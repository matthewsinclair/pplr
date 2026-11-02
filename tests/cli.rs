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
  // sync stays on the fallback until WP10; with no mode it prints its usage
  assert!(pplr::cli::native("sync").is_none());
  let out = run(sandbox.pplr().arg("sync"))?;
  assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
  assert!(text(&out.stdout).starts_with("Usage: pplr sync --check"));
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
  let about = sandbox.data().join("K/Kemp, Jon/About");
  fs::create_dir_all(&about)?;
  fs::write(about.join("Jon Kemp (About).md"), "# Jon Kemp (About)\n")?;
  sandbox.legacy_script("links", r#"for a in "$@"; do echo "$a"; done"#)?;
  let bin = sandbox.legacy_script("rename", r#"for a in "$@"; do echo "$a"; done"#)?;

  let tag = run(sandbox.pplr().env("PPLR_BIN_DIR", &bin).args([
    "rename",
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
  let bin = sandbox.legacy_script("rename", "echo ran")?;
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_BIN_DIR", &bin)
      .args(["rename", "pplr://x/nobody"]),
  )?;
  assert_eq!(out.status.code(), Some(1));
  assert_eq!(text(&out.stdout), "");
  assert_eq!(
    text(&out.stderr),
    "pplr: no one answers to pplr://x/nobody\n"
  );
  Ok(())
}

// AT-03.2 (ST0002 AC-03.2): the version line, from the stamps taken at build time.
#[test]
fn version_and_its_flags_print_the_version_line() -> Result<()> {
  use pplr::install::{Kind, SOURCE_COMMIT, SOURCE_DIRTY, line};
  // The dev build names the version in Cargo.toml and the commit it was built from (T4)
  let expected = line(
    env!("CARGO_PKG_VERSION"),
    Kind::Dev,
    None,
    SOURCE_COMMIT,
    SOURCE_DIRTY,
  );
  let sandbox = Sandbox::new()?;
  for args in [&["version"][..], &["-v"], &["--version"]] {
    let out = run(sandbox.pplr().args(args))?;
    assert!(out.status.success());
    assert_eq!(text(&out.stdout), format!("{expected}\n"));
  }
  Ok(())
}

#[test]
fn help_lists_native_and_fallback_commands_and_details_is_the_readme() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(sandbox.pplr().arg("help"))?;
  assert!(out.status.success());
  let listing = text(&out.stdout);
  for command in ["count", "meetings", "sync", "tags"] {
    assert!(
      listing
        .lines()
        .any(|l| l.starts_with(&format!("{command} "))),
      "{command} is not listed"
    );
  }
  let details = run(sandbox.pplr().args(["help", "--details"]))?;
  assert_eq!(
    text(&details.stdout),
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))?
  );
  Ok(())
}

// The golden replay (ST0001 WP03, AC-03.1). Every case in tests/golden/cases
// was captured from the legacy code; each one whose steps all name a native
// command runs again here against this binary, in a sandbox seeded the same
// way, and must match: stdout, stderr, exit code and the resulting tree, after
// the rewrites tests/golden/README.md lists. A case with a step on the
// fallback is skipped, so each porting WP turns its own goldens on.

/// The capture's sandbox root, whose length the replay must keep: rename cuts
/// its running-text lines at 200 columns, absolute path included.
const GOLDEN_ROOT: &str = "/private/tmp/pplr-golden.XXXXXXXX";
const GOLDEN_DAY: &str = "2026-06-15";
const GOLDEN_NOW: &str = "2026-06-15 09:30:00";
/// Inputs the capture does not record.
const GOLDEN_INPUTS: [&str; 3] = ["home", "mail", "inputs"];

fn golden_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// A sandbox root as long as the capture's, at a real path (no symlink in it).
fn golden_root() -> Result<PathBuf> {
  let unique = format!(
    "{:08x}",
    (std::process::id() as usize).wrapping_mul(1 << 12) ^ NEXT.fetch_add(1, Ordering::Relaxed)
  );
  let unique = &unique[unique.len() - 8..];
  let mut root = format!("/private/tmp/pplr-golden.{unique}");
  if !fs::symlink_metadata("/private/tmp").is_ok_and(|m| m.is_dir()) {
    let stem = "/tmp/pplr-golden";
    let pad = GOLDEN_ROOT.len() - stem.len() - 1 - unique.len();
    root = format!("{stem}{}.{unique}", "-".repeat(pad));
  }
  if root.len() != GOLDEN_ROOT.len() {
    return Err(format!("the replay root {root} is not as long as {GOLDEN_ROOT}").into());
  }
  Ok(PathBuf::from(root))
}

/// Copies a tree, keeping symlinks as symlinks.
fn copy_tree(from: &std::path::Path, to: &std::path::Path) -> Result<()> {
  fs::create_dir_all(to)?;
  for entry in fs::read_dir(from)? {
    let entry = entry?;
    let (source, target) = (entry.path(), to.join(entry.file_name()));
    let kind = entry.file_type()?;
    if kind.is_symlink() {
      std::os::unix::fs::symlink(fs::read_link(&source)?, &target)?;
    } else if kind.is_dir() {
      copy_tree(&source, &target)?;
    } else {
      fs::copy(&source, &target)?;
    }
  }
  Ok(())
}

/// Every path under `root`, relative and bytewise sorted, skipping `skip` at the top.
fn walk(root: &std::path::Path, skip: &[&str]) -> Result<Vec<String>> {
  fn visit(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
      let path = entry?.path();
      let rel = path.strip_prefix(root)?.to_string_lossy().into_owned();
      out.push(rel);
      if fs::symlink_metadata(&path)?.is_dir() {
        visit(root, &path, out)?;
      }
    }
    Ok(())
  }
  let mut out = Vec::new();
  visit(root, root, &mut out)?;
  out.retain(|rel| {
    !skip
      .iter()
      .any(|top| rel == top || rel.starts_with(&format!("{top}/")))
  });
  out.sort();
  Ok(out)
}

/// Text, as the capture's `grep -I` judges it: no NUL byte.
fn is_text(bytes: &[u8]) -> bool {
  !bytes.contains(&0)
}

/// Replaces each `<digits:8>-<digits:6>` with @STAMP@.
fn mask_stamps(text: &str) -> String {
  let bytes = text.as_bytes();
  let digits = |range: std::ops::Range<usize>| {
    bytes
      .get(range)
      .is_some_and(|b| b.iter().all(u8::is_ascii_digit))
  };
  let (mut out, mut i) = (String::new(), 0);
  while i < bytes.len() {
    if digits(i..i + 8) && bytes.get(i + 8) == Some(&b'-') && digits(i + 9..i + 15) {
      out.push_str("@STAMP@");
      i += 15;
    } else {
      let ch = text[i..].chars().next().unwrap_or_default();
      out.push(ch);
      i += ch.len_utf8().max(1);
    }
  }
  out
}

/// Replaces the quoted value after `key` (with any spaces and a colon) by `token`.
fn mask_value(text: &str, key: &str, token: &str) -> String {
  let (mut out, mut rest) = (String::new(), text);
  while let Some(at) = rest.find(key) {
    let (head, tail) = rest.split_at(at + key.len());
    out.push_str(head);
    let after = tail.trim_start_matches(' ');
    let Some(after) = after.strip_prefix(':') else {
      rest = tail;
      continue;
    };
    let after = after.trim_start_matches(' ');
    let Some(value) = after.strip_prefix('"') else {
      rest = tail;
      continue;
    };
    let Some(end) = value.find('"') else {
      rest = tail;
      continue;
    };
    out.push_str(&tail[..tail.len() - after.len()]);
    out.push('"');
    out.push_str(token);
    out.push('"');
    rest = &value[end + 1..];
  }
  out.push_str(rest);
  out
}

// Test-only switches, for proving the replay itself (never read by pplr):
//   PPLR_GOLDEN_ALL=1           replay every case, fallback commands included
//   PPLR_GOLDEN_REAL_DAY=<day>  the real date the legacy tags.py reads, rewritten
//                               to the pinned day as the capture does
//   PPLR_GOLDEN_CASES=<dir>     replay the cases in <dir> (eg a mutated copy)

/// The README's rewrites. Today's date only under PPLR_GOLDEN_REAL_DAY: the
/// port reads PPLR_TODAY.
fn golden_normalise(text: &str, sandbox: &str) -> String {
  let mut text = text.to_owned();
  if let Ok(day) = std::env::var("PPLR_GOLDEN_REAL_DAY") {
    text = text.replace(&day, GOLDEN_DAY);
  }
  let bare = sandbox.strip_prefix("/private").unwrap_or(sandbox);
  for form in [
    sandbox.to_owned(),
    bare.to_owned(),
    sandbox.replace('/', "\\/"),
    bare.replace('/', "\\/"),
  ] {
    text = text.replace(&form, "@SANDBOX@");
  }
  text = mask_value(&text, "\"generated_at\"", "@GENERATED_AT@");
  text = mask_value(&text, "\"generated\"", "@GENERATED@");
  if let Some(at) = text.find("sync plan, ") {
    let start = at + "sync plan, ".len();
    let stamp = "YYYY-MM-DD HH:MM".len();
    if text
      .get(start..start + stamp)
      .is_some_and(|s| s.bytes().filter(u8::is_ascii_digit).count() == 12)
    {
      text.replace_range(start..start + stamp, "@GENERATED@");
    }
  }
  mask_stamps(&text)
}

/// The lines of a golden file. Split on newlines alone: a path such as
/// Finder's `Icon\r` ends in a carriage return that `str::lines` would drop.
fn lines_of(path: &std::path::Path) -> Result<Vec<String>> {
  let text = fs::read_to_string(path)?;
  let text = text.strip_suffix('\n').unwrap_or(&text);
  Ok(text.split('\n').map(str::to_owned).collect())
}

/// Replays one case; `Ok(false)` when a step is still on the fallback.
fn replay(case: &std::path::Path) -> Result<bool> {
  let steps_dir = case.join("steps");
  let mut steps: Vec<PathBuf> = fs::read_dir(&steps_dir)?
    .map(|e| e.map(|e| e.path()))
    .collect::<std::io::Result<_>>()?;
  steps.retain(|p| p.extension().is_some_and(|x| x == "args"));
  steps.sort();
  let all = std::env::var_os("PPLR_GOLDEN_ALL").is_some();
  let mut argv = Vec::new();
  for step in &steps {
    let args = lines_of(step)?;
    if !all
      && args
        .first()
        .is_none_or(|name| pplr::cli::native(name).is_none())
    {
      return Ok(false);
    }
    argv.push((step.with_extension(""), args));
  }

  let golden = golden_dir();
  let fixture = golden.join("fixture");
  let root = golden_root()?;
  let _cleanup = Sandbox { root: root.clone() };
  let sb = root.join("sandbox");
  let sandbox = sb.to_string_lossy().into_owned();
  copy_tree(&fixture, &sb)?;
  for rel in walk(&sb, &[])? {
    let path = sb.join(&rel);
    if fs::symlink_metadata(&path)?.is_file() {
      let bytes = fs::read(&path)?;
      if is_text(&bytes) && String::from_utf8_lossy(&bytes).contains("@SANDBOX@") {
        fs::write(
          &path,
          String::from_utf8_lossy(&bytes).replace("@SANDBOX@", &sandbox),
        )?;
      }
    }
  }
  for dir in ["home", "cache", "backups"] {
    fs::create_dir_all(sb.join(dir))?;
  }
  fs::write(
    sb.join("home/.gitconfig"),
    "[user]\n\tname = Matthew Sinclair\n\temail = golden@example.invalid\n",
  )?;

  let fakes = golden.join("fakes");
  let path = format!(
    "{}:{}",
    fakes.display(),
    std::env::var("PATH").unwrap_or_default()
  );
  let case_env = if case.join("env").exists() {
    lines_of(&case.join("env"))?
  } else {
    Vec::new()
  };
  // The capture's fake curl answered for the CMS; the port probes it itself
  let _cms = if case_env.iter().any(|l| l == "PPLR_GOLDEN_CMS=up") {
    Some(CmsStub::start()?)
  } else {
    None
  };

  for (stem, args) in &argv {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pplr"));
    command
      .env_clear()
      .env("HOME", sb.join("home"))
      .env("PATH", &path)
      .env("LANG", "en_GB.UTF-8")
      .env("TZ", "Europe/London")
      .env("PPLR_DATA", sb.join("Career/People"))
      .env("PPLR_DIR", sb.join("Career/People"))
      .env("PPLR_TODAY", GOLDEN_DAY)
      .env("PPLR_CONTACTS_JSON", sb.join("contacts.json"))
      .env("PPLR_BACKUP_DIR", sb.join("backups"))
      .env("PPLR_CACHE_DIR", sb.join("cache"))
      .env("PPLR_REFS_DIR", &sb)
      .env("PPLR_MAIL_RECENT", fakes.join("mail-recent"))
      .env(
        "PPLR_HANDLER_APP",
        sb.join("home/Applications/Pplr Links.app"),
      )
      .env("PPLR_GOLDEN_NOW", GOLDEN_NOW)
      .env("PPLR_GOLDEN_SANDBOX", &sb)
      .env("PPLR_GOLDEN_TRIPWIRE", root.join("tripwire.log"))
      .envs(
        // The legacy code's uv, when the fallback runs it under PPLR_GOLDEN_ALL
        ["UV_CACHE_DIR", "UV_PYTHON_INSTALL_DIR", "UV_OFFLINE"]
          .iter()
          .filter_map(|key| std::env::var_os(key).map(|value| (*key, value))),
      )
      .current_dir(&sb)
      .stdin(std::process::Stdio::null());
    for line in &case_env {
      let (key, value) = line
        .split_once('=')
        .ok_or_else(|| format!("a setting without '=': {line}"))?;
      command.env(key, value.replace("@SANDBOX@", &sandbox));
    }
    command.args(args.iter().map(|a| a.replace("@SANDBOX@", &sandbox)));
    let out = command.output()?;
    let step = stem
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    let expect = |ext: &str| fs::read_to_string(stem.with_extension(ext));
    same(
      out.status.code().map(|c| c.to_string()),
      Some(expect("exit")?.trim().to_owned()),
      &format!("step {step}: exit code"),
    )?;
    same(
      golden_normalise(&text(&out.stdout), &sandbox),
      expect("stdout")?,
      &format!("step {step}: stdout"),
    )?;
    same(
      golden_normalise(&text(&out.stderr), &sandbox),
      expect("stderr")?,
      &format!("step {step}: stderr"),
    )?;
  }
  if fs::metadata(root.join("tripwire.log")).is_ok_and(|m| m.len() > 0) {
    return Err(
      format!(
        "reached a tool no scenario should: {}",
        fs::read_to_string(root.join("tripwire.log"))?
      )
      .into(),
    );
  }

  // Stamped backup names, then the tree against the manifest
  for entry in fs::read_dir(sb.join("backups"))? {
    let path = entry?.path();
    let file = path
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    let masked = mask_stamps(&file);
    if masked != file {
      fs::rename(&path, sb.join("backups").join(masked))?;
    }
  }
  let mut expected = Vec::new();
  for line in lines_of(&case.join("manifest"))? {
    let mut parts = line.splitn(3, ' ');
    let (kind, value, rel) = (
      parts.next().unwrap_or_default(),
      parts.next().unwrap_or_default(),
      parts.next().unwrap_or_default(),
    );
    let actual = sb.join(rel);
    let meta = fs::symlink_metadata(&actual).map_err(|e| format!("{rel} is missing: {e}"))?;
    match kind {
      "d" => same(meta.is_dir(), true, &format!("{rel} is a directory"))?,
      "l" => same(
        fs::read_link(&actual)?.to_string_lossy().into_owned(),
        value.to_owned(),
        &format!("symlink {rel}"),
      )?,
      "f" => {
        let bytes = fs::read(&actual)?;
        let stored = case.join("files").join(rel);
        let want = if stored.exists() {
          fs::read(&stored)?
        } else {
          fs::read(fixture.join(rel))?
        };
        let got = if is_text(&bytes) && bytes != want {
          golden_normalise(&text(&bytes), &sandbox).into_bytes()
        } else {
          bytes
        };
        same(
          text(&got),
          text(&want),
          &format!("{rel} against its golden"),
        )?;
      }
      "x" => {
        // A workbook by its cells, read by the same script the capture uses
        let dump = Command::new(golden.join("xlsx-cells"))
          .arg(&actual)
          .output()?;
        same(
          dump.status.success(),
          true,
          &format!("xlsx-cells reads {rel}"),
        )?;
        let want = fs::read_to_string(case.join("files").join(format!("{rel}.cells")))?;
        same(
          golden_normalise(&text(&dump.stdout), &sandbox),
          want,
          &format!("{rel}'s cells against their golden"),
        )?;
      }
      other => return Err(format!("unknown manifest kind {other}").into()),
    }
    expected.push(rel.to_owned());
  }
  let actual = walk(&sb, &GOLDEN_INPUTS)?;
  let extra: Vec<_> = actual.iter().filter(|p| !expected.contains(p)).collect();
  let missing: Vec<_> = expected.iter().filter(|p| !actual.contains(p)).collect();
  same(
    (extra, missing),
    (Vec::new(), Vec::new()),
    "the tree's paths (extra, missing)",
  )?;
  Ok(true)
}

/// Where the CMS-up scenarios put the CMS (tests/golden/scenarios.zsh).
const GOLDEN_CMS: &str = "127.0.0.1:47360";

/// A CMS that answers every request with 200, until dropped.
struct CmsStub {
  stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CmsStub {
  fn start() -> Result<Self> {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind(GOLDEN_CMS)?;
    listener.set_nonblocking(true)?;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let running = stop.clone();
    std::thread::spawn(move || {
      while !running.load(Ordering::Relaxed) {
        match listener.accept() {
          Ok((mut stream, _)) => {
            let _ = stream.set_nonblocking(false);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(b"HTTP/1.0 200 OK\r\nContent-Length: 0\r\n\r\n");
          }
          Err(_) => std::thread::sleep(std::time::Duration::from_millis(5)),
        }
      }
    });
    Ok(Self { stop })
  }
}

impl Drop for CmsStub {
  fn drop(&mut self) {
    self.stop.store(true, Ordering::Relaxed);
  }
}

/// Ok when `got` is `want`; otherwise an error showing both.
fn same<T: PartialEq + std::fmt::Debug>(got: T, want: T, what: &str) -> Result<()> {
  if got == want {
    Ok(())
  } else {
    Err(format!("{what}\n    got:  {got:?}\n    want: {want:?}").into())
  }
}

/// AT-03.1 (ST0001 AC-03.1): every golden replays against the binary,
/// skipping only the commands still on the fallback.
#[test]
fn every_golden_of_a_ported_command_replays() -> Result<()> {
  let cases =
    std::env::var_os("PPLR_GOLDEN_CASES").map_or_else(|| golden_dir().join("cases"), PathBuf::from);
  let mut names: Vec<PathBuf> = fs::read_dir(&cases)?
    .map(|e| e.map(|e| e.path()))
    .collect::<std::io::Result<_>>()?;
  names.retain(|p| p.is_dir());
  names.sort();
  assert!(!names.is_empty(), "no goldens in {}", cases.display());
  let (mut ran, mut failed) = (Vec::new(), Vec::new());
  for case in &names {
    let name = case
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    match replay(case) {
      Ok(true) => ran.push(name),
      Ok(false) => {}
      Err(error) => failed.push(format!("{name}: {error}")),
    }
  }
  eprintln!(
    "golden replay: {} of {} case(s) ran: {}",
    ran.len(),
    names.len(),
    ran.join(", ")
  );
  if failed.is_empty() {
    Ok(())
  } else {
    Err(
      format!(
        "{} case(s) differ from their goldens:\n{}",
        failed.len(),
        failed.join("\n")
      )
      .into(),
    )
  }
}

#[test]
fn the_golden_rewrites_mask_what_varies_between_runs() {
  let sb = "/private/tmp/pplr-golden.ab12cd34/sandbox";
  let raw = format!(
    "{sb}/x and /tmp/pplr-golden.ab12cd34/sandbox/y and {}\n\"generated_at\": \"2026-11-02T10:00:00Z\"\n\"generated\" : \"2026-11-02 10:00\"\nsync plan, 2026-11-02 10:00\nbackups/20261102-100000-contacts.json\n",
    sb.replace('/', "\\/")
  );
  let want = "@SANDBOX@/x and @SANDBOX@/y and @SANDBOX@\n\"generated_at\": \"@GENERATED_AT@\"\n\"generated\" : \"@GENERATED@\"\nsync plan, @GENERATED@\nbackups/@STAMP@-contacts.json\n";
  assert_eq!(golden_normalise(&raw, sb), want);
}

/// ST0001 WP09, read-only and by hand on macOS: `cargo test -- --ignored live`.
/// The live store through the sidecar, and its dump through the Rust card
/// model and back, byte for byte. It prints a count, never a card (D11).
#[test]
#[ignore = "reads the live Contacts store; run by hand on macOS"]
fn live_contacts_round_trip_through_the_sidecar_store() -> pplr::Result<()> {
  use pplr::contacts::{Card, SidecarStore, Store, to_swift_json};
  let sidecar = std::path::Path::new(env!("CARGO_BIN_EXE_pplr")).with_file_name("pplr-contacts");
  let dump = Command::new(&sidecar).arg("dump").output()?;
  assert!(dump.status.success(), "the sidecar's dump failed");
  let text = text(&dump.stdout);
  let cards: Vec<Card> = serde_json::from_str(&text)?;
  assert_eq!(
    to_swift_json(&serde_json::to_value(&cards)?),
    text.trim_end_matches('\n')
  );
  let through_store = SidecarStore::new(sidecar)?.cards()?;
  assert_eq!(through_store.len(), cards.len());
  println!(
    "live Contacts: {} cards, read through the store and round-tripped",
    cards.len()
  );
  Ok(())
}

/// AT-04.1 (ST0001 AC-04.1): the twelve read commands run natively, and every
/// golden made of them alone replays; the bats suite (tools/bats-rust) covers
/// them against the binary too.
#[test]
fn the_read_commands_are_native_and_their_goldens_replay() -> Result<()> {
  const WP04: [&str; 12] = [
    "resolve", "count", "about", "cp", "open", "linkedin", "edit", "meetings", "refresh", "grep",
    "help", "version",
  ];
  for name in WP04 {
    assert!(pplr::cli::native(name).is_some(), "{name} is not native");
  }
  let cases = golden_dir().join("cases");
  let mut replayed = Vec::new();
  for case in fs::read_dir(&cases)? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut commands = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        commands.push(lines_of(&step)?.first().cloned().unwrap_or_default());
      }
    }
    if commands.iter().all(|c| WP04.contains(&c.as_str())) {
      same(replay(&case)?, true, &format!("{} replays", case.display()))?;
      replayed.push(case);
    }
  }
  assert!(!replayed.is_empty(), "no golden is made of WP04's commands");
  Ok(())
}

/// reindex's own files against its golden, while its tags step is still the
/// legacy script (stubbed here): index.json as `jq .` prints it, and index.md.
#[test]
fn reindex_writes_index_json_as_jq_prints_it_and_index_md() -> Result<()> {
  let sandbox = Sandbox::new()?;
  copy_tree(&golden_dir().join("fixture/Career/People"), &sandbox.data())?;
  let bin = sandbox.legacy_script("tags", "exit 0")?;
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_BIN_DIR", &bin)
      .env("PPLR_TODAY", GOLDEN_DAY)
      .arg("reindex"),
  )?;
  assert!(out.status.success(), "{}", text(&out.stderr));
  let golden = golden_dir().join("cases/reindex/files/Career/People/.index");
  for file in ["index.json", "index.md"] {
    assert_eq!(
      fs::read_to_string(sandbox.data().join(".index").join(file))?,
      fs::read_to_string(golden.join(file))?,
      "{file}"
    );
  }
  Ok(())
}

// ST0002 WP03: the dev/opt/brew trees, the manifest, `install` and `use`.

/// The running test binary's own pplr, laid out as a tree at `prefix`.
fn publish_tree(prefix: &std::path::Path, managed_by: &str, force: bool) -> Result<()> {
  let sources = [("bin/pplr", PathBuf::from(env!("CARGO_BIN_EXE_pplr")))];
  pplr::install::publish(
    &sources,
    prefix,
    "2.0.0",
    "0123456789abcdef0123456789abcdef01234567",
    managed_by,
    force,
  )
  .map_err(|e| e.to_string().into())
}

// AT-03.1 (ST0002 AC-03.1): install publishes from dev only, writes the T3
// manifest, and replaces only its own prefix, only with --force (this test and
// the two after it).
#[test]
fn publish_writes_the_runtime_set_and_its_manifest_and_names_the_tree_opt() -> Result<()> {
  use pplr::install::{Kind, MANIFEST, Manifest, drift, kind};
  let sandbox = Sandbox::new()?;
  let prefix = sandbox.path("opt");
  publish_tree(&prefix, "pplr", false)?;
  let manifest =
    Manifest::parse(&fs::read_to_string(prefix.join(MANIFEST))?).ok_or("no manifest")?;
  assert_eq!(
    (manifest.tool.as_str(), manifest.managed_by.as_str()),
    ("pplr", "pplr")
  );
  assert_eq!(
    manifest
      .files
      .iter()
      .map(|(_, p)| p.as_str())
      .collect::<Vec<_>>(),
    ["bin/pplr"]
  );
  assert_eq!(kind(&prefix).0, Kind::Opt);
  assert!(drift(&prefix, &manifest).is_empty());
  fs::write(prefix.join("bin/pplr"), "tampered")?;
  assert_eq!(drift(&prefix, &manifest), ["bin/pplr"]);
  Ok(())
}

#[test]
fn publish_replaces_only_its_own_prefix_and_only_with_force() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let prefix = sandbox.path("opt");
  publish_tree(&prefix, "pplr", false)?;
  let again = publish_tree(&prefix, "pplr", false)
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
  assert!(again.contains("--force"), "{again}");
  publish_tree(&prefix, "pplr", true)?;

  let foreign = sandbox.path("foreign");
  fs::create_dir_all(&foreign)?;
  fs::write(foreign.join("notes.txt"), "not pplr's")?;
  let refused = publish_tree(&foreign, "pplr", true)
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
  assert!(refused.contains("did not install"), "{refused}");

  let keg = sandbox.path("keg");
  publish_tree(&keg, "brew", false)?;
  let brews = publish_tree(&keg, "pplr", true)
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
  assert!(brews.contains("brew upgrade"), "{brews}");
  Ok(())
}

#[test]
fn install_needs_a_prefix_and_runs_only_from_dev() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(sandbox.pplr().arg("install"))?;
  assert!(!out.status.success());
  assert!(
    text(&out.stderr).contains("PPLR_OPT_PREFIX"),
    "{}",
    text(&out.stderr)
  );

  let opt = sandbox.path("opt");
  publish_tree(&opt, "pplr", false)?;
  let from_opt = run(
    Command::new(opt.join("bin/pplr"))
      .env_clear()
      .env("PATH", std::env::var_os("PATH").unwrap_or_default())
      .env("HOME", sandbox.path("home"))
      .args(["install", "--prefix"])
      .arg(sandbox.path("other")),
  )?;
  assert!(
    text(&from_opt.stderr).contains("from a dev checkout only"),
    "{}",
    text(&from_opt.stderr)
  );
  Ok(())
}

/// A fake Homebrew prefix holding pplr's keg, linked as brew links it.
fn fake_brew(sandbox: &Sandbox) -> Result<PathBuf> {
  let brew = sandbox.path("brew");
  let keg = brew.join("Cellar/pplr/2.0.0");
  publish_tree(&keg.join("libexec"), "brew", false)?;
  fs::create_dir_all(brew.join("opt"))?;
  fs::create_dir_all(brew.join("bin"))?;
  std::os::unix::fs::symlink(&keg, brew.join("opt/pplr"))?;
  std::os::unix::fs::symlink(
    brew.join("opt/pplr/libexec/bin/pplr"),
    brew.join("bin/pplr"),
  )?;
  Ok(brew)
}

// AT-03.3 (ST0002 AC-03.3): use moves the one link, refuses a brew copy that
// would not answer, and the shadow check fails a shadowed link (this test and
// the next).
#[test]
fn use_points_the_one_link_and_use_brew_hands_back_to_homebrew() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let opt = sandbox.path("opt");
  publish_tree(&opt, "pplr", false)?;
  let brew = fake_brew(&sandbox)?;
  let links = sandbox.path("home/.local/bin");
  let path = format!(
    "{}:{}:/usr/bin:/bin",
    links.display(),
    brew.join("bin").display()
  );
  let pplr = || {
    let mut c = sandbox.pplr();
    c.env("PATH", &path)
      .env("PPLR_OPT_PREFIX", &opt)
      .env("HOMEBREW_PREFIX", &brew);
    c
  };

  let to_opt = run(pplr().args(["use", "opt"]))?;
  assert!(to_opt.status.success(), "{}", text(&to_opt.stderr));
  assert!(
    text(&to_opt.stdout).starts_with("pplr 2.0.0 (opt, 0123456789ab)"),
    "{}",
    text(&to_opt.stdout)
  );
  assert_eq!(fs::read_link(links.join("pplr"))?, opt.join("bin/pplr"));
  let report = run(pplr().arg("use"))?;
  assert!(
    text(&report.stdout).starts_with("pplr -> opt"),
    "{}",
    text(&report.stdout)
  );

  let to_brew = run(pplr().args(["use", "brew"]))?;
  assert!(to_brew.status.success(), "{}", text(&to_brew.stderr));
  assert!(
    text(&to_brew.stdout).starts_with("pplr 2.0.0 (brew, 0123456789ab)"),
    "{}",
    text(&to_brew.stdout)
  );
  assert!(links.join("pplr").symlink_metadata().is_err());
  Ok(())
}

#[test]
fn use_brew_keeps_the_link_when_no_brew_copy_would_answer_and_bare_use_finds_a_shadow() -> Result<()>
{
  let sandbox = Sandbox::new()?;
  let opt = sandbox.path("opt");
  publish_tree(&opt, "pplr", false)?;
  let links = sandbox.path("home/.local/bin");
  let empty_brew = sandbox.path("nobrew");
  fs::create_dir_all(&empty_brew)?;
  let shadow = sandbox.path("shadow");
  fs::create_dir_all(&shadow)?;
  fs::write(shadow.join("pplr"), "#!/bin/sh\n")?;
  fs::set_permissions(shadow.join("pplr"), fs::Permissions::from_mode(0o755))?;
  let pplr = |path: String| {
    let mut c = sandbox.pplr();
    c.env("PATH", path)
      .env("PPLR_OPT_PREFIX", &opt)
      .env("HOMEBREW_PREFIX", &empty_brew);
    c
  };
  let ours = format!("{}:/usr/bin:/bin", links.display());
  assert!(
    run(pplr(ours.clone()).args(["use", "opt"]))?
      .status
      .success()
  );

  let refused = run(pplr(ours).args(["use", "brew"]))?;
  assert!(!refused.status.success());
  assert!(
    text(&refused.stderr).contains("is kept"),
    "{}",
    text(&refused.stderr)
  );
  assert!(links.join("pplr").symlink_metadata().is_ok());

  let shadowed = run(
    pplr(format!(
      "{}:{}:/usr/bin:/bin",
      shadow.display(),
      links.display()
    ))
    .arg("use"),
  )?;
  assert!(!shadowed.status.success());
  assert!(
    text(&shadowed.stdout).contains("shadowed"),
    "{}",
    text(&shadowed.stdout)
  );
  Ok(())
}

/// AT-05.1 (ST0001 AC-05.1): json, index, reindex, new, pictures and search
/// run natively; every golden made of them alone (and of commands already
/// native) replays, reindex's own files match theirs in the test above, and
/// search never puts the query on a command line (search::tests).
#[test]
fn the_index_new_pictures_and_search_commands_are_native_and_their_goldens_replay() -> Result<()> {
  const WP05: [&str; 6] = ["json", "index", "reindex", "new", "pictures", "search"];
  for name in WP05 {
    assert!(pplr::cli::native(name).is_some(), "{name} is not native");
  }
  let mut replayed = 0;
  for case in fs::read_dir(golden_dir().join("cases"))? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut commands = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        commands.push(lines_of(&step)?.first().cloned().unwrap_or_default());
      }
    }
    if commands.iter().any(|c| WP05.contains(&c.as_str()))
      && commands.iter().all(|c| pplr::cli::native(c).is_some())
    {
      same(replay(&case)?, true, &format!("{} replays", case.display()))?;
      replayed += 1;
    }
  }
  assert!(replayed > 0, "no golden of WP05's commands replayed");
  Ok(())
}

/// AT-06.1 (ST0001 AC-06.1): tag and tags run natively, every golden of
/// theirs that is fully native replays (tags.yaml from the ported emitter,
/// the _tags pages, the Tags lines and tags_index.json in Python's form), and
/// reindex, its tags step now native, replays too.
#[test]
fn tag_and_tags_are_native_and_their_goldens_replay() -> Result<()> {
  const WP06: [&str; 2] = ["tag", "tags"];
  for name in WP06 {
    assert!(pplr::cli::native(name).is_some(), "{name} is not native");
  }
  let mut replayed = Vec::new();
  for case in fs::read_dir(golden_dir().join("cases"))? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut commands = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        commands.push(lines_of(&step)?.first().cloned().unwrap_or_default());
      }
    }
    if commands.iter().any(|c| c == "tags" || c == "reindex")
      && commands.iter().all(|c| pplr::cli::native(c).is_some())
    {
      same(replay(&case)?, true, &format!("{} replays", case.display()))?;
      replayed.push(case);
    }
  }
  assert!(
    !replayed.is_empty(),
    "no golden of tags or reindex replayed"
  );
  Ok(())
}

#[test]
fn commands_about_pplr_itself_run_where_there_is_no_people_tree() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let missing = sandbox.path("no-such-tree");
  let pplr = || {
    let mut c = sandbox.pplr();
    c.env("PPLR_DATA", &missing);
    c
  };
  let version = run(pplr().arg("version"))?;
  assert!(version.status.success(), "{}", text(&version.stdout));
  assert!(
    text(&version.stdout).starts_with("pplr "),
    "{}",
    text(&version.stdout)
  );
  // doctor runs, and reports the missing tree as one of its rows
  let doctor = run(pplr().arg("doctor"))?;
  assert!(
    text(&doctor.stdout).contains(&format!("Data directory not found: {}", missing.display())),
    "{}",
    text(&doctor.stdout)
  );
  assert!(run(pplr().arg("help"))?.status.success());
  // a command about the People tree still refuses, naming it
  let count = run(pplr().arg("count"))?;
  assert_eq!(
    text(&count.stdout),
    format!(
      "Error: Could not change to directory {}.\n",
      missing.display()
    )
  );
  Ok(())
}

/// AT-07.1 (ST0001 AC-07.1): contact runs natively (log --email excepted
/// until WP14 brings pplr's own mail reader), contact.yaml and (Contact).md
/// come out byte-identical, and every golden of contact and tag that is
/// native throughout replays.
#[test]
fn contact_is_native_and_its_goldens_replay() -> Result<()> {
  assert!(
    pplr::cli::native("contact").is_some(),
    "contact is not native"
  );
  let mut replayed = Vec::new();
  for case in fs::read_dir(golden_dir().join("cases"))? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut commands = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        commands.push(lines_of(&step)?.first().cloned().unwrap_or_default());
      }
    }
    if commands.iter().any(|c| c == "contact" || c == "tag")
      && commands.iter().all(|c| pplr::cli::native(c).is_some())
    {
      same(replay(&case)?, true, &format!("{} replays", case.display()))?;
      replayed.push(case);
    }
  }
  assert!(!replayed.is_empty(), "no golden of contact or tag replayed");
  Ok(())
}
