//! The JSON store: a dump-format file standing in for Contacts, read and
//! rewritten as the engine's `--contacts-json` path does, byte for byte.

use super::{Card, CardChange, LabelledValue, MarkerWrite, NewCard, Store, linkedin_url};
use crate::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub struct JsonStore {
  path: PathBuf,
  cards: Vec<Card>,
}

impl JsonStore {
  pub fn open(path: &Path) -> Result<Self> {
    let text = std::fs::read_to_string(path)
      .map_err(|e| format!("cannot read the Contacts dump {}: {e}", path.display()))?;
    let cards = serde_json::from_str(&text)
      .map_err(|e| format!("{} is not a Contacts dump: {e}", path.display()))?;
    Ok(Self {
      path: path.to_path_buf(),
      cards,
    })
  }

  fn card(&mut self, id: &str) -> Option<&mut Card> {
    self.cards.iter_mut().find(|card| card.id == id)
  }
}

impl Store for JsonStore {
  fn cards(&mut self) -> Result<Vec<Card>> {
    Ok(self.cards.clone())
  }

  /// The file as it stands on disk, before this run's writes are saved.
  fn backup(&mut self, dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)
      .map_err(|e| format!("cannot make the backup directory {}: {e}", dir.display()))?;
    let to = dir.join(format!("{}-contacts.json", stamp()));
    std::fs::copy(&self.path, &to).map_err(|e| {
      format!(
        "cannot back up {} to {}: {e}",
        self.path.display(),
        to.display()
      )
    })?;
    Ok(to)
  }

  fn link(&mut self, id: &str, marker: Option<&MarkerWrite>, group: Option<&str>) -> Result<()> {
    let Some(card) = self.card(id) else {
      return Ok(());
    };
    if let Some(marker) = marker {
      // An old pplr URL is found by its label or by its scheme
      let old = card
        .urls
        .iter_mut()
        .find(|u| u.label.to_lowercase() == "pplr" || u.value.starts_with("pplr://"));
      match old {
        Some(url) if marker.replace => url.value = marker.url.clone(),
        _ => card.urls.push(pplr_url(&marker.url)),
      }
    }
    if let Some(group) = group {
      card.groups.push(group.to_owned());
    }
    Ok(())
  }

  fn create(&mut self, new: &NewCard, group: &str) -> Result<String> {
    let id = format!("new-{}", self.cards.len() + 1);
    let mut urls = vec![pplr_url(&new.marker)];
    if let Some(slug) = &new.linkedin {
      urls.insert(
        0,
        LabelledValue {
          label: "LinkedIn".to_owned(),
          value: linkedin_url(slug),
        },
      );
    }
    self.cards.push(Card {
      id: id.clone(),
      given: new.given.clone(),
      family: new.family.clone(),
      organization: new.organization.clone(),
      job_title: new.job_title.clone(),
      emails: new.emails.clone(),
      phones: new.phones.clone(),
      urls,
      linkedin: new.linkedin.iter().cloned().collect(),
      groups: vec![group.to_owned()],
      account: Some("home".to_owned()),
      has_image: None,
    });
    Ok(id)
  }

  fn update(&mut self, id: &str, change: &CardChange, group: &str) -> Result<()> {
    let Some(card) = self.card(id) else {
      return Ok(());
    };
    if let Some((given, family)) = &change.name {
      card.given = given.clone();
      card.family = family.clone();
    }
    if let Some(organization) = &change.organization {
      card.organization = organization.clone();
    }
    if let Some(job_title) = &change.job_title {
      card.job_title = job_title.clone();
    }
    card.emails.extend(change.add_emails.iter().cloned());
    card.phones.extend(change.add_phones.iter().cloned());
    if let Some(slug) = &change.add_linkedin {
      card.urls.push(LabelledValue {
        label: "LinkedIn".to_owned(),
        value: linkedin_url(slug),
      });
      card.linkedin.push(slug.clone());
    }
    match &change.marker {
      // Here the old URL is found by its exact label, as the engine's update does
      Some(marker) if marker.replace => {
        if let Some(url) = card.urls.iter_mut().find(|u| u.label == "pplr") {
          url.value = marker.url.clone();
        }
      }
      Some(marker) => card.urls.push(pplr_url(&marker.url)),
      None => {}
    }
    if change.add_group {
      card.groups.push(group.to_owned());
    }
    Ok(())
  }

  fn set_photo(&mut self, id: &str, _picture: &Path) -> Result<()> {
    if let Some(card) = self.card(id) {
      card.has_image = Some(true);
    }
    Ok(())
  }

  fn save(&mut self) -> Result<()> {
    let text = to_swift_json(&serde_json::to_value(&self.cards)?);
    crate::fs::write_atomic(&self.path, text.as_bytes())
  }
}

fn pplr_url(url: &str) -> LabelledValue {
  LabelledValue {
    label: "pplr".to_owned(),
    value: url.to_owned(),
  }
}

/// The engine's backup stamp: local time, to the second.
pub fn stamp() -> String {
  jiff::Zoned::now().strftime("%Y%m%d-%H%M%S").to_string()
}

/// JSON exactly as Swift's `JSONEncoder` writes it with `.prettyPrinted` and
/// `.sortedKeys`: two-space indents, `"key" : value`, `/` escaped, an empty
/// array or object as an open bracket, a blank line and the close, and no
/// newline at the end. serde_json's map keeps keys sorted.
pub fn to_swift_json(value: &Value) -> String {
  let mut out = String::new();
  write_value(value, 0, &mut out);
  out
}

fn write_value(value: &Value, indent: usize, out: &mut String) {
  match value {
    Value::Null => out.push_str("null"),
    Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
    Value::Number(n) => out.push_str(&n.to_string()),
    Value::String(s) => write_string(s, out),
    Value::Array(items) => write_container(
      ('[', ']'),
      items.iter().map(|item| (None, item)),
      items.len(),
      indent,
      out,
    ),
    Value::Object(map) => write_container(
      ('{', '}'),
      map.iter().map(|(key, item)| (Some(key.as_str()), item)),
      map.len(),
      indent,
      out,
    ),
  }
}

fn write_container<'a>(
  (open, close): (char, char),
  items: impl Iterator<Item = (Option<&'a str>, &'a Value)>,
  len: usize,
  indent: usize,
  out: &mut String,
) {
  out.push(open);
  out.push('\n');
  if len == 0 {
    out.push('\n');
  }
  for (index, (key, item)) in items.enumerate() {
    out.push_str(&" ".repeat(indent + 2));
    if let Some(key) = key {
      write_string(key, out);
      out.push_str(" : ");
    }
    write_value(item, indent + 2, out);
    out.push_str(if index + 1 < len { ",\n" } else { "\n" });
  }
  out.push_str(&" ".repeat(indent));
  out.push(close);
}

fn write_string(s: &str, out: &mut String) {
  out.push('"');
  for c in s.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '/' => out.push_str("\\/"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      '\u{8}' => out.push_str("\\b"),
      '\u{c}' => out.push_str("\\f"),
      c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
      c => out.push(c),
    }
  }
  out.push('"');
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn swift_pretty_print_matches_jsonencoder() {
    // Printed by Swift's JSONEncoder (.prettyPrinted, .sortedKeys) on rhadamanth
    let swift = "[\n  {\n    \"emails\" : [\n\n    ],\n    \"given\" : \"Zs\u{f3}fia \\\"Zs\u{f3}\\\" O'Brien\\\\x\\t\",\n    \"groups\" : [\n\n    ],\n    \"hasImage\" : true,\n    \"id\" : \"a:ABPerson\",\n    \"urls\" : [\n      {\n        \"label\" : \"pplr\",\n        \"value\" : \"pplr:\\/\\/v\\/varga-zsofia\"\n      }\n    ]\n  }\n]";
    let value = json!([{
      "id": "a:ABPerson", "given": "Zs\u{f3}fia \"Zs\u{f3}\" O'Brien\\x\t", "emails": [],
      "urls": [{"label": "pplr", "value": "pplr://v/varga-zsofia"}], "groups": [], "hasImage": true
    }]);
    assert_eq!(to_swift_json(&value), swift);
    assert_eq!(to_swift_json(&json!([])), "[\n\n]");
  }

  #[test]
  fn control_characters_escape_as_swift_does() {
    // Swift: "a\u0001b\u001fc\bd\fe\rf\ng<DEL>h"
    let value = json!(["a\u{1}b\u{1f}c\u{8}d\u{c}e\rf\ng\u{7f}h"]);
    assert_eq!(
      to_swift_json(&value),
      "[\n  \"a\\u0001b\\u001fc\\bd\\fe\\rf\\ng\u{7f}h\"\n]"
    );
  }

  /// A dump file in its own temporary directory, removed on drop.
  struct Dump(PathBuf);

  impl Dump {
    fn new(name: &str, cards: Value) -> Result<Self> {
      let dir = std::env::temp_dir().join(format!("pplr-json-test-{}-{name}", std::process::id()));
      std::fs::create_dir_all(&dir)?;
      let file = dir.join("contacts.json");
      std::fs::write(&file, cards.to_string())?;
      Ok(Self(file))
    }
  }

  impl Drop for Dump {
    fn drop(&mut self) {
      if let Some(dir) = self.0.parent() {
        let _ = std::fs::remove_dir_all(dir);
      }
    }
  }

  fn card(id: &str, urls: Value) -> Value {
    json!({"id": id, "given": "Ada", "family": "Lovelace", "organization": "", "jobTitle": "",
           "emails": [], "phones": [], "urls": urls, "linkedin": [], "groups": [], "account": "home"})
  }

  #[test]
  fn link_adds_or_replaces_the_marker_and_joins_the_group() -> Result<()> {
    let dump = Dump::new(
      "link",
      json!([
        card("c1", json!([])),
        card("c2", json!([{"label": "PPLR", "value": "pplr://l/old"}]))
      ]),
    )?;
    let mut store = JsonStore::open(&dump.0)?;
    let add = MarkerWrite {
      url: "pplr://l/lovelace-ada".into(),
      replace: false,
    };
    let replace = MarkerWrite {
      url: "pplr://l/lovelace-ada".into(),
      replace: true,
    };
    store.link("c1", Some(&add), Some("PPLR"))?;
    store.link("c2", Some(&replace), None)?;
    store.link("absent", Some(&add), Some("PPLR"))?;
    let cards = store.cards()?;
    assert_eq!(cards[0].urls, vec![pplr_url("pplr://l/lovelace-ada")]);
    assert_eq!(cards[0].groups, vec!["PPLR".to_owned()]);
    assert_eq!(
      cards[1].urls,
      vec![LabelledValue {
        label: "PPLR".into(),
        value: "pplr://l/lovelace-ada".into()
      }]
    );
    assert!(cards[1].groups.is_empty());
    Ok(())
  }

  #[test]
  fn create_numbers_the_new_card_and_files_it_in_the_group() -> Result<()> {
    let dump = Dump::new("create", json!([card("c1", json!([]))]))?;
    let mut store = JsonStore::open(&dump.0)?;
    let new = NewCard {
      given: "Grace".into(),
      family: "Hopper".into(),
      linkedin: Some("ghopper".into()),
      marker: "pplr://h/hopper-grace".into(),
      ..NewCard::default()
    };
    let id = store.create(&new, "PPLR")?;
    let cards = store.cards()?;
    assert_eq!(id, format!("new-{}", cards.len()));
    let made = &cards[cards.len() - 1];
    let labels: Vec<&str> = made.urls.iter().map(|u| u.label.as_str()).collect();
    assert_eq!(labels, ["LinkedIn", "pplr"]);
    assert_eq!(
      (made.groups.clone(), made.account.clone(), made.has_image),
      (vec!["PPLR".to_owned()], Some("home".to_owned()), None)
    );
    Ok(())
  }

  #[test]
  fn update_and_photo_change_only_what_they_name_and_save_writes_swifts_bytes() -> Result<()> {
    let dump = Dump::new(
      "update",
      json!([card(
        "c1",
        json!([{"label": "pplr", "value": "pplr://l/old"}])
      )]),
    )?;
    let mut store = JsonStore::open(&dump.0)?;
    let change = CardChange {
      job_title: Some("Countess".into()),
      add_emails: vec!["ada@example.org".into()],
      marker: Some(MarkerWrite {
        url: "pplr://l/lovelace-ada".into(),
        replace: true,
      }),
      add_group: true,
      ..CardChange::default()
    };
    store.update("c1", &change, "PPLR")?;
    store.set_photo("c1", Path::new("/nowhere.jpg"))?;
    store.save()?;
    let written = std::fs::read_to_string(&dump.0)?;
    let cards: Vec<Card> = serde_json::from_str(&written)?;
    assert_eq!(written, to_swift_json(&serde_json::to_value(&cards)?));
    let c = &cards[0];
    assert_eq!(
      (c.job_title.as_str(), c.emails.clone(), c.has_image),
      ("Countess", vec!["ada@example.org".to_owned()], Some(true))
    );
    assert_eq!(
      (c.urls[0].value.as_str(), c.groups.clone(), c.given.as_str()),
      ("pplr://l/lovelace-ada", vec!["PPLR".to_owned()], "Ada")
    );
    Ok(())
  }

  #[test]
  fn backup_copies_the_file_as_it_stands_under_a_stamp() -> Result<()> {
    let dump = Dump::new("backup", json!([card("c1", json!([]))]))?;
    let mut store = JsonStore::open(&dump.0)?;
    store.link("c1", None, Some("PPLR"))?;
    let dir = dump.0.with_file_name("backups");
    let saved = store.backup(&dir)?;
    let name = saved
      .file_name()
      .map(|n| n.to_string_lossy().into_owned())
      .unwrap_or_default();
    assert!(
      name.ends_with("-contacts.json") && name.len() == "YYYYMMDD-HHMMSS-contacts.json".len(),
      "{name}"
    );
    assert_eq!(std::fs::read(&saved)?, std::fs::read(&dump.0)?);
    Ok(())
  }

  #[test]
  fn every_golden_contacts_file_reencodes_byte_for_byte() -> Result<()> {
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/cases");
    let mut seen = 0;
    for case in std::fs::read_dir(&cases)? {
      let file = case?.path().join("files/contacts.json");
      if !file.is_file() {
        continue;
      }
      let swift = std::fs::read_to_string(&file)?;
      let cards: Vec<Card> = serde_json::from_str(&swift)?;
      assert_eq!(
        to_swift_json(&serde_json::to_value(&cards)?),
        swift,
        "{}",
        file.display()
      );
      seen += 1;
    }
    assert!(
      seen > 0,
      "no golden contacts.json under {}",
      cases.display()
    );
    Ok(())
  }
}
