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

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::BTreeSet;
  use std::os::unix::fs::PermissionsExt;

  /// The sidecar's modes, as AC-09.1 names them.
  const MODES: &[&str] = &["dump", "backup", "group"];

  /// A temporary directory of the test's own, removed on drop.
  struct Scratch(PathBuf);

  impl Scratch {
    fn new(name: &str) -> Result<Self> {
      let dir =
        std::env::temp_dir().join(format!("pplr-sidecar-test-{}-{name}", std::process::id()));
      std::fs::create_dir_all(&dir)?;
      Ok(Self(dir))
    }
  }

  impl Drop for Scratch {
    fn drop(&mut self) {
      let _ = std::fs::remove_dir_all(&self.0);
    }
  }

  /// AT-09.1 (ST0001 AC-09.1): the live store asks the sidecar for dump,
  /// backup and group alone, and never builds it. A missing sidecar is
  /// refused, naming the step that builds it, and nothing appears at its path;
  /// a stand-in sidecar is asked for each of the three modes and for nothing
  /// else; and the sidecar's own mode switch names exactly those three. That
  /// sync reaches Contacts only through the store rests on AT-10.1 and
  /// AT-11.1, which run it on the JSON store alone, and the dump format on
  /// json.rs's tests.
  #[test]
  fn the_live_store_asks_the_sidecar_for_dump_backup_and_group_only() -> Result<()> {
    let modes: BTreeSet<String> = MODES.iter().map(|mode| mode.to_string()).collect();
    let scratch = Scratch::new("modes")?;

    // Built ahead of time: a missing sidecar is refused by name, and nothing
    // is built in its place
    let missing = scratch.0.join("absent").join("pplr-contacts");
    let refusal = match SidecarStore::new(missing.clone()) {
      Ok(_) => return Err("a missing sidecar was accepted".into()),
      Err(e) => e.to_string(),
    };
    assert!(
      refusal.contains("bin/devbin build"),
      "the refusal does not name the step that builds the sidecar: {refusal}"
    );
    assert!(
      !missing.exists(),
      "something was written at the sidecar's path"
    );

    // A stand-in that records each mode it is asked for, and answers it
    let asked_log = scratch.0.join("asked");
    let stand_in = scratch.0.join("pplr-contacts");
    std::fs::write(
      &stand_in,
      format!(
        "#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\ncase \"$1\" in\n  dump) echo '[]' ;;\n  backup) echo \"Backup: $3/contacts.vcf\" ;;\n  group) echo group-id ;;\nesac\n",
        asked_log.display()
      ),
    )?;
    std::fs::set_permissions(&stand_in, std::fs::Permissions::from_mode(0o755))?;

    // Every read the store makes of the sidecar: the cards, a backup and the
    // group (the writes go to Contacts.app through osascript instead)
    let mut store = SidecarStore::new(stand_in)?;
    store.cards()?;
    store.backup(&scratch.0.join("backups"))?;
    store.group_id("PPLR")?;
    let asked: BTreeSet<String> = std::fs::read_to_string(&asked_log)?
      .lines()
      .map(str::to_owned)
      .collect();
    for mode in &modes {
      assert!(
        asked.contains(mode),
        "the store never asked the sidecar for {mode}"
      );
    }
    for mode in &asked {
      assert!(
        modes.contains(mode),
        "the store asked the sidecar for {mode}, which AC-09.1 does not name"
      );
    }

    // The sidecar's own mode switch names the three, read from its source
    let main_swift = Path::new(env!("CARGO_MANIFEST_DIR")).join("swift/pplr-contacts/main.swift");
    let labels = mode_switch_labels(&std::fs::read_to_string(main_swift)?)?;
    assert_eq!(
      labels, modes,
      "swift/pplr-contacts/main.swift's mode switch names other modes than AC-09.1's"
    );
    Ok(())
  }

  /// The case labels of main.swift's mode switch, `switch cmd { ... }`; an
  /// error, naming why, when there is no such switch or it has no case label.
  fn mode_switch_labels(source: &str) -> Result<BTreeSet<String>> {
    let start = source
      .find("switch cmd {")
      .ok_or("swift/pplr-contacts/main.swift has no mode switch (switch cmd)")?;
    let open = start + "switch cmd ".len();
    // The switch's body runs to the brace that closes its opening one
    let mut depth = 0_i32;
    let mut close = None;
    for (at, c) in source[open..].char_indices() {
      match c {
        '{' => depth += 1,
        '}' => {
          depth -= 1;
          if depth == 0 {
            close = Some(open + at);
            break;
          }
        }
        _ => {}
      }
    }
    let close = close.ok_or("main.swift's mode switch never closes")?;
    let mut labels = BTreeSet::new();
    for line in source[open..close].lines() {
      if let Some(case) = line.trim().strip_prefix("case ") {
        labels.extend(case.split('"').skip(1).step_by(2).map(str::to_owned));
      }
    }
    if labels.is_empty() {
      return Err("main.swift's mode switch has no case label".into());
    }
    Ok(labels)
  }
}
