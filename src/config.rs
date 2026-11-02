//! Every `PPLR_*` setting, read once (ST0001 design D5 and D8).

use crate::Result;
use std::path::PathBuf;

/// Where the People tree is when neither PPLR_DATA nor PPLR_DIR names it.
const DEFAULT_DATA: &str = "Dropbox/Career/People";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
  /// The People tree: PPLR_DATA, then the legacy PPLR_DIR (D8).
  pub data: PathBuf,
  /// PPLR_DIR as given, which Gtools reads first; doctor compares it with `data`.
  pub data_legacy: Option<PathBuf>,
  /// PPLR_BACKUP_DIR, default $XDG_DATA_HOME/pplr/contacts-backups (D5).
  pub backup_dir: PathBuf,
  /// PPLR_CACHE_DIR, default $XDG_CACHE_HOME/pplr (D5).
  pub cache_dir: PathBuf,
}

impl Config {
  /// The configuration from the process environment.
  pub fn from_env() -> Result<Self> {
    Self::from_lookup(|name| std::env::var_os(name).map(PathBuf::from))
  }

  /// The configuration from any lookup of environment variables. An empty
  /// value counts as unset, as `${VAR:-default}` treats it in the legacy code.
  pub fn from_lookup(lookup: impl Fn(&str) -> Option<PathBuf>) -> Result<Self> {
    let get = |name: &str| lookup(name).filter(|value| !value.as_os_str().is_empty());
    let home = || get("HOME").ok_or("HOME is not set, so pplr cannot find its default paths");

    let data_legacy = get("PPLR_DIR");
    let data = match get("PPLR_DATA").or_else(|| data_legacy.clone()) {
      Some(data) => data,
      None => home()?.join(DEFAULT_DATA),
    };
    let xdg = |name: &str, under_home: &str| -> Result<PathBuf> {
      Ok(get(name).map_or_else(|| home().map(|h| h.join(under_home)), Ok)?)
    };
    let backup_dir = match get("PPLR_BACKUP_DIR") {
      Some(dir) => dir,
      None => xdg("XDG_DATA_HOME", ".local/share")?.join("pplr/contacts-backups"),
    };
    let cache_dir = match get("PPLR_CACHE_DIR") {
      Some(dir) => dir,
      None => xdg("XDG_CACHE_HOME", ".cache")?.join("pplr"),
    };
    Ok(Self {
      data,
      data_legacy,
      backup_dir,
      cache_dir,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  fn config(vars: &[(&str, &str)]) -> Result<Config> {
    let vars: HashMap<String, PathBuf> = vars
      .iter()
      .map(|(k, v)| (k.to_string(), PathBuf::from(v)))
      .collect();
    Config::from_lookup(|name| vars.get(name).cloned())
  }

  #[test]
  fn pplr_data_names_the_tree_and_pplr_dir_is_its_fallback() -> Result<()> {
    assert_eq!(
      config(&[("HOME", "/h"), ("PPLR_DATA", "/d"), ("PPLR_DIR", "/l")])?.data,
      PathBuf::from("/d")
    );
    assert_eq!(
      config(&[("HOME", "/h"), ("PPLR_DIR", "/l")])?.data,
      PathBuf::from("/l")
    );
    assert_eq!(
      config(&[("HOME", "/h")])?.data,
      PathBuf::from("/h").join(DEFAULT_DATA)
    );
    assert_eq!(
      config(&[("HOME", "/h"), ("PPLR_DATA", ""), ("PPLR_DIR", "/l")])?.data,
      PathBuf::from("/l")
    );
    Ok(())
  }

  #[test]
  fn backups_and_cache_default_under_xdg() -> Result<()> {
    let bare = config(&[("HOME", "/h")])?;
    assert_eq!(
      bare.backup_dir,
      PathBuf::from("/h/.local/share/pplr/contacts-backups")
    );
    assert_eq!(bare.cache_dir, PathBuf::from("/h/.cache/pplr"));

    let xdg = config(&[
      ("HOME", "/h"),
      ("XDG_DATA_HOME", "/x/data"),
      ("XDG_CACHE_HOME", "/x/cache"),
    ])?;
    assert_eq!(
      xdg.backup_dir,
      PathBuf::from("/x/data/pplr/contacts-backups")
    );
    assert_eq!(xdg.cache_dir, PathBuf::from("/x/cache/pplr"));

    let set = config(&[
      ("PPLR_DATA", "/d"),
      ("PPLR_BACKUP_DIR", "/b"),
      ("PPLR_CACHE_DIR", "/c"),
    ])?;
    assert_eq!(
      (set.backup_dir, set.cache_dir),
      (PathBuf::from("/b"), PathBuf::from("/c"))
    );
    Ok(())
  }

  #[test]
  fn a_missing_home_is_named_when_a_default_needs_it() {
    let message = config(&[])
      .map(|_| ())
      .err()
      .map(|e| e.to_string())
      .unwrap_or_default();
    assert!(message.contains("HOME"), "{message}");
  }
}
