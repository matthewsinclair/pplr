//! `pplr contact`: when you last spoke to each person, and when to get in
//! touch next, in About/contact.yaml (the port of contact.py).
//!
//! ```text
//! last: {date: 2026-01-28, via: meeting, link: "pplr://b/bray-martin/Meetings/20260128 Catch-up"}
//! next: 2026-04-28
//! cadence: 90
//! ```
//!
//! Who is on the roster, and how often, comes from the person's tags and
//! _pplr/cadence.yaml (`roster:` defaults, `every:` cadence tags overriding);
//! a cadence in contact.yaml overrides both. A Meetings folder dated after
//! today is booked: that person is not due.

use crate::Result;
use crate::config::Config;
use crate::marker::marker_url;
use crate::tags::read_tags;
use crate::tree::{display_name, letter_of, names, person_dirs};
use crate::yaml::{self, Value};
use jiff::ToSpan;
use jiff::civil::Date;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VIAS: [&str; 4] = ["meeting", "email", "call", "message"];
/// The lines of a meeting's notes `context` shows.
const CONTEXT_LINES: usize = 80;
/// Days in each unit of `next +N[dwm]`.
const DAYS_PER: [(&str, i64); 3] = [("d", 1), ("w", 7), ("m", 30)];
/// What `pplr contact` with no command prints: the usage contact.py printed.
const USAGE: &str = "
    contact.py scan [--dry-run]     last from each person's newest meeting, and next from the cadence
    contact.py due [DAYS] [--all]   who to get in touch with: next on or before today (plus DAYS);
                                    --all lists everyone with a next date
    contact.py show PERSON          a person's contact.yaml, with their cadence
    contact.py log [PERSON] [DATE] [--via email|call|message|meeting] [--email N|MESSAGE-ID]
                   [--link URL] [--note TEXT]
                                    record a contact (DATE defaults to today); next moves on;
                                    --email links the message: N is its number in the last
                                    pplr email listing, which also gives the date, subject and
                                    person (the sender, or who you sent it to), so PERSON can go
    contact.py render               About/<First Surname> (Contact).md from contact.yaml, for everyone
    contact.py next PERSON WHEN     set next: a date (2026-11-01) or from today (+2w, +10d, +3m)
    contact.py context PERSON       what to write from: role, latest update, last contact and its notes
";

/// A person's contact.yaml as contact.py held it.
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
  /// The last contact's keys in the file's order, its date read as a date.
  pub last: Option<Vec<(String, Value)>>,
  pub next: Option<Date>,
  pub cadence: Value,
}

impl Contact {
  fn last_get(&self, key: &str) -> Option<&Value> {
    self
      .last
      .as_ref()?
      .iter()
      .find(|(k, _)| k == key)
      .map(|(_, v)| v)
  }

  fn last_date(&self) -> Option<Date> {
    match self.last_get("date") {
      Some(Value::Date(d)) => d.parse().ok(),
      _ => None,
    }
  }

  /// `last.get('via', 'meeting')`.
  fn via(&self) -> String {
    self
      .last_get("via")
      .map_or_else(|| "meeting".to_owned(), Value::py_str)
  }

  /// A value of last's that Python would find truthy, as text.
  fn last_text(&self, key: &str) -> Option<String> {
    self
      .last_get(key)
      .filter(|v| v.is_truthy())
      .map(Value::py_str)
  }
}

/// `as_date`: a date as it is, anything else read as YYYY-MM-DD.
fn as_date(v: &Value, file: &Path) -> Result<Option<Date>> {
  match v {
    v if !v.is_truthy() => Ok(None),
    Value::Date(d) | Value::Str(d) => {
      Ok(Some(d.parse().map_err(|e| {
        format!("{}: '{d}' is not a date: {e}", file.display())
      })?))
    }
    other => Err(format!("{}: '{}' is not a date", file.display(), other.py_str()).into()),
  }
}

fn contact_file(dir: &Path) -> PathBuf {
  dir.join("About/contact.yaml")
}

pub fn read_contact(dir: &Path) -> Result<Contact> {
  let file = contact_file(dir);
  let doc = match fs::read_to_string(&file) {
    Ok(text) => yaml::parse(&text).map_err(|e| format!("{}: {e}", file.display()))?,
    Err(_) => Value::Null,
  };
  let last = match doc.get("last").filter(|l| l.is_truthy()) {
    Some(Value::Map(pairs)) => {
      let mut out = Vec::new();
      for (k, v) in pairs {
        let k = k.py_str();
        let v = if k == "date" {
          as_date(v, &file)?.map_or(Value::Null, |d| Value::Date(d.to_string()))
        } else {
          v.clone()
        };
        out.push((k, v));
      }
      if !out.iter().any(|(k, _)| k == "date") {
        out.push(("date".to_owned(), Value::Null));
      }
      Some(out)
    }
    _ => None,
  };
  Ok(Contact {
    last,
    next: as_date(doc.get("next").unwrap_or(&Value::Null), &file)?,
    cadence: doc.get("cadence").cloned().unwrap_or(Value::Null),
  })
}

/// contact.yaml from contact.py's own emitter, then its (Contact).md.
fn write_contact(dir: &Path, c: &Contact, defaults: &Defaults, today: Date) -> Result<()> {
  let mut lines = vec![
    format!(
      "# Contact with {}: last spoke, and when to get in touch next.",
      display_name(dir)
    ),
    "# pplr contact scan fills last from Meetings; pplr contact log records an email or call."
      .to_owned(),
  ];
  if c.last.is_some() {
    let date = c.last_get("date").map(Value::py_str).unwrap_or_default();
    let extra: String = ["link", "note"]
      .iter()
      .filter_map(|k| {
        let v = c.last_get(k).filter(|v| v.is_truthy())?;
        Some(format!(
          ", {k}: {}",
          serde_json::to_string(&v.py_str()).unwrap_or_default()
        ))
      })
      .collect();
    lines.push(format!("last: {{date: {date}, via: {}{extra}}}", c.via()));
  }
  if let Some(next) = c.next {
    lines.push(format!("next: {next}"));
  }
  if c.cadence.is_truthy() {
    lines.push(format!("cadence: {}", c.cadence.py_str()));
  }
  fs::write(contact_file(dir), lines.join("\n") + "\n")?;
  render_one(dir, c, defaults, today)?;
  Ok(())
}

/// "[label](target)", angle-bracketed when the target holds a space, ( ) < or >.
fn md_link(label: &str, target: &str) -> String {
  if target.contains([' ', '(', ')', '<', '>']) {
    format!("[{label}](<{target}>)")
  } else {
    format!("[{label}]({target})")
  }
}

/// "4 Mar 2026", as `f"{date.day} {date:%b %Y}"` wrote it.
fn nice(date: Date) -> String {
  format!("{} {}", date.day(), date.strftime("%b %Y"))
}

/// The roster's and the cadence tags' days, from _pplr/cadence.yaml.
pub struct Defaults {
  roster: Vec<(String, i64)>,
  every: Vec<(String, i64)>,
}

impl Defaults {
  pub fn load(root: &Path) -> Result<Self> {
    let file = root.join("_pplr/cadence.yaml");
    let doc = match fs::read_to_string(&file) {
      Ok(text) => yaml::parse(&text).map_err(|e| format!("{}: {e}", file.display()))?,
      Err(_) => Value::Null,
    };
    let kind = |name: &str| -> Result<Vec<(String, i64)>> {
      doc
        .get(name)
        .map(Value::pairs)
        .unwrap_or_default()
        .iter()
        .map(|(tag, days)| {
          Ok((
            tag.py_str(),
            int(days).ok_or_else(|| {
              format!(
                "{}: {name}: '{}' is not a number of days",
                file.display(),
                days.py_str()
              )
            })?,
          ))
        })
        .collect()
    };
    Ok(Self {
      roster: kind("roster")?,
      every: kind("every")?,
    })
  }
}

/// Python's int() of a YAML value.
fn int(v: &Value) -> Option<i64> {
  match v {
    Value::Int(i) => Some(*i),
    Value::Bool(b) => Some(i64::from(*b)),
    Value::Float(f) if f.is_finite() => Some(f.trunc() as i64),
    Value::Str(s) => s.trim().parse().ok(),
    _ => None,
  }
}

/// (days, why): the file's own cadence, else a cadence tag, else the
/// shortest roster tag's.
fn cadence_of(dir: &Path, c: &Contact, defaults: &Defaults) -> Result<(Option<i64>, String)> {
  if c.cadence.is_truthy() {
    let days = int(&c.cadence).ok_or_else(|| {
      format!(
        "{}: cadence '{}' is not a number",
        dir.display(),
        c.cadence.py_str()
      )
    })?;
    return Ok((Some(days), "contact.yaml".to_owned()));
  }
  let tags: Vec<String> = read_tags(dir)?.into_iter().map(|t| t.tag).collect();
  for kind in [&defaults.every, &defaults.roster] {
    let mut hits: Vec<(i64, &String)> = kind
      .iter()
      .filter(|(tag, _)| tags.contains(tag))
      .map(|(tag, days)| (*days, tag))
      .collect();
    hits.sort();
    if let Some((days, tag)) = hits.first() {
      return Ok((Some(*days), format!("#{tag}")));
    }
  }
  Ok((None, String::new()))
}

/// The dated meeting folders, oldest first: a name starting YYYYMMDD and a
/// word boundary, with a real date.
fn meetings(dir: &Path) -> Vec<(Date, String)> {
  let meetings = dir.join("Meetings");
  let mut out: Vec<(Date, String)> = names(&meetings)
    .into_iter()
    .filter(|n| !n.starts_with('.') && meetings.join(n).is_dir())
    .filter_map(|n| {
      let digits = n
        .get(..8)
        .filter(|d| d.bytes().all(|b| b.is_ascii_digit()))?;
      let boundary = n[8..]
        .chars()
        .next()
        .is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
      if !boundary {
        return None;
      }
      let date = Date::new(
        digits[..4].parse().ok()?,
        digits[4..6].parse().ok()?,
        digits[6..].parse().ok()?,
      )
      .ok()?;
      Some((date, n))
    })
    .collect();
  out.sort();
  out
}

fn booked(dir: &Path, today: Date) -> Option<(Date, String)> {
  meetings(dir).into_iter().find(|(d, _)| *d > today)
}

/// next from last and the cadence; with no contact yet, today.
fn moved_on(c: &Contact, days: Option<i64>, today: Date) -> Result<Option<Date>> {
  let Some(days) = days else {
    return Ok(c.next);
  };
  match c.last_date() {
    Some(last) => Ok(Some(last.checked_add(days.days())?)),
    None => Ok(Some(today)),
  }
}

fn since(date: Option<Date>, today: Date) -> String {
  match date {
    Some(d) => format!("{} days ago", (today - d).get_days()),
    None => "never".to_owned(),
  }
}

/// About/<First Surname> (Contact).md, contact.yaml to read; true if it changed.
fn render_one(dir: &Path, c: &Contact, defaults: &Defaults, today: Date) -> Result<bool> {
  let (days, why) = cadence_of(dir, c, defaults)?;
  let mut lines = vec![
    "<!-- Generated from contact.yaml by pplr contact: edit contact.yaml, not this file. -->"
      .to_owned(),
    format!("# Contact: {}", display_name(dir)),
    String::new(),
  ];
  if c.last.is_some() {
    let mut target = c.last_text("link").unwrap_or_default();
    let meeting = Regex::new(r"^pplr://[a-z]/[^/]+/(Meetings/.+)$")?
      .captures(&target)
      .and_then(|m| m.get(1))
      .map(|g| format!("../{}/", g.as_str().trim_end_matches('/')));
    if let Some(m) = &meeting {
      target = m.clone();
    }
    let date = c.last_date().map(nice).unwrap_or_else(|| "None".to_owned());
    let what = format!("{date}, {}", c.via());
    let tail = if target.is_empty() {
      String::new()
    } else {
      let label = if meeting.is_some() {
        target
          .trim_end_matches('/')
          .rsplit('/')
          .next()
          .unwrap_or_default()
          .to_owned()
      } else if target.starts_with("message:") {
        "the email".to_owned()
      } else {
        "link".to_owned()
      };
      format!(": {}", md_link(&label, &target))
    };
    lines.push(format!("- Last contact: {what}{tail}"));
    if let Some(note) = c.last_text("note") {
      lines.push(format!("- Note: {note}"));
    }
  } else {
    lines.push("- Last contact: none recorded".to_owned());
  }
  lines.push(c.next.map_or_else(
    || "- Next: none".to_owned(),
    |n| format!("- Next: {}", nice(n)),
  ));
  if let Some((date, folder)) = booked(dir, today) {
    lines.push(format!(
      "- Booked: {}, {}",
      nice(date),
      md_link(&folder, &format!("../Meetings/{folder}/"))
    ));
  }
  lines.push(match days {
    Some(days) => format!("- Cadence: every {days} days (from {why})"),
    None => "- Cadence: not on the contact roster".to_owned(),
  });
  let text = lines.join("\n") + "\n";
  // Its first name, before it matched the other About files
  let old = dir.join("About/contact.md");
  if old.exists() {
    fs::remove_file(old)?;
  }
  let file = dir.join(format!("About/{} (Contact).md", display_name(dir)));
  if fs::read_to_string(&file).is_ok_and(|t| t == text) {
    return Ok(false);
  }
  fs::write(file, text)?;
  Ok(true)
}

/// The person a name means; a missing one stops the command, as sys.exit did.
fn find_person(root: &Path, name: &str) -> std::result::Result<PathBuf, ExitCode> {
  let name = name.split_once('/').map_or(name, |(_, rest)| rest);
  let dir = root.join(letter_of(name)).join(name);
  if dir.is_dir() {
    Ok(dir)
  } else {
    eprintln!("Error: no such person: {name}");
    Err(ExitCode::FAILURE)
  }
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let Some((command, rest)) = args
    .split_first()
    .filter(|(c, _)| !matches!(c.as_str(), "-h" | "--help"))
  else {
    println!("{USAGE}");
    return Ok(ExitCode::SUCCESS);
  };
  let root = &config.data;
  let today = config.today();
  match (command.as_str(), rest) {
    ("scan", _) => scan(root, rest.iter().any(|a| a == "--dry-run"), today),
    ("due", _) => {
      let days = rest
        .iter()
        .find(|a| !a.is_empty() && a.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|a| a.parse().ok())
        .unwrap_or(0);
      due(root, days, rest.iter().any(|a| a == "--all"), today)
    }
    ("show", [name, ..]) => show(root, name, today),
    ("log", [first, more @ ..]) => {
      let (name, args) = if first.starts_with("--") {
        (None, rest)
      } else {
        (Some(first), more)
      };
      let emailed = match emailed(args, &config.cache_dir) {
        Ok(emailed) => emailed,
        Err(why) => {
          eprintln!("{why}");
          return Ok(ExitCode::FAILURE);
        }
      };
      log(root, name, args, emailed, today)
    }
    ("next", [name, when]) => set_next(root, name, when, today),
    ("context", [name, ..]) => context(root, name, today),
    ("render", _) => render(root, today),
    _ => {
      println!(
        "pplr contact: unknown command {}",
        Value::Str(command.clone()).py_repr()
      );
      Ok(ExitCode::from(2))
    }
  }
}

fn scan(root: &Path, dry: bool, today: Date) -> Result<ExitCode> {
  let defaults = Defaults::load(root)?;
  let (mut wrote, mut created) = (0, 0);
  for dir in person_dirs(root) {
    let before = read_contact(&dir)?;
    let mut c = before.clone();
    let past: Vec<(Date, String)> = meetings(&dir)
      .into_iter()
      .filter(|(d, _)| *d <= today)
      .collect();
    if let Some((date, folder)) = past.last()
      && (c.last.is_none() || c.last_date().is_none_or(|last| *date > last))
    {
      let key = dir.strip_prefix(root).unwrap_or(&dir).display().to_string();
      let link = format!("{}/Meetings/{folder}", marker_url(&key));
      c.last = Some(vec![
        ("date".to_owned(), Value::Date(date.to_string())),
        ("via".to_owned(), Value::Str("meeting".to_owned())),
        (
          "link".to_owned(),
          Value::Str(link.trim_start_matches('/').to_owned()),
        ),
      ]);
      c.next = None;
    }
    let (days, _) = cadence_of(&dir, &c, &defaults)?;
    if days.is_some() && c.next.is_none() {
      c.next = moved_on(&c, days, today)?;
    }
    if same_contact(&c, &before) || (c.last.is_none() && c.next.is_none()) {
      continue;
    }
    if !contact_file(&dir).exists() {
      created += 1;
    }
    wrote += 1;
    if !dry {
      write_contact(&dir, &c, &defaults, today)?;
    }
  }
  println!(
    "pplr contact scan{}: {wrote} contact.yaml {} ({created} new)",
    if dry { " (dry run)" } else { "" },
    if dry { "to write" } else { "written" }
  );
  Ok(ExitCode::SUCCESS)
}

/// `json.dumps(c, default=str, sort_keys=True)` equality: the same keys and
/// values, whatever their order.
fn same_contact(a: &Contact, b: &Contact) -> bool {
  let sorted = |c: &Contact| {
    c.last.as_ref().map(|l| {
      let mut l: Vec<(String, String)> = l.iter().map(|(k, v)| (k.clone(), v.py_repr())).collect();
      l.sort();
      l
    })
  };
  sorted(a) == sorted(b) && a.next == b.next && a.cadence == b.cadence
}

fn due(root: &Path, within: i64, everyone: bool, today: Date) -> Result<ExitCode> {
  let horizon = today.checked_add(within.days())?;
  let mut rows = Vec::new();
  for dir in person_dirs(root) {
    let c = read_contact(&dir)?;
    let Some(next) = c.next else { continue };
    if !everyone && (next > horizon || booked(&dir, today).is_some()) {
      continue;
    }
    rows.push((next, dir, c));
  }
  rows.sort_by(|a, b| {
    a.0
      .cmp(&b.0)
      .then_with(|| a.1.to_string_lossy().cmp(&b.1.to_string_lossy()))
  });
  for (next, dir, c) in &rows {
    let key = dir.strip_prefix(root).unwrap_or(dir).display().to_string();
    let was = if c.last.is_some() {
      let date = c.last_get("date").map(Value::py_str).unwrap_or_default();
      format!("last {date} {} ({})", c.via(), since(c.last_date(), today))
    } else {
      "no contact yet".to_owned()
    };
    let booked = booked(dir, today)
      .map(|(d, _)| format!("  booked {d}"))
      .unwrap_or_default();
    println!("{next}  {key}  {was}{booked}");
  }
  if rows.is_empty() {
    println!("Nobody is due.");
  }
  Ok(ExitCode::SUCCESS)
}

fn show(root: &Path, name: &str, today: Date) -> Result<ExitCode> {
  let dir = match find_person(root, name) {
    Ok(dir) => dir,
    Err(code) => return Ok(code),
  };
  let c = read_contact(&dir)?;
  let (days, why) = cadence_of(&dir, &c, &Defaults::load(root)?)?;
  println!("{}", display_name(&dir));
  if c.last.is_some() {
    let date = c.last_get("date").map(Value::py_str).unwrap_or_default();
    println!(
      "  last:    {date} {} ({})",
      c.via(),
      since(c.last_date(), today)
    );
    for key in ["link", "note"] {
      if let Some(v) = c.last_text(key) {
        println!("           {v}");
      }
    }
  } else {
    println!("  last:    none recorded");
  }
  let next = c.next.map_or_else(|| "none".to_owned(), |n| n.to_string());
  let booked = booked(&dir, today)
    .map(|(d, _)| format!("  (booked {d})"))
    .unwrap_or_default();
  println!("  next:    {next}{booked}");
  println!(
    "  cadence: {}",
    days.map_or_else(|| "none".to_owned(), |d| format!("{d} days, from {why}"))
  );
  Ok(ExitCode::SUCCESS)
}

/// What `--email N|MESSAGE-ID` gives `contact log`: the message's link and,
/// for message N of the last `pplr email` listing, its date, subject and person.
#[derive(Debug, Default)]
struct Emailed {
  link: String,
  /// N, when the message came from the listing.
  listed: Option<usize>,
  date: Option<Date>,
  subject: Option<String>,
  person: Option<String>,
}

/// The message `--email` names, if it names one; an Err is the line to print,
/// as the legacy contact printed its errors. A contact is one message, so a
/// second `--email` is refused (vc's ruling B on WP14: the legacy took the
/// first's date, person and note, and the last's link).
fn emailed(args: &[String], cache_dir: &Path) -> Result<Option<Emailed>> {
  let mut references = Vec::new();
  let mut it = args.iter();
  while let Some(arg) = it.next() {
    if arg == "--email" {
      references.push(it.next().map(|r| r.trim()).unwrap_or_default());
    }
  }
  let reference = match references.as_slice() {
    [] => return Ok(None),
    [reference] => *reference,
    _ => {
      return Err("Error: --email is given more than once: a contact log links one message".into());
    }
  };
  if reference.is_empty() || !reference.bytes().all(|b| b.is_ascii_digit()) {
    return Ok(Some(Emailed {
      link: crate::email::message_link(reference),
      ..Emailed::default()
    }));
  }
  let n = reference
    .parse()
    .map_err(|e| format!("Error: --email {reference}: {e}"))?;
  let hit = crate::email::listed(cache_dir, n)?;
  let date = hit.date.parse::<Date>().map_err(|e| {
    format!(
      "Error: message {n} of the last pplr email listing has no date ({}): {e}",
      hit.date
    )
  })?;
  Ok(Some(Emailed {
    link: crate::email::message_link(&hit.message_id),
    listed: Some(hit.n),
    date: Some(date),
    subject: Some(hit.subject),
    person: hit.person,
  }))
}

/// `contact log`: a contact on a date (today unless given), by `--via`, with
/// a `--link` and a `--note`; `--email` gives what its message knows, and
/// what the command line says wins.
fn log(
  root: &Path,
  name: Option<&String>,
  args: &[String],
  emailed: Option<Emailed>,
  today: Date,
) -> Result<ExitCode> {
  let name = match (name, &emailed) {
    (Some(name), _) => name.as_str(),
    (
      None,
      Some(Emailed {
        person: Some(person),
        ..
      }),
    ) => person.as_str(),
    (
      None,
      Some(Emailed {
        listed: Some(n), ..
      }),
    ) => {
      eprintln!("Error: message {n} is with no one in pplr: give the person");
      return Ok(ExitCode::FAILURE);
    }
    (None, _) => {
      eprintln!("Error: who? Give the person, or --email N from a pplr email listing");
      return Ok(ExitCode::FAILURE);
    }
  };
  let dir = match find_person(root, name) {
    Ok(dir) => dir,
    Err(code) => return Ok(code),
  };
  let defaults = Defaults::load(root)?;
  let mut c = read_contact(&dir)?;
  let (mut date, mut via, mut link, mut note) = match emailed {
    Some(e) => (e.date, "email".to_owned(), Some(e.link), e.subject),
    None => (None, "email".to_owned(), None, None),
  };
  let mut it = args.iter();
  while let Some(a) = it.next() {
    match a.as_str() {
      "--email" => {
        it.next();
      }
      "--via" => via = it.next().cloned().unwrap_or_else(|| "email".to_owned()),
      "--link" => link = it.next().cloned(),
      "--note" => note = it.next().cloned(),
      other => {
        date = Some(
          other
            .parse::<Date>()
            .map_err(|e| format!("'{other}' is not a date (YYYY-MM-DD): {e}"))?,
        )
      }
    }
  }
  let date = date.unwrap_or(today);
  if !VIAS.contains(&via.as_str()) {
    eprintln!("Error: --via is one of {}", VIAS.join(", "));
    return Ok(ExitCode::FAILURE);
  }
  if let Some(last) = c.last_date().filter(|last| *last > date) {
    println!("{}: kept the later contact on {last}", display_name(&dir));
    return Ok(ExitCode::SUCCESS);
  }
  let text = |v: &Option<String>| v.clone().map_or(Value::Null, Value::Str);
  c.last = Some(vec![
    ("date".to_owned(), Value::Date(date.to_string())),
    ("via".to_owned(), Value::Str(via.clone())),
    ("link".to_owned(), text(&link)),
    ("note".to_owned(), text(&note)),
  ]);
  let (days, _) = cadence_of(&dir, &c, &defaults)?;
  c.next = if days.is_some() {
    moved_on(&c, days, today)?
  } else {
    None
  };
  write_contact(&dir, &c, &defaults, today)?;
  let quoted = match (&note, &link) {
    (Some(n), Some(_)) if !n.is_empty() => format!(" (\"{n}\")"),
    _ => String::new(),
  };
  let next = c.next.map(|n| format!("; next {n}")).unwrap_or_default();
  println!("{}: {via} on {date}{quoted}{next}", display_name(&dir));
  Ok(ExitCode::SUCCESS)
}

fn set_next(root: &Path, name: &str, when: &str, today: Date) -> Result<ExitCode> {
  let dir = match find_person(root, name) {
    Ok(dir) => dir,
    Err(code) => return Ok(code),
  };
  let defaults = Defaults::load(root)?;
  let mut c = read_contact(&dir)?;
  let relative = when.strip_prefix('+').and_then(|rest| {
    let unit = rest.get(rest.len().checked_sub(1)?..)?;
    let per = DAYS_PER.iter().find(|(u, _)| *u == unit)?.1;
    let n: i64 = rest[..rest.len() - 1]
      .parse()
      .ok()
      .filter(|_| rest.len() > 1 && rest[..rest.len() - 1].bytes().all(|b| b.is_ascii_digit()))?;
    Some(n * per)
  });
  c.next = Some(match relative {
    Some(days) => today.checked_add(days.days())?,
    None => when
      .parse()
      .map_err(|e| format!("'{when}' is not a date (YYYY-MM-DD) or +N[dwm]: {e}"))?,
  });
  write_contact(&dir, &c, &defaults, today)?;
  println!(
    "{}: next {}",
    display_name(&dir),
    c.next.map(|n| n.to_string()).unwrap_or_default()
  );
  Ok(ExitCode::SUCCESS)
}

/// The meeting's summary, else its notes, else its one note file, else its first .md.
fn notes_of(folder: &Path) -> Option<PathBuf> {
  let own = format!("{}.md", folder.file_name()?.to_string_lossy());
  ["Summary.md", "Notes.md", own.as_str()]
    .iter()
    .map(|f| folder.join(f))
    .find(|p| p.exists())
    .or_else(|| {
      names(folder)
        .into_iter()
        .find(|n| n.ends_with(".md"))
        .map(|n| folder.join(n))
    })
}

fn context(root: &Path, name: &str, today: Date) -> Result<ExitCode> {
  let dir = match find_person(root, name) {
    Ok(dir) => dir,
    Err(code) => return Ok(code),
  };
  let c = read_contact(&dir)?;
  let about = names(&dir.join("About"))
    .into_iter()
    .find(|n| n.ends_with(" (About).md"));
  let text = about
    .map(|a| fs::read_to_string(dir.join("About").join(a)).unwrap_or_default())
    .unwrap_or_default();
  println!("# {}\n", display_name(&dir));
  let link_text = Regex::new(r"\[(.*?)\]\(.*?\)")?;
  for field in ["Role", "Company", "Headline", "Email"] {
    let re = Regex::new(&format!(r"(?m)^- {field}:[ \t]*(.*)$"))?;
    let value = re
      .captures(&text)
      .and_then(|m| m.get(1))
      .map(|m| link_text.replace_all(m.as_str(), "$1").trim().to_owned())
      .unwrap_or_default();
    if !value.is_empty() {
      println!("- {field}: {value}");
    }
  }
  let mut tags: Vec<String> = read_tags(&dir)?.into_iter().map(|t| t.tag).collect();
  tags.sort();
  tags.dedup();
  println!("- Tags: {}", tags.join(" "));
  if let Some(update) = Regex::new(r"(?m)^\*\*Update, [^*]+\*\* .*$")?.find(&text) {
    println!("\nLatest update: {}", update.as_str());
  }
  let last = if c.last.is_some() {
    let date = c.last_get("date").map(Value::py_str).unwrap_or_default();
    format!("{date} {} ({})", c.via(), since(c.last_date(), today))
  } else {
    "none recorded".to_owned()
  };
  println!("\nLast contact: {last}");
  if let Some(note) = c.last_text("note") {
    println!("Note: {note}");
  }
  let past: Vec<(Date, String)> = meetings(&dir)
    .into_iter()
    .filter(|(d, _)| *d <= today)
    .collect();
  if let Some((_, folder)) = past.last() {
    let path = dir.join("Meetings").join(folder);
    println!("\n## Last meeting: {folder}\n");
    if let Some(notes) = notes_of(&path) {
      println!(
        "({})\n",
        notes.strip_prefix(root).unwrap_or(&notes).display()
      );
      let body = String::from_utf8_lossy(&fs::read(&notes)?).into_owned();
      println!(
        "{}",
        body
          .lines()
          .take(CONTEXT_LINES)
          .collect::<Vec<_>>()
          .join("\n")
      );
    }
  }
  Ok(ExitCode::SUCCESS)
}

fn render(root: &Path, today: Date) -> Result<ExitCode> {
  let written = render_all(root, today)?;
  println!("pplr contact render: {written} (Contact).md written");
  Ok(ExitCode::SUCCESS)
}

/// Every (Contact).md from its contact.yaml; how many changed.
pub fn render_all(root: &Path, today: Date) -> Result<usize> {
  let defaults = Defaults::load(root)?;
  let mut written = 0;
  for dir in person_dirs(root) {
    if contact_file(&dir).exists() && render_one(&dir, &read_contact(&dir)?, &defaults, today)? {
      written += 1;
    }
  }
  Ok(written)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dates_read_as_python_wrote_them() {
    assert_eq!(nice(jiff::civil::date(2026, 3, 4)), "4 Mar 2026");
    assert_eq!(
      md_link("x", "../Meetings/20260304 Platform review/"),
      "[x](<../Meetings/20260304 Platform review/>)"
    );
    assert_eq!(
      md_link("link", "https://example.com/a"),
      "[link](https://example.com/a)"
    );
  }

  #[test]
  fn ints_read_as_python_int_reads_them() {
    assert_eq!(int(&Value::Int(90)), Some(90));
    assert_eq!(int(&Value::Str(" 30 ".to_owned())), Some(30));
    assert_eq!(int(&Value::Float(2.9)), Some(2));
    assert_eq!(int(&Value::Str("x".to_owned())), None);
  }
}
