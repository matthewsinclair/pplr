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

/// Ok when `got` is `want`; otherwise an error showing both.
fn same<T: PartialEq + std::fmt::Debug>(got: T, want: T, what: &str) -> Result<()> {
  if got == want {
    Ok(())
  } else {
    Err(format!("{what}\n    got:  {got:?}\n    want: {want:?}").into())
  }
}

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
