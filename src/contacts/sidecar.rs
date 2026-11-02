//! The live store on macOS: reads through the `pplr-contacts` sidecar (dump,
//! backup, group), writes through Contacts.app scripts and `sips`.

use super::{Card, CardChange, MarkerWrite, NewCard, Store, scripts};
use crate::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The sidecar's exit status when macOS refuses it access to Contacts.
const NO_ACCESS: i32 = 3;

/// macOS refused access to Contacts. The caller exits 3, as the engine did.
#[derive(Debug)]
pub struct AccessDenied(pub String);

impl std::fmt::Display for AccessDenied {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(&self.0)
  }
}

impl std::error::Error for AccessDenied {}

pub struct SidecarStore {
  sidecar: PathBuf,
  group_ids: HashMap<String, String>,
  tiffs: usize,
}

impl SidecarStore {
  pub fn new(sidecar: PathBuf) -> Result<Self> {
    if !sidecar.is_file() {
      return Err(
        format!(
          "the Contacts sidecar is missing: {} (build it with bin/devbin build)",
          sidecar.display()
        )
        .into(),
      );
    }
    Ok(Self {
      sidecar,
      group_ids: HashMap::new(),
      tiffs: 0,
    })
  }

  /// The sidecar's stdout, trimmed of its final newline.
  fn sidecar(&self, args: &[&str]) -> Result<String> {
    let out = Command::new(&self.sidecar)
      .args(args)
      .output()
      .map_err(|e| format!("cannot run {}: {e}", self.sidecar.display()))?;
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();
    match out.status.code() {
      Some(0) => Ok(
        String::from_utf8_lossy(&out.stdout)
          .trim_end_matches('\n')
          .to_owned(),
      ),
      Some(NO_ACCESS) => Err(Box::new(AccessDenied(stderr))),
      _ => Err(format!("pplr-contacts {} failed: {stderr}", args.join(" ")).into()),
    }
  }

  fn group_id(&mut self, group: &str) -> Result<String> {
    if let Some(id) = self.group_ids.get(group) {
      return Ok(id.clone());
    }
    let id = self.sidecar(&["group", "--group", group])?;
    self.group_ids.insert(group.to_owned(), id.clone());
    Ok(id)
  }

  /// The picture as a TIFF, which is what Contacts.app takes from a script.
  fn tiff(&mut self, picture: &Path) -> Result<Tiff> {
    self.tiffs += 1;
    let path =
      std::env::temp_dir().join(format!("pplr-{}-{}.tiff", std::process::id(), self.tiffs));
    let ok = Command::new("/usr/bin/sips")
      .args(["-s", "format", "tiff", "-Z", "512"])
      .arg(picture)
      .arg("--out")
      .arg(&path)
      .output()
      .is_ok_and(|out| out.status.success());
    if !ok {
      return Err(format!("could not convert {}", picture.display()).into());
    }
    Ok(Tiff(path))
  }
}

/// A temporary TIFF, removed when dropped.
struct Tiff(PathBuf);

impl Drop for Tiff {
  fn drop(&mut self) {
    let _ = std::fs::remove_file(&self.0);
  }
}

/// Run a script with osascript; its stdout, or its stderr as the error.
fn osascript(script: &str, args: &[&str]) -> Result<String> {
  let out = Command::new("/usr/bin/osascript")
    .arg("-e")
    .arg(script)
    .args(args)
    .output()
    .map_err(|e| format!("cannot run osascript: {e}"))?;
  if out.status.success() {
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
  } else {
    Err(
      String::from_utf8_lossy(&out.stderr)
        .trim()
        .to_owned()
        .into(),
    )
  }
}

impl Store for SidecarStore {
  fn cards(&mut self) -> Result<Vec<Card>> {
    let dump = self.sidecar(&["dump"])?;
    Ok(
      serde_json::from_str(&dump)
        .map_err(|e| format!("pplr-contacts dump is not a Contacts dump: {e}"))?,
    )
  }

  fn backup(&mut self, dir: &Path) -> Result<PathBuf> {
    let out = self.sidecar(&["backup", "--backup-dir", &dir.to_string_lossy()])?;
    let path = out
      .strip_prefix("Backup: ")
      .ok_or_else(|| format!("pplr-contacts backup said: {out}"))?;
    Ok(PathBuf::from(path))
  }

  fn link(&mut self, card: &str, marker: Option<&MarkerWrite>, group: Option<&str>) -> Result<()> {
    let group_id = group
      .map(|g| self.group_id(g))
      .transpose()?
      .unwrap_or_default();
    let url = marker.map_or("", |m| m.url.as_str());
    let replacing = if marker.is_some_and(|m| m.replace) {
      "yes"
    } else {
      "no"
    };
    osascript(scripts::LINK, &[card, url, &group_id, replacing]).map(|_| ())
  }

  fn create(&mut self, card: &NewCard, group: &str) -> Result<String> {
    let group_id = self.group_id(group)?;
    // A picture that will not convert leaves the card without a photo, as before
    let tiff = card.picture.as_deref().and_then(|p| self.tiff(p).ok());
    osascript(
      &scripts::create(card, &group_id, tiff.as_ref().map(|t| t.0.as_path())),
      &[],
    )
  }

  fn update(&mut self, card: &str, change: &CardChange, group: &str) -> Result<()> {
    let group_id = if change.add_group {
      self.group_id(group)?
    } else {
      String::new()
    };
    osascript(&scripts::update(card, change, &group_id), &[]).map(|_| ())
  }

  fn set_photo(&mut self, card: &str, picture: &Path) -> Result<()> {
    let tiff = self.tiff(picture)?;
    osascript(&scripts::photo(card, &tiff.0), &[]).map(|_| ())
  }

  fn save(&mut self) -> Result<()> {
    Ok(())
  }
}
