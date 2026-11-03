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
  /// PPLR_CONTACTS_JSON: a Contacts dump file standing in for the live store.
  pub contacts_json: Option<PathBuf>,
  /// PPLR_ICON_DIR: the Finder icons applyicons sets, over `_pplr/icons` and the embedded set.
  pub icon_dir: Option<PathBuf>,
  /// PPLR_OPT_PREFIX: the opt tree, the one configured place (devbin T1; no default).
  pub opt_prefix: Option<PathBuf>,
  /// PPLR_REFS_DIR, where rename repoints links; default ~/Dropbox when HOME is set.
  pub refs_dir: Option<PathBuf>,
  /// PPLR_TODAY, which pins the clock (design 3.5): "YYYY-MM-DD", meaning its
  /// midnight, or "YYYY-MM-DD HH:MM:SS", local time.
  pub today: Option<jiff::civil::DateTime>,
  /// HOME, under which Mail and the system keep the stores `pplr email` reads.
  pub home: Option<PathBuf>,
  /// PPLR_CMS_URL, default the local CMS; set empty, files only, never the CMS.
  pub cms_url: String,
  /// PPLR_PICTURE_WIDTH, the width of the picture line `pictures` writes.
  pub picture_width: String,
}

/// The CMS that `open` sends a person to when it answers.
const DEFAULT_CMS: &str = "http://localhost:4360/people";

/// The picture line's width when PPLR_PICTURE_WIDTH does not set it.
const DEFAULT_PICTURE_WIDTH: &str = "160";

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
    let contacts_json = get("PPLR_CONTACTS_JSON");
    let icon_dir = get("PPLR_ICON_DIR");
    let opt_prefix = get("PPLR_OPT_PREFIX");
    let refs_dir = get("PPLR_REFS_DIR").or_else(|| home().ok().map(|h| h.join("Dropbox")));
    let today = match get("PPLR_TODAY") {
      Some(pin) => Some(
        pinned_clock(&pin.to_string_lossy())
          .map_err(|e| format!("PPLR_TODAY is not YYYY-MM-DD or YYYY-MM-DD HH:MM:SS: {e}"))?,
      ),
      None => None,
    };
    // `${PPLR_CMS_URL=default}`: empty is a value here, and means files only
    let picture_width = get("PPLR_PICTURE_WIDTH").map_or_else(
      || DEFAULT_PICTURE_WIDTH.to_owned(),
      |w| w.to_string_lossy().into_owned(),
    );
    let cms_url = lookup("PPLR_CMS_URL").map_or_else(
      || DEFAULT_CMS.to_owned(),
      |u| u.to_string_lossy().into_owned(),
    );
    Ok(Self {
      data,
      data_legacy,
      backup_dir,
      cache_dir,
      contacts_json,
      icon_dir,
      opt_prefix,
      refs_dir,
      today,
      home: home().ok(),
      cms_url,
      picture_width,
    })
  }

  /// Today: PPLR_TODAY's date, or the local date.
  pub fn today(&self) -> jiff::civil::Date {
    self
      .today
      .map_or_else(|| jiff::Zoned::now().date(), |pin| pin.date())
  }

  /// Now: PPLR_TODAY's instant in the local time zone, or the clock's.
  pub fn now(&self) -> Result<jiff::Zoned> {
    match self.today {
      Some(pin) => Ok(pin.to_zoned(jiff::tz::TimeZone::system())?),
      None => Ok(jiff::Zoned::now()),
    }
  }
}

/// PPLR_TODAY's two forms: a date, at its midnight, or a date and a time.
fn pinned_clock(pin: &str) -> std::result::Result<jiff::civil::DateTime, jiff::Error> {
  if pin.len() == "YYYY-MM-DD".len() {
    Ok(pin.parse::<jiff::civil::Date>()?.at(0, 0, 0, 0))
  } else {
    jiff::civil::DateTime::strptime("%Y-%m-%d %H:%M:%S", pin)
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
  fn pplr_today_pins_the_day_and_a_bad_one_is_named() -> Result<()> {
    let pinned = config(&[
      ("HOME", "/h"),
      ("PPLR_DATA", "/d"),
      ("PPLR_TODAY", "2026-06-15"),
    ])?;
    assert_eq!(pinned.today(), jiff::civil::date(2026, 6, 15));
    assert_eq!(
      pinned.today,
      Some(jiff::civil::date(2026, 6, 15).at(0, 0, 0, 0))
    );
    let timed = config(&[
      ("HOME", "/h"),
      ("PPLR_DATA", "/d"),
      ("PPLR_TODAY", "2026-06-15 09:30:00"),
    ])?;
    assert_eq!(timed.today(), jiff::civil::date(2026, 6, 15));
    assert_eq!(
      timed.today,
      Some(jiff::civil::date(2026, 6, 15).at(9, 30, 0, 0))
    );
    assert_eq!(
      timed.now()?.datetime(),
      jiff::civil::date(2026, 6, 15).at(9, 30, 0, 0)
    );
    let bad = config(&[
      ("HOME", "/h"),
      ("PPLR_DATA", "/d"),
      ("PPLR_TODAY", "15/06/2026"),
    ])
    .err()
    .map(|e| e.to_string())
    .unwrap_or_default();
    assert!(bad.contains("PPLR_TODAY"), "{bad}");
    Ok(())
  }

  #[test]
  fn an_empty_cms_url_means_files_only_and_unset_means_the_local_cms() -> Result<()> {
    assert_eq!(
      config(&[("HOME", "/h"), ("PPLR_DATA", "/d")])?.cms_url,
      DEFAULT_CMS
    );
    assert_eq!(
      config(&[("HOME", "/h"), ("PPLR_DATA", "/d"), ("PPLR_CMS_URL", "")])?.cms_url,
      ""
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
