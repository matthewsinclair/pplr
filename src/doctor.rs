//! `pplr doctor [--fix]`: checks the data tree, its index files, the
//! settings and the tools pplr needs, one row per check, each saying what to
//! fix. `--fix` makes the letter folders and `.index`, reindexes, and removes
//! old `Index/` folders, as the legacy doctor did. It exits with the number of
//! failed checks.

use crate::Result;
use crate::config::Config;
use crate::layout::Layout;
use crate::tools::on_path;
use serde_json::Value;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
  Pass,
  Warn,
  Fail,
  Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
  pub status: Status,
  pub text: String,
}

pub struct Section {
  pub title: &'static str,
  pub rows: Vec<Row>,
}

fn row(status: Status, text: impl Into<String>) -> Row {
  Row {
    status,
    text: text.into(),
  }
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let fix = match args {
    [] => false,
    [flag] if flag == "--fix" => true,
    _ => {
      println!("Usage: pplr doctor [--fix]");
      return Ok(ExitCode::FAILURE);
    }
  };
  let sections = check(config, fix)?;
  print!(
    "{}",
    render(&sections, fix, std::io::stdout().is_terminal())
  );
  let failed = sections
    .iter()
    .flat_map(|s| &s.rows)
    .filter(|r| r.status == Status::Fail)
    .count();
  Ok(ExitCode::from(u8::try_from(failed).unwrap_or(u8::MAX)))
}

/// Every check, in the order the report shows them; with `fix`, the fixes
/// are made as their checks run.
pub fn check(config: &Config, fix: bool) -> Result<Vec<Section>> {
  let data = &config.data;
  Ok(vec![
    Section {
      title: "Install",
      rows: install_rows(),
    },
    Section {
      title: "Environment Variables",
      rows: settings(config),
    },
    Section {
      title: "Directory Structure",
      rows: directories(data, fix),
    },
    Section {
      title: "Dependencies",
      rows: tools(),
    },
    Section {
      title: "Index Files",
      rows: index_files(data, fix),
    },
    Section {
      title: "Common Issues",
      rows: common_issues(data, fix)?,
    },
  ])
}

/// The running tree and its version line (T4), its manifest against its files
/// (T3), and the link switch's shadow check (T5).
fn install_rows() -> Vec<Row> {
  use crate::install::{self, Kind, Links};
  let line = install::version_line();
  let Ok(layout) = Layout::running() else {
    return vec![row(
      Status::Fail,
      format!("{line}: cannot find the running pplr's tree"),
    )];
  };
  let (kind, manifest) = install::kind(&layout.root);
  let mut rows = Vec::new();
  match (kind, &manifest) {
    (Kind::Unknown, _) => rows.push(row(
      Status::Fail,
      format!("{line}: this pplr is in no tree pplr knows (no manifest, and not a checkout)"),
    )),
    (Kind::Opt | Kind::Brew, Some(m)) => {
      rows.push(row(Status::Pass, line));
      let drifted = install::drift(&layout.root, m);
      if drifted.is_empty() {
        rows.push(row(Status::Pass, "The manifest matches every runtime file"));
      } else {
        let fix = if kind == Kind::Brew {
          "brew reinstall pplr"
        } else {
          "pplr install --force, from dev"
        };
        rows.push(row(
          Status::Fail,
          format!(
            "Files differ from the manifest: {} ({fix})",
            drifted.join(", ")
          ),
        ));
      }
    }
    _ => {
      rows.push(row(Status::Pass, line));
      // In dev the line names the commit the binary was built from (T4)
      let head = Command::new("git")
        .arg("-C")
        .arg(&layout.root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
      match head {
        Some(head) if head != install::SOURCE_COMMIT => rows.push(row(
          Status::Warn,
          format!("HEAD has moved past this build ({head:.12}): rebuild with bin/devbin build"),
        )),
        Some(_) => {}
        None => rows.push(row(
          Status::Fail,
          "git cannot read this checkout, so its version line names no commit",
        )),
      }
    }
  }
  if cfg!(target_os = "macos") {
    rows.extend(signatures(kind, &layout.root));
  }
  if let Some(home) = std::env::var_os("HOME") {
    let report = Links::new(PathBuf::from(home).join(".local/bin")).report();
    let status = if report.ok {
      Status::Pass
    } else {
      Status::Fail
    };
    rows.push(row(status, report.text.replace('\n', "; ")));
  }
  rows
}

/// How pplr's binaries are signed: the one source, shared with the signer.
const SIGNING: &str = include_str!("../packaging/macos/signing");

/// A `key=value` fact from packaging/macos/signing.
fn signing_fact(key: &str) -> Option<&'static str> {
  SIGNING
    .lines()
    .find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
}

/// Each binary's signature against its designated requirement (ST0002 E2). A
/// dev build is the linker's ad-hoc signature until the release signs it.
fn signatures(kind: crate::install::Kind, root: &Path) -> Vec<Row> {
  use crate::install::Kind;
  if kind == Kind::Dev {
    return vec![row(
      Status::Pass,
      "Signatures: a dev build is not Developer ID signed (a release is: bin/devbin macos prepare)",
    )];
  }
  ["pplr", "pplr-contacts"]
    .iter()
    .map(|binary| {
      let path = root.join("bin").join(binary);
      let Some(requirement) = signing_fact(&format!("requirement.{binary}")) else {
        return row(
          Status::Fail,
          format!("packaging/macos/signing names no requirement for {binary}"),
        );
      };
      let verified = Command::new("codesign")
        .args(["--verify", "--strict"])
        .arg(format!("-R={requirement}"))
        .arg(&path)
        .output()
        .is_ok_and(|out| out.status.success());
      if verified {
        row(
          Status::Pass,
          format!(
            "{binary} is signed as {}",
            signing_fact(&format!("id.{binary}")).unwrap_or(binary)
          ),
        )
      } else {
        row(
          Status::Fail,
          format!(
            "{} is not signed with pplr's Developer ID (reinstall it from a release)",
            path.display()
          ),
        )
      }
    })
    .collect()
}

fn settings(config: &Config) -> Vec<Row> {
  let mut rows = Vec::new();
  match std::env::var_os("PPLR_DATA").filter(|v| !v.is_empty()) {
    Some(_) => rows.push(row(
      Status::Pass,
      format!("PPLR_DATA is set: {}", config.data.display()),
    )),
    None => rows.push(row(
      Status::Warn,
      format!("PPLR_DATA not set (using {})", config.data.display()),
    )),
  }
  // Gtools reads PPLR_DIR first, so the two must name the same tree (D8)
  if let Some(legacy) = &config.data_legacy
    && std::env::var_os("PPLR_DATA").is_some_and(|v| !v.is_empty())
    && legacy != &config.data
  {
    rows.push(row(
      Status::Warn,
      format!(
        "PPLR_DIR ({}) differs from PPLR_DATA: unset PPLR_DIR, or make them the same",
        legacy.display()
      ),
    ));
  }
  // The backups and the cache moved to XDG (D5); the old folders stay where they are
  if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
    let moved = [
      (
        "Library/Application Support/pplr/contacts-backups",
        &config.backup_dir,
        "backups",
      ),
      ("Library/Caches/pplr", &config.cache_dir, "cache"),
    ];
    for (old, new, what) in moved {
      let old = home.join(old);
      if old.is_dir() && &old != new {
        rows.push(row(
          Status::Warn,
          format!(
            "Legacy {what} folder {}: pplr now uses {}; move what you need, or remove it",
            old.display(),
            new.display()
          ),
        ));
      }
    }
  }
  rows
}

fn directories(data: &Path, fix: bool) -> Vec<Row> {
  if !data.is_dir() {
    return vec![row(
      Status::Fail,
      format!(
        "Data directory not found: {} (set PPLR_DATA to the People tree)",
        data.display()
      ),
    )];
  }
  let mut rows = vec![row(
    Status::Pass,
    format!("Data directory exists: {}", data.display()),
  )];
  let missing: Vec<String> = ('A'..='Z')
    .map(String::from)
    .filter(|l| !data.join(l).is_dir())
    .collect();
  if missing.is_empty() {
    rows.push(row(Status::Pass, "All A-Z directories present"));
  } else {
    let list = missing.join(" ");
    if fix
      && missing
        .iter()
        .all(|l| std::fs::create_dir_all(data.join(l)).is_ok())
    {
      rows.push(row(
        Status::Fixed,
        format!("Created missing directories: {list}"),
      ));
    } else {
      rows.push(row(
        Status::Warn,
        format!("Missing directories: {list} (use --fix to create)"),
      ));
    }
  }
  let index = data.join(".index");
  if index.is_dir() {
    rows.push(row(Status::Pass, ".index directory exists"));
  } else if fix && std::fs::create_dir_all(&index).is_ok() {
    rows.push(row(Status::Fixed, "Created .index directory"));
  } else {
    rows.push(row(
      Status::Fail,
      ".index directory missing (use --fix to create)",
    ));
  }
  rows
}

fn tools() -> Vec<Row> {
  let mut rows = Vec::new();
  let mut tool = |name: &str, needed_for: &str| match on_path(name) {
    Some(path) => rows.push(row(
      Status::Pass,
      format!("{name} found: {}", path.display()),
    )),
    None => rows.push(row(
      Status::Warn,
      format!("{name} not found ({needed_for})"),
    )),
  };
  tool("jq", "the commands not yet ported to Rust need it");
  tool("claude", "pplr search needs it");
  tool(
    "mail-recent",
    "pplr email needs it until ST0001 WP14; or set PPLR_MAIL_RECENT",
  );
  if cfg!(target_os = "macos") {
    tool(
      "fileicon",
      "applyicons and setpicfordir need it: brew install fileicon",
    );
    tool("osascript", "the live Contacts writes need it");
  }
  rows.push(sidecar());
  rows
}

fn sidecar() -> Row {
  if !cfg!(target_os = "macos") {
    return row(
      Status::Pass,
      "Contacts sidecar not needed here (live Contacts is macOS only)",
    );
  }
  match Layout::running() {
    Ok(layout) if layout.sidecar().is_file() => row(
      Status::Pass,
      format!("Contacts sidecar found: {}", layout.sidecar().display()),
    ),
    Ok(layout) => row(
      Status::Warn,
      format!(
        "Contacts sidecar missing: {} (pplr sync needs it: bin/devbin build)",
        layout.sidecar().display()
      ),
    ),
    Err(e) => row(Status::Warn, format!("Contacts sidecar not checked: {e}")),
  }
}

fn index_files(data: &Path, fix: bool) -> Vec<Row> {
  let index = data.join(".index");
  let mut rows = Vec::new();
  let mut reindexed = false;
  let mut reindex = || {
    if !reindexed {
      reindexed = true;
      let _ = std::env::current_exe().map(|pplr| Command::new(pplr).arg("reindex").output());
    }
  };

  let json = index.join("index.json");
  match read_json(&json) {
    Some(Ok(value)) => rows.push(row(
      Status::Pass,
      format!(
        "index.json is valid (contains {} people)",
        people_in_index(&value)
      ),
    )),
    Some(Err(())) => rows.push(row(
      Status::Fail,
      "index.json is invalid JSON (run 'pplr reindex' to rebuild it)",
    )),
    None if fix => {
      reindex();
      rows.push(row(Status::Fixed, "Generated index.json"));
    }
    None => rows.push(row(
      Status::Warn,
      "index.json not found (run 'pplr reindex' to generate)",
    )),
  }

  if index.join("index.md").is_file() {
    rows.push(row(Status::Pass, "index.md exists"));
  } else if fix {
    reindex();
    rows.push(row(Status::Fixed, "Generated index.md"));
  } else {
    rows.push(row(
      Status::Warn,
      "index.md not found (run 'pplr reindex' to generate)",
    ));
  }

  match read_json(&index.join("tags_index.json")) {
    Some(Ok(value)) => {
      let count = value
        .get("people")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
      rows.push(row(
        Status::Pass,
        format!("tags_index.json is valid ({count} people)"),
      ))
    }
    Some(Err(())) => rows.push(row(
      Status::Fail,
      "tags_index.json is invalid JSON (run 'pplr reindex' to rebuild it)",
    )),
    None => rows.push(row(
      Status::Warn,
      "tags_index.json not found (run pplr reindex; search needs it)",
    )),
  }
  rows
}

/// None when the file is absent; Err when it is there and is not JSON.
fn read_json(path: &Path) -> Option<std::result::Result<Value, ()>> {
  let text = std::fs::read_to_string(path).ok()?;
  Some(serde_json::from_str(&text).map_err(|_| ()))
}

/// index.json's `people` maps each letter to its people.
fn people_in_index(index: &Value) -> usize {
  index
    .get("people")
    .and_then(Value::as_object)
    .map_or(0, |letters| {
      letters
        .values()
        .filter_map(Value::as_array)
        .map(Vec::len)
        .sum()
    })
}

fn common_issues(data: &Path, fix: bool) -> Result<Vec<Row>> {
  let mut rows = Vec::new();
  let people = letter_dirs(data, false);

  let old: Vec<PathBuf> = letter_dirs(data, true)
    .iter()
    .map(|p| p.join("Index"))
    .filter(|i| i.is_dir())
    .collect();
  if old.is_empty() {
    rows.push(row(Status::Pass, "No old Index/ directories found"));
  } else if fix {
    for dir in &old {
      std::fs::remove_dir_all(dir).map_err(|e| format!("cannot remove {}: {e}", dir.display()))?;
    }
    rows.push(row(
      Status::Fixed,
      format!("Removed {} old Index/ directories", old.len()),
    ));
  } else {
    rows.push(row(
      Status::Warn,
      format!(
        "Found {} old Index/ directories (use --fix to remove)",
        old.len()
      ),
    ));
  }

  let without_about = people.iter().filter(|p| !about_file(p).is_file()).count();
  if without_about == 0 {
    rows.push(row(Status::Pass, "All people have About files"));
  } else {
    rows.push(row(
      Status::Warn,
      format!("{without_about} of {} people missing About files (each needs About/<First Surname> (About).md)", people.len()),
    ));
  }
  Ok(rows)
}

/// Every `<A-Z>/<folder>`: everyone, plus the `_`-prefixed folders when asked.
fn letter_dirs(data: &Path, with_underscored: bool) -> Vec<PathBuf> {
  let mut people = Vec::new();
  for letter in 'A'..='Z' {
    let Ok(entries) = std::fs::read_dir(data.join(letter.to_string())) else {
      continue;
    };
    people.extend(
      entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
          p.is_dir()
            && (with_underscored
              || !p
                .file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with('_')))
        }),
    );
  }
  people.sort();
  people
}

/// `About/<First Surname> (About).md` for a "Surname, First" folder.
fn about_file(person: &Path) -> PathBuf {
  let name = person
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_default();
  let (surname, first) = name
    .split_once(',')
    .map_or((name.as_str(), ""), |(s, f)| (s.trim(), f.trim()));
  person
    .join("About")
    .join(format!("{first} {surname} (About).md"))
}

/// The report: a heading, each section's rows, and the summary.
pub fn render(sections: &[Section], fix: bool, colour: bool) -> String {
  let paint = |code: &str, text: &str| {
    if colour {
      format!("\x1b[{code}m{text}\x1b[0m")
    } else {
      text.to_owned()
    }
  };
  let mut out =
    String::from("pplr doctor - System Health Check\n=================================\n\n");
  for section in sections {
    out.push_str(&format!("{}:\n", section.title));
    for r in &section.rows {
      let line = match r.status {
        Status::Pass => format!("{} {}", paint("0;32", "✓"), r.text),
        Status::Fixed => format!("{} {} (fixed)", paint("0;32", "✓"), r.text),
        Status::Warn => format!("{} {}", paint("1;33", "⚠"), r.text),
        Status::Fail => format!("{} {}", paint("0;31", "✗"), r.text),
      };
      out.push_str(&line);
      out.push('\n');
    }
    out.push('\n');
  }
  let count = |status| {
    sections
      .iter()
      .flat_map(|s| &s.rows)
      .filter(|r| r.status == status)
      .count()
  };
  let (errors, warnings, fixed) = (
    count(Status::Fail),
    count(Status::Warn),
    count(Status::Fixed),
  );
  out.push_str("Summary:\n========\n");
  if errors == 0 && warnings == 0 {
    out.push_str("All checks passed! Your pplr installation is healthy.\n");
  } else {
    out.push_str(&format!("Found {errors} errors and {warnings} warnings\n"));
    if fix {
      out.push_str(&format!("Fixed {fixed} issues\n"));
    } else {
      out.push_str("\nRun 'pplr doctor --fix' to attempt automatic fixes\n");
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_shipped_binary_has_an_identifier_and_a_requirement_to_sign_against() {
    for path in crate::install::runtime_set() {
      let binary = path.trim_start_matches("bin/");
      let id = signing_fact(&format!("id.{binary}"));
      assert!(id.is_some(), "{binary}");
      let requirement = signing_fact(&format!("requirement.{binary}")).unwrap_or_default();
      assert!(
        requirement.starts_with(&format!("identifier \"{}\"", id.unwrap_or_default())),
        "{requirement}"
      );
    }
  }

  #[test]
  fn the_report_names_each_row_and_counts_the_summary() {
    let sections = [Section {
      title: "Index Files",
      rows: vec![
        row(Status::Pass, "index.md exists"),
        row(Status::Fail, "index.json is invalid JSON"),
        row(Status::Fixed, "Created .index directory"),
      ],
    }];
    let report = render(&sections, true, false);
    assert!(report.contains("Index Files:\n✓ index.md exists\n✗ index.json is invalid JSON\n✓ Created .index directory (fixed)\n"), "{report}");
    let count = |status| {
      sections[0]
        .rows
        .iter()
        .filter(|r| r.status == status)
        .count()
    };
    let summary = format!(
      "Found {} errors and {} warnings\nFixed {} issues\n",
      count(Status::Fail),
      count(Status::Warn),
      count(Status::Fixed)
    );
    assert!(report.ends_with(&summary), "{report}");
  }

  #[test]
  fn a_clean_report_says_so_and_colour_is_only_for_a_terminal() {
    let sections = [Section {
      title: "Dependencies",
      rows: vec![row(Status::Pass, "jq found: /bin/jq")],
    }];
    assert!(
      render(&sections, false, false)
        .ends_with("All checks passed! Your pplr installation is healthy.\n")
    );
    assert!(render(&sections, false, true).contains("\x1b[0;32m✓\x1b[0m jq found"));
    assert!(!render(&sections, false, false).contains('\x1b'));
  }

  #[test]
  fn index_json_counts_the_people_under_every_letter() -> Result<()> {
    let letters = [("A", 2), ("K", 1), ("Z", 0)];
    let people: serde_json::Map<String, Value> = letters
      .iter()
      .map(|(l, n)| (l.to_string(), Value::Array(vec![Value::Null; *n])))
      .collect();
    let index = serde_json::json!({ "people": people });
    assert_eq!(
      people_in_index(&index),
      letters.iter().map(|(_, n)| n).sum::<usize>()
    );
    Ok(())
  }

  #[test]
  fn a_persons_about_file_is_named_first_then_surname() {
    assert_eq!(
      about_file(Path::new("/d/V/Van der Berg, Ann")),
      PathBuf::from("/d/V/Van der Berg, Ann/About/Ann Van der Berg (About).md")
    );
  }
}
