//! Dev-only scaffolding (ST0001 design D3 and 3.3): a command not yet ported
//! runs its legacy `bin/pplr_<cmd>` from the checkout this binary was built in,
//! exactly as `bin/pplr` runs it. ST0001 WP13 deletes this module.

use crate::Result;
use crate::config::Config;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

/// The checkout this binary was built from, which holds the legacy scripts.
const CHECKOUT: &str = env!("CARGO_MANIFEST_DIR");

/// The commands that take a `pplr://` URL as it is; every other command gets
/// a person's marker rewritten to their folder name ("Surname, First").
const TAKES_URLS: &[&str] = &["open", "resolve", "links", "handler"];

const USAGE: &str = "Usage: pplr <cmd>";

/// Run `args` (a command and its arguments) through the legacy scripts.
pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let env = LegacyEnv::new(config)?;

  if matches!(args.first().map(String::as_str), Some("-v" | "--version")) {
    return status(env.command(&env.script("version")).status());
  }
  let Some((name, rest)) = args.split_first() else {
    println!("{USAGE}");
    return Ok(ExitCode::FAILURE);
  };
  let script = env.script(name);
  if !is_executable(&script) {
    println!("Error: Command '{name}' not found.\n{USAGE}");
    return Ok(ExitCode::FAILURE);
  }
  if !env.data.is_dir() {
    println!(
      "Error: Could not change to directory {}.",
      env.data.display()
    );
    return Ok(ExitCode::FAILURE);
  }

  let mut pass = Vec::with_capacity(rest.len());
  for arg in rest {
    if TAKES_URLS.contains(&name.as_str()) || !is_person_marker(arg) {
      pass.push(arg.clone());
      continue;
    }
    let resolved = env.command(&env.script("resolve")).arg(arg).output()?;
    if !resolved.status.success() {
      eprint!("{}", String::from_utf8_lossy(&resolved.stderr));
      return Ok(ExitCode::FAILURE);
    }
    pass.push(folder_of_about(
      String::from_utf8_lossy(&resolved.stdout).trim_end_matches('\n'),
    ));
  }
  status(env.command(&script).args(&pass).status())
}

/// The environment `bin/pplr` gives a command script.
struct LegacyEnv {
  root: PathBuf,
  bin_dir: PathBuf,
  data: PathBuf,
  data_legacy: PathBuf,
  caller: PathBuf,
}

impl LegacyEnv {
  fn new(config: &Config) -> Result<Self> {
    // `bin/pplr` keeps an inherited PPLR_ROOT and PPLR_BIN_DIR (`:=`), and so does this
    let set = |name: &str| {
      std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
    };
    let root = set("PPLR_ROOT").unwrap_or_else(|| PathBuf::from(CHECKOUT));
    let bin_dir = set("PPLR_BIN_DIR").unwrap_or_else(|| root.join("bin"));
    let data = config.data.clone();
    let data_legacy = config.data_legacy.clone().unwrap_or_else(|| data.clone());
    let caller =
      std::env::current_dir().map_err(|e| format!("cannot read the current directory: {e}"))?;
    Ok(Self {
      root,
      bin_dir,
      data,
      data_legacy,
      caller,
    })
  }

  fn script(&self, name: &str) -> PathBuf {
    self.bin_dir.join(format!("pplr_{name}"))
  }

  /// A script run from the data root, as `bin/pplr` runs it.
  fn command(&self, script: &Path) -> Command {
    let mut command = Command::new(script);
    command
      .current_dir(&self.data)
      .env("PPLR_ROOT", &self.root)
      .env("PPLR_BIN_DIR", &self.bin_dir)
      .env("PPLR_DATA", &self.data)
      .env("PPLR_DIR", &self.data_legacy)
      .env("PPLR_CALLER_DIR", &self.caller);
    command
  }
}

/// `^pplr://[a-z]/[^/]+/?$`: a person's marker, with no path inside their folder.
fn is_person_marker(arg: &str) -> bool {
  let Some(rest) = arg.strip_prefix("pplr://") else {
    return false;
  };
  let mut chars = rest.chars();
  let letter = chars.next().is_some_and(|c| c.is_ascii_lowercase());
  let key = chars
    .as_str()
    .strip_prefix('/')
    .map(|k| k.strip_suffix('/').unwrap_or(k));
  letter && key.is_some_and(|k| !k.is_empty() && !k.contains('/'))
}

/// `basename "$(dirname "$(dirname "$about")")"`: the person's folder name.
fn folder_of_about(about: &str) -> String {
  Path::new(about)
    .parent()
    .and_then(Path::parent)
    .and_then(Path::file_name)
    .map(|name| name.to_string_lossy().into_owned())
    .unwrap_or_default()
}

fn is_executable(path: &Path) -> bool {
  use std::os::unix::fs::PermissionsExt;
  path
    .metadata()
    .is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

/// The script's exit status, as bash reports it: 128 plus the signal for a kill.
fn status(result: std::io::Result<ExitStatus>) -> Result<ExitCode> {
  use std::os::unix::process::ExitStatusExt;
  let status = result.map_err(|e| format!("cannot run the legacy script: {e}"))?;
  let code = status
    .code()
    .or_else(|| status.signal().map(|s| 128 + s))
    .unwrap_or(1);
  Ok(ExitCode::from((code & 0xff) as u8))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_person_marker_is_a_letter_and_a_key_with_no_path() {
    for marker in ["pplr://k/kemp-jon", "pplr://k/kemp-jon/"] {
      assert!(is_person_marker(marker), "{marker}");
    }
    for other in [
      "pplr://k/kemp-jon/About/x.md",
      "pplr://K/kemp-jon",
      "pplr://k/",
      "pplr://kk/x",
      "Kemp, Jon",
      "http://k/x",
    ] {
      assert!(!is_person_marker(other), "{other}");
    }
  }

  #[test]
  fn the_folder_is_two_levels_above_the_about_file() {
    assert_eq!(
      folder_of_about("/d/K/Kemp, Jon/About/Jon Kemp (About).md"),
      "Kemp, Jon"
    );
    assert_eq!(folder_of_about("x"), "");
  }
}
