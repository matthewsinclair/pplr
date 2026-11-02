//! Dev-only scaffolding (ST0001 design D3 and 3.3): a command not yet ported
//! runs its legacy `bin/pplr_<cmd>` from the checkout this binary was built in,
//! exactly as `bin/pplr` runs it. ST0001 WP13 deletes this module.

use crate::Result;
use crate::cli::data_missing;
use crate::config::Config;
use crate::marker::rewrite_args;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, ExitStatus};

/// The checkout this binary was built from, which holds the legacy scripts.
const CHECKOUT: &str = env!("CARGO_MANIFEST_DIR");

const USAGE: &str = "Usage: pplr <cmd>";
const SCRIPT_PREFIX: &str = "pplr_";
const USAGE_PREFIX: &str = "# USAGE: ";

/// Run `args` (a command and its arguments) through the legacy scripts.
pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  run_with(args, config, false)
}

/// `run`, with the script's stdout discarded (a legacy `>/dev/null`).
pub fn run_quietly(args: &[String], config: &Config) -> Result<ExitCode> {
  run_with(args, config, true)
}

fn run_with(args: &[String], config: &Config, quiet: bool) -> Result<ExitCode> {
  let env = LegacyEnv::new(config)?;
  let Some((name, rest)) = args.split_first() else {
    println!("{USAGE}");
    return Ok(ExitCode::FAILURE);
  };
  let script = env.script(name);
  if !is_executable(&script) {
    println!("Error: Command '{name}' not found.\n{USAGE}");
    return Ok(ExitCode::FAILURE);
  }
  if let Some(code) = data_missing(config) {
    return Ok(code);
  }
  let pass = rewrite_args(name, rest, config)?;
  let mut command = env.command(&script);
  if quiet {
    command.stdout(std::process::Stdio::null());
  }
  status(command.args(&pass).status())
}

/// Each legacy script still in the checkout, with its `# USAGE:` lines, for `help`.
pub fn usages() -> Vec<(String, Vec<String>)> {
  let Ok(env) = LegacyEnv::new_without_config() else {
    return Vec::new();
  };
  crate::tree::names(&env.bin_dir)
    .into_iter()
    .filter_map(|file| {
      let name = file.strip_prefix(SCRIPT_PREFIX)?.to_owned();
      let text = std::fs::read_to_string(env.bin_dir.join(&file)).ok()?;
      let lines = text
        .lines()
        .filter_map(|l| l.strip_prefix(USAGE_PREFIX))
        .map(str::to_owned)
        .collect();
      Some((name, lines))
    })
    .collect()
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
    let mut env = Self::new_without_config()?;
    env.data = config.data.clone();
    env.data_legacy = config
      .data_legacy
      .clone()
      .unwrap_or_else(|| config.data.clone());
    Ok(env)
  }

  fn new_without_config() -> Result<Self> {
    // `bin/pplr` keeps an inherited PPLR_ROOT and PPLR_BIN_DIR (`:=`), and so does this
    let set = |name: &str| {
      std::env::var_os(name)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
    };
    let root = set("PPLR_ROOT").unwrap_or_else(|| PathBuf::from(CHECKOUT));
    let bin_dir = set("PPLR_BIN_DIR").unwrap_or_else(|| root.join("bin"));
    let caller =
      std::env::current_dir().map_err(|e| format!("cannot read the current directory: {e}"))?;
    Ok(Self {
      root,
      bin_dir,
      data: PathBuf::new(),
      data_legacy: PathBuf::new(),
      caller,
    })
  }

  fn script(&self, name: &str) -> PathBuf {
    self.bin_dir.join(format!("{SCRIPT_PREFIX}{name}"))
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
