//! The Contacts.app scripts that make every live write, as the engine wrote
//! them. The Contacts framework refused to update a card from a command-line
//! tool (CoreData, Cocoa error 134092); the app takes the same ids.

use super::{CardChange, NewCard, linkedin_url};
use std::path::Path;

/// Run with: card id, the pplr URL or "", the group id or "", "yes" to replace.
pub const LINK: &str = r#"on run argv
    set pid to item 1 of argv
    set marker to item 2 of argv
    set gid to item 3 of argv
    set replacing to item 4 of argv
    tell application "Contacts"
        set p to person id pid
        if marker is not "" then
            if replacing is "yes" then
                set value of (first url of p whose label is "pplr") to marker
            else
                make new url at end of urls of p with properties {label:"pplr", value:marker}
            end if
        end if
        if gid is not "" then add p to group id gid
        save
    end tell
end run"#;

/// A string as an AppleScript literal.
pub fn as_text(s: &str) -> String {
  format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn add_email(e: &str) -> String {
  format!(
    "make new email at end of emails of p with properties {{label:\"work\", value:{}}}",
    as_text(e)
  )
}

fn add_phone(ph: &str) -> String {
  format!(
    "make new phone at end of phones of p with properties {{label:\"mobile\", value:{}}}",
    as_text(ph)
  )
}

fn add_url(label: &str, value: &str) -> String {
  format!(
    "make new url at end of urls of p with properties {{label:\"{label}\", value:{}}}",
    as_text(value)
  )
}

fn set_image(tiff: &Path) -> String {
  format!(
    "set image of p to (read (POSIX file {}) as TIFF picture)",
    as_text(&tiff.to_string_lossy())
  )
}

/// A new card in the group; the script returns its id.
pub fn create(card: &NewCard, group_id: &str, tiff: Option<&Path>) -> String {
  let mut props = vec![
    format!("first name:{}", as_text(&card.given)),
    format!("last name:{}", as_text(&card.family)),
  ];
  if !card.organization.is_empty() {
    props.push(format!("organization:{}", as_text(&card.organization)));
  }
  if !card.job_title.is_empty() {
    props.push(format!("job title:{}", as_text(&card.job_title)));
  }
  let mut lines = vec!["tell application \"Contacts\"".to_owned()];
  lines.push(format!(
    "set p to make new person with properties {{{}}}",
    props.join(", ")
  ));
  lines.extend(card.emails.iter().map(|e| add_email(e)));
  lines.extend(card.phones.iter().map(|ph| add_phone(ph)));
  if let Some(slug) = &card.linkedin {
    lines.push(add_url("LinkedIn", &linkedin_url(slug)));
  }
  lines.push(add_url("pplr", &card.marker));
  if let Some(tiff) = tiff {
    lines.push(set_image(tiff));
  }
  lines.push("save".to_owned());
  lines.push(format!("add p to group id {}", as_text(group_id)));
  finish(lines)
}

/// Changes to an existing card; the script returns its id.
pub fn update(id: &str, change: &CardChange, group_id: &str) -> String {
  let mut lines = vec![
    "tell application \"Contacts\"".to_owned(),
    format!("set p to person id {}", as_text(id)),
  ];
  if let Some((given, family)) = &change.name {
    lines.push(format!("set first name of p to {}", as_text(given)));
    lines.push(format!("set last name of p to {}", as_text(family)));
  }
  if let Some(organization) = &change.organization {
    lines.push(format!(
      "set organization of p to {}",
      as_text(organization)
    ));
  }
  if let Some(job_title) = &change.job_title {
    lines.push(format!("set job title of p to {}", as_text(job_title)));
  }
  lines.extend(change.add_emails.iter().map(|e| add_email(e)));
  lines.extend(change.add_phones.iter().map(|ph| add_phone(ph)));
  if let Some(slug) = &change.add_linkedin {
    lines.push(add_url("LinkedIn", &linkedin_url(slug)));
  }
  match &change.marker {
    Some(marker) if marker.replace => lines.push(format!(
      "set value of (first url of p whose label is \"pplr\") to {}",
      as_text(&marker.url)
    )),
    Some(marker) => lines.push(add_url("pplr", &marker.url)),
    None => {}
  }
  if change.add_group {
    lines.push(format!("add p to group id {}", as_text(group_id)));
  }
  finish(lines)
}

fn finish(mut lines: Vec<String>) -> String {
  lines.extend(["save", "return id of p", "end tell"].map(str::to_owned));
  lines.join("\n")
}

/// A card's photo, from a TIFF.
pub fn photo(id: &str, tiff: &Path) -> String {
  format!(
    "tell application \"Contacts\"\nset p to person id {}\n{}\nsave\nend tell",
    as_text(id),
    set_image(tiff)
  )
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::contacts::MarkerWrite;
  use std::path::PathBuf;

  #[test]
  fn text_literals_escape_backslashes_and_quotes() {
    assert_eq!(as_text(r#"O"Brien\x"#), r#""O\"Brien\\x""#);
  }

  #[test]
  fn a_new_card_script_matches_the_engines() {
    let card = NewCard {
      given: "Ada".into(),
      family: "Lovelace".into(),
      organization: "Engines".into(),
      job_title: String::new(),
      emails: vec!["ada@example.org".into()],
      phones: vec!["+447700900001".into()],
      linkedin: Some("ada-l".into()),
      marker: "pplr://l/lovelace-ada".into(),
      picture: None,
    };
    let expected = [
      "tell application \"Contacts\"",
      "set p to make new person with properties {first name:\"Ada\", last name:\"Lovelace\", organization:\"Engines\"}",
      "make new email at end of emails of p with properties {label:\"work\", value:\"ada@example.org\"}",
      "make new phone at end of phones of p with properties {label:\"mobile\", value:\"+447700900001\"}",
      "make new url at end of urls of p with properties {label:\"LinkedIn\", value:\"https://www.linkedin.com/in/ada-l/\"}",
      "make new url at end of urls of p with properties {label:\"pplr\", value:\"pplr://l/lovelace-ada\"}",
      "set image of p to (read (POSIX file \"/t/p.tiff\") as TIFF picture)",
      "save",
      "add p to group id \"G1\"",
      "save",
      "return id of p",
      "end tell",
    ];
    assert_eq!(
      create(&card, "G1", Some(&PathBuf::from("/t/p.tiff"))),
      expected.join("\n")
    );
  }

  #[test]
  fn an_update_script_names_only_what_changes() {
    let change = CardChange {
      job_title: Some("CTO".into()),
      marker: Some(MarkerWrite {
        url: "pplr://l/lovelace-ada".into(),
        replace: true,
      }),
      add_group: true,
      ..CardChange::default()
    };
    let expected = [
      "tell application \"Contacts\"",
      "set p to person id \"c1:ABPerson\"",
      "set job title of p to \"CTO\"",
      "set value of (first url of p whose label is \"pplr\") to \"pplr://l/lovelace-ada\"",
      "add p to group id \"G1\"",
      "save",
      "return id of p",
      "end tell",
    ];
    assert_eq!(update("c1:ABPerson", &change, "G1"), expected.join("\n"));
  }
}
