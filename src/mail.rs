//! Mail's own index, read-only (ST0001 WP14): every query pplr makes of
//! Mail's store is here. Mail keeps no public framework for its mailboxes, so
//! pplr reads Mail's SQLite index, `~/Library/Mail/V10/MailData/Envelope
//! Index`, and the system's accounts store, `~/Library/Accounts/
//! Accounts4.sqlite`, as `mail-recent index` did. Both schemas are Apple's and
//! unversioned, so the tables and columns read here are checked first, and a
//! store that lacks one is refused by name rather than read for wrong rows.

use crate::Result;
use crate::normalise::percent_decode;
use rusqlite::{Connection, OpenFlags, params};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The `pplr email --json` document's version: it changes when a field does.
pub const SCHEMA: u32 = 1;

/// Each table read from the Envelope Index, and the columns read from it.
const INDEX_COLUMNS: &[(&str, &[&str])] = &[
  (
    "messages",
    &[
      "ROWID",
      "date_received",
      "sender",
      "subject",
      "global_message_id",
      "mailbox",
      "deleted",
    ],
  ),
  ("subjects", &["ROWID", "subject"]),
  ("addresses", &["ROWID", "address", "comment"]),
  ("message_global_data", &["ROWID", "message_id_header"]),
  ("mailboxes", &["ROWID", "url"]),
  ("labels", &["message_id", "mailbox_id"]),
  ("recipients", &["message", "address", "type", "position"]),
];

/// The accounts store's one table, and the columns read from it.
const ACCOUNT_COLUMNS: &[(&str, &[&str])] = &[(
  "ZACCOUNT",
  &["Z_PK", "ZIDENTIFIER", "ZUSERNAME", "ZPARENTACCOUNT"],
)];

/// Where Mail's index and the accounts store are, under a home directory.
pub fn stores(home: &Path) -> (PathBuf, PathBuf) {
  (
    home.join("Library/Mail/V10/MailData/Envelope Index"),
    home.join("Library/Accounts/Accounts4.sqlite"),
  )
}

/// What to list: the inbox now, else what arrived in the last `hours`.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
  pub inbox: bool,
  pub hours: f64,
  /// Only accounts whose address contains this, case-insensitively.
  pub account: Option<String>,
  /// Sent mailboxes too.
  pub sent: bool,
  /// The newest this many, listed oldest first.
  pub limit: usize,
}

impl Default for Query {
  fn default() -> Self {
    Self {
      inbox: false,
      hours: 24.0,
      account: None,
      sent: false,
      limit: 200,
    }
  }
}

/// One message, as `pplr email --json` documents it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Message {
  /// When it arrived, local time with its offset (RFC 3339).
  pub date: String,
  /// The address of the account it arrived in.
  pub account: String,
  /// "Name <address>", or the address alone.
  pub from: String,
  /// The sender's address.
  pub address: String,
  /// Every recipient's address: To, then Cc, then Bcc, each in order.
  pub to: Vec<String>,
  pub subject: String,
  /// The Message-ID header, angle brackets included, or empty.
  pub message_id: String,
}

/// A message row as the index holds it, before the mailbox rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
  pub rowid: i64,
  pub received: i64,
  pub name: Option<String>,
  pub address: Option<String>,
  pub subject: Option<String>,
  pub message_id: Option<String>,
  pub mailbox: i64,
}

/// A mailbox: its account's address (or id, when the accounts store does not
/// name it) and its name, eg `INBOX` or `[Gmail]/All Mail`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
  pub account: String,
  pub name: String,
}

/// The messages the query asks for, read from the two stores, never written.
pub fn read(index: &Path, accounts: &Path, query: &Query, now: i64) -> Result<Vec<Message>> {
  let index_db = open(index)?;
  let accounts_db = open(accounts)?;
  check(&index_db, index, INDEX_COLUMNS)?;
  check(&accounts_db, accounts, ACCOUNT_COLUMNS)?;

  let addresses = account_addresses(&accounts_db)?;
  let mut boxes = HashMap::new();
  let mut stmt = index_db.prepare("select ROWID, url from mailboxes")?;
  for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
    let (rowid, url) = row?;
    boxes.insert(rowid, mailbox(&url, &addresses));
  }

  let cutoff = if query.inbox {
    0
  } else {
    now - (query.hours * 3600.0) as i64
  };
  let mut labels: HashMap<i64, HashSet<i64>> = HashMap::new();
  let mut stmt = index_db.prepare(
    "select l.message_id, l.mailbox_id from labels l join messages m on m.ROWID = l.message_id \
     where m.date_received >= ?1",
  )?;
  for row in stmt.query_map(params![cutoff], |r| Ok((r.get(0)?, r.get(1)?)))? {
    let (message, mailbox) = row?;
    labels.entry(message).or_default().insert(mailbox);
  }

  let mut stmt = index_db.prepare(
    "select m.ROWID, m.date_received, a.comment, a.address, s.subject, g.message_id_header, m.mailbox \
     from messages m join subjects s on s.ROWID = m.subject \
     left join addresses a on a.ROWID = m.sender \
     left join message_global_data g on g.ROWID = m.global_message_id \
     where m.deleted = 0 and m.date_received >= ?1 order by m.ROWID",
  )?;
  let rows = stmt
    .query_map(params![cutoff], |r| {
      Ok(Row {
        rowid: r.get(0)?,
        received: r.get(1)?,
        name: r.get(2)?,
        address: r.get(3)?,
        subject: r.get(4)?,
        message_id: r.get(5)?,
        mailbox: r.get(6)?,
      })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;

  let mut stmt = index_db.prepare(
    "select a.address from recipients r join addresses a on a.ROWID = r.address \
     where r.message = ?1 order by r.type, r.position",
  )?;
  let mut out = Vec::new();
  for (row, account) in select(rows, &boxes, &labels, query) {
    let to = stmt
      .query_map(params![row.rowid], |r| r.get::<_, String>(0))?
      .collect::<rusqlite::Result<Vec<_>>>()?;
    out.push(message(row, account, to)?);
  }
  Ok(out)
}

/// The setting a process needs before macOS lets it read Mail's stores.
pub const FULL_DISK_ACCESS: &str = "give the app running pplr Full Disk Access \
  (System Settings > Privacy & Security > Full Disk Access)";

/// Whether this process can read a store, as `pplr email` and doctor ask it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Access {
  Readable,
  /// No store there: no Mail on this machine, or none under this HOME.
  Missing,
  /// A store this process may not read, for want of Full Disk Access.
  Denied,
  /// Opening it failed some other way, with the reason.
  Failed(String),
}

pub fn access(path: &Path) -> Access {
  match std::fs::File::open(path) {
    Ok(_) => Access::Readable,
    Err(e) => match e.kind() {
      std::io::ErrorKind::NotFound => Access::Missing,
      std::io::ErrorKind::PermissionDenied => Access::Denied,
      _ => Access::Failed(e.to_string()),
    },
  }
}

/// The store opened read-only, or why it cannot be: no store at the path it
/// looked in, or a process without Full Disk Access.
fn open(path: &Path) -> Result<Connection> {
  let why = match access(path) {
    Access::Readable => None,
    Access::Missing => Some(format!(
      "no Mail store at {} (pplr email reads macOS Mail's own index)",
      path.display()
    )),
    Access::Denied => Some(format!(
      "cannot read {}: {FULL_DISK_ACCESS}",
      path.display()
    )),
    Access::Failed(e) => Some(format!("cannot read {}: {e}", path.display())),
  };
  if let Some(why) = why {
    return Err(why.into());
  }
  let db = Connection::open_with_flags(
    path,
    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
  )?;
  db.pragma_update(None, "query_only", true)?;
  Ok(db)
}

/// Refuses a store that lacks a table or column pplr reads, naming each.
fn check(db: &Connection, path: &Path, want: &[(&str, &[&str])]) -> Result<()> {
  let mut have = HashMap::new();
  for (table, _) in want {
    let mut stmt = db.prepare("select name from pragma_table_info(?1)")?;
    let columns = stmt
      .query_map(params![table], |r| r.get::<_, String>(0))?
      .collect::<rusqlite::Result<HashSet<_>>>()?;
    have.insert(*table, columns);
  }
  let missing = missing(&have, want);
  if missing.is_empty() {
    Ok(())
  } else {
    Err(
      format!(
        "{} is not the schema pplr reads (missing {}), so pplr reads nothing from it",
        path.display(),
        missing.join(", ")
      )
      .into(),
    )
  }
}

/// Each wanted `table.column` the store does not have. A table with no
/// columns is missing; ROWID is every ordinary table's, so it is never listed.
pub fn missing(have: &HashMap<&str, HashSet<String>>, want: &[(&str, &[&str])]) -> Vec<String> {
  let mut out = Vec::new();
  for (table, columns) in want {
    match have.get(table).filter(|c| !c.is_empty()) {
      None => out.push((*table).to_owned()),
      Some(got) => out.extend(
        columns
          .iter()
          .filter(|c| **c != "ROWID" && !got.contains(**c))
          .map(|c| format!("{table}.{c}")),
      ),
    }
  }
  out
}

/// Mail's account id -> its address: the account's own user name, or for a
/// child account (a mail account under an internet account) its parent's.
fn account_addresses(db: &Connection) -> Result<HashMap<String, String>> {
  let mut stmt = db.prepare(
    "select c.ZIDENTIFIER, coalesce(p.ZUSERNAME, c.ZUSERNAME) from ZACCOUNT c \
     left join ZACCOUNT p on p.Z_PK = c.ZPARENTACCOUNT",
  )?;
  let rows = stmt.query_map([], |r| {
    Ok((
      r.get::<_, Option<String>>(0)?,
      r.get::<_, Option<String>>(1)?,
    ))
  })?;
  let mut out = HashMap::new();
  for row in rows {
    if let (Some(id), Some(user)) = row?
      && !id.is_empty()
      && !user.is_empty()
    {
      out.insert(id, user);
    }
  }
  Ok(out)
}

/// A mailbox from its URL, `<scheme>://<account id>/<name, %-escaped>`.
pub fn mailbox(url: &str, addresses: &HashMap<String, String>) -> Mailbox {
  let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
  let (id, name) = rest.split_once('/').unwrap_or((rest, ""));
  Mailbox {
    account: addresses.get(id).cloned().unwrap_or_else(|| id.to_owned()),
    name: percent_decode(name).unwrap_or_else(|| name.to_owned()),
  }
}

/// Junk, Trash, Deleted, Drafts, Spam and Bin, whatever the account calls them.
fn skipped(name: &str) -> bool {
  let name = name.to_lowercase();
  ["junk", "trash", "deleted", "drafts", "spam"]
    .iter()
    .any(|w| name.contains(w))
    || name.ends_with("bin")
}

/// The rows the query keeps, each with its account, oldest first: by account,
/// never in a skipped mailbox, in the inbox when it asks, Sent only when it
/// asks, one per Message-ID (the first in the index), the newest `limit`.
pub fn select(
  rows: Vec<Row>,
  boxes: &HashMap<i64, Mailbox>,
  labels: &HashMap<i64, HashSet<i64>>,
  query: &Query,
) -> Vec<(Row, String)> {
  let unknown = Mailbox {
    account: "?".to_owned(),
    name: String::new(),
  };
  let needle = query.account.as_deref().map(str::to_lowercase);
  let mut seen = HashSet::new();
  let mut kept = Vec::new();
  for row in rows {
    let mut where_ = vec![row.mailbox];
    where_.extend(labels.get(&row.rowid).into_iter().flatten());
    let names: Vec<&str> = where_
      .iter()
      .map(|b| boxes.get(b).unwrap_or(&unknown).name.as_str())
      .collect();
    let account = boxes.get(&row.mailbox).unwrap_or(&unknown).account.clone();
    if needle
      .as_ref()
      .is_some_and(|n| !account.to_lowercase().contains(n))
      || names.iter().any(|n| skipped(n))
      || (query.inbox && !names.iter().any(|n| n.eq_ignore_ascii_case("INBOX")))
      || (!query.sent && names.iter().any(|n| n.to_lowercase().contains("sent")))
    {
      continue;
    }
    let key = row
      .message_id
      .clone()
      .filter(|m| !m.is_empty())
      .unwrap_or_else(|| format!("row{}", row.rowid));
    if seen.insert(key) {
      kept.push((row, account));
    }
  }
  kept.sort_by_key(|(row, _)| row.received);
  let skip = kept.len().saturating_sub(query.limit);
  kept.split_off(skip)
}

fn message(row: Row, account: String, to: Vec<String>) -> Result<Message> {
  let address = row.address.unwrap_or_default();
  let from = match row.name.filter(|n| !n.is_empty()) {
    Some(name) => format!("{name} <{address}>"),
    None => address.clone(),
  };
  Ok(Message {
    date: local_time(row.received)?,
    account,
    from,
    address,
    to,
    subject: row.subject.unwrap_or_default(),
    message_id: row.message_id.unwrap_or_default(),
  })
}

/// Seconds since the epoch as local time with its offset, eg
/// `2026-06-15T08:12:00+01:00`.
pub fn local_time(seconds: i64) -> Result<String> {
  let zoned = jiff::Timestamp::from_second(seconds)?.to_zoned(jiff::tz::TimeZone::system());
  Ok(zoned.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string())
}

#[cfg(test)]
mod tests {
  //! The mailbox rules here, over rows as the index holds them; reading a
  //! synthetic store end to end is tests/cli.rs's, through `pplr email`.
  use super::*;

  fn row(rowid: i64, received: i64, message_id: &str, mailbox: i64) -> Row {
    Row {
      rowid,
      received,
      name: None,
      address: Some("someone@example.org".to_owned()),
      subject: None,
      message_id: Some(message_id.to_owned()),
      mailbox,
    }
  }

  /// Mailboxes 1 to 6 are me@example.com's, and 7 is work@example.org's INBOX.
  fn boxes() -> HashMap<i64, Mailbox> {
    let mine = |name: &str| Mailbox {
      account: "me@example.com".to_owned(),
      name: name.to_owned(),
    };
    HashMap::from([
      (1, mine("INBOX")),
      (2, mine("[Gmail]/All Mail")),
      (3, mine("Sent Messages")),
      (4, mine("[Gmail]/Spam")),
      (5, mine("Archive")),
      (6, mine("Bin")),
      (
        7,
        Mailbox {
          account: "work@example.org".to_owned(),
          name: "INBOX".to_owned(),
        },
      ),
    ])
  }

  fn kept(rows: Vec<Row>, labels: &[(i64, i64)], query: &Query) -> Vec<i64> {
    let mut by_message: HashMap<i64, HashSet<i64>> = HashMap::new();
    for (message, mailbox) in labels {
      by_message.entry(*message).or_default().insert(*mailbox);
    }
    select(rows, &boxes(), &by_message, query)
      .into_iter()
      .map(|(row, _)| row.rowid)
      .collect()
  }

  #[test]
  fn a_message_in_a_skipped_mailbox_goes_even_when_another_mailbox_holds_it() {
    let rows = vec![
      row(1, 10, "<a>", 1),
      row(2, 20, "<b>", 4),
      row(3, 30, "<c>", 2),
      row(4, 40, "<d>", 6),
    ];
    assert_eq!(kept(rows, &[(3, 4)], &Query::default()), [1]);
  }

  #[test]
  fn sent_mail_is_listed_only_when_asked_for() {
    let rows = || vec![row(1, 10, "<a>", 1), row(2, 20, "<b>", 3)];
    assert_eq!(kept(rows(), &[], &Query::default()), [1]);
    let sent = Query {
      sent: true,
      ..Query::default()
    };
    assert_eq!(kept(rows(), &[], &sent), [1, 2]);
  }

  #[test]
  fn the_inbox_is_what_an_inbox_holds_by_mailbox_or_by_label() {
    let rows = vec![
      row(1, 10, "<a>", 1),
      row(2, 20, "<b>", 2),
      row(3, 30, "<c>", 5),
      row(4, 40, "<d>", 7),
    ];
    let inbox = Query {
      inbox: true,
      ..Query::default()
    };
    assert_eq!(kept(rows, &[(2, 1)], &inbox), [1, 2, 4]);
  }

  #[test]
  fn account_keeps_the_accounts_whose_address_holds_its_text_in_any_case() {
    let rows = vec![row(1, 10, "<a>", 1), row(2, 20, "<b>", 7)];
    let work = Query {
      account: Some("WORK".to_owned()),
      ..Query::default()
    };
    assert_eq!(kept(rows, &[], &work), [2]);
  }

  #[test]
  fn a_message_id_is_listed_once_from_its_first_row_and_a_row_with_none_stands_alone() {
    let rows = vec![
      row(1, 10, "<a>", 2),
      row(2, 10, "<a>", 5),
      row(3, 30, "", 1),
      row(4, 40, "", 1),
    ];
    assert_eq!(kept(rows, &[], &Query::default()), [1, 3, 4]);
  }

  #[test]
  fn the_listing_is_oldest_first_and_limit_keeps_the_newest() {
    let rows = || {
      vec![
        row(1, 30, "<a>", 1),
        row(2, 10, "<b>", 1),
        row(3, 20, "<c>", 1),
      ]
    };
    assert_eq!(kept(rows(), &[], &Query::default()), [2, 3, 1]);
    let two = Query {
      limit: 2,
      ..Query::default()
    };
    assert_eq!(kept(rows(), &[], &two), [3, 1]);
  }

  #[test]
  fn a_missing_table_is_named_and_rowid_is_never_asked_for() {
    let mut have = HashMap::new();
    have.insert("subjects", HashSet::from(["subject".to_owned()]));
    assert_eq!(
      missing(
        &have,
        &[
          ("subjects", &["ROWID", "subject"]),
          ("labels", &["mailbox_id"])
        ]
      ),
      ["labels"]
    );
    have.insert("labels", HashSet::from(["message_id".to_owned()]));
    assert_eq!(
      missing(&have, &[("labels", &["message_id", "mailbox_id"])]),
      ["labels.mailbox_id"]
    );
  }

  #[test]
  fn a_mailbox_url_names_its_account_and_its_unescaped_name() {
    let addresses = HashMap::from([("ID-1".to_owned(), "me@example.com".to_owned())]);
    assert_eq!(
      mailbox("imap://ID-1/%5BGmail%5D/All%20Mail", &addresses),
      Mailbox {
        account: "me@example.com".to_owned(),
        name: "[Gmail]/All Mail".to_owned()
      }
    );
    assert_eq!(mailbox("ews://ID-2", &addresses).account, "ID-2");
  }

  #[test]
  fn skipped_mailboxes_are_matched_as_mail_recent_matched_them() {
    for name in [
      "Junk",
      "[Gmail]/Trash",
      "Deleted Messages",
      "Drafts",
      "[Gmail]/Spam",
      "Bin",
    ] {
      assert!(skipped(name), "{name}");
    }
    for name in ["INBOX", "Binders", "Sent Messages", "[Gmail]/All Mail"] {
      assert!(!skipped(name), "{name}");
    }
  }

  /// Reads this Mac's own store, read-only, on demand: `cargo test --lib
  /// mail -- --ignored`. It checks the shape of what it reads and prints none
  /// of it (D11).
  #[test]
  #[ignore = "reads this Mac's own Mail store; needs Full Disk Access"]
  fn the_live_store_reads() -> Result<()> {
    let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
    let (index, accounts) = stores(Path::new(&home));
    let inbox = Query {
      inbox: true,
      ..Query::default()
    };
    for m in read(
      &index,
      &accounts,
      &inbox,
      jiff::Timestamp::now().as_second(),
    )? {
      assert!(
        m.date.parse::<jiff::Timestamp>().is_ok(),
        "a date does not parse"
      );
      assert!(!m.account.is_empty(), "a message has no account");
    }
    Ok(())
  }
}
