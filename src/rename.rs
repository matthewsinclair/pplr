//! `pplr rename "Old, First" "New, First" [--dry-run] [--refs DIR]...`, ported
//! from bin/pplr_rename (ST0001 WP08): the folder moved, the files named after
//! the person renamed, the About text updated, the old name recorded as an
//! alias, every path-style link under the refs directories repointed in one
//! longest-first pass per file, and the running prose that still names the
//! person listed, never changed.

use crate::Result;
use crate::config::Config;
use crate::tree::letter_of;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "Usage: pplr rename \"Old, First\" \"New, First\" [--dry-run] [--refs DIR]...
  Moves the folder (to a new letter if the surname's initial changes), renames
  files named \"First Surname (...)\", updates the name in the About file, records
  the old name in .index/aliases so an old pplr:// URL still resolves, and repoints
  path-style links under the refs directories. Names in running prose elsewhere are
  listed, not changed.";

/// The files whose links are repointed, as grep's --include named them.
const REF_EXTENSIONS: &[&str] = &[
  "md", "csv", "txt", "yaml", "yml", "json", "fileloc", "webloc", "inetloc", "html",
];
/// The directories never searched for links.
const SKIP_DIRS: &[&str] = &[".git", ".index", "node_modules", "_build", "deps", ".build"];
/// Running prose: the files searched, and the directories passed over.
const PROSE_EXTENSIONS: &[&str] = &["md", "csv", "txt"];
const PROSE_SKIP_DIRS: &[&str] = &[".git", ".index"];
/// Prose lines listed, and the width each is cut to (path and line included).
const PROSE_LINES: usize = 40;
const PROSE_WIDTH: usize = 200;

/// A person's name, as the folder and their files spell it.
struct Name {
  letter: String,
  family: String,
  first: String,
}

impl Name {
  /// "H/Horely, Steve" or "Horely, Steve".
  fn parse(arg: &str) -> Option<Self> {
    let name = arg.split_once('/').map_or(arg, |(_, rest)| rest);
    let (family, first) = name.split_once(", ")?;
    Some(Self {
      letter: letter_of(family),
      family: family.to_owned(),
      first: first.to_owned(),
    })
  }

  fn folder(&self) -> String {
    format!("{}, {}", self.family, self.first)
  }

  fn key(&self) -> String {
    format!("{}/{}", self.letter, self.folder())
  }

  fn full(&self) -> String {
    format!("{} {}", self.first, self.family)
  }
}

fn enc(s: &str) -> String {
  s.replace(',', "%2C")
    .replace(' ', "%20")
    .replace('(', "%28")
    .replace(')', "%29")
}

fn enc_space(s: &str) -> String {
  s.replace(' ', "%20")
}

/// Every path-style form of the old name, and what each becomes.
fn pairs(old: &Name, new: &Name) -> Vec<(String, String)> {
  vec![
    (old.key(), new.key()),
    (
      format!("{}/{}", old.letter, enc(&old.folder())),
      format!("{}/{}", new.letter, enc(&new.folder())),
    ),
    (
      format!("{}/{}", old.letter, enc_space(&old.folder())),
      format!("{}/{}", new.letter, enc_space(&new.folder())),
    ),
    (old.folder(), new.folder()),
    (enc(&old.folder()), enc(&new.folder())),
    (format!("{} (", old.full()), format!("{} (", new.full())),
    (
      format!("{} (", enc_space(&old.full())),
      format!("{} (", enc_space(&new.full())),
    ),
    (
      enc(&format!("{} (", old.full())),
      enc(&format!("{} (", new.full())),
    ),
  ]
}

/// Every form replaced in one pass, longest first at each place, so a new
/// name that contains the old one is never rewritten again.
pub fn replace_all(text: &str, pairs: &[(String, String)]) -> String {
  let mut sorted: Vec<&(String, String)> =
    pairs.iter().filter(|(from, _)| !from.is_empty()).collect();
  sorted.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));
  let mut out = String::with_capacity(text.len());
  let mut rest = text;
  'scan: while !rest.is_empty() {
    for (from, to) in &sorted {
      if let Some(after) = rest.strip_prefix(from.as_str()) {
        out.push_str(to);
        rest = after;
        continue 'scan;
      }
    }
    let step = rest.chars().next().map_or(1, char::len_utf8);
    out.push_str(&rest[..step]);
    rest = &rest[step..];
  }
  out
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let caller = std::env::current_dir()?;
  let (mut dry, mut refs, mut names) = (false, Vec::new(), Vec::new());
  let mut args = args.iter();
  while let Some(arg) = args.next() {
    match arg.as_str() {
      "--dry-run" => dry = true,
      "--refs" => refs.push(caller.join(args.next().map(String::as_str).unwrap_or_default())),
      "-h" | "--help" => {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
      }
      name => names.push(name.to_owned()),
    }
  }
  let [old_arg, new_arg] = &names[..] else {
    println!("{USAGE}");
    return Ok(ExitCode::FAILURE);
  };
  if refs.is_empty() {
    refs.push(
      config
        .refs_dir
        .clone()
        .ok_or("pplr rename needs --refs DIR, or PPLR_REFS_DIR, or HOME for ~/Dropbox")?,
    );
  }
  let mut roots = Vec::new();
  for r in &refs {
    match r.canonicalize() {
      Ok(physical) if physical.is_dir() => roots.push(physical),
      _ => {
        println!("Error: no such directory: {}", r.display());
        return Ok(ExitCode::FAILURE);
      }
    }
  }
  let mut parsed = Vec::new();
  for arg in [old_arg, new_arg] {
    match Name::parse(arg) {
      Some(name) => parsed.push(name),
      None => {
        eprintln!("Error: '{arg}' is not \"Surname, First\"");
        return Ok(ExitCode::FAILURE);
      }
    }
  }
  let (old, new) = (&parsed[0], &parsed[1]);
  let (old_dir, new_dir) = (config.data.join(old.key()), config.data.join(new.key()));
  if !old_dir.is_dir() {
    println!("Error: no such person: {}", old.key());
    return Ok(ExitCode::FAILURE);
  }
  if old.key() == new.key() {
    println!("Error: the names are the same");
    return Ok(ExitCode::FAILURE);
  }
  if new_dir.symlink_metadata().is_ok() {
    println!("Error: {} already exists", new.key());
    return Ok(ExitCode::FAILURE);
  }

  let pairs = pairs(old, new);
  println!(
    "pplr rename: {} -> {}{}",
    old.key(),
    new.key(),
    if dry { " (dry run)" } else { "" }
  );

  let own_physical = old_dir.canonicalize().unwrap_or_else(|_| old_dir.clone());
  let needles: Vec<&str> = pairs.iter().map(|(from, _)| from.as_str()).collect();
  let mut ref_files: Vec<PathBuf> = search(&roots, REF_EXTENSIONS, SKIP_DIRS)
    .into_iter()
    .filter(|f| contains_any(f, &needles))
    .filter(|f| !f.starts_with(&own_physical))
    .collect();
  ref_files.sort_by(|a, b| {
    a.as_os_str()
      .as_encoded_bytes()
      .cmp(b.as_os_str().as_encoded_bytes())
  });
  ref_files.dedup();

  let prefix = format!("{} (", old.full());
  let own_files = named_after(&old_dir, &prefix);

  if dry {
    println!(
      "\nWould move the folder, and rename {} file(s) in it:",
      own_files.len()
    );
    for f in &own_files {
      println!("  {}", f.strip_prefix(&old_dir).unwrap_or(f).display());
    }
    println!("\nWould repoint links in {} file(s):", ref_files.len());
    for f in &ref_files {
      println!("  {}", f.display());
    }
  } else {
    std::fs::create_dir_all(config.data.join(&new.letter))?;
    std::fs::rename(&old_dir, &new_dir)?;
    for f in &own_files {
      let moved = new_dir.join(f.strip_prefix(&old_dir).unwrap_or(f));
      let name = moved
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
      let renamed = moved.with_file_name(format!("{}{}", new.full(), &name[old.full().len()..]));
      std::fs::rename(&moved, &renamed)?;
    }
    // The About file: its title, header and running text name the person
    if let Some(about) = about_file(&new_dir)
      && let Some(text) = crate::fs::read_exact(&about)?
    {
      crate::fs::write_atomic(&about, text.replace(&old.full(), &new.full()).as_bytes())?;
    }
    // The person's own notes and meetings link to their About by file name
    let own_needles = [prefix.as_str(), &format!("{} (", enc_space(&old.full()))];
    for f in search(std::slice::from_ref(&new_dir), &["md"], &[])
      .into_iter()
      .filter(|f| contains_any(f, &own_needles))
    {
      rewrite(&f, &pairs)?;
    }
    // An old pplr:// URL on a card still finds this person
    record_alias(&new_dir, &old.key())?;
    for f in &ref_files {
      rewrite(f, &pairs)?;
    }
    println!(
      "\nMoved the folder, renamed {} file(s), recorded the alias {}.",
      own_files.len(),
      old.key()
    );
    println!("Repointed links in {} file(s):", ref_files.len());
    for f in &ref_files {
      println!("  {}", f.display());
    }
    if reindex() {
      println!("Reindexed.");
    }
  }

  // Running prose is a person's call: list it, never change it
  let (full, folder) = (old.full(), old.folder());
  let skip = [prefix.clone(), format!("{folder}/")];
  let mut prose = Vec::new();
  // Sorted, so the listing (and where PROSE_LINES cuts it) is the same on
  // every filesystem; directory order is APFS's on macOS and another on Linux
  let mut prose_files = search(&roots, PROSE_EXTENSIONS, PROSE_SKIP_DIRS);
  prose_files.sort();
  'files: for f in prose_files {
    let Some(text) = text_of(&f) else { continue };
    for (n, line) in text.lines().enumerate() {
      if (line.contains(&full) || line.contains(&folder))
        && !skip.iter().any(|s| line.contains(s.as_str()))
      {
        prose.push(format!("{}:{}:{line}", f.display(), n + 1));
        if prose.len() == PROSE_LINES {
          break 'files;
        }
      }
    }
  }
  if !prose.is_empty() {
    println!("\nThe old name still appears in running text (not changed):");
    for line in prose {
      println!("  {}", line.chars().take(PROSE_WIDTH).collect::<String>());
    }
  }
  Ok(ExitCode::SUCCESS)
}

/// Text files under `roots` with one of `extensions`, in directory order,
/// passing over `skip` directories and every symlink below a root, as
/// `grep -r --include --exclude-dir` walks.
fn search(roots: &[PathBuf], extensions: &[&str], skip: &[&str]) -> Vec<PathBuf> {
  fn walk(dir: &Path, extensions: &[&str], skip: &[&str], out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
      return;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      let Ok(kind) = entry.file_type() else {
        continue;
      };
      let name = entry.file_name();
      if kind.is_dir() {
        if !skip.iter().any(|s| name == *s) {
          walk(&path, extensions, skip, out);
        }
      } else if kind.is_file()
        && path
          .extension()
          .is_some_and(|e| extensions.iter().any(|x| e == *x))
      {
        out.push(path);
      }
    }
  }
  let mut out = Vec::new();
  for root in roots {
    walk(root, extensions, skip, &mut out);
  }
  out
}

/// A file's text, unless it is binary (holds a NUL byte), as grep -I reads it.
fn text_of(path: &Path) -> Option<String> {
  let bytes = std::fs::read(path).ok()?;
  if bytes.contains(&0) {
    return None;
  }
  Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn contains_any(path: &Path, needles: &[&str]) -> bool {
  text_of(path).is_some_and(|text| needles.iter().any(|n| text.contains(n)))
}

fn rewrite(path: &Path, pairs: &[(String, String)]) -> Result<()> {
  let text =
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
  let out = replace_all(&text, pairs);
  if out != text {
    crate::fs::write_atomic(path, out.as_bytes())?;
  }
  Ok(())
}

/// Files and folders inside `dir` whose names start with `prefix`, in the
/// order `find` meets them.
fn named_after(dir: &Path, prefix: &str) -> Vec<PathBuf> {
  fn walk(dir: &Path, prefix: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
      return;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if entry.file_name().to_string_lossy().starts_with(prefix) {
        out.push(path.clone());
      }
      if entry.file_type().is_ok_and(|t| t.is_dir()) {
        walk(&path, prefix, out);
      }
    }
  }
  let mut out = Vec::new();
  walk(dir, prefix, &mut out);
  out
}

/// The first `About/*(About).md`, in name order, as `ls | head -1`.
fn about_file(dir: &Path) -> Option<PathBuf> {
  let mut abouts: Vec<PathBuf> = std::fs::read_dir(dir.join("About"))
    .ok()?
    .flatten()
    .map(|e| e.path())
    .filter(|p| {
      p.file_name()
        .is_some_and(|n| n.to_string_lossy().ends_with("(About).md"))
    })
    .collect();
  abouts.sort();
  abouts.into_iter().next()
}

fn record_alias(dir: &Path, key: &str) -> Result<()> {
  let index = dir.join(".index");
  std::fs::create_dir_all(&index)?;
  let aliases = index.join("aliases");
  let existing = crate::fs::read_exact(&aliases)?.unwrap_or_default();
  if existing.lines().any(|line| line == key) {
    return Ok(());
  }
  let sep = if existing.is_empty() || existing.ends_with('\n') {
    ""
  } else {
    "\n"
  };
  crate::fs::write_atomic(&aliases, format!("{existing}{sep}{key}\n").as_bytes())
}

/// `pplr reindex`, quietly; whether it succeeded.
fn reindex() -> bool {
  std::env::current_exe().is_ok_and(|pplr| {
    Command::new(pplr)
      .arg("reindex")
      .output()
      .is_ok_and(|o| o.status.success())
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn names(old: &str, new: &str) -> Vec<(String, String)> {
    let (Some(old), Some(new)) = (Name::parse(old), Name::parse(new)) else {
      return Vec::new();
    };
    pairs(&old, &new)
  }

  #[test]
  fn every_form_is_replaced_once_longest_first() {
    let p = names("Varley, Hanna", "Varley, Hannah");
    let text = "[x](../V/Varley, Hanna/About/Hanna Varley (About).md) and V/Varley%2C%20Hanna";
    assert_eq!(
      replace_all(text, &p),
      "[x](../V/Varley, Hannah/About/Hannah Varley (About).md) and V/Varley%2C%20Hannah"
    );
  }

  #[test]
  fn a_name_may_carry_its_letter_and_the_surname_letters_the_folder() {
    let key = |s: &str| Name::parse(s).map(|n| n.key());
    assert_eq!(key("Q/Quennell, Imogen"), Some("Q/Quennell, Imogen".into()));
    assert_eq!(key("quennell, Imogen"), Some("Q/quennell, Imogen".into()));
    assert_eq!(key("Quennell"), None);
  }
}
