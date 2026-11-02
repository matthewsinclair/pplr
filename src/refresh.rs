//! `pplr refresh`: when each person was last checked against their public
//! profile, one line in `<person>/.index/refreshed`: "2026-09-25 linkedin".

use crate::Result;
use crate::config::Config;
use crate::tree::folder_of_name;
use jiff::ToSpan;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_NEXT: usize = 20;
const DEFAULT_SOURCE: &str = "linkedin";
const RECENT_DAYS: i64 = 90;
const NEVER: &str = "0000-00-00";
const USAGE: &str =
  "Usage: pplr refresh next [N] [--older-than DAYS] | stamp \"Surname, First\" [SOURCE] | status";

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let arg = |i: usize| args.get(i).map(String::as_str);
  match arg(0) {
    Some("next") => {
      let n = arg(1)
        .filter(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|n| n.parse().ok())
        .unwrap_or(DEFAULT_NEXT);
      // `days="${4:-$3}"`; days the date tool cannot read leave no cutoff
      let cutoff = if arg(2) == Some("--older-than") || arg(1) == Some("--older-than") {
        let days = arg(3)
          .filter(|d| !d.is_empty())
          .or(arg(2))
          .unwrap_or_default();
        days_before(config, days)
      } else {
        None
      };
      for key in next(&config.data, n, cutoff.as_deref()) {
        println!("{key}");
      }
      Ok(ExitCode::SUCCESS)
    }
    Some("stamp") => {
      let Some(name) = arg(1).filter(|n| !n.is_empty()) else {
        println!("Usage: pplr refresh stamp \"Surname, First\" [SOURCE]");
        return Ok(ExitCode::FAILURE);
      };
      let (name, dir) = folder_of_name(&config.data, name);
      if !dir.is_dir() {
        println!("Error: no such person: {name}");
        return Ok(ExitCode::FAILURE);
      }
      let day = config.today().to_string();
      let source = arg(2).unwrap_or(DEFAULT_SOURCE);
      fs::create_dir_all(dir.join(".index"))?;
      fs::write(dir.join(".index/refreshed"), format!("{day} {source}\n"))?;
      println!(
        "Checked: {} ({day}, {source})",
        relative(&config.data, &dir)
      );
      Ok(ExitCode::SUCCESS)
    }
    Some("status") => {
      let cutoff = days_before(config, &RECENT_DAYS.to_string()).unwrap_or_default();
      let (mut total, mut never, mut recent) = (0, 0, 0);
      for dir in people(&config.data) {
        total += 1;
        match stamp_of(&dir) {
          None => never += 1,
          Some(s) if s.as_str() > cutoff.as_str() => recent += 1,
          Some(_) => {}
        }
      }
      println!(
        "pplr refresh: {total} people, {recent} checked in the last {RECENT_DAYS} days, {} older, {never} never",
        total - never - recent
      );
      Ok(ExitCode::SUCCESS)
    }
    other => {
      println!("{USAGE}");
      Ok(if matches!(other, Some("-h" | "--help")) {
        ExitCode::SUCCESS
      } else {
        ExitCode::FAILURE
      })
    }
  }
}

/// The day `days` days before today, as `date -v-<days>d` gives it.
fn days_before(config: &Config, days: &str) -> Option<String> {
  let n: i64 = days.parse().ok()?;
  Some(config.today().checked_sub(n.days()).ok()?.to_string())
}

/// The next `n` people to check: never checked first, then the oldest
/// stamps, each "L/Surname, First"; with a cutoff, only those at or before it.
fn next(root: &Path, n: usize, cutoff: Option<&str>) -> Vec<String> {
  let mut rows: Vec<(String, String)> = people(root)
    .into_iter()
    .map(|dir| {
      (
        stamp_of(&dir).unwrap_or_else(|| NEVER.to_owned()),
        relative(root, &dir),
      )
    })
    .filter(|(stamp, _)| cutoff.is_none_or(|c| stamp.as_str() <= c))
    .collect();
  rows.sort_by(|a, b| format!("{} {}", a.0, a.1).cmp(&format!("{} {}", b.0, b.1)));
  rows.into_iter().take(n).map(|(_, key)| key).collect()
}

/// Every person folder two levels down, by path: `find -mindepth 2 -maxdepth 2
/// -type d -name '*, *'`, skipping anything under a `_` or `.` folder.
fn people(root: &Path) -> Vec<PathBuf> {
  let mut out = Vec::new();
  for top in crate::tree::names(root) {
    if top.starts_with(['_', '.']) || !root.join(&top).is_dir() {
      continue;
    }
    for name in crate::tree::names(&root.join(&top)) {
      let dir = root.join(&top).join(&name);
      let real_dir = fs::symlink_metadata(&dir).is_ok_and(|m| m.is_dir());
      if name.contains(", ") && !name.starts_with(['_', '.']) && real_dir {
        out.push(dir);
      }
    }
  }
  // `| sort`: by the path's bytes, not its components
  out.sort_by_key(|p| p.to_string_lossy().into_owned());
  out
}

/// The day on the first line of the person's refreshed file, if any.
fn stamp_of(dir: &Path) -> Option<String> {
  let text = fs::read_to_string(dir.join(".index/refreshed")).ok()?;
  let day = text.lines().next()?.split(' ').next()?.to_owned();
  (!day.is_empty()).then_some(day)
}

fn relative(root: &Path, dir: &Path) -> String {
  dir
    .strip_prefix(root)
    .unwrap_or(dir)
    .to_string_lossy()
    .into_owned()
}
