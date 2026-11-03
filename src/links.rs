//! `pplr links`: Markdown links into the People tree rewritten as pplr://
//! URLs, as the engine's `links` mode did (ST0001 WP08). A link to a person's
//! About file or folder becomes the bare person URL; a link to anything else
//! keeps its path inside the folder. A link whose person or file is not there
//! is left as it is, and listed.

use crate::Result;
use crate::config::Config;
use crate::contacts::to_swift_json;
use crate::marker::marker_url;
use crate::normalise::percent_decode;
use crate::tree::{self, Person};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "Usage: pplr links <file or dir>... [--apply] [--json]
  Finds links into the people tree, as paths (../../Career/People/K/Kemp, Jon/...)
  or CMS URLs (http://localhost:4360/people/...), and rewrites them as pplr:// URLs:
  a link to a person's About becomes pplr://k/kemp-jon, anything else keeps its
  path, eg pplr://k/kemp-jon/Meetings/20260924 Intro/Summary.md. A link whose
  person or file cannot be found is left as it is and listed. A dry run unless --apply.";

/// One link found: converted (`to`), or left as it was (`problem`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkChange {
  pub file: String,
  pub line: usize,
  pub from: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub to: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub problem: Option<String>,
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let (mut apply, mut json, mut paths) = (false, false, Vec::new());
  let caller = std::env::current_dir()?;
  for arg in args {
    match arg.as_str() {
      "--apply" => apply = true,
      "--json" => json = true,
      "-h" | "--help" => {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
      }
      path => paths.push(physical(&caller.join(path))),
    }
  }
  if paths.is_empty() {
    println!("{USAGE}");
    return Ok(ExitCode::FAILURE);
  }

  let people = tree::people(&config.data);
  let by_key = by_key(&people);
  let mut all = Vec::new();
  for file in markdown_files(&paths) {
    let Ok(text) = std::fs::read_to_string(&file) else {
      continue;
    };
    let (out, changes) = convert(&text, &file.to_string_lossy(), &by_key);
    all.extend(changes);
    if apply && out != text {
      crate::fs::write_atomic(&file, out.as_bytes())?;
    }
  }

  if json {
    println!("{}", to_swift_json(&serde_json::to_value(&all)?));
    return Ok(ExitCode::SUCCESS);
  }
  let (done, left): (Vec<_>, Vec<_>) = all.iter().partition(|c| c.to.is_some());
  let files: std::collections::BTreeSet<&str> = done.iter().map(|c| c.file.as_str()).collect();
  println!(
    "pplr links{}\n",
    if apply {
      ""
    } else {
      " (dry run: nothing written; --apply to write)"
    }
  );
  println!("  links into the people tree  {}", all.len());
  println!(
    "  {:<28}{}",
    if apply { "converted" } else { "to convert" },
    done.len()
  );
  println!("  left as they are            {}", left.len());
  println!("  files                       {}", files.len());
  if !left.is_empty() {
    println!("\nLeft as they are:");
    for c in &left {
      println!(
        "  {}:{}  {}",
        c.file,
        c.line,
        c.problem.as_deref().unwrap_or_default()
      );
    }
  }
  Ok(ExitCode::SUCCESS)
}

/// A path with its directory resolved physically, as `cd "$(dirname)" && pwd -P`.
fn physical(path: &Path) -> PathBuf {
  match (
    path.parent().and_then(|d| d.canonicalize().ok()),
    path.file_name(),
  ) {
    (Some(dir), Some(name)) => dir.join(name),
    _ => path.to_path_buf(),
  }
}

/// Everyone by their key and by each former name.
pub fn by_key(people: &[Person]) -> HashMap<String, &Person> {
  let mut out = HashMap::new();
  for person in people {
    for key in person.keys() {
      out.insert(key.clone(), person);
    }
  }
  out
}

/// `](<target>)` and `](target)`, the Markdown link targets the engine read.
fn link_pattern() -> Result<Regex> {
  Ok(Regex::new(r"\]\(\s*(?:<([^>\n]+)>|([^)\s]+))\s*\)")?)
}

/// The text with each link into the tree converted, and every link found.
pub fn convert(
  text: &str,
  file: &str,
  by_key: &HashMap<String, &Person>,
) -> (String, Vec<LinkChange>) {
  let Ok(re) = link_pattern() else {
    return (text.to_owned(), Vec::new());
  };
  let mut out = text.to_owned();
  let mut changes = Vec::new();
  let matches: Vec<_> = re.captures_iter(text).collect();
  for caps in matches.iter().rev() {
    let (Some(whole), Some(target)) = (caps.get(0), caps.get(1).or_else(|| caps.get(2))) else {
      continue;
    };
    let target = target.as_str();
    let Some((key, raw_rest)) = people_link_parts(target) else {
      continue;
    };
    let line = text[..whole.start()].matches('\n').count() + 1;
    let change = |to: Option<String>, problem: Option<String>| LinkChange {
      file: file.to_owned(),
      line,
      from: target.to_owned(),
      to,
      problem,
    };
    let Some(person) = by_key.get(&key) else {
      changes.push(change(None, Some(format!("no such person: {key}"))));
      continue;
    };
    let mut rest = raw_rest.strip_suffix('/').unwrap_or(&raw_rest).to_owned();
    // The About file is the person: its name follows a rename, the person URL does not
    if (rest.starts_with("About/") && rest.ends_with("(About).md")) || rest == "About" {
      rest.clear();
    }
    if !rest.is_empty() && !person.dir.join(&rest).exists() {
      changes.push(change(
        None,
        Some(format!("no such file in {}: {rest}", person.key)),
      ));
      continue;
    }
    let url = if rest.is_empty() {
      marker_url(&person.key)
    } else {
      format!("{}/{rest}", marker_url(&person.key))
    };
    let link = if url.contains([' ', '(', ')', '<', '>']) {
      format!("](<{url}>)")
    } else {
      format!("]({url})")
    };
    out.replace_range(whole.range(), &link);
    changes.push(change(Some(url), None));
  }
  changes.reverse();
  (out, changes)
}

/// ("L/Surname, First", the path inside the folder) for a link into the tree:
/// a path through `Career/People/` or a URL on the local CMS's `/people/`.
pub fn people_link_parts(target: &str) -> Option<(String, String)> {
  let sub = ["Career/People/", "localhost:4360/people/"]
    .iter()
    .find_map(|anchor| target.find(anchor).map(|i| &target[i + anchor.len()..]))?;
  let sub = sub.split(['?', '#']).next().unwrap_or_default();
  let sub = percent_decode(sub).unwrap_or_else(|| sub.to_owned());
  let mut parts = sub.splitn(3, '/');
  let (letter, name) = (parts.next()?, parts.next()?);
  if letter.chars().count() != 1 || !name.contains(", ") {
    return None;
  }
  Some((
    format!("{letter}/{name}"),
    parts.next().unwrap_or_default().to_owned(),
  ))
}

/// Each real Markdown file once: a symlinked note is followed to its file, so
/// it is counted once and written in place. A directory is searched for `.md`,
/// skipping hidden files and folders.
pub fn markdown_files(roots: &[PathBuf]) -> Vec<PathBuf> {
  let mut raw = Vec::new();
  for root in roots {
    if root.is_file() {
      raw.push(root.clone());
    } else if root.is_dir() {
      let base = root.canonicalize().unwrap_or_else(|_| root.clone());
      walk(&base, &mut raw);
    }
  }
  let mut seen = std::collections::BTreeSet::new();
  let mut files: Vec<PathBuf> = raw
    .into_iter()
    .map(|p| p.canonicalize().unwrap_or(p))
    .filter(|p| seen.insert(p.clone()))
    .collect();
  files.sort_by(|a, b| {
    a.as_os_str()
      .as_encoded_bytes()
      .cmp(b.as_os_str().as_encoded_bytes())
  });
  files
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if entry.file_name().to_string_lossy().starts_with('.') {
      continue;
    }
    if path.is_dir() {
      walk(&path, out);
    } else if path.extension().is_some_and(|e| e == "md") {
      out.push(path);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_link_into_the_tree_names_its_person_and_path() {
    assert_eq!(
      people_link_parts(
        "../../Career/People/K/Kemp,%20Jon/Meetings/20260924%20Intro/Summary.md#top"
      ),
      Some((
        "K/Kemp, Jon".into(),
        "Meetings/20260924 Intro/Summary.md".into()
      ))
    );
    assert_eq!(
      people_link_parts("http://localhost:4360/people/K/Kemp, Jon"),
      Some(("K/Kemp, Jon".into(), String::new()))
    );
    assert_eq!(people_link_parts("https://example.org/K/Kemp, Jon"), None);
    assert_eq!(people_link_parts("Career/People/KK/Kemp, Jon"), None);
  }
}
