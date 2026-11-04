//! `pplr grep [-t|--tag] <text>`: a basic regular expression over every
//! Markdown file, or over each person's About/tags.yaml, shown person by person.

use crate::Result;
use crate::config::Config;
use crate::fs::{Order, read_text, walk};
use crate::tree::names;
use regex::{Regex, RegexBuilder};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Matching lines of a tags.yaml shown per person.
const TAG_LINES: usize = 3;

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let mut tags = false;
  let mut text = None;
  for arg in args {
    match arg.as_str() {
      "-t" | "--tag" if text.is_none() => tags = true,
      _ => {
        text = Some(arg.clone());
        break;
      }
    }
  }
  let Some(text) = text.filter(|t| !t.is_empty()) else {
    println!("Usage: pplr grep [options] <text-to-search>");
    println!("Options:");
    println!("  -t, --tag    search in tag files instead of markdown files");
    return Ok(ExitCode::FAILURE);
  };
  let root = &config.data;
  // grep reports a pattern it cannot read on stderr, which the script discards
  let pattern = RegexBuilder::new(&from_basic(&text))
    .case_insensitive(tags)
    .build()
    .ok();
  let mut found = false;
  if tags {
    println!("Searching in tag files for: {text}\n");
    for file in tag_files(root) {
      let lines: Vec<String> = matching(&file, pattern.as_ref())
        .into_iter()
        .take(TAG_LINES)
        .collect();
      if !lines.is_empty() {
        show(&file, lines.join(" ").trim_end())?;
        found = true;
      }
    }
    if !found {
      println!("No matches found in tag files.");
    }
  } else {
    println!("Searching for: {text}\n");
    for file in markdown_files(root) {
      for line in matching(&file, pattern.as_ref()) {
        show(&file, &line)?;
        found = true;
      }
    }
    if !found {
      println!("No matches found.");
    }
  }
  Ok(ExitCode::SUCCESS)
}

/// One match, under the person two folders above the file, with the role and
/// company from their About header (each line's link text).
fn show(file: &Path, text: &str) -> Result<()> {
  let person = file
    .parent()
    .and_then(Path::parent)
    .unwrap_or(file)
    .to_path_buf();
  let name = person
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_default();
  let about = names(&person.join("About"))
    .into_iter()
    .find(|n| n.ends_with("(About).md"))
    .map(|n| person.join("About").join(n));
  println!(
    "\u{1f464} [{name}](<{}>)",
    about.as_deref().unwrap_or(&person).display()
  );
  let header = match &about {
    Some(a) => read_text(a)?,
    None => None,
  };
  if let Some(header) = header {
    if let Some(role) = field(&header, "Role").filter(|r| !r.is_empty()) {
      println!("   {role}");
    }
    if let Some(company) = field(&header, "Company").filter(|c| !c.is_empty()) {
      println!("   at {company}");
    }
  }
  println!("   \u{2192} {text}\n");
  Ok(())
}

/// The first `- <name>:` line's value, or its link's text when it starts with one.
fn field(text: &str, name: &str) -> Option<String> {
  let prefix = format!("- {name}:");
  let line = text.lines().find(|l| l.starts_with(&prefix))?;
  let value = line[prefix.len()..].trim_start_matches([' ', '\t']);
  if let Some(rest) = value.strip_prefix('[')
    && let Some(close) = rest.find(']')
  {
    return Some(rest[..close].to_owned());
  }
  Some(value.to_owned())
}

/// The lines of a file the pattern matches.
fn matching(file: &Path, pattern: Option<&Regex>) -> Vec<String> {
  let (Some(pattern), Ok(text)) = (pattern, fs::read(file)) else {
    return Vec::new();
  };
  String::from_utf8_lossy(&text)
    .lines()
    .filter(|line| pattern.is_match(line))
    .map(str::to_owned)
    .collect()
}

/// Every .md file under each top-level folder (`grep -r <root>/*/`), by path,
/// not following symlinks below the top. A name that is not UTF-8 is passed
/// over, as 2.0.0 joined each name back from its lossy form and found nothing.
pub(crate) fn markdown_files(root: &Path) -> Vec<PathBuf> {
  let mut out = Vec::new();
  for top in names(root) {
    let dir = root.join(&top);
    if !top.starts_with('.') && dir.is_dir() {
      walk(&dir, Order::Name, &mut |entry| {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
          return false;
        };
        if entry.file_type().is_ok_and(|t| t.is_file()) && name.ends_with(".md") {
          out.push(entry.path());
        }
        true
      });
    }
  }
  out
}

/// `<root>/<top>/<person>/About/tags.yaml`, outside any `_` folder, by path.
fn tag_files(root: &Path) -> Vec<PathBuf> {
  let mut out = Vec::new();
  for top in names(root).into_iter().filter(|t| !t.starts_with('_')) {
    for person in names(&root.join(&top))
      .into_iter()
      .filter(|p| !p.starts_with('_'))
    {
      let file = root.join(&top).join(&person).join("About/tags.yaml");
      if file.is_file() {
        out.push(file);
      }
    }
  }
  out
}

/// A POSIX basic regular expression, as GNU grep reads it, in the regex
/// crate's syntax: `\(` `\)` `\{` `\}` `\|` `\+` `\?` are operators and their
/// bare forms literals; `*` is literal first, `^` only anchors first and `$`
/// only last; bracket expressions keep their POSIX meaning.
pub fn from_basic(bre: &str) -> String {
  let chars: Vec<char> = bre.chars().collect();
  let mut out = String::new();
  let mut i = 0;
  // Whether the next character starts an expression (where * is literal)
  let mut at_start = true;
  while i < chars.len() {
    let c = chars[i];
    let started = at_start;
    at_start = false;
    match c {
      '\\' if i + 1 < chars.len() => {
        let next = chars[i + 1];
        i += 1;
        match next {
          '(' => {
            out.push('(');
            at_start = true;
          }
          ')' | '{' | '}' | '|' | '+' | '?' => {
            out.push(next);
            at_start = next == '|';
          }
          '<' | '>' => out.push_str(r"\b"),
          'w' | 'W' | 's' | 'S' | 'b' | 'B' => {
            out.push('\\');
            out.push(next);
          }
          other => out.push_str(&regex::escape(&other.to_string())),
        }
      }
      '*' if started => out.push_str(r"\*"),
      '^' if started => {
        out.push('^');
        at_start = true;
      }
      '$' if i + 1 == chars.len() || bre[byte_at(&chars, i + 1)..].starts_with("\\)") => {
        out.push('$')
      }
      '[' => {
        let (class, end) = bracket(&chars, i);
        out.push_str(&class);
        i = end;
      }
      '.' | '*' => out.push(c),
      other => out.push_str(&regex::escape(&other.to_string())),
    }
    i += 1;
  }
  out
}

fn byte_at(chars: &[char], i: usize) -> usize {
  chars[..i].iter().map(|c| c.len_utf8()).sum()
}

/// A bracket expression from `chars[start]` ('['), and the index of its ']'.
/// An unclosed one is a literal '['.
fn bracket(chars: &[char], start: usize) -> (String, usize) {
  let mut out = String::from("[");
  let mut i = start + 1;
  if chars.get(i) == Some(&'^') {
    out.push('^');
    i += 1;
  }
  let first = i;
  while i < chars.len() {
    let c = chars[i];
    if c == ']' && i > first {
      out.push(']');
      return (out, i);
    }
    if c == '[' && matches!(chars.get(i + 1), Some(':' | '.' | '=')) {
      let kind = chars[i + 1];
      let close: String = [kind, ']'].iter().collect();
      let rest: String = chars[i..].iter().collect();
      if let Some(end) = rest.find(&close) {
        let token: String = rest[..end + close.len()].to_owned();
        out.push_str(&token);
        i += token.chars().count();
        continue;
      }
    }
    match c {
      '\\' | '[' | '&' | '~' | ']' => {
        out.push('\\');
        out.push(c);
      }
      _ => out.push(c),
    }
    i += 1;
  }
  (r"\[".to_owned(), start)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn matches(bre: &str, line: &str) -> bool {
    Regex::new(&from_basic(bre)).is_ok_and(|re| re.is_match(line))
  }

  #[test]
  fn a_basic_expression_keeps_its_posix_meaning() {
    assert!(matches("ledger", "Talked about ledgers."));
    assert!(matches("led.er", "ledger"));
    assert!(matches("a(b)", "xa(b)y"));
    assert!(!matches("a(b)", "ab"));
    assert!(matches(r"\(ab\)\{2\}", "abab"));
    assert!(matches("a+b", "a+b"));
    assert!(!matches("a+b", "aab"));
    assert!(matches(r"a\+b", "aab"));
    assert!(matches("*star", "a *star"));
    assert!(matches("^- Role", "- Role: x"));
    assert!(matches("a^b", "a^b"));
    assert!(matches("end$", "the end"));
    assert!(matches("a$b", "a$b"));
    assert!(matches("[]x]", "]"));
    assert!(matches("[[:digit:]]\\{8\\}", "20260120"));
    assert!(matches("[", "a [ b"));
    assert!(matches(r"\.md", "x.md"));
    assert!(!matches(r"\.md", "xamd"));
  }

  #[test]
  fn a_field_reads_its_link_text() {
    let text = "- Role:     [CTO](https://x)\n- Company:  Larkspur Ledger\n";
    assert_eq!(field(text, "Role").as_deref(), Some("CTO"));
    assert_eq!(field(text, "Company").as_deref(), Some("Larkspur Ledger"));
    assert_eq!(field(text, "Email"), None);
  }
}
