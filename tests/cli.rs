//! The binary, end to end, in a sandbox per test. This is the one integration
//! file (devbin's `check autotests`); the golden replay (ST0001 WP03) extends
//! `Sandbox` rather than adding another.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// An io failure that names the path it was about, so a failing run says
/// which file without a guess (No Silent Errors).
trait Named<T> {
  fn named(self, path: &std::path::Path) -> Result<T>;
}

impl<T> Named<T> for std::io::Result<T> {
  fn named(self, path: &std::path::Path) -> Result<T> {
    self.map_err(|e| format!("{}: {e}", path.display()).into())
  }
}

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

/// AT-02.2 (ST0001 AC-02.2): a command not yet ported runs its legacy script
/// from the data root, with the variables bin/pplr sets, its arguments and its
/// exit status. The bats suite against the Rust pplr is the AC's other half.
#[test]
fn a_person_marker_stands_for_their_folder_name() -> Result<()> {
  // contact show takes "Surname, First": given the person's marker it shows
  // the same person. Which arguments are rewritten, and the URL-taking
  // commands' exemption, are marker::rewrite_args' own tests'
  let sandbox = Sandbox::new()?;
  let about = sandbox.data().join("K/Kemp, Jon/About");
  fs::create_dir_all(&about)?;
  fs::write(about.join("Jon Kemp (About).md"), "# Jon Kemp (About)\n")?;

  let by_marker = run(
    sandbox
      .pplr()
      .args(["contact", "show", "pplr://k/kemp-jon"]),
  )?;
  let by_name = run(sandbox.pplr().args(["contact", "show", "Kemp, Jon"]))?;
  assert_eq!(
    by_marker.status.code(),
    Some(0),
    "{}",
    text(&by_marker.stderr)
  );
  assert!(text(&by_name.stdout).starts_with("Jon Kemp\n"));
  assert_eq!(text(&by_marker.stdout), text(&by_name.stdout));
  Ok(())
}

#[test]
fn a_marker_that_does_not_resolve_stops_the_command() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(sandbox.pplr().args(["rename", "pplr://x/nobody"]))?;
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
fn help_lists_every_command_and_details_is_the_readme() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let out = run(sandbox.pplr().arg("help"))?;
  assert!(out.status.success());
  let listing = text(&out.stdout);
  // A command's first line is its name, padded to the longest, then ": "
  let listed: Vec<&str> = listing
    .lines()
    .filter_map(|l| l.split_once(": ").map(|(name, _)| name.trim_end()))
    .collect();
  for command in pplr::cli::COMMANDS {
    assert!(
      listed.contains(&command.name),
      "{} is not listed",
      command.name
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
// was captured from the legacy code, and runs again here against this binary,
// in a sandbox seeded the same way, and must match: stdout, stderr, exit code
// and the resulting tree, after the rewrites tests/golden/README.md lists.

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
  fs::create_dir_all(to).named(to)?;
  for entry in fs::read_dir(from).named(from)? {
    let entry = entry.named(from)?;
    let (source, target) = (entry.path(), to.join(entry.file_name()));
    let kind = entry.file_type().named(&source)?;
    if kind.is_symlink() {
      std::os::unix::fs::symlink(fs::read_link(&source).named(&source)?, &target).named(&target)?;
    } else if kind.is_dir() {
      copy_tree(&source, &target)?;
    } else {
      fs::copy(&source, &target)
        .map_err(|e| format!("{} to {}: {e}", source.display(), target.display()))?;
    }
  }
  Ok(())
}

/// Every path under `root`, relative and bytewise sorted, skipping `skip` at the top.
fn walk(root: &std::path::Path, skip: &[&str]) -> Result<Vec<String>> {
  fn visit(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).named(dir)? {
      let path = entry.named(dir)?.path();
      let rel = path.strip_prefix(root)?.to_string_lossy().into_owned();
      out.push(rel);
      if fs::symlink_metadata(&path).named(&path)?.is_dir() {
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

// A test-only switch, for proving the replay itself (never read by pplr):
//   PPLR_GOLDEN_CASES=<dir>     replay the cases in <dir> (eg a mutated copy)

/// The README's rewrites, less today's date: the port reads PPLR_TODAY.
fn golden_normalise(text: &str, sandbox: &str) -> String {
  let mut text = text.to_owned();
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
  let text = fs::read_to_string(path).named(path)?;
  let text = text.strip_suffix('\n').unwrap_or(&text);
  Ok(text.split('\n').map(str::to_owned).collect())
}

/// The file under `dir` that `rel` names, or None. Each component is matched
/// by its decomposed form against the names the directory lists, never by
/// the filesystem: git on macOS, where the goldens are captured, stores every
/// path precomposed (core.precomposeunicode), while a manifest names a file as
/// macOS showed it, and pplr writes an .inetloc's name decomposed, as
/// Foundation did. APFS takes either form for the other; ext4 does not.
fn golden_file(dir: &std::path::Path, rel: &str) -> Result<Option<PathBuf>> {
  let mut at = dir.to_path_buf();
  for part in rel.split('/') {
    let entries = match fs::read_dir(&at) {
      Ok(entries) => entries,
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
      Err(e) => return Err(format!("{}: {e}", at.display()).into()),
    };
    let listed = entries
      .map(|e| e.map(|e| e.path()))
      .collect::<std::io::Result<Vec<_>>>()
      .named(&at)?;
    let Some(next) = one_named(part, listed)? else {
      return Ok(None);
    };
    at = next;
  }
  Ok(Some(at))
}

/// The one entry of `listed` whose name is `part` once both are decomposed;
/// two such entries are refused, naming both, rather than one picked.
fn one_named(part: &str, listed: Vec<PathBuf>) -> Result<Option<PathBuf>> {
  use pplr::normalise::decompose;
  let want = decompose(part);
  let mut matches: Vec<PathBuf> = listed
    .into_iter()
    .filter(|p| {
      p.file_name()
        .is_some_and(|n| decompose(&n.to_string_lossy()) == want)
    })
    .collect();
  if matches.len() > 1 {
    return Err(format!("{part} names more than one entry: {matches:?}").into());
  }
  Ok(matches.pop())
}

/// Replays one case: Ok when every step and the resulting tree match.
fn replay(case: &std::path::Path) -> Result<()> {
  let steps_dir = case.join("steps");
  let mut steps: Vec<PathBuf> = fs::read_dir(&steps_dir)
    .named(&steps_dir)?
    .map(|e| e.map(|e| e.path()))
    .collect::<std::io::Result<_>>()
    .named(&steps_dir)?;
  steps.retain(|p| p.extension().is_some_and(|x| x == "args"));
  steps.sort();
  let mut argv = Vec::new();
  for step in &steps {
    argv.push((step.with_extension(""), lines_of(step)?));
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
    if fs::symlink_metadata(&path).named(&path)?.is_file() {
      let bytes = fs::read(&path).named(&path)?;
      if is_text(&bytes) && String::from_utf8_lossy(&bytes).contains("@SANDBOX@") {
        fs::write(
          &path,
          String::from_utf8_lossy(&bytes).replace("@SANDBOX@", &sandbox),
        )
        .named(&path)?;
      }
    }
  }
  for dir in ["home", "cache", "backups"] {
    fs::create_dir_all(sb.join(dir)).named(&sb.join(dir))?;
  }
  let gitconfig = sb.join("home/.gitconfig");
  fs::write(
    &gitconfig,
    "[user]\n\tname = Matthew Sinclair\n\temail = golden@example.invalid\n",
  )
  .named(&gitconfig)?;
  // pplr email reads Mail's own stores under HOME, built from the messages the
  // capture's fake mail-recent served the legacy code
  mail_stores(&sb.join("home"), &sb.join("mail/messages.json"))?;

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
      .env("PPLR_TODAY", GOLDEN_NOW)
      .env("PPLR_CONTACTS_JSON", sb.join("contacts.json"))
      .env("PPLR_BACKUP_DIR", sb.join("backups"))
      .env("PPLR_CACHE_DIR", sb.join("cache"))
      .env("PPLR_REFS_DIR", &sb)
      .env("PPLR_GOLDEN_TRIPWIRE", root.join("tripwire.log"))
      .current_dir(&sb)
      .stdin(std::process::Stdio::null());
    for line in &case_env {
      let (key, value) = line
        .split_once('=')
        .ok_or_else(|| format!("a setting without '=': {line}"))?;
      command.env(key, value.replace("@SANDBOX@", &sandbox));
    }
    command.args(args.iter().map(|a| a.replace("@SANDBOX@", &sandbox)));
    let out = command.output().map_err(|e| {
      format!(
        "step {}: cannot run {}: {e}",
        stem.display(),
        env!("CARGO_BIN_EXE_pplr")
      )
    })?;
    let step = stem
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    let expect = |ext: &str| {
      let path = stem.with_extension(ext);
      fs::read_to_string(&path).named(&path)
    };
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
  let tripwire = root.join("tripwire.log");
  if fs::metadata(&tripwire).is_ok_and(|m| m.len() > 0) {
    return Err(
      format!(
        "reached a tool no scenario should: {}",
        fs::read_to_string(&tripwire).named(&tripwire)?
      )
      .into(),
    );
  }

  // Stamped backup names, then the tree against the manifest
  let backups = sb.join("backups");
  for entry in fs::read_dir(&backups).named(&backups)? {
    let path = entry.named(&backups)?.path();
    let file = path
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    let masked = mask_stamps(&file);
    if masked != file {
      fs::rename(&path, backups.join(masked)).named(&path)?;
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
        fs::read_link(&actual)
          .named(&actual)?
          .to_string_lossy()
          .into_owned(),
        value.to_owned(),
        &format!("symlink {rel}"),
      )?,
      "f" => {
        let bytes = fs::read(&actual).named(&actual)?;
        // The case's own golden, else the file as the fixture holds it
        let stored = match golden_file(&case.join("files"), rel)? {
          Some(stored) => stored,
          None => golden_file(&fixture, rel)?
            .ok_or_else(|| format!("{rel} has no golden in the case or the fixture"))?,
        };
        let want = fs::read(&stored).named(&stored)?;
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
        let script = golden.join("xlsx-cells");
        let dump = Command::new(&script).arg(&actual).output().named(&script)?;
        same(
          dump.status.success(),
          true,
          &format!("xlsx-cells reads {rel}"),
        )?;
        let cells = format!("{rel}.cells");
        let stored = golden_file(&case.join("files"), &cells)?
          .ok_or_else(|| format!("{cells} has no golden in the case"))?;
        let want = fs::read_to_string(&stored).named(&stored)?;
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
  Ok(())
}

/// Where the CMS-up cases put the CMS: the PPLR_CMS_URL in their env files
/// (tests/golden/cases/open-print-cms-up/env).
const GOLDEN_CMS: &str = "127.0.0.1:47360";

/// GOLDEN_CMS's port, one stub at a time. This lock covers one test process,
/// whose tests replay CMS-up cases at once; the estate's cargo lane covers the
/// host, where another process's tests would bind the same port.
static CMS_PORT: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// A CMS that answers every request with 200, until dropped. It holds the
/// port for its whole life and frees it before the next stub can take it.
struct CmsStub {
  stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
  serving: Option<std::thread::JoinHandle<()>>,
  // Released last, once the listener has closed
  _port: std::sync::MutexGuard<'static, ()>,
}

impl CmsStub {
  fn start() -> Result<Self> {
    use std::io::{Read, Write};
    // A test that failed while holding the port poisoned the lock, but its
    // stub still closed the port on the way out
    let port = CMS_PORT
      .lock()
      .unwrap_or_else(std::sync::PoisonError::into_inner);
    let listener = std::net::TcpListener::bind(GOLDEN_CMS)?;
    listener.set_nonblocking(true)?;
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let running = stop.clone();
    let serving = std::thread::spawn(move || {
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
    Ok(Self {
      stop,
      serving: Some(serving),
      _port: port,
    })
  }
}

impl Drop for CmsStub {
  fn drop(&mut self) {
    self.stop.store(true, Ordering::Relaxed);
    // The listener closes as its thread ends, so wait for that before the
    // port's lock goes; the loop has nothing in it that can panic
    if let Some(serving) = self.serving.take() {
      let _ = serving.join();
    }
  }
}

/// Every CMS-up case names GOLDEN_CMS, and the tests that replay those cases
/// run at once: a second stub waits for the first to go, rather than failing
/// to bind the port (CI run 41491091725, open-print-cms-up).
#[test]
fn a_second_stub_cms_waits_for_the_first() -> Result<()> {
  let first = CmsStub::start()?;
  let (asking, asked) = std::sync::mpsc::channel();
  let second = std::thread::spawn(move || -> std::result::Result<(), String> {
    asking.send(()).map_err(|e| e.to_string())?;
    CmsStub::start().map(drop).map_err(|e| e.to_string())
  });
  asked.recv()?;
  // Long enough for the second stub to reach the port while the first holds it
  std::thread::sleep(std::time::Duration::from_millis(100));
  drop(first);
  let second = second
    .join()
    .map_err(|_| "the second stub's thread panicked")?;
  same(
    second,
    Ok(()),
    "the second stub, started while the first ran",
  )
}

/// Ok when `got` is `want`; otherwise an error showing both.
fn same<T: PartialEq + std::fmt::Debug>(got: T, want: T, what: &str) -> Result<()> {
  if got == want {
    Ok(())
  } else {
    Err(format!("{what}\n    got:  {got:?}\n    want: {want:?}").into())
  }
}

/// AT-03.1 (ST0001 AC-03.1): every golden replays against the binary.
/// AT-08.1 (ST0001 AC-08.1): rename, links and handler are native, so their
/// goldens replay here too; the bats rename tests run them against the binary,
/// and handler's stable opt path is src/handler.rs's own test.
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
      Ok(()) => ran.push(name),
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

/// A golden is found whichever Unicode form names it. Git stores the path
/// precomposed, a manifest may name it decomposed (as pplr names an
/// .inetloc), and ext4, unlike APFS, keeps the two apart (ubuntu's run
/// 41591508539). The lookup lists and compares, so this runs the same match
/// on either filesystem; two entries of one name are refused, naming both.
#[test]
fn a_golden_is_found_whichever_unicode_form_names_it() -> Result<()> {
  let (composed, decomposed) = (
    "Zs\u{f3}fia Varga (Contacts).inetloc",
    "Zso\u{301}fia Varga (Contacts).inetloc",
  );
  let sandbox = Sandbox::new()?;
  let files = sandbox.path("files");
  let about = files.join("V/Varga, Zs\u{f3}fia/About");
  fs::create_dir_all(&about)?;
  fs::write(about.join(composed), "the golden")?;
  let rel = format!("V/Varga, Zs\u{f3}fia/About/{decomposed}");
  let found = golden_file(&files, &rel)?.ok_or("no golden found")?;
  assert_eq!(fs::read_to_string(found)?, "the golden");
  assert_eq!(
    golden_file(&files, "V/Varga, Zs\u{f3}fia/About/nobody.inetloc")?,
    None
  );
  assert_eq!(golden_file(&files, "W/Nobody, Here/About")?, None);

  let both = vec![about.join(composed), about.join(decomposed)];
  let refused = one_named(decomposed, both)
    .err()
    .ok_or("two entries of one name were not refused")?
    .to_string();
  assert!(
    refused.contains("more than one entry") && refused.contains(composed),
    "{refused}"
  );
  Ok(())
}

// pplr's own mail reader (ST0001 WP14). Mail's two stores are built here from
// the fixture's messages, holding only the tables and columns pplr email
// reads, so no test touches a real mail store.

/// One message of the fixture's mail/messages.json: its fields as the
/// capture's fake mail-recent served them, and the mailboxes it is in.
#[derive(serde::Deserialize)]
struct FixtureMessage {
  date: String,
  account: String,
  address: String,
  from: String,
  to: Vec<String>,
  subject: String,
  message_id: String,
  mailboxes: Vec<String>,
}

/// Where macOS keeps Mail's index and the accounts store, under a HOME: the
/// test's own statement of it, so a reader looking elsewhere fails.
fn mail_store_paths(home: &std::path::Path) -> (PathBuf, PathBuf) {
  (
    home.join("Library/Mail/V10/MailData/Envelope Index"),
    home.join("Library/Accounts/Accounts4.sqlite"),
  )
}

/// Mail's Envelope Index and the accounts store under `home`, holding the
/// messages of `messages_json` in its order, each in its first mailbox and
/// labelled with the rest, as Gmail's All Mail holds a message labelled INBOX.
fn mail_stores(home: &std::path::Path, messages_json: &std::path::Path) -> Result<()> {
  use rusqlite::{Connection, ToSql};
  let messages: Vec<FixtureMessage> =
    serde_json::from_slice(&fs::read(messages_json).named(messages_json)?)?;
  let (index_path, accounts_path) = mail_store_paths(home);
  for path in [&index_path, &accounts_path] {
    let folder = path.parent().ok_or("a store has no folder")?;
    fs::create_dir_all(folder).named(folder)?;
  }
  let open =
    |path: &std::path::Path| Connection::open(path).map_err(|e| format!("{}: {e}", path.display()));
  let accounts = open(&accounts_path)?;
  accounts.execute_batch(
    "create table ZACCOUNT (Z_PK integer primary key, ZIDENTIFIER, ZUSERNAME, ZPARENTACCOUNT);",
  )?;
  let index = open(&index_path)?;
  index.execute_batch(
    "create table messages (date_received, sender, subject, global_message_id, mailbox, deleted);
     create table subjects (subject);
     create table addresses (address, comment);
     create table message_global_data (message_id_header);
     create table mailboxes (url);
     create table labels (message_id, mailbox_id);
     create table recipients (message, address, type, position);",
  )?;
  let insert = |db: &Connection, sql: &str, values: &[&dyn ToSql]| -> Result<i64> {
    db.execute(sql, values)?;
    Ok(db.last_insert_rowid())
  };
  let mut ids = std::collections::HashMap::<String, String>::new();
  let mut boxes = std::collections::HashMap::<(String, String), i64>::new();
  for m in &messages {
    let id = match ids.get(&m.account) {
      Some(id) => id.clone(),
      None => {
        let id = format!("ACCOUNT-{}", ids.len() + 1);
        insert(
          &accounts,
          "insert into ZACCOUNT (ZIDENTIFIER, ZUSERNAME) values (?1, ?2)",
          &[&id, &m.account],
        )?;
        ids.insert(m.account.clone(), id.clone());
        id
      }
    };
    let mut held = Vec::new();
    for name in &m.mailboxes {
      let key = (id.clone(), name.clone());
      let rowid = match boxes.get(&key) {
        Some(rowid) => *rowid,
        None => {
          // unescaped: decoding an escaped name is mail.rs's own test's
          let url = format!("imap://{id}/{name}");
          let rowid = insert(&index, "insert into mailboxes (url) values (?1)", &[&url])?;
          boxes.insert(key, rowid);
          rowid
        }
      };
      held.push(rowid);
    }
    let (mailbox, labels) = held
      .split_first()
      .ok_or("a fixture message is in no mailbox")?;
    let name = m.from.strip_suffix(&format!(" <{}>", m.address));
    let sender = insert(
      &index,
      "insert into addresses (address, comment) values (?1, ?2)",
      &[&m.address, &name],
    )?;
    let subject = insert(
      &index,
      "insert into subjects (subject) values (?1)",
      &[&m.subject],
    )?;
    let global = insert(
      &index,
      "insert into message_global_data (message_id_header) values (?1)",
      &[&m.message_id],
    )?;
    let received = m.date.parse::<jiff::Timestamp>()?.as_second();
    let message = insert(
      &index,
      "insert into messages (date_received, sender, subject, global_message_id, mailbox, deleted) \
       values (?1, ?2, ?3, ?4, ?5, 0)",
      &[&received, &sender, &subject, &global, mailbox],
    )?;
    for label in labels {
      insert(
        &index,
        "insert into labels (message_id, mailbox_id) values (?1, ?2)",
        &[&message, label],
      )?;
    }
    for (position, to) in m.to.iter().enumerate() {
      let address = insert(&index, "insert into addresses (address) values (?1)", &[to])?;
      insert(
        &index,
        "insert into recipients (message, address, type, position) values (?1, ?2, 0, ?3)",
        &[&message, &address, &i64::try_from(position)?],
      )?;
    }
  }
  Ok(())
}

/// Stand-ins on PATH for tools a command must never run: each one that runs
/// leaves `tripped-<name>` in the sandbox, and fails.
fn tripwires(sandbox: &Sandbox, names: &[&str]) -> Result<PathBuf> {
  let bin = sandbox.path("tripwires");
  fs::create_dir_all(&bin)?;
  for name in names {
    let script = bin.join(name);
    let mark = sandbox.path(&format!("tripped-{name}"));
    fs::write(
      &script,
      format!("#!/bin/sh\n/usr/bin/touch '{}'\nexit 1\n", mark.display()),
    )?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
  }
  Ok(bin)
}

/// AT-14.1 (ST0001 AC-14.1): pplr reads recent and inbox mail itself, from
/// Mail's own stores. Every golden of pplr email and of contact log --email
/// replays against stores built from the fixture's messages; and with the
/// tools the legacy code ran (mail-recent, and Python through its runner) as
/// tripwires on PATH, pplr email lists the inbox and contact log --email
/// <Message-ID> links a message, running none of them.
/// The bats contact log --email tests run both forms against the binary.
#[test]
fn email_and_contact_log_email_are_native_and_their_goldens_replay() -> Result<()> {
  assert!(pplr::cli::native("email").is_some(), "email is not native");
  let (mut listed, mut logged) = (false, false);
  for case in fs::read_dir(golden_dir().join("cases"))? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut steps = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        steps.push(lines_of(&step)?);
      }
    }
    let lists = steps
      .iter()
      .any(|s| s.first().is_some_and(|c| c == "email"));
    let logs = steps
      .iter()
      .any(|s| s.first().is_some_and(|c| c == "contact") && s.iter().any(|a| a == "--email"));
    if lists || logs {
      replay(&case)?;
      listed |= lists;
      logged |= logs;
    }
  }
  assert!(listed, "no golden of pplr email replayed");
  assert!(logged, "no golden of contact log --email replayed");

  let sandbox = Sandbox::new()?;
  mail_stores(
    &sandbox.path("home"),
    &golden_dir().join("fixture/mail/messages.json"),
  )?;
  let about = sandbox.data().join("O/Okafor, Chidi/About");
  fs::create_dir_all(&about)?;
  fs::write(
    about.join("Chidi Okafor (About).md"),
    "# Chidi Okafor (About)\n",
  )?;
  let tools = ["mail-recent", "python3", "uv"]; // nopython: tripwires, never run
  let bin = tripwires(&sandbox, &tools)?;
  let path = format!("{}:/usr/bin:/bin", bin.display());
  let inbox = run(
    sandbox
      .pplr()
      .env("PATH", &path)
      .env("PPLR_MAIL_RECENT", bin.join("mail-recent"))
      .args(["email", "inbox"]),
  )?;
  assert_eq!(inbox.status.code(), Some(0), "{}", text(&inbox.stderr));
  assert!(
    text(&inbox.stdout).contains("  ID:   <ledger-1@larkspur.example>\n"),
    "{}",
    text(&inbox.stdout)
  );
  let log = run(
    sandbox
      .pplr()
      .env("PATH", &path)
      .env("PPLR_TODAY", GOLDEN_DAY)
      .args([
        "contact",
        "log",
        "Okafor, Chidi",
        "--email",
        "<ledger-1@larkspur.example>",
      ]),
  )?;
  assert_eq!(log.status.code(), Some(0), "{}", text(&log.stderr));
  let yaml = fs::read_to_string(about.join("contact.yaml"))?;
  assert!(
    yaml.contains("link: \"message://%3Cledger-1@larkspur.example%3E\""),
    "{yaml}"
  );
  for tool in tools {
    assert!(
      !sandbox.path(&format!("tripped-{tool}")).exists(),
      "pplr ran {tool}"
    );
  }
  Ok(())
}

/// AT-14.2 (ST0001 AC-14.2): pplr email --json is a versioned contract. It
/// prints a document with its schema and the documented fields, read from
/// Mail's stores, and leaves the saved listing alone. It exits 1 naming the
/// cause, with nothing on stdout, for a store without a column it reads, a
/// store it may not read (naming Full Disk Access) and no store at all (naming
/// the path it looked in).
#[test]
fn email_json_is_a_versioned_contract_and_refuses_a_store_it_cannot_trust() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let home = sandbox.path("home");
  mail_stores(&home, &golden_dir().join("fixture/mail/messages.json"))?;
  let json = || run(sandbox.pplr().args(["email", "inbox", "--json"]));

  let out = json()?;
  assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
  let document: serde_json::Value = serde_json::from_slice(&out.stdout)?;
  assert!(document["schema"].is_u64(), "no schema version: {document}");
  let messages = document["messages"].as_array().ok_or("no messages array")?;
  let ids: Vec<&str> = messages
    .iter()
    .filter_map(|m| m["message_id"].as_str())
    .collect();
  assert_eq!(
    ids,
    ["<seed-9@kestrel.example>", "<ledger-1@larkspur.example>"]
  );
  for m in messages {
    let mut fields: Vec<&str> = m
      .as_object()
      .ok_or("a message is not an object")?
      .keys()
      .map(String::as_str)
      .collect();
    fields.sort_unstable();
    assert_eq!(
      fields,
      [
        "account",
        "address",
        "date",
        "from",
        "message_id",
        "subject",
        "to"
      ]
    );
    assert!(
      m["date"]
        .as_str()
        .is_some_and(|d| d.parse::<jiff::Timestamp>().is_ok()),
      "{m}"
    );
  }
  let saved = home.join(".cache/pplr/email-last.json");
  assert!(!saved.exists(), "--json wrote the saved listing");
  // The control: the listing without --json saves where that check looks
  let listing = run(sandbox.pplr().args(["email", "inbox"]))?;
  assert_eq!(listing.status.code(), Some(0), "{}", text(&listing.stderr));
  assert!(saved.exists(), "no saved listing at {}", saved.display());

  let (index, _) = mail_store_paths(&home);
  let shown = index.display().to_string();
  let refused = |causes: &[&str]| -> Result<()> {
    let out = json()?;
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout), "");
    let stderr = text(&out.stderr);
    for cause in causes {
      assert!(stderr.contains(cause), "{stderr}");
    }
    Ok(())
  };
  rusqlite::Connection::open(&index)?.execute_batch("alter table messages drop column deleted")?;
  refused(&["messages.deleted", "not the schema pplr reads"])?;
  fs::set_permissions(&index, fs::Permissions::from_mode(0o000))?;
  refused(&["Full Disk Access", &shown])?;
  fs::remove_file(&index)?;
  refused(&["no Mail store at", &shown])?;
  Ok(())
}

/// vc's ruling B on ST0001 WP14: a contact is one message, so `contact log`
/// refuses a second --email, exits 1 naming why and writes nothing. The legacy
/// mixed the two, the first's date, person and note with the last's link;
/// WP14's body records the divergence for WP13's CHANGELOG.
#[test]
fn contact_log_refuses_a_second_email_and_writes_nothing() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let about = sandbox.data().join("O/Okafor, Chidi/About");
  fs::create_dir_all(&about)?;
  fs::write(
    about.join("Chidi Okafor (About).md"),
    "# Chidi Okafor (About)\n",
  )?;
  let contact = about.join("contact.yaml");
  let log = |emails: &[&str]| {
    let mut args = vec!["contact", "log", "Okafor, Chidi"];
    for email in emails {
      args.extend(["--email", *email]);
    }
    run(sandbox.pplr().env("PPLR_TODAY", GOLDEN_DAY).args(args))
  };

  let twice = log(&["<a-1@larkspur.example>", "<a-2@larkspur.example>"])?;
  assert_eq!(twice.status.code(), Some(1), "{}", text(&twice.stdout));
  let stderr = text(&twice.stderr);
  assert!(
    stderr.contains("--email") && stderr.contains("more than once"),
    "{stderr}"
  );
  assert!(
    !contact.exists(),
    "a refused log wrote {}",
    contact.display()
  );

  // The control: one --email writes the file the refusal must not
  let once = log(&["<a-1@larkspur.example>"])?;
  assert_eq!(once.status.code(), Some(0), "{}", text(&once.stderr));
  assert!(contact.exists(), "a log wrote no {}", contact.display());
  Ok(())
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
      replay(&case)?;
      replayed.push(case);
    }
  }
  assert!(!replayed.is_empty(), "no golden is made of WP04's commands");
  Ok(())
}

/// reindex's own files against its golden: index.json as `jq .` prints it,
/// and index.md.
#[test]
fn reindex_writes_index_json_as_jq_prints_it_and_index_md() -> Result<()> {
  let sandbox = Sandbox::new()?;
  copy_tree(&golden_dir().join("fixture/Career/People"), &sandbox.data())?;
  let out = run(sandbox.pplr().env("PPLR_TODAY", GOLDEN_DAY).arg("reindex"))?;
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
      replay(&case)?;
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
      replay(&case)?;
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

/// AT-07.1 (ST0001 AC-07.1): contact runs natively, contact.yaml and
/// (Contact).md come out byte-identical, and every golden of contact and tag
/// that is native throughout replays (log --email's are AT-14.1's too).
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
      replay(&case)?;
      replayed.push(case);
    }
  }
  assert!(!replayed.is_empty(), "no golden of contact or tag replayed");
  Ok(())
}

/// AT-10.1 (ST0001 AC-10.1): sync runs natively, its --check, --link, --pull,
/// --photos and --backup against the JSON store, and every golden of sync
/// replays.
#[test]
fn sync_is_native_and_its_goldens_replay() -> Result<()> {
  const WP10: [&str; 5] = ["--check", "--link", "--pull", "--photos", "--backup"];
  assert!(pplr::cli::native("sync").is_some(), "sync is not native");
  let mut modes = Vec::new();
  for case in fs::read_dir(golden_dir().join("cases"))? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut syncs = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        let args = lines_of(&step)?;
        if args.first().is_some_and(|c| c == "sync") {
          syncs.extend(args.into_iter().skip(1));
        }
      }
    }
    if !syncs.is_empty() {
      replay(&case)?;
      modes.extend(syncs);
    }
  }
  for mode in WP10 {
    assert!(
      modes.iter().any(|m| m == mode),
      "no golden of sync {mode} replayed"
    );
  }
  Ok(())
}

/// AT-11.1 (ST0001 AC-11.1): sync --plan and --apply-plan run in Rust, and
/// their goldens replay: the plan's JSON, its summary, the workbook's four
/// sheets cell for cell, and a reviewed workbook carried out against the JSON
/// store.
#[test]
fn sync_plan_and_apply_plan_are_native_and_their_goldens_replay() -> Result<()> {
  let sandbox = Sandbox::new()?;
  fs::create_dir_all(sandbox.data())?;
  let contacts = sandbox.path("contacts.json");
  fs::write(&contacts, "[]")?;
  let plan = |args: &[&str]| {
    run(
      sandbox
        .pplr()
        .env("PPLR_CONTACTS_JSON", &contacts)
        .env("PPLR_BACKUP_DIR", sandbox.path("backups"))
        .args(args),
    )
  };
  let json = plan(&["sync", "--plan", "--json"])?;
  assert_eq!(json.status.code(), Some(0), "{}", text(&json.stderr));
  assert!(text(&json.stdout).starts_with("{\n  \"cards\" : ["));
  let book = sandbox.path("plan.xlsx");
  let book = book.to_string_lossy();
  let wrote = plan(&["sync", "--plan", "--out", &book])?;
  assert_eq!(wrote.status.code(), Some(0), "{}", text(&wrote.stderr));
  let applied = plan(&["sync", "--apply-plan", &book])?;
  assert_eq!(applied.status.code(), Some(0), "{}", text(&applied.stderr));
  assert!(text(&applied.stdout).starts_with("pplr sync --apply-plan (dry run"));

  const WP11: [&str; 2] = ["--plan", "--apply-plan"];
  let mut modes = Vec::new();
  for case in fs::read_dir(golden_dir().join("cases"))? {
    let case = case?.path();
    if !case.is_dir() {
      continue;
    }
    let mut ours = Vec::new();
    for step in fs::read_dir(case.join("steps"))? {
      let step = step?.path();
      if step.extension().is_some_and(|x| x == "args") {
        let args = lines_of(&step)?;
        if args.first().is_some_and(|c| c == "sync") {
          ours.extend(
            args
              .into_iter()
              .skip(1)
              .filter(|a| WP11.contains(&a.as_str())),
          );
        }
      }
    }
    if !ours.is_empty() {
      replay(&case)?;
      modes.extend(ours);
    }
  }
  for mode in WP11 {
    assert!(
      modes.iter().any(|m| m == mode),
      "no golden of sync {mode} replayed"
    );
  }
  Ok(())
}

/// ST0001 WP11, a deliberate divergence from the legacy (WP11's body): sync
/// refuses a missing option value and a --limit that is not a number, with
/// the error and the usage, and writes nothing. The legacy read them as no
/// limit or an empty group, so `--apply-plan wb.xlsx --apply --limit l`
/// wrote every change.
#[test]
fn sync_refuses_a_missing_value_or_a_bad_limit_and_writes_nothing() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let about = sandbox.data().join("K/Kemp, Jon/About");
  fs::create_dir_all(&about)?;
  fs::write(about.join("Jon Kemp (About).md"), "# Jon Kemp (About)\n")?;
  let contacts = sandbox.path("contacts.json");
  fs::write(&contacts, "[]")?;
  let backups = sandbox.path("backups");
  let sync = |args: &[&str]| {
    run(
      sandbox
        .pplr()
        .env("PPLR_CONTACTS_JSON", &contacts)
        .env("PPLR_BACKUP_DIR", &backups)
        .args(args),
    )
  };
  let book = sandbox.path("plan.xlsx");
  let book = book.to_string_lossy();
  let planned = sync(&["sync", "--plan", "--out", &book])?;
  assert_eq!(planned.status.code(), Some(0), "{}", text(&planned.stderr));
  for args in [
    &["sync", "--apply-plan", &book, "--apply", "--limit", "l"][..],
    &["sync", "--apply-plan", &book, "--apply", "--limit"],
    &["sync", "--link", "--apply", "--group"],
    &["sync", "--link", "--apply", "--name"],
    &["sync", "--pull", "--apply", "--skip"],
    &["sync", "--plan", "--out"],
    &["sync", "--apply-plan"],
  ] {
    let refused = sync(args)?;
    let out = text(&refused.stdout);
    assert_eq!(refused.status.code(), Some(1), "{args:?}: {out}");
    assert!(
      out.starts_with("Error: ") && out.contains("\nUsage: pplr sync"),
      "{args:?}: {out}"
    );
    assert_eq!(
      fs::read_to_string(&contacts)?,
      "[]",
      "{args:?} wrote to Contacts"
    );
    assert!(!backups.exists(), "{args:?} made a backup");
  }
  // The control: asked properly, the same workbook writes, so the checks above can fail
  let applied = sync(&["sync", "--apply-plan", &book, "--apply", "--limit", "1"])?;
  assert_eq!(applied.status.code(), Some(0), "{}", text(&applied.stdout));
  assert_ne!(fs::read_to_string(&contacts)?, "[]");
  assert!(backups.exists());
  Ok(())
}

/// One card, as the JSON store holds it.
fn card_json(given: &str, family: &str, emails: &str, phones: &str, urls: &str) -> String {
  format!(
    r#"[{{"id":"c1","given":"{given}","family":"{family}","organization":"","jobTitle":"","emails":[{emails}],"phones":[{phones}],"urls":[{urls}],"linkedin":[],"groups":[]}}]"#
  )
}

/// vc decision 10: an About is read as the legacy's byte-wise reads took it,
/// so one that is not UTF-8 keeps its header: here its email matches a card
/// that shares nothing else with it.
#[test]
fn an_about_that_is_not_utf8_keeps_its_header() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let dir = sandbox.data().join("L/Lovelace, Ada/About");
  fs::create_dir_all(&dir)?;
  fs::write(
    dir.join("Ada Lovelace (About).md"),
    b"# Ada Lovelace (About)\n\n- Email:    [ada@engine.example](mailto:ada@engine.example)\n\nMet at the caf\xe9.\n",
  )?;
  let contacts = sandbox.path("contacts.json");
  fs::write(
    &contacts,
    card_json("Augusta", "Byron", r#""ada@engine.example""#, "", ""),
  )?;
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_CONTACTS_JSON", &contacts)
      .args(["sync", "--check", "--json"]),
  )?;
  assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
  let report: serde_json::Value = serde_json::from_slice(&out.stdout)?;
  assert_eq!(report["matched"][0]["how"], "email", "{report}");
  Ok(())
}

/// vc decision 10: an About that is there but cannot be read fails the
/// command, naming it, where it was once read as no header at all.
#[test]
fn an_about_that_cannot_be_read_fails_the_command_naming_it() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let dir = sandbox.data().join("K/Kemp, Jon/About");
  fs::create_dir_all(&dir)?;
  let about = dir.join("Jon Kemp (About).md");
  fs::write(&about, "# Jon Kemp (About)\n")?;
  fs::set_permissions(&about, fs::Permissions::from_mode(0o000))?;
  assert!(
    fs::read(&about).is_err(),
    "the test needs a user that mode 000 keeps out"
  );
  let contacts = sandbox.path("contacts.json");
  fs::write(&contacts, "[]")?;
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_CONTACTS_JSON", &contacts)
      .args(["sync", "--check"]),
  )?;
  let stderr = text(&out.stderr);
  assert_eq!(out.status.code(), Some(1), "{}", text(&out.stdout));
  assert!(
    stderr.contains(&format!("cannot read {}: ", about.display())),
    "{stderr}"
  );
  Ok(())
}

/// vc decision 10: --pull writes the About back, so one that is not UTF-8 is
/// left byte for byte, and the failure names it rather than 'no About file'.
#[test]
fn pull_leaves_an_about_that_is_not_utf8_as_it_is_and_names_it() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let dir = sandbox.data().join("L/Lovelace, Ada/About");
  fs::create_dir_all(&dir)?;
  let about = dir.join("Ada Lovelace (About).md");
  let bytes = b"# Ada Lovelace (About)\n\n- Phone:\n\nMet at the caf\xe9.\n";
  fs::write(&about, bytes)?;
  let contacts = sandbox.path("contacts.json");
  fs::write(
    &contacts,
    card_json(
      "Ada",
      "Lovelace",
      "",
      r#""+447700900123""#,
      r#"{"label":"pplr","value":"pplr://l/lovelace-ada"}"#,
    ),
  )?;
  let out = run(
    sandbox
      .pplr()
      .env("PPLR_CONTACTS_JSON", &contacts)
      .env("PPLR_BACKUP_DIR", sandbox.path("backups"))
      .args(["sync", "--pull", "--apply"]),
  )?;
  let stdout = text(&out.stdout);
  assert_eq!(out.status.code(), Some(1), "{stdout}");
  assert!(
    stdout.contains(&format!(
      "{} is not UTF-8, so it is left as it is",
      about.display()
    )),
    "{stdout}"
  );
  assert_eq!(fs::read(&about)?, bytes);
  Ok(())
}

#[test]
fn publish_lays_out_an_app_bundle_file_by_file_with_each_mode_kept() -> Result<()> {
  use pplr::install::{MANIFEST, Manifest, drift};
  let sandbox = Sandbox::new()?;
  let app = sandbox.path("built/Pplr Links.app");
  fs::create_dir_all(app.join("Contents/MacOS"))?;
  fs::write(app.join("Contents/Info.plist"), "<plist/>")?;
  fs::set_permissions(
    app.join("Contents/Info.plist"),
    fs::Permissions::from_mode(0o644),
  )?;
  fs::write(app.join("Contents/MacOS/applet"), "#!/bin/sh\n")?;
  fs::set_permissions(
    app.join("Contents/MacOS/applet"),
    fs::Permissions::from_mode(0o755),
  )?;
  let prefix = sandbox.path("opt");
  let sources = [
    ("bin/pplr", PathBuf::from(env!("CARGO_BIN_EXE_pplr"))),
    ("Pplr Links.app", app),
  ];
  pplr::install::publish(
    &sources,
    &prefix,
    "2.0.0",
    "0123456789abcdef0123456789abcdef01234567",
    "pplr",
    false,
  )
  .map_err(|e| e.to_string())?;
  let manifest =
    Manifest::parse(&fs::read_to_string(prefix.join(MANIFEST))?).ok_or("no manifest")?;
  let paths: Vec<&str> = manifest.files.iter().map(|(_, p)| p.as_str()).collect();
  assert_eq!(
    paths,
    [
      "Pplr Links.app/Contents/Info.plist",
      "Pplr Links.app/Contents/MacOS/applet",
      "bin/pplr"
    ]
  );
  assert!(drift(&prefix, &manifest).is_empty());
  let mode = |p: &str| fs::metadata(prefix.join(p)).map(|m| m.permissions().mode() & 0o777);
  assert_eq!(
    (
      mode("Pplr Links.app/Contents/MacOS/applet")?,
      mode("Pplr Links.app/Contents/Info.plist")?
    ),
    (0o755, 0o644)
  );
  Ok(())
}

/// ST0001 WP13, vc decision 14 (a D10 fix): `pplr tags` and `pplr contact`
/// with no command print their usage naming the pplr command, where the port
/// had kept the legacy helper's name, in its layout, and exit 0 as it did.
#[test]
fn tags_and_contact_with_no_command_print_their_usage_naming_pplr() -> Result<()> {
  let tags = "
    pplr tags check                 every tags.yaml against the vocabulary
    pplr tags show PERSON           a person's tags, by facet
    pplr tags render [--dry-run]    the \"- Tags:\" line in each About, linking each tag to
                                    its page, and the tag pages in $PPLR_DATA/_tags/
    pplr tags index                 .index/tags_index.json: everyone with name, marker,
                                    role, company, picture and tags by facet
    pplr tags edit PERSON +tag -tag add or remove hand tags, checked against the vocabulary
    pplr tags apply FILE [--dry-run]
                                    write tags.yaml from a JSON list of
                                    {person, auto: [tag], inferred: [{tag, evidence}], hand: [tag]};
                                    hand tags already in the file are kept
";
  let contact = "
    pplr contact scan [--dry-run]     last from each person's newest meeting, and next from the cadence
    pplr contact due [DAYS] [--all]   who to get in touch with: next on or before today (plus DAYS);
                                      --all lists everyone with a next date
    pplr contact show PERSON          a person's contact.yaml, with their cadence
    pplr contact log [PERSON] [DATE] [--via email|call|message|meeting] [--email N|MESSAGE-ID]
                     [--link URL] [--note TEXT]
                                      record a contact (DATE defaults to today); next moves on;
                                      --email links the message: N is its number in the last
                                      pplr email listing, which also gives the date, subject and
                                      person (the sender, or who you sent it to), so PERSON can go
    pplr contact render               About/<First Surname> (Contact).md from contact.yaml, for everyone
    pplr contact next PERSON WHEN     set next: a date (2026-11-01) or from today (+2w, +10d, +3m)
    pplr contact context PERSON       what to write from: role, latest update, last contact and its notes
";
  let sandbox = Sandbox::new()?;
  for (command, usage) in [("tags", tags), ("contact", contact)] {
    let out = run(sandbox.pplr().arg(command))?;
    assert_eq!(out.status.code(), Some(0), "pplr {command}");
    assert_eq!(text(&out.stdout), format!("{usage}\n"), "pplr {command}");
  }
  Ok(())
}

/// AT-00.2 (ST0001 AC-00.2): one binary. Every command the shell pplr offered
/// (recon section 4) is served here, held to its arguments and flags by the
/// goldens and the bats suite (AT-00.4, AT-00.3), and the checkout keeps no
/// legacy dispatcher, command script, fallback or run-time engine build.
#[test]
fn one_binary_serves_every_shell_command_and_no_legacy_script_remains() -> Result<()> {
  // Recon section 4's catalogue: the commands the shell pplr offered
  const SHELL_COMMANDS: &[&str] = &[
    "about",
    "applyicons",
    "contact",
    "count",
    "cp",
    "doctor",
    "edit",
    "email",
    "grep",
    "handler",
    "help",
    "index",
    "json",
    "linkedin",
    "links",
    "meetings",
    "new",
    "open",
    "pictures",
    "refresh",
    "reindex",
    "rename",
    "resolve",
    "search",
    "setpicfordir",
    "setpicsfordirs",
    "sync",
    "tag",
    "tags",
    "version",
  ];
  let unserved: Vec<&str> = SHELL_COMMANDS
    .iter()
    .copied()
    .filter(|name| pplr::cli::native(name).is_none())
    .collect();
  assert!(unserved.is_empty(), "not served: {unserved:?}");

  let checkout = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
  let scripts: Vec<String> = fs::read_dir(checkout.join("bin"))?
    .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
    .collect::<std::io::Result<Vec<_>>>()?
    .into_iter()
    .filter(|name| name == "pplr" || name.starts_with("pplr_"))
    .collect();
  assert!(scripts.is_empty(), "legacy scripts in bin/: {scripts:?}");
  for legacy in ["lib/engine.sh", "src/fallback.rs"] {
    assert!(!checkout.join(legacy).exists(), "{legacy} remains");
  }
  Ok(())
}

/// AT-00.6 (ST0001 AC-00.6): an opt tree's pplr, run from outside any
/// checkout with PPLR_ROOT, PPLR_BIN_DIR and PPLR_TEMPLATE_DIR naming a decoy
/// legacy layout, runs nothing from it and reads nothing from it: an unknown
/// command is refused, new writes the About the dev build writes from its own
/// template, and help --details is the README the binary holds.
#[test]
fn an_installed_pplr_needs_no_checkout_and_no_legacy_setting() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let prefix = sandbox.path("opt");
  publish_tree(&prefix, "pplr", false)?;
  // The decoy: a command script that leaves a mark, a person template and a README
  let decoy = sandbox.path("decoy");
  let mark = sandbox.path("decoy-ran");
  fs::create_dir_all(decoy.join("bin"))?;
  let script = decoy.join("bin/pplr_decoy");
  fs::write(&script, format!("#!/bin/sh\ntouch '{}'\n", mark.display()))?;
  fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
  let template = decoy.join("templates/person/About");
  fs::create_dir_all(&template)?;
  fs::write(template.join("Firstname Surname (About).md"), "decoy\n")?;
  fs::write(decoy.join("README.md"), "decoy\n")?;
  let elsewhere = sandbox.path("elsewhere");
  let (opt_data, dev_data) = (sandbox.path("opt-data"), sandbox.path("dev-data"));
  for dir in [&elsewhere, &opt_data, &dev_data] {
    fs::create_dir_all(dir)?;
  }
  let installed = |args: &[&str]| {
    run(
      Command::new(prefix.join("bin/pplr"))
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap_or_default())
        .env("HOME", sandbox.path("home"))
        .env("LANG", "en_GB.UTF-8")
        .env("PPLR_DATA", &opt_data)
        .env("PPLR_TODAY", GOLDEN_DAY)
        .env("PPLR_ROOT", &decoy)
        .env("PPLR_BIN_DIR", decoy.join("bin"))
        .env("PPLR_TEMPLATE_DIR", decoy.join("templates"))
        .current_dir(&elsewhere)
        .args(args),
    )
  };

  let unknown = installed(&["decoy"])?;
  assert_eq!(
    text(&unknown.stdout),
    "Error: Command 'decoy' not found.\nUsage: pplr <cmd>\n"
  );
  assert_eq!(unknown.status.code(), Some(1));
  assert!(!mark.exists(), "the decoy's script ran");

  let new = installed(&["new", "Ada", "Lovelace"])?;
  assert!(new.status.success(), "{}", text(&new.stderr));
  let dev = run(
    sandbox
      .pplr()
      .env("PPLR_DATA", &dev_data)
      .env("PPLR_TODAY", GOLDEN_DAY)
      .args(["new", "Ada", "Lovelace"]),
  )?;
  assert!(dev.status.success(), "{}", text(&dev.stderr));
  let about = "L/Lovelace, Ada/About/Ada Lovelace (About).md";
  assert_eq!(
    fs::read_to_string(opt_data.join(about))?,
    fs::read_to_string(dev_data.join(about))?
  );

  let details = installed(&["help", "--details"])?;
  assert_eq!(
    text(&details.stdout),
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))?
  );
  Ok(())
}

/// ST0001 WP13, found by the live parity: rename lists the running text that
/// still names the person, but passes over every hit whose path or text holds
/// a skip, the person's own folder ("Surname, First/") or a file named after
/// them ("First Surname ("), as the legacy's `grep -vF` over grep's output
/// did. So the running text of their own About, which a rename rewrites, is
/// never listed as left unchanged.
#[test]
fn rename_lists_running_text_outside_the_persons_own_files() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let about = sandbox.data().join("B/Blackwood, Esme/About");
  fs::create_dir_all(&about)?;
  fs::write(
    about.join("Esme Blackwood (About).md"),
    "# Esme Blackwood (About)\n\n_Esme Blackwood_\n\nEsme Blackwood designs bridges.\n",
  )?;
  let refs = sandbox.path("refs");
  fs::create_dir_all(&refs)?;
  let notes = refs.join("notes.md");
  fs::write(&notes, "Lunch with Esme Blackwood.\n")?;

  let out = run(sandbox.pplr().args([
    "rename",
    "Blackwood, Esme",
    "Blackwood, Esmeralda",
    "--dry-run",
    "--refs",
    &sandbox.root.to_string_lossy(),
  ]))?;
  assert!(out.status.success(), "{}", text(&out.stderr));
  let listing = text(&out.stdout);
  let (_, prose) = listing
    .split_once("The old name still appears in running text (not changed):\n")
    .ok_or(format!("no running text listed: {listing}"))?;
  let listed: Vec<&str> = prose.lines().map(str::trim_start).collect();
  assert_eq!(
    listed,
    [format!("{}:1:Lunch with Esme Blackwood.", notes.display())]
  );
  Ok(())
}

/// ST0001 WP13, vc decision 15 (a D10 fix the live parity found): grep and
/// grep -t list people in name order, the same on every filesystem, where the
/// legacy's grep -r and find gave the filesystem's own directory order.
#[test]
fn grep_lists_people_in_name_order() -> Result<()> {
  let sandbox = Sandbox::new()?;
  for (letter, folder, name) in [
    ("Z", "Zeal, Zara", "Zara Zeal"),
    ("A", "Abbott, Ann", "Ann Abbott"),
  ] {
    let about = sandbox.data().join(format!("{letter}/{folder}/About"));
    fs::create_dir_all(&about)?;
    fs::write(
      about.join(format!("{name} (About).md")),
      format!("# {name} (About)\n\n- Role:     Engineer\n"),
    )?;
    fs::write(about.join("tags.yaml"), "tags:\n  - engineer\n")?;
  }
  for args in [&["grep", "Engineer"][..], &["grep", "-t", "engineer"]] {
    let out = run(sandbox.pplr().args(args))?;
    let listing = text(&out.stdout);
    let first = listing.find("[Abbott, Ann]");
    let last = listing.find("[Zeal, Zara]");
    assert!(
      matches!((first, last), (Some(a), Some(z)) if a < z),
      "pplr {args:?} is not in name order: {listing}"
    );
  }
  Ok(())
}

/// ST0001 WP13, vc decision 15 (a D10 fix the live parity found): grep prints
/// a matched line as the file holds it, so an empty "- Role:" keeps its colon,
/// which the legacy's `IFS=: read` dropped when it was the line's only one.
#[test]
fn grep_prints_a_matched_line_as_the_file_holds_it() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let about = sandbox.data().join("P/Pemberton, Rufus/About");
  fs::create_dir_all(&about)?;
  fs::write(
    about.join("Rufus Pemberton (About).md"),
    "# Rufus Pemberton (About)\n\n- Role:\n",
  )?;
  let out = run(sandbox.pplr().args(["grep", "Role"]))?;
  let listing = text(&out.stdout);
  assert!(
    listing.lines().any(|line| line == "   \u{2192} - Role:"),
    "{listing}"
  );
  Ok(())
}

/// ST0001 WP13, found by the live parity: links searches a directory through
/// its real subdirectories and never through a symlinked one, as the legacy's
/// FileManager enumerator walked, so a link out of the tree, or one back up
/// it that a following walk would go round and round, is never followed.
#[test]
fn links_searches_no_symlinked_directory() -> Result<()> {
  let sandbox = Sandbox::new()?;
  let people = sandbox.path("Career/People");
  let about = people.join("K/Kemp, Jon/About");
  fs::create_dir_all(&about)?;
  fs::write(about.join("Jon Kemp (About).md"), "# Jon Kemp (About)\n")?;
  let link = "[Jon](<../Career/People/K/Kemp, Jon/About/Jon Kemp (About).md>)\n";
  let (notes, outside) = (sandbox.path("notes"), sandbox.path("outside"));
  fs::create_dir_all(&notes)?;
  fs::create_dir_all(&outside)?;
  fs::write(notes.join("inside.md"), link)?;
  fs::write(outside.join("away.md"), link)?;
  std::os::unix::fs::symlink(&outside, notes.join("elsewhere"))?;
  std::os::unix::fs::symlink(&notes, notes.join("loop"))?;

  let out = run(sandbox.pplr().env("PPLR_DATA", &people).args([
    "links",
    "--json",
    &notes.to_string_lossy(),
  ]))?;
  assert!(out.status.success(), "{}", text(&out.stderr));
  let listing = text(&out.stdout);
  assert!(listing.contains("inside.md"), "{listing}");
  assert!(!listing.contains("away.md"), "{listing}");
  Ok(())
}
