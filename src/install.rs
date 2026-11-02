//! devbin's dev/opt/brew contract for pplr (ST0002 WP03, install-release.md
//! section 2): the trees and the one function that names them (T1, T4), the
//! manifest (T3), the version line (T4), `pplr install` (T2) and `pplr use`
//! with its shadow check (T5).

use crate::Result;
use crate::config::Config;
use crate::layout::Layout;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The manifest, at a tree's root.
pub const MANIFEST: &str = ".pplr-install";
const TOOL: &str = "pplr";

/// The commit this binary was built from, or `unknown` (build.rs).
pub const SOURCE_COMMIT: &str = env!("PPLR_SOURCE_COMMIT");
/// Whether its build inputs were dirty when it was built.
pub const SOURCE_DIRTY: bool = matches!(env!("PPLR_SOURCE_DIRTY").as_bytes(), b"1");
/// The checkout it was built in: the dev tree.
const CHECKOUT: &str = env!("CARGO_MANIFEST_DIR");

/// The link set (T5): `pplr` alone. The sidecar is found beside the binary.
const LINK: &str = "pplr";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
  Dev,
  Opt,
  Brew,
  Unknown,
}

impl Kind {
  pub fn word(self) -> &'static str {
    match self {
      Kind::Dev => "dev",
      Kind::Opt => "opt",
      Kind::Brew => "brew",
      Kind::Unknown => "unknown",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
  pub tool: String,
  pub version: String,
  pub source_commit: String,
  pub managed_by: String,
  /// (sha256, path relative to the root), sorted by path bytes.
  pub files: Vec<(String, String)>,
}

impl Manifest {
  pub fn parse(text: &str) -> Option<Self> {
    let mut header = std::collections::HashMap::new();
    let mut files = Vec::new();
    for line in text.lines() {
      if let Some(field) = line.strip_prefix("# ") {
        if let Some((key, value)) = field.split_once(": ") {
          header.insert(key.to_owned(), value.to_owned());
        }
      } else if let Some((sha, path)) = line.split_once("  ") {
        files.push((sha.to_owned(), path.to_owned()));
      }
    }
    let field = |key: &str| header.get(key).cloned();
    Some(Self {
      tool: field("tool")?,
      version: field("version")?,
      source_commit: field("source_commit")?,
      managed_by: field("managed_by")?,
      files,
    })
  }

  /// T3's form: the header, then one `shasum -a 256 -c` line per file, sorted
  /// by `LC_ALL=C sort` (byte order).
  pub fn render(&self) -> String {
    let mut files = self.files.clone();
    files.sort_by(|a, b| a.1.as_bytes().cmp(b.1.as_bytes()));
    let mut out = format!(
      "# tool: {}\n# version: {}\n# source_commit: {}\n# managed_by: {}\n",
      self.tool, self.version, self.source_commit, self.managed_by
    );
    for (sha, path) in files {
      out.push_str(&format!("{sha}  {path}\n"));
    }
    out
  }

  fn read(root: &Path) -> Option<Self> {
    Self::parse(&std::fs::read_to_string(root.join(MANIFEST)).ok()?)
  }
}

/// The one function that names a tree (T4): brew when its manifest says
/// brew manages it, opt when the manifest is pplr's, dev when the root is a git
/// checkout of pplr with no manifest, else unknown.
pub fn kind(root: &Path) -> (Kind, Option<Manifest>) {
  match Manifest::read(root) {
    Some(m) if m.managed_by == "brew" => (Kind::Brew, Some(m)),
    Some(m) if m.tool == TOOL => (Kind::Opt, Some(m)),
    Some(m) => (Kind::Unknown, Some(m)),
    None if root.join(".git").exists() && root.join("Cargo.toml").is_file() => (Kind::Dev, None),
    None => (Kind::Unknown, None),
  }
}

/// `pplr <version> (<tree>, <sha12>[+dirty])`, from the stamps or the manifest.
pub fn line(
  version: &str,
  kind: Kind,
  manifest: Option<&Manifest>,
  commit: &str,
  dirty: bool,
) -> String {
  let sha = match (kind, manifest) {
    (Kind::Opt | Kind::Brew, Some(m)) => short(&m.source_commit),
    _ if commit == "unknown" => "unknown".to_owned(),
    (Kind::Dev, _) if dirty => format!("{}+dirty", short(commit)),
    _ => short(commit),
  };
  format!("{TOOL} {version} ({}, {sha})", kind.word())
}

fn short(sha: &str) -> String {
  sha.chars().take(12).collect()
}

/// The running binary's version line.
pub fn version_line() -> String {
  let (kind, manifest) = Layout::running().map_or((Kind::Unknown, None), |l| kind(&l.root));
  line(
    env!("CARGO_PKG_VERSION"),
    kind,
    manifest.as_ref(),
    SOURCE_COMMIT,
    SOURCE_DIRTY,
  )
}

/// The runtime set (T1), relative to a tree's root; install and doctor read it.
pub fn runtime_set() -> Vec<&'static str> {
  let mut set = vec!["bin/pplr"];
  if cfg!(target_os = "macos") {
    set.push("bin/pplr-contacts");
  }
  set
}

/// Every runtime file whose bytes no longer match the manifest, by path.
pub fn drift(root: &Path, manifest: &Manifest) -> Vec<String> {
  manifest
    .files
    .iter()
    .filter(|(sha, path)| sha256(&root.join(path)).as_deref() != Some(sha.as_str()))
    .map(|(_, path)| path.clone())
    .collect()
}

fn sha256(path: &Path) -> Option<String> {
  let bytes = std::fs::read(path).ok()?;
  Some(
    Sha256::digest(bytes)
      .iter()
      .map(|b| format!("{b:02x}"))
      .collect(),
  )
}

/// Lay a runtime set out in `prefix` and write its manifest (T2, T3). Each
/// source maps to its path in the set. An existing prefix is replaced only with
/// `force`, and only when pplr's own manifest says pplr wrote it.
pub fn publish(
  sources: &[(&str, PathBuf)],
  prefix: &Path,
  version: &str,
  commit: &str,
  managed_by: &str,
  force: bool,
) -> Result<()> {
  let occupied = prefix
    .read_dir()
    .is_ok_and(|mut entries| entries.next().is_some());
  if occupied {
    let old = Manifest::read(prefix).filter(|m| m.tool == TOOL);
    match old {
      None => {
        return Err(
          format!(
            "{} holds something pplr did not install; choose another prefix",
            prefix.display()
          )
          .into(),
        );
      }
      Some(m) if m.managed_by == "brew" => {
        return Err(
          format!(
            "{} is managed by brew; upgrade it with brew upgrade pplr",
            prefix.display()
          )
          .into(),
        );
      }
      Some(_) if !force => {
        return Err(
          format!(
            "{} already holds pplr; give --force to replace it",
            prefix.display()
          )
          .into(),
        );
      }
      Some(m) => {
        // Files the old install had that this one does not
        for (_, path) in m
          .files
          .iter()
          .filter(|(_, p)| !sources.iter().any(|(s, _)| s == p))
        {
          if !path.contains("..") {
            let _ = std::fs::remove_file(prefix.join(path));
          }
        }
      }
    }
  }
  let mut files = Vec::new();
  for (path, source) in sources {
    let bytes =
      std::fs::read(source).map_err(|e| format!("cannot read {}: {e}", source.display()))?;
    let target = prefix.join(path);
    if let Some(dir) = target.parent() {
      std::fs::create_dir_all(dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
    }
    crate::fs::write_atomic(&target, &bytes)?;
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755))?;
    files.push((
      Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect(),
      (*path).to_owned(),
    ));
  }
  let manifest = Manifest {
    tool: TOOL.to_owned(),
    version: version.to_owned(),
    source_commit: commit.to_owned(),
    managed_by: managed_by.to_owned(),
    files,
  };
  crate::fs::write_atomic(&prefix.join(MANIFEST), manifest.render().as_bytes())
}

/// `pplr install [--prefix <dir>] [--force] [--managed-by brew]`
pub fn install(args: &[String], config: &Config) -> Result<ExitCode> {
  let mut args = args.to_vec();
  let force = take_flag(&mut args, "--force");
  let prefix = take_value(&mut args, "--prefix")?.map(PathBuf::from);
  let managed_by = match take_value(&mut args, "--managed-by")?.as_deref() {
    None => TOOL,
    Some("brew") => "brew",
    Some(other) => return Err(format!("--managed-by takes brew only, not {other}").into()),
  };
  if let Some(extra) = args.first() {
    return Err(format!("install does not take {extra}; usage: pplr install [--prefix <dir>] [--force] [--managed-by brew]").into());
  }
  let layout = Layout::running()?;
  let (here, _) = kind(&layout.root);
  if here != Kind::Dev {
    return Err(
      format!(
        "pplr install publishes from a dev checkout only, and this pplr is the {} tree",
        here.word()
      )
      .into(),
    );
  }
  let prefix = prefix.or_else(|| config.opt_prefix.clone()).ok_or(
    "pplr install needs a prefix: give --prefix <dir>, or set PPLR_OPT_PREFIX (the opt tree has no default)",
  )?;
  if SOURCE_COMMIT == "unknown" {
    return Err(
      "this pplr was built where git could not name the commit, so it cannot be published".into(),
    );
  }
  if SOURCE_DIRTY {
    return Err("this pplr was built from uncommitted changes: commit them, rebuild (bin/devbin build) and install again".into());
  }
  let mut sources = vec![("bin/pplr", std::env::current_exe()?)];
  if cfg!(target_os = "macos") {
    let sidecar = layout.sidecar();
    if !sidecar.is_file() {
      return Err(
        format!(
          "the Contacts sidecar is missing at {}: build it with bin/devbin build",
          sidecar.display()
        )
        .into(),
      );
    }
    sources.push(("bin/pplr-contacts", sidecar));
  }
  publish(
    &sources,
    &prefix,
    env!("CARGO_PKG_VERSION"),
    SOURCE_COMMIT,
    managed_by,
    force,
  )?;
  println!(
    "Installed {} into {}",
    line(
      env!("CARGO_PKG_VERSION"),
      Kind::Opt,
      None,
      SOURCE_COMMIT,
      false
    ),
    prefix.display()
  );
  Ok(ExitCode::SUCCESS)
}

/// `pplr use [dev|opt|brew]`
pub fn use_tree(args: &[String], config: &Config) -> Result<ExitCode> {
  let home = std::env::var_os("HOME")
    .map(PathBuf::from)
    .ok_or("HOME is not set, so pplr cannot find ~/.local/bin")?;
  let links = Links::new(home.join(".local/bin"));
  match args.first().map(String::as_str) {
    None => {
      let report = links.report();
      println!("{}", report.text);
      Ok(if report.ok {
        ExitCode::SUCCESS
      } else {
        ExitCode::FAILURE
      })
    }
    Some("dev") => {
      let target = Path::new(CHECKOUT).join("target/release/pplr");
      if !target.is_file() {
        return Err(
          format!(
            "no dev build at {}: run tools/build --release",
            target.display()
          )
          .into(),
        );
      }
      links.point_at(&target)
    }
    Some("opt") => {
      let prefix = config
        .opt_prefix
        .clone()
        .ok_or("pplr use opt needs PPLR_OPT_PREFIX, the opt tree's one place")?;
      match kind(&prefix).0 {
        Kind::Opt => links.point_at(&prefix.join("bin/pplr")),
        Kind::Brew => Err(format!("{} is managed by brew: use brew", prefix.display()).into()),
        _ => Err(
          format!(
            "no opt tree at {}: publish one with pplr install from dev",
            prefix.display()
          )
          .into(),
        ),
      }
    }
    Some("brew") => links.to_brew(&brew_prefix()),
    Some(other) => Err(format!("pplr use takes dev, opt or brew, not {other}").into()),
  }
}

/// Homebrew's prefix: HOMEBREW_PREFIX, else what `brew --prefix` says.
fn brew_prefix() -> Option<PathBuf> {
  if let Some(prefix) = std::env::var_os("HOMEBREW_PREFIX").filter(|p| !p.is_empty()) {
    return Some(PathBuf::from(prefix));
  }
  let out = Command::new("brew").arg("--prefix").output().ok()?;
  out
    .status
    .success()
    .then(|| PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
}

/// The link set in `dir` (T5).
pub struct Links {
  dir: PathBuf,
}

pub struct Report {
  pub ok: bool,
  pub text: String,
}

impl Links {
  pub fn new(dir: PathBuf) -> Self {
    Self { dir }
  }

  fn link(&self) -> PathBuf {
    self.dir.join(LINK)
  }

  /// The tree the link serves, or None with no link.
  fn serves(&self) -> Option<(PathBuf, Kind)> {
    let target = std::fs::read_link(self.link()).ok()?;
    let target = if target.is_relative() {
      self.dir.join(target)
    } else {
      target
    };
    let kind = target
      .canonicalize()
      .ok()
      .and_then(|t| Layout::from_exe(&t))
      .map_or(Kind::Unknown, |l| kind(&l.root).0);
    Some((target, kind))
  }

  /// What the link serves, and the shadow check: the link must be the first
  /// `pplr` on PATH, compared as resolved directories.
  pub fn report(&self) -> Report {
    let first = first_on_path(LINK, None);
    match self.serves() {
      None => Report {
        ok: true,
        text: "pplr: no link in ~/.local/bin, so brew answers".to_owned(),
      },
      Some((target, kind)) => {
        let ours = resolved_dir(&self.link());
        let theirs = first.as_deref().and_then(resolved_dir);
        let shadowed = ours.is_some() && ours != theirs;
        let text = format!("pplr -> {} ({})", kind.word(), target.display());
        match (shadowed, first) {
          (true, Some(first)) => Report {
            ok: false,
            text: format!(
              "{text}\nshadowed: {} comes first on PATH; put ~/.local/bin first",
              first.display()
            ),
          },
          _ => Report { ok: true, text },
        }
      }
    }
  }

  /// Point the link at `target`, replacing a link but never a file.
  fn point_at(&self, target: &Path) -> Result<ExitCode> {
    std::fs::create_dir_all(&self.dir)?;
    let link = self.link();
    if link
      .symlink_metadata()
      .is_ok_and(|m| !m.file_type().is_symlink())
    {
      return Err(
        format!(
          "{} is a file, not a link; move it aside first",
          link.display()
        )
        .into(),
      );
    }
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(target, &link)?;
    let out = Command::new(&link).arg("version").output()?;
    print!("{}", String::from_utf8_lossy(&out.stdout));
    Ok(ExitCode::SUCCESS)
  }

  /// Remove the link so Homebrew's answers, refusing when none would.
  fn to_brew(&self, brew: &Option<PathBuf>) -> Result<ExitCode> {
    let link = self.link();
    let serving = self.serves();
    let brew_bin = brew.as_ref().map(|p| p.join("bin").join(LINK));
    let keg = brew.as_ref().map(|p| p.join("opt/pplr"));
    let brews = brew_bin.as_ref().and_then(|b| b.canonicalize().ok());
    let answers = first_on_path(LINK, Some(&link)).and_then(|p| p.canonicalize().ok());
    let in_keg = |p: &Path| {
      keg
        .as_ref()
        .and_then(|k| k.canonicalize().ok())
        .is_some_and(|k| p.starts_with(k))
    };
    let would = match (&brews, &answers) {
      (Some(b), Some(a)) if b == a && in_keg(b) => None,
      (None, _) => {
        Some("brew's pplr is not installed: brew install matthewsinclair/pplr/pplr".to_owned())
      }
      (Some(_), Some(a)) if !in_keg(a) => Some(format!("{} would come first on PATH", a.display())),
      _ => Some("brew's pplr is not linked: brew link pplr".to_owned()),
    };
    if let Some(why) = would {
      let serves = serving
        .map(|(t, k)| format!(" (it serves {} at {})", k.word(), t.display()))
        .unwrap_or_default();
      return Err(
        format!(
          "pplr use brew would leave no brew copy answering, so {} is kept{serves}: {why}",
          link.display()
        )
        .into(),
      );
    }
    if matches!(serving, Some((_, Kind::Dev | Kind::Opt | Kind::Brew))) {
      std::fs::remove_file(&link)?;
    }
    let out = Command::new(brew_bin.unwrap_or_default())
      .arg("version")
      .output()?;
    print!("{}", String::from_utf8_lossy(&out.stdout));
    Ok(ExitCode::SUCCESS)
  }
}

/// The first `name` on PATH, passing over `skip`.
fn first_on_path(name: &str, skip: Option<&Path>) -> Option<PathBuf> {
  let path = std::env::var_os("PATH")?;
  std::env::split_paths(&path)
    .map(|dir| dir.join(name))
    .find(|c| Some(c.as_path()) != skip && c.is_file())
}

fn resolved_dir(link: &Path) -> Option<PathBuf> {
  link.parent()?.canonicalize().ok()
}

fn take_flag(args: &mut Vec<String>, flag: &str) -> bool {
  args
    .iter()
    .position(|a| a == flag)
    .map(|i| args.remove(i))
    .is_some()
}

fn take_value(args: &mut Vec<String>, flag: &str) -> Result<Option<String>> {
  let Some(i) = args.iter().position(|a| a == flag) else {
    return Ok(None);
  };
  args.remove(i);
  if i >= args.len() || args[i].starts_with("--") {
    return Err(format!("{flag} needs a value").into());
  }
  Ok(Some(args.remove(i)))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn manifest(managed_by: &str) -> Manifest {
    Manifest {
      tool: TOOL.into(),
      version: "2.0.0".into(),
      source_commit: "0123456789abcdef0123456789abcdef01234567".into(),
      managed_by: managed_by.into(),
      files: vec![
        ("bb".into(), "bin/pplr-contacts".into()),
        ("aa".into(), "bin/pplr".into()),
      ],
    }
  }

  #[test]
  fn the_manifest_renders_t3s_form_sorted_and_parses_back() {
    let m = manifest("pplr");
    let text = m.render();
    assert!(text.starts_with("# tool: pplr\n# version: 2.0.0\n# source_commit: 0123456789abcdef0123456789abcdef01234567\n# managed_by: pplr\n"));
    assert!(
      text.ends_with("aa  bin/pplr\nbb  bin/pplr-contacts\n"),
      "{text}"
    );
    let back = Manifest::parse(&text);
    assert_eq!(
      back.map(|b| (b.tool, b.managed_by, b.files.len())),
      Some((m.tool, m.managed_by, m.files.len()))
    );
  }

  #[test]
  fn the_version_line_names_the_tree_and_its_commit() {
    let commit = "fedcba9876543210fedcba9876543210fedcba98";
    let opt = manifest("pplr");
    assert_eq!(
      line("2.0.0", Kind::Opt, Some(&opt), commit, true),
      format!("pplr 2.0.0 (opt, {})", &opt.source_commit[..12])
    );
    assert_eq!(
      line("2.0.0", Kind::Dev, None, commit, false),
      format!("pplr 2.0.0 (dev, {})", &commit[..12])
    );
    assert_eq!(
      line("2.0.0", Kind::Dev, None, commit, true),
      format!("pplr 2.0.0 (dev, {}+dirty)", &commit[..12])
    );
    assert_eq!(
      line("2.0.0", Kind::Dev, None, "unknown", true),
      "pplr 2.0.0 (dev, unknown)"
    );
  }

  #[test]
  fn the_tree_is_named_by_its_manifest_then_its_checkout() -> Result<()> {
    let root = std::env::temp_dir().join(format!("pplr-kind-test-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    assert_eq!(kind(&root).0, Kind::Unknown);
    for (managed_by, expected) in [("pplr", Kind::Opt), ("brew", Kind::Brew)] {
      std::fs::write(root.join(MANIFEST), manifest(managed_by).render())?;
      assert_eq!(kind(&root).0, expected);
    }
    std::fs::remove_file(root.join(MANIFEST))?;
    std::fs::create_dir_all(root.join(".git"))?;
    std::fs::write(root.join("Cargo.toml"), "")?;
    assert_eq!(kind(&root).0, Kind::Dev);
    std::fs::remove_dir_all(&root)?;
    Ok(())
  }
}
