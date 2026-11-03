//! `pplr email` (ST0001 WP14): recent mail, or what is in the inbox,
//! numbered, each message marked with the person it is with when they are in
//! pplr. The listing is saved, so `pplr contact log --email N` knows message
//! N. `--json` prints the messages as a versioned document instead, the
//! contract other tools call, and leaves the saved listing alone. Both exit 0
//! with what they print, or 1: with the cause on stderr for no Mail store, no
//! Full Disk Access or a store whose schema pplr does not read, and with the
//! usage line for a usage error, as the legacy `pplr email` did.

use crate::Result;
use crate::config::Config;
use crate::mail::{self, Message, Query};
use crate::marker::marker_url;
use crate::normalise::percent_encode;
use crate::tree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "Usage: pplr email [recent|inbox] [--account TEXT] [--hours N] [--sent]";

/// What `pplr email -h` prints, and `pplr help` shows for email: the legacy
/// script's `# USAGE:` lines, which it printed for both.
pub const HELP: &str = "recent mail, or what is in the inbox, numbered: pplr contact log --email N logs message N\nrecent [--hours N]        what arrived in the last 24 hours (the default)\ninbox                     what is in the inbox now, whatever its age\n--account TEXT            only accounts whose address contains TEXT (eg work)\n--sent                    include sent mail\n--limit N                 the newest N (default 200)\n--json                    the messages as a versioned JSON document, for other tools";

/// The saved listing's file in the cache directory.
const CACHE: &str = "email-last.json";

/// One message of the saved listing, as `contact log --email N` reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listed {
  pub n: usize,
  pub date: String,
  pub message_id: String,
  pub subject: String,
  /// "in" or "out".
  pub direction: String,
  /// The person's key, eg "K/Kemp, Jon", or null.
  pub person: Option<String>,
}

/// `pplr email --json`'s document.
#[derive(Serialize)]
struct Document<'a> {
  schema: u32,
  messages: &'a [Message],
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let mut query = Query::default();
  let mut json = false;
  let mut it = args.iter();
  while let Some(arg) = it.next() {
    match arg.as_str() {
      "recent" => query.inbox = false,
      "inbox" => query.inbox = true,
      "--sent" => query.sent = true,
      "--json" => json = true,
      "--account" | "--hours" | "--limit" => {
        let Some(value) = it.next() else {
          println!("{USAGE}");
          return Ok(ExitCode::FAILURE);
        };
        match arg.as_str() {
          "--account" => query.account = Some(value.clone()),
          "--hours" => query.hours = value.parse().map_err(|e| format!("--hours {value}: {e}"))?,
          _ => query.limit = value.parse().map_err(|e| format!("--limit {value}: {e}"))?,
        }
      }
      "-h" | "--help" => {
        println!("{HELP}");
        return Ok(ExitCode::SUCCESS);
      }
      _ => {
        println!("{USAGE}");
        return Ok(ExitCode::FAILURE);
      }
    }
  }

  let home = config
    .home
    .as_deref()
    .ok_or("HOME is not set, so pplr cannot find Mail's store")?;
  let (index, accounts) = mail::stores(home);
  let now = config.now()?.timestamp().as_second();
  let messages = mail::read(&index, &accounts, &query, now)?;

  if json {
    let document = Document {
      schema: mail::SCHEMA,
      messages: &messages,
    };
    println!("{}", serde_json::to_string_pretty(&document)?);
    return Ok(ExitCode::SUCCESS);
  }

  let who = people_by_address(&config.data)?;
  let mut saved = Vec::new();
  for (i, m) in messages.into_iter().enumerate() {
    let (text, entry) = listing(i + 1, m, &who)?;
    print!("{text}");
    saved.push(entry);
  }
  save(&config.cache_dir, &saved)?;
  Ok(ExitCode::SUCCESS)
}

/// address -> (key, pplr:// URL), from the addresses on each person's About
/// Email line, as every command reads them (about.rs); an address on two
/// people's Abouts is the later one's.
fn people_by_address(root: &Path) -> Result<HashMap<String, (String, String)>> {
  let mut out = HashMap::new();
  for person in tree::people(root)? {
    let url = marker_url(&person.key);
    for address in person.header.emails {
      out.insert(address, (person.key.clone(), url.clone()));
    }
  }
  Ok(out)
}

/// Message n's four lines and a blank one, and its entry in the saved listing.
/// Mail from the account is "out", marked with its first recipient in pplr;
/// anything else is "in", marked with its sender.
fn listing(
  n: usize,
  m: Message,
  who: &HashMap<String, (String, String)>,
) -> Result<(String, Listed)> {
  let sent = m.address.to_lowercase() == m.account.to_lowercase();
  let candidates: &[String] = if sent {
    &m.to
  } else {
    std::slice::from_ref(&m.address)
  };
  let person = candidates.iter().find_map(|a| who.get(&a.to_lowercase()));
  // mail::local_time wrote the date in this zone, so this is its own time
  let when = m
    .date
    .parse::<jiff::Timestamp>()?
    .to_zoned(jiff::tz::TimeZone::system())
    .strftime("%a %d %b %H:%M")
    .to_string();
  let tag = person
    .map(|(_, url)| format!("   {url}"))
    .unwrap_or_default();
  let mut text = if sent {
    format!(
      "[{n}] {when}  from {}\n  To:   {}{tag}\n",
      m.account,
      m.to.join(", ")
    )
  } else {
    format!("[{n}] {when}  to {}\n  From: {}{tag}\n", m.account, m.from)
  };
  text.push_str(&format!(
    "  Subj: {}\n  ID:   {}\n\n",
    m.subject, m.message_id
  ));
  let entry = Listed {
    n,
    date: m.date,
    message_id: m.message_id,
    subject: m.subject,
    direction: if sent { "out" } else { "in" }.to_owned(),
    person: person.map(|(key, _)| key.clone()),
  };
  Ok((text, entry))
}

/// The listing in the cache, as the legacy listing wrote it: one-space
/// indents, non-ASCII as it is, no final newline.
fn save(cache_dir: &Path, saved: &[Listed]) -> Result<()> {
  std::fs::create_dir_all(cache_dir)?;
  let mut bytes = Vec::new();
  let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
  let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, formatter);
  saved.serialize(&mut serializer)?;
  crate::fs::write_atomic(&cache_dir.join(CACHE), &bytes)?;
  Ok(())
}

/// Message n of the last listing, for `contact log --email N`. Each error is
/// the line to print: no listing yet is the legacy contact's line, and a
/// listing pplr cannot read or parse names its file and why.
pub fn listed(cache_dir: &Path, n: usize) -> Result<Listed> {
  let path = cache_dir.join(CACHE);
  let unreadable = |e: &dyn std::fmt::Display| {
    format!(
      "Error: cannot read the last pplr email listing, {}: {e}",
      path.display()
    )
  };
  let bytes = match std::fs::read(&path) {
    Ok(bytes) => bytes,
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
      return Err("Error: no pplr email listing yet: run pplr email first".into());
    }
    Err(e) => return Err(unreadable(&e).into()),
  };
  let rows: Vec<Listed> = serde_json::from_slice(&bytes).map_err(|e| unreadable(&e))?;
  let count = rows.len();
  rows.into_iter().find(|r| r.n == n).ok_or_else(|| {
    format!("Error: the last pplr email listing has no message {n} (it has 1 to {count})").into()
  })
}

/// Mail's own link to a message, from its Message-ID with or without its
/// angle brackets, escaped as the legacy contact escaped it.
pub fn message_link(message_id: &str) -> String {
  let id = message_id.trim_matches(['<', '>']);
  format!("message://%3C{}%3E", percent_encode(id, b"@="))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn message(address: &str, to: &[&str]) -> Message {
    Message {
      date: "2026-06-15T08:12:00+01:00".to_owned(),
      account: "me@example.com".to_owned(),
      from: format!("Someone <{address}>"),
      address: address.to_owned(),
      to: to.iter().map(|s| (*s).to_owned()).collect(),
      subject: "Hello".to_owned(),
      message_id: "<m-1@example.com>".to_owned(),
    }
  }

  #[test]
  fn mail_in_is_marked_by_its_sender_and_mail_out_by_its_first_known_recipient() -> Result<()> {
    let who = HashMap::from([(
      "ada@example.org".to_owned(),
      (
        "L/Lovelace, Ada".to_owned(),
        "pplr://l/lovelace-ada".to_owned(),
      ),
    )]);
    let (text, entry) = listing(1, message("ADA@example.org", &["me@example.com"]), &who)?;
    assert!(
      text.contains("  From: Someone <ADA@example.org>   pplr://l/lovelace-ada\n"),
      "{text}"
    );
    assert_eq!(entry.direction, "in");
    assert_eq!(entry.person.as_deref(), Some("L/Lovelace, Ada"));

    let (text, entry) = listing(
      2,
      message("me@example.com", &["x@example.com", "Ada@example.org"]),
      &who,
    )?;
    assert!(text.starts_with("[2] "), "{text}");
    assert!(
      text.contains("  To:   x@example.com, Ada@example.org   pplr://l/lovelace-ada\n"),
      "{text}"
    );
    assert_eq!(entry.direction, "out");
    Ok(())
  }

  #[test]
  fn a_message_link_escapes_its_id_as_the_legacy_contact_did() {
    assert_eq!(
      message_link("<CAL=x@mail.gmail.com>"),
      "message://%3CCAL=x@mail.gmail.com%3E"
    );
    assert_eq!(message_link("a+b/c d@x"), "message://%3Ca%2Bb%2Fc%20d@x%3E");
  }
}
