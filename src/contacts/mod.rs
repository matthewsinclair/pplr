//! Contacts, behind one store interface (ST0001 design D2 and 3.2). Sync logic
//! reaches cards only through `Store`: the JSON store for the tests and Linux
//! (`PPLR_CONTACTS_JSON`), the Swift sidecar plus Contacts.app for the live
//! store on macOS. The writes are the engine's, primitive by primitive; what to
//! write, and when to stop, stays with the caller.

mod json;
mod scripts;
mod sidecar;

pub use json::{JsonStore, stamp, to_swift_json};
pub use sidecar::{AccessDenied, SidecarStore};

use crate::Result;
use crate::config::Config;
use crate::layout::Layout;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A card, in the engine's `dump` format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
  pub id: String,
  pub given: String,
  pub family: String,
  pub organization: String,
  pub job_title: String,
  pub emails: Vec<String>,
  pub phones: Vec<String>,
  pub urls: Vec<LabelledValue>,
  pub linkedin: Vec<String>,
  pub groups: Vec<String>,
  /// "home" (iCloud, where the group lives), "other" or "none"; absent means home.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub account: Option<String>,
  /// A photo or thumbnail; absent means unknown, which counts as none.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub has_image: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelledValue {
  pub label: String,
  pub value: String,
}

/// The pplr URL a write puts on a card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerWrite {
  pub url: String,
  /// Replace the card's pplr URL in an older form, rather than add one.
  pub replace: bool,
}

/// A new card, made from a pplr person.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewCard {
  pub given: String,
  pub family: String,
  pub organization: String,
  pub job_title: String,
  pub emails: Vec<String>,
  pub phones: Vec<String>,
  /// The person's LinkedIn slug, when they have one.
  pub linkedin: Option<String>,
  /// The person's pplr URL.
  pub marker: String,
  /// Their picture, set as the card's photo (the live store only).
  pub picture: Option<PathBuf>,
}

/// What an update changes on an existing card; every field is optional.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardChange {
  /// (given, family)
  pub name: Option<(String, String)>,
  pub organization: Option<String>,
  pub job_title: Option<String>,
  pub add_emails: Vec<String>,
  pub add_phones: Vec<String>,
  /// A LinkedIn slug to add, as a URL and as the card's slug.
  pub add_linkedin: Option<String>,
  pub marker: Option<MarkerWrite>,
  pub add_group: bool,
}

pub trait Store {
  /// Every card.
  fn cards(&mut self) -> Result<Vec<Card>>;
  /// Every card saved into `dir` before anything is written; the file written.
  fn backup(&mut self, dir: &Path) -> Result<PathBuf>;
  /// Link a card: its pplr URL and, when named, membership of the group.
  fn link(&mut self, card: &str, marker: Option<&MarkerWrite>, group: Option<&str>) -> Result<()>;
  /// Make a card in the default account, in the group; its id.
  fn create(&mut self, card: &NewCard, group: &str) -> Result<String>;
  /// Change an existing card.
  fn update(&mut self, card: &str, change: &CardChange, group: &str) -> Result<()>;
  /// Give a card the picture as its photo.
  fn set_photo(&mut self, card: &str, picture: &Path) -> Result<()>;
  /// Make every write so far durable: the JSON store writes its file.
  fn save(&mut self) -> Result<()>;
}

/// The store this run reads: the JSON store when PPLR_CONTACTS_JSON names one,
/// else the live store, which needs macOS and the sidecar beside the binary.
pub fn open(config: &Config, layout: &Layout) -> Result<Box<dyn Store>> {
  if let Some(path) = &config.contacts_json {
    return Ok(Box::new(JsonStore::open(path)?));
  }
  crate::tools::macos_only(
    "the live Contacts store needs macOS; elsewhere pplr reaches Contacts only through a dump file named by PPLR_CONTACTS_JSON",
  )?;
  Ok(Box::new(SidecarStore::new(layout.sidecar())?))
}

/// A LinkedIn slug as the URL the engine writes.
pub fn linkedin_url(slug: &str) -> String {
  format!("https://www.linkedin.com/in/{slug}/")
}
