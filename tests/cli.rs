//! The binary, end to end, in a sandbox per test. This is the one integration
//! file (devbin's `check autotests`); the golden replay (ST0001 WP03) extends
//! `Sandbox` rather than adding another.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A private directory with its own HOME and People tree, removed on drop.
/// Every run gets a clean environment: PATH, HOME, LANG and PPLR_DATA only.
struct Sandbox {
  root: PathBuf,
}

impl Sandbox {
  fn new() -> Result<Self> {
    let root = std::env::temp_dir().join(format!(
      "pplr-test-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("home"))?;
    fs::create_dir_all(root.join("data"))?;
    let root = root.canonicalize()?;
    Ok(Self { root })
  }

  fn path(&self, relative: &str) -> PathBuf {
    self.root.join(relative)
  }

  fn data(&self) -> PathBuf {
    self.path("data")
  }

  /// `pplr` with the sandbox's environment, run from the sandbox root.
  fn pplr(&self) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pplr"));
    command
      .env_clear()
      .env("PATH", std::env::var_os("PATH").unwrap_or_default())
      .env("HOME", self.path("home"))
      .env("LANG", "en_GB.UTF-8")
      .env("PPLR_DATA", self.data())
      .current_dir(&self.root);
    command
  }

  /// A stand-in for a legacy `bin/pplr_<name>` script, in the sandbox's own bin.
  fn legacy_script(&self, name: &str, body: &str) -> Result<PathBuf> {
    let bin = self.path("legacy-bin");
    fs::create_dir_all(&bin)?;
    let script = bin.join(format!("pplr_{name}"));
    fs::write(&script, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    Ok(bin)
  }
}

impl Drop for Sandbox {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.root);
  }
}

fn text(bytes: &[u8]) -> String {
  String::from_utf8_lossy(bytes).into_owned()
}

fn run(command: &mut Command) -> Result<Output> {
  Ok(command.output()?)
}

#[test]
fn no_command_prints_the_legacy_usage_and_fails() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(&mut sandbox.pplr())?;
  assert_eq!(out.status.code(), Some(1));
  assert_eq!(text(&out.stdout), "Usage: pplr <cmd>\n");
  Ok(())
}

#[test]
fn an_unknown_command_is_named_and_fails() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(sandbox.pplr().arg("nosuchcommand"))?;
  assert_eq!(out.status.code(), Some(1));
  assert_eq!(
    text(&out.stdout),
    "Error: Command 'nosuchcommand' not found.\nUsage: pplr <cmd>\n"
  );
  Ok(())
}

#[test]
fn a_missing_data_directory_is_named_and_fails() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let missing = sandbox.path("no-such-tree");
  let out = run(sandbox.pplr().env("PPLR_DATA", &missing).arg("count"))?;
  assert_eq!(out.status.code(), Some(1));
  assert_eq!(
    text(&out.stdout),
    format!(
      "Error: Could not change to directory {}.\n",
      missing.display()
    )
  );
  Ok(())
}

#[test]
fn the_fallback_runs_the_real_legacy_script() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(sandbox.pplr().arg("count"))?;
  assert!(out.status.success(), "{}", text(&out.stderr));
  assert_eq!(text(&out.stdout).trim(), "0");
  Ok(())
}

#[test]
fn the_fallback_gives_a_script_bin_pplrs_environment_and_its_exit_status() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let bin = sandbox.legacy_script(
    "probe",
    r#"echo "pwd=$(pwd -P)"; echo "data=$PPLR_DATA"; echo "dir=$PPLR_DIR"; echo "caller=$PPLR_CALLER_DIR"; echo "bin=$PPLR_BIN_DIR"; for a in "$@"; do echo "arg=$a"; done; exit 7"#,
  )?;
  let caller = sandbox.path("home");
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_BIN_DIR", &bin)
      .current_dir(&caller)
      .args(["probe", "a b", "c"]),
  )?;
  assert_eq!(out.status.code(), Some(7));
  let data = sandbox.data();
  let expected = [
    format!("pwd={}", data.display()),
    format!("data={}", data.display()),
    format!("dir={}", data.display()),
    format!("caller={}", caller.display()),
    format!("bin={}", bin.display()),
    "arg=a b".to_owned(),
    "arg=c".to_owned(),
  ];
  assert_eq!(text(&out.stdout).lines().collect::<Vec<_>>(), expected);
  Ok(())
}

#[test]
fn a_person_marker_becomes_their_folder_except_for_commands_that_take_urls() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let about = sandbox.data().join("K/Kemp, Jon/About/Jon Kemp (About).md");
  sandbox.legacy_script("resolve", &format!("echo '{}'", about.display()))?;
  sandbox.legacy_script("links", r#"for a in "$@"; do echo "$a"; done"#)?;
  let bin = sandbox.legacy_script("tag", r#"for a in "$@"; do echo "$a"; done"#)?;

  let tag = run(sandbox.pplr().env("PPLR_BIN_DIR", &bin).args([
    "tag",
    "pplr://k/kemp-jon",
    "pplr://k/kemp-jon/About",
  ]))?;
  assert_eq!(text(&tag.stdout), "Kemp, Jon\npplr://k/kemp-jon/About\n");

  let links = run(
    sandbox
      .pplr()
      .env("PPLR_BIN_DIR", &bin)
      .args(["links", "pplr://k/kemp-jon"]),
  )?;
  assert_eq!(text(&links.stdout), "pplr://k/kemp-jon\n");
  Ok(())
}

#[test]
fn a_marker_that_does_not_resolve_stops_the_command() -> Result<()> {
  let sandbox = Sandbox::new()?;
  sandbox.legacy_script("resolve", "echo 'no one is pplr://x/nobody' >&2; exit 1")?;
  let bin = sandbox.legacy_script("tag", "echo ran")?;
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_BIN_DIR", &bin)
      .args(["tag", "pplr://x/nobody"]),
  )?;
  assert_eq!(out.status.code(), Some(1));
  assert_eq!(text(&out.stdout), "");
  assert_eq!(text(&out.stderr), "no one is pplr://x/nobody\n");
  Ok(())
}

#[test]
fn version_flags_run_the_legacy_version_script() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let bin = sandbox.legacy_script("version", "echo v-from-legacy")?;
  for flag in ["-v", "--version"] {
    let out = run(sandbox.pplr().env("PPLR_BIN_DIR", &bin).arg(flag))?;
    assert!(out.status.success());
    assert_eq!(text(&out.stdout), "v-from-legacy\n");
  }
  Ok(())
}
