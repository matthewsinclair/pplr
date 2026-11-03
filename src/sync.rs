//! `pplr sync`: pplr and Contacts, compared (`--check`), linked (`--link`),
//! pulled into pplr (`--pull`), given photos (`--photos`) and backed up
//! (`--backup`), through the store (ST0001 design 3.2). The engine's logic,
//! ported, and the plan with its review workbook (`--plan`, `--apply-plan`),
//! which src/plan.rs scores and src/workbook.rs writes and reads back.

use crate::Result;
use crate::config::Config;
use crate::contacts::{self, AccessDenied, Card, MarkerWrite, Store, linkedin_url, to_swift_json};
use crate::fs::read_exact;
use crate::layout::Layout;
use crate::marker::{by_marker, marker_path, marker_paths, marker_url};
use crate::normalise::{decompose, email_key, fold, percent_decode};
use crate::plan::{self, Decision, Reviewed, Work};
use crate::tree::{Person, people};
use crate::workbook;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const DEFAULT_GROUP: &str = "PPLR";
/// A phone shorter than this is not matched or pulled.
const MIN_PHONE: usize = 8;
/// Writes stop after this many failures; --link and --apply-plan say so,
/// and --photos, as the engine did, does not.
const MAX_FAILURES: usize = 3;
const STOPPED: &str = "(stopped after three failures)";
/// The engine's exit for no Contacts access.
const NO_ACCESS: u8 = 3;
/// The width of a header field's label in an About file ("- Company:  ").
const LABEL_WIDTH: usize = 12;

fn usage() {
  for line in [
    "Usage: pplr sync --check [--json] [-v|--verbose] [--group NAME]",
    "       pplr sync --plan [--out FILE] [--json] [--group NAME]",
    "       pplr sync --apply-plan WORKBOOK [--apply] [--limit N] [--name \"Surname, First\"]...",
    "       pplr sync --photos [--apply] [--limit N]",
    "       pplr sync --pull [--apply] [--name \"Surname, First\"]... [--skip \"Surname, First\"]... [--limit N]",
    "       pplr sync --backup",
    "       pplr sync --link [--apply] [--name \"Surname, First\"]... [--all-names] [--group NAME]",
    "  --check         Compare pplr with Apple Contacts; changes nothing",
    "  --link          Mark matched cards as pplr's: the pplr URL and the group, nothing else",
    "  --plan          Write a workbook for review: pplr, Contacts, and a proposed merge; changes nothing",
    "  --apply-plan F  Carry out a reviewed --plan workbook: Add, Update, Link only (a dry run without --apply)",
    "  --photos        Give linked cards with no photo the pplr picture (a dry run without --apply)",
    "  --pull          Copy into pplr the emails, phones and LinkedIn a linked card has and pplr lacks, and its",
    "                  company or role where pplr has none; pplr wins, nothing is removed (a dry run without --apply)",
    "  --backup        Save every Contacts card to a dated .vcf in $PPLR_BACKUP_DIR",
    "  --apply         With --link, write (a dry run otherwise); every card is saved to a .vcf first",
    "  --name NAME     With --link, also link this name-only match (repeatable)",
    "  --limit N       With --link --apply, write only the first N",
    "  --all-names     With --link, also link every name-only match",
    "  --skip NAME     With --pull, leave this person alone (repeatable)",
    "  --out FILE      With --plan, the workbook to write (default: $PPLR_DATA/_out/contacts-plan-<stamp>.xlsx)",
    "  --json          Print the report (or with --plan, the plan) as JSON",
    "  -v, --verbose   Also list everyone who is only in pplr",
    "  --group NAME    The Contacts group that marks pplr people (default: PPLR)",
  ] {
    println!("{line}");
  }
}

#[derive(Default)]
struct Options {
  mode: Option<String>,
  apply: bool,
  json: bool,
  verbose: bool,
  all_names: bool,
  limit: Option<usize>,
  group: Option<String>,
  names: Vec<String>,
  skips: Vec<String>,
  /// --plan's workbook, as given.
  out: Option<String>,
  /// --apply-plan's workbook, as given.
  workbook: Option<String>,
}

/// The options, or None when they ask for the usage. A mistake in them is
/// the error: an unknown option, an option without its value, or a --limit
/// that is not a number.
fn options(args: &[String]) -> std::result::Result<Option<Options>, String> {
  let mut o = Options::default();
  let mut it = args.iter();
  while let Some(arg) = it.next() {
    let mut value = || {
      it.next()
        .cloned()
        .ok_or_else(|| format!("{arg} needs a value"))
    };
    match arg.as_str() {
      "--check" | "--link" | "--backup" | "--plan" | "--photos" | "--pull" => {
        o.mode = Some(arg.trim_start_matches('-').to_owned());
      }
      "--apply-plan" => {
        o.workbook = Some(value()?);
        o.mode = Some("applyplan".to_owned());
      }
      "--out" => o.out = Some(value()?),
      "--apply" => o.apply = true,
      "--name" => o.names.push(value()?),
      "--skip" => o.skips.push(value()?),
      "--all-names" => o.all_names = true,
      "--limit" => {
        let n = value()?;
        o.limit = Some(
          n.parse()
            .map_err(|_| format!("--limit needs a number, not '{n}'"))?,
        );
      }
      "--json" => o.json = true,
      "-v" | "--verbose" => o.verbose = true,
      "--group" => o.group = Some(value()?),
      "-h" | "--help" => return Ok(None),
      other => return Err(format!("unknown option '{other}'")),
    }
  }
  Ok(Some(o))
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let o = match options(args) {
    Ok(Some(o)) => o,
    Ok(None) => {
      usage();
      return Ok(ExitCode::SUCCESS);
    }
    Err(mistake) => {
      println!("Error: {mistake}");
      usage();
      return Ok(ExitCode::FAILURE);
    }
  };
  let Some(mode) = o.mode.clone() else {
    usage();
    return Ok(ExitCode::FAILURE);
  };
  match run_mode(&mode, &o, config) {
    Ok(code) => Ok(code),
    Err(e) if e.downcast_ref::<AccessDenied>().is_some() => {
      eprintln!("{e}");
      Ok(ExitCode::from(NO_ACCESS))
    }
    Err(e) => Err(e),
  }
}

fn run_mode(mode: &str, o: &Options, config: &Config) -> Result<ExitCode> {
  let group = o.group.clone().unwrap_or_else(|| DEFAULT_GROUP.to_owned());
  let mut store = contacts::open(config, &Layout::running()?)?;
  match mode {
    "backup" => {
      println!("Backup: {}", store.backup(&config.backup_dir)?.display());
      Ok(ExitCode::SUCCESS)
    }
    "check" => {
      let report = check(&people(&config.data)?, &store.cards()?, &group);
      if o.json {
        println!("{}", to_swift_json(&report.to_json()));
      } else {
        print_report(&report, o.verbose);
      }
      Ok(ExitCode::SUCCESS)
    }
    "link" => link(store.as_mut(), o, &group, config),
    "photos" => photos(store.as_mut(), o, config),
    "pull" => pull(store.as_mut(), o, &group, config),
    "plan" => plan_mode(store.as_mut(), o, &group, config),
    "applyplan" => apply_plan(store.as_mut(), o, &group, config),
    _ => Ok(ExitCode::FAILURE),
  }
}

// The model and the check

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiff {
  pub field: &'static str,
  pub pplr: Vec<String>,
  pub contacts: Vec<String>,
}

/// A pplr person and the card they match, and what a link still needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pair {
  pub person: String,
  pub contact: String,
  pub contact_name: String,
  /// linked | email | linkedin | phone | name
  pub how: &'static str,
  pub in_group: bool,
  /// An iCloud card not yet in the group.
  pub needs_group: bool,
  /// A directory or Other Known card: nothing can be written.
  pub no_account: bool,
  /// In an account other than iCloud: the group cannot hold it.
  pub other_account: bool,
  /// The card's pplr URL is in the current form.
  pub has_marker: bool,
  /// The card has a pplr URL in an older form, to replace.
  pub marker_stale: bool,
  pub inetloc: PathBuf,
  /// It exists and opens this card: check() reads it, matches() leaves it false.
  pub has_inetloc: bool,
  pub diffs: Vec<FieldDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
  pub group: String,
  pub people: usize,
  pub contacts: usize,
  pub group_members: usize,
  pub linked: Vec<Pair>,
  pub matched: Vec<Pair>,
  pub name_only: Vec<Pair>,
  /// Person key -> "Name (id)" of each card it could be, by key.
  pub ambiguous: Vec<(String, Vec<String>)>,
  pub pplr_only: Vec<String>,
  pub group_orphans: Vec<String>,
}

impl Report {
  /// The report as the engine encoded it (keys sorted by the encoder).
  pub fn to_json(&self) -> Value {
    let pairs = |ps: &[Pair]| -> Value {
      Value::Array(
        ps.iter()
          .map(|p| {
            json!({
              "person": p.person, "contact": p.contact, "contactName": p.contact_name, "how": p.how,
              "inGroup": p.in_group, "needsGroup": p.needs_group, "noAccount": p.no_account,
              "otherAccount": p.other_account, "hasMarker": p.has_marker, "markerStale": p.marker_stale,
              "inetloc": p.inetloc.display().to_string(), "hasInetloc": p.has_inetloc,
              "diffs": p.diffs.iter().map(|d| json!({"field": d.field, "pplr": d.pplr, "contacts": d.contacts})).collect::<Vec<_>>(),
            })
          })
          .collect(),
      )
    };
    let ambiguous: serde_json::Map<String, Value> = self
      .ambiguous
      .iter()
      .map(|(k, v)| (k.clone(), json!(v)))
      .collect();
    json!({
      "group": self.group, "people": self.people, "contacts": self.contacts, "groupMembers": self.group_members,
      "linked": pairs(&self.linked), "matched": pairs(&self.matched), "nameOnly": pairs(&self.name_only),
      "ambiguous": ambiguous, "pplrOnly": self.pplr_only, "groupOrphans": self.group_orphans,
    })
  }
}

/// The path of the card's pplr URL, as written: "b/bray-martin".
pub fn pplr_marker(c: &Card) -> Option<String> {
  c.urls
    .iter()
    .find(|u| u.label.to_lowercase() == "pplr" || u.value.starts_with("pplr://"))
    .map(|u| u.value.replace("pplr://", ""))
}

/// An iCloud card, where the group lives: in "home", or no account named.
pub fn is_home(c: &Card) -> bool {
  c.account.as_deref().unwrap_or("home") == "home"
}

/// Whether two values the legacy engine held as Swift Strings are the same
/// text. Swift's equality is canonical: a letter and its accent written as one
/// character or as two (é, or e and U+0301) are one value. So each such value
/// is compared and keyed by its decomposed form, and shown as it was written
/// (vc decision 22).
pub(crate) fn same_text(a: &str, b: &str) -> bool {
  decompose(a) == decompose(b)
}

/// The card is in the group, its name compared as `same_text` compares.
pub fn in_group(c: &Card, group: &str) -> bool {
  c.groups.iter().any(|g| same_text(g, group))
}

/// Linking puts an iCloud card in the group, if it is not in it yet.
pub fn needs_group(c: &Card, group: &str) -> bool {
  is_home(c) && !in_group(c, group)
}

/// The pplr URL linking writes on the card: none when it has the person's
/// current one, else theirs, in place of a URL in an older form.
pub fn marker_write(c: &Card, key: &str) -> Option<MarkerWrite> {
  match pplr_marker(c) {
    Some(m) if m == marker_path(key) => None,
    m => Some(MarkerWrite {
      url: marker_url(key),
      replace: m.is_some(),
    }),
  }
}

/// The card's given and family names, whichever it has.
pub fn card_name(c: &Card) -> String {
  [c.given.as_str(), c.family.as_str()]
    .iter()
    .filter(|s| !s.is_empty())
    .copied()
    .collect::<Vec<_>>()
    .join(" ")
}

/// The card's name, else its company, else "(no name)".
pub fn display_name(c: &Card) -> String {
  let n = card_name(c);
  if !n.is_empty() {
    n
  } else if c.organization.is_empty() {
    "(no name)".to_owned()
  } else {
    c.organization.clone()
  }
}

/// About/<First Surname> (Contacts).inetloc, beside the About file.
fn inetloc_path(p: &Person) -> PathBuf {
  let about = p
    .about
    .as_ref()
    .and_then(|a| a.parent().map(Path::to_path_buf))
    .unwrap_or_else(|| p.dir.join("About"));
  about.join(format!("{} {} (Contacts).inetloc", p.given, p.family))
}

fn contacts_url(id: &str) -> String {
  format!("addressbook://{id}")
}

/// Whether the .inetloc at `path` opens this card.
fn inetloc_opens(path: &Path, id: &str) -> bool {
  let Ok(text) = fs::read_to_string(path) else {
    return false;
  };
  let Some(after) = text.split_once("<key>URL</key>").map(|(_, rest)| rest) else {
    return false;
  };
  let url = after
    .split_once("<string>")
    .and_then(|(_, rest)| rest.split_once("</string>"))
    .map(|(url, _)| xml_unescape(url));
  url.as_deref() == Some(contacts_url(id).as_str())
}

fn xml_unescape(s: &str) -> String {
  s.replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&apos;", "'")
    .replace("&amp;", "&")
}

fn xml_escape(s: &str) -> String {
  s.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
}

/// An .inetloc, as PropertyListSerialization writes one; a .webloc beside it
/// goes, since macOS opens a .webloc only for web addresses. Its name on disk
/// is decomposed, as Foundation wrote it; reports keep the name as composed.
fn write_inetloc(path: &Path, id: &str) -> Result<()> {
  let name = path.file_name().map(|n| decompose(&n.to_string_lossy()));
  let path = &name.map_or_else(|| path.to_path_buf(), |n| path.with_file_name(n));
  if let Some(dir) = path.parent() {
    fs::create_dir_all(dir)?;
  }
  let plist = format!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n\t<key>URL</key>\n\t<string>{}</string>\n</dict>\n</plist>\n",
    xml_escape(&contacts_url(id))
  );
  fs::write(path, plist)?;
  match fs::remove_file(path.with_extension("webloc")) {
    Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
    _ => Ok(()),
  }
}

/// Values once each, as a Swift Set holds them: spellings `same_text` calls
/// one value are one, the first kept. Sorted.
fn sorted_set(values: impl IntoIterator<Item = String>) -> Vec<String> {
  let mut seen = HashSet::new();
  let mut v: Vec<String> = values
    .into_iter()
    .filter(|s| seen.insert(decompose(s)))
    .collect();
  v.sort();
  v
}

fn diffs(p: &Person, c: &Card) -> Vec<FieldDiff> {
  let mut out = Vec::new();
  let mut scalar = |field, a: String, b: String| {
    if fold(&a) != fold(&b) {
      let one = |s: String| if s.is_empty() { Vec::new() } else { vec![s] };
      out.push(FieldDiff {
        field,
        pplr: one(a),
        contacts: one(b),
      });
    }
  };
  scalar(
    "name",
    format!("{} {}", p.given, p.family),
    format!("{} {}", c.given, c.family),
  );
  scalar("company", p.header.company.clone(), c.organization.clone());
  scalar("role", p.header.role.clone(), c.job_title.clone());
  // Each set compared as a Swift Set is (`same_text`), and each side's
  // differences shown as that side holds them
  let mut set = |field, a: Vec<String>, b: Vec<String>| {
    let keys = |v: &[String]| v.iter().map(|s| decompose(s)).collect::<HashSet<_>>();
    let (ka, kb) = (keys(&a), keys(&b));
    if ka != kb {
      out.push(FieldDiff {
        field,
        pplr: sorted_set(a.into_iter().filter(|s| !kb.contains(&decompose(s)))),
        contacts: sorted_set(b.into_iter().filter(|s| !ka.contains(&decompose(s)))),
      });
    }
  };
  set(
    "email",
    p.header.emails.iter().map(|e| email_key(e)).collect(),
    c.emails.iter().map(|e| email_key(e)).collect(),
  );
  set("phone", p.header.phones.clone(), c.phones.clone());
  let slug = if p.header.linkedin.is_empty() {
    Vec::new()
  } else {
    vec![p.header.linkedin.clone()]
  };
  set("linkedin", slug, c.linkedin.clone());
  out
}

/// The cards that share a key: an email key or a slug, decomposed as
/// `same_text` compares them; a phone; or a folded name.
type CardsBy<'a> = HashMap<String, Vec<&'a Card>>;

/// Cards once each, in their first order.
fn uniq(cs: Vec<&Card>) -> Vec<&Card> {
  let mut seen = HashSet::new();
  cs.into_iter()
    .filter(|c| seen.insert(c.id.clone()))
    .collect()
}

/// matches(), with each pair's .inetloc read to see whether it opens the card.
pub fn check(people: &[Person], cards: &[Card], group: &str) -> Report {
  let mut r = matches(people, cards, group);
  for p in r
    .linked
    .iter_mut()
    .chain(&mut r.matched)
    .chain(&mut r.name_only)
  {
    p.has_inetloc = inetloc_opens(&p.inetloc, &p.contact);
  }
  r
}

/// Every person matched to a card: the pplr URL first, then a unique email,
/// LinkedIn or phone, then the name alone; ambiguity and orphans reported.
/// Nothing is read from disk, so no .inetloc is known to open its card.
pub fn matches(people: &[Person], cards: &[Card], group: &str) -> Report {
  // A card's pplr URL names a person by a marker path, or by the key itself,
  // each keyed as `same_text` compares
  let mut marker_keys: HashMap<String, String> = HashMap::new();
  for p in people {
    for path in marker_paths(p) {
      marker_keys.insert(decompose(&path), p.key.clone());
    }
    for k in p.keys() {
      marker_keys.insert(decompose(k), p.key.clone());
    }
  }
  let mut by_marker: HashMap<String, &Card> = HashMap::new();
  let mut by_email: CardsBy = HashMap::new();
  let mut by_slug: CardsBy = HashMap::new();
  let mut by_phone: CardsBy = HashMap::new();
  let mut by_name: CardsBy = HashMap::new();
  for c in cards {
    if let Some(m) = pplr_marker(c) {
      let key = marker_keys
        .get(&decompose(&m))
        .or_else(|| marker_keys.get(&decompose(&percent_decode(&m).unwrap_or_else(|| m.clone()))));
      if let Some(key) = key {
        by_marker.insert(key.clone(), c);
      }
    }
    for e in &c.emails {
      by_email
        .entry(decompose(&email_key(e)))
        .or_default()
        .push(c);
    }
    for s in &c.linkedin {
      by_slug.entry(decompose(s)).or_default().push(c);
    }
    let phones: HashSet<&String> = c.phones.iter().collect();
    for ph in phones
      .into_iter()
      .filter(|ph| ph.chars().count() >= MIN_PHONE)
    {
      by_phone.entry(ph.clone()).or_default().push(c);
    }
    by_name
      .entry(fold(&format!("{} {}", c.given, c.family)))
      .or_default()
      .push(c);
  }
  let mut r = Report {
    group: group.to_owned(),
    people: people.len(),
    contacts: cards.len(),
    group_members: cards.iter().filter(|c| in_group(c, group)).count(),
    linked: Vec::new(),
    matched: Vec::new(),
    name_only: Vec::new(),
    ambiguous: Vec::new(),
    pplr_only: Vec::new(),
    group_orphans: Vec::new(),
  };
  let mut claimed: HashSet<String> = HashSet::new();
  let mut pair = |p: &Person, c: &Card, how: &'static str| -> Pair {
    claimed.insert(c.id.clone());
    let marker = marker_write(c, &p.key);
    let account = c.account.as_deref();
    Pair {
      person: p.key.clone(),
      contact: c.id.clone(),
      contact_name: display_name(c),
      how,
      in_group: in_group(c, group),
      needs_group: needs_group(c, group),
      no_account: account == Some("none"),
      other_account: account == Some("other"),
      has_marker: marker.is_none(),
      marker_stale: marker.is_some_and(|m| m.replace),
      inetloc: inetloc_path(p),
      has_inetloc: false,
      diffs: diffs(p, c),
    }
  };
  let names_of = |cs: &[&Card]| {
    cs.iter()
      .map(|c| format!("{} ({})", display_name(c), c.id))
      .collect::<Vec<_>>()
  };
  for p in people {
    if let Some(c) = by_marker.get(&p.key) {
      r.linked.push(pair(p, c, "linked"));
      continue;
    }
    let via_email = uniq(
      p.header
        .emails
        .iter()
        .flat_map(|e| {
          by_email
            .get(&decompose(&email_key(e)))
            .cloned()
            .unwrap_or_default()
        })
        .collect(),
    );
    let via_slug = if p.header.linkedin.is_empty() {
      Vec::new()
    } else {
      uniq(
        by_slug
          .get(&decompose(&p.header.linkedin))
          .cloned()
          .unwrap_or_default(),
      )
    };
    let via_phone = uniq(
      p.header
        .phones
        .iter()
        .flat_map(|ph| by_phone.get(ph).cloned().unwrap_or_default())
        .collect(),
    );
    let via_name = by_name
      .get(&fold(&format!("{} {}", p.given, p.family)))
      .cloned()
      .unwrap_or_default();
    if via_email.len() == 1 {
      r.matched.push(pair(p, via_email[0], "email"));
    } else if via_email.len() > 1 {
      r.ambiguous.push((p.key.clone(), names_of(&via_email)));
    } else if via_slug.len() == 1 {
      r.matched.push(pair(p, via_slug[0], "linkedin"));
    } else if via_phone.len() == 1 {
      r.matched.push(pair(p, via_phone[0], "phone"));
    } else if via_name.len() == 1 {
      r.name_only.push(pair(p, via_name[0], "name"));
    } else if via_name.len() > 1 {
      r.ambiguous.push((p.key.clone(), names_of(&via_name)));
    } else {
      r.pplr_only.push(p.key.clone());
    }
  }
  r.ambiguous.sort_by(|a, b| a.0.cmp(&b.0));
  let mut orphans: Vec<String> = cards
    .iter()
    .filter(|c| in_group(c, group) && !claimed.contains(&c.id))
    .map(display_name)
    .collect();
  orphans.sort();
  r.group_orphans = orphans;
  r
}

fn print_report(r: &Report, verbose: bool) {
  let both: Vec<&Pair> = r.linked.iter().chain(&r.matched).collect();
  let with_diffs: Vec<&Pair> = both
    .iter()
    .copied()
    .filter(|p| !p.diffs.is_empty())
    .collect();
  let how = |h: &str| r.matched.iter().filter(|p| p.how == h).count();
  println!("pplr sync --check (read-only)\n");
  println!("  pplr people             {}", r.people);
  println!("  Contacts cards          {}", r.contacts);
  println!("  in group {:<15}{}", r.group, r.group_members);
  println!();
  println!("  linked (pplr marker)    {}", r.linked.len());
  let elsewhere = both.iter().filter(|p| p.other_account).count();
  if elsewhere > 0 {
    println!(
      "    in another account    {elsewhere}  (marked by the pplr URL; the group is iCloud's only)"
    );
  }
  println!("  matched by email        {}", how("email"));
  println!("  matched by LinkedIn     {}", how("linkedin"));
  println!("  matched by phone        {}", how("phone"));
  println!("  name only (confirm)     {}", r.name_only.len());
  println!("  ambiguous               {}", r.ambiguous.len());
  println!("  only in pplr            {}", r.pplr_only.len());
  println!("  group orphans           {}", r.group_orphans.len());
  println!("  matched, fields differ  {}", with_diffs.len());
  let section = |title: &str, lines: Vec<String>| {
    if !lines.is_empty() {
      println!("\n{title}");
      for line in lines {
        println!("  {line}");
      }
    }
  };
  section(
    "Name only: same name, no shared email, LinkedIn or phone. Confirm before linking:",
    r.name_only
      .iter()
      .map(|p| format!("{}  <->  {}", p.person, p.contact_name))
      .collect(),
  );
  section(
    "Ambiguous: more than one card could be this person:",
    r.ambiguous
      .iter()
      .map(|(k, v)| format!("{k}: {}", v.join("; ")))
      .collect(),
  );
  let diff_lines = with_diffs
    .iter()
    .flat_map(|p| {
      let head = format!(
        "{}  <->  {}  [{}{}{}]",
        p.person,
        p.contact_name,
        p.how,
        if p.in_group { ", in group" } else { "" },
        if p.other_account {
          ", other account"
        } else {
          ""
        }
      );
      std::iter::once(head).chain(p.diffs.iter().map(|d| {
        let side = |v: &[String]| {
          if v.is_empty() {
            "-".to_owned()
          } else {
            v.join(", ")
          }
        };
        format!(
          "    {:<9} pplr: {}  |  Contacts: {}",
          d.field,
          side(&d.pplr),
          side(&d.contacts)
        )
      }))
    })
    .collect();
  section("Matched, with differences:", diff_lines);
  section(
    &format!(
      "In the \"{}\" group but not matched to anyone in pplr:",
      r.group
    ),
    r.group_orphans.clone(),
  );
  if verbose {
    section("Only in pplr:", r.pplr_only.clone());
  } else if !r.pplr_only.is_empty() {
    println!(
      "\n({} only in pplr; --verbose lists them)",
      r.pplr_only.len()
    );
  }
}

/// "Kemp, Jon" or "K/Kemp, Jon" -> "K/Kemp, Jon".
fn person_key(name: &str) -> String {
  if name.contains('/') {
    name.to_owned()
  } else {
    let first: String = name.chars().take(1).flat_map(char::to_uppercase).collect();
    format!("{first}/{name}")
  }
}

/// Each --name or --skip value as a person's key, keyed as `same_text`
/// compares.
fn key_set(names: &[String]) -> HashSet<String> {
  names.iter().map(|n| decompose(&person_key(n))).collect()
}

// --link

struct LinkPlan {
  to_link: Vec<Pair>,
  already_done: usize,
  names_skipped: Vec<String>,
  unknown_names: Vec<String>,
  ambiguous: Vec<String>,
  no_account: Vec<String>,
}

fn plan_link(r: &Report, names: &[String], all_names: bool) -> LinkPlan {
  // Each --name as a person's key, keyed as `same_text` compares and shown as
  // typed; of two spellings of one name the first is kept, as a Swift Set does
  let mut wanted: HashMap<String, String> = HashMap::new();
  for k in names.iter().map(|n| person_key(n)) {
    wanted.entry(decompose(&k)).or_insert(k);
  }
  let named = |p: &Pair| wanted.contains_key(&decompose(&p.person));
  let name_keys: HashSet<String> = r.name_only.iter().map(|p| decompose(&p.person)).collect();
  let confirmed = r.name_only.iter().filter(|p| all_names || named(p));
  let all: Vec<&Pair> = r.linked.iter().chain(&r.matched).chain(confirmed).collect();
  let done = |p: &Pair| p.has_marker && !p.needs_group && p.has_inetloc;
  let strict: HashSet<String> = r
    .linked
    .iter()
    .chain(&r.matched)
    .map(|p| decompose(&p.person))
    .collect();
  let mut unknown: Vec<String> = wanted
    .iter()
    .filter(|(k, _)| !name_keys.contains(*k) && !strict.contains(*k))
    .map(|(_, typed)| typed.clone())
    .collect();
  unknown.sort();
  LinkPlan {
    to_link: all
      .iter()
      .filter(|p| !done(p) && !p.no_account)
      .map(|p| (*p).clone())
      .collect(),
    already_done: all.iter().filter(|p| done(p)).count(),
    names_skipped: r
      .name_only
      .iter()
      .filter(|p| !(all_names || named(p)))
      .map(|p| p.person.clone())
      .collect(),
    unknown_names: unknown,
    ambiguous: r.ambiguous.iter().map(|(k, _)| k.clone()).collect(),
    no_account: all
      .iter()
      .filter(|p| p.no_account)
      .map(|p| p.person.clone())
      .collect(),
  }
}

/// Each write's person, with its error when it failed.
type Outcome = Vec<(String, Option<String>)>;

/// Each item written in turn, the first `limit` of them, stopping after
/// MAX_FAILURES failures.
fn write_each<T>(
  items: &[T],
  limit: Option<usize>,
  person: impl Fn(&T) -> String,
  mut write: impl FnMut(&T) -> Result<()>,
) -> Outcome {
  let mut outcome = Outcome::new();
  let mut failures = 0;
  for item in items.iter().take(limit.unwrap_or(usize::MAX)) {
    let error = write(item).err().map(|e| e.to_string());
    let failed = error.is_some();
    outcome.push((person(item), error));
    if failed {
      failures += 1;
      if failures >= MAX_FAILURES {
        break;
      }
    }
  }
  outcome
}

/// FAILURE when a write failed.
fn exit_code(outcome: Option<&Outcome>) -> ExitCode {
  if outcome.is_some_and(|o| o.iter().any(|(_, e)| e.is_some())) {
    ExitCode::FAILURE
  } else {
    ExitCode::SUCCESS
  }
}

/// The writes that failed, each with its error, and whether they stopped the rest.
fn print_failed(bad: &[&(String, Option<String>)]) {
  if bad.is_empty() {
    return;
  }
  println!("\nFailed:");
  for (person, error) in bad {
    println!("  {person}: {}", error.as_deref().unwrap_or_default());
  }
  if bad.len() >= MAX_FAILURES {
    println!("  {STOPPED}");
  }
}

fn link(store: &mut dyn Store, o: &Options, group: &str, config: &Config) -> Result<ExitCode> {
  let report = check(&people(&config.data)?, &store.cards()?, group);
  let plan = plan_link(&report, &o.names, o.all_names);
  let mut backup = None;
  let outcome = if !o.apply {
    None
  } else if plan.to_link.is_empty() {
    Some(Outcome::new())
  } else {
    backup = Some(store.backup(&config.backup_dir)?);
    let done = write_each(
      &plan.to_link,
      o.limit,
      |p| p.person.clone(),
      |p| {
        let marker = (!p.has_marker).then(|| MarkerWrite {
          url: marker_url(&p.person),
          replace: p.marker_stale,
        });
        if marker.is_some() || p.needs_group {
          store.link(&p.contact, marker.as_ref(), p.needs_group.then_some(group))?;
        }
        if p.has_inetloc {
          Ok(())
        } else {
          write_inetloc(&p.inetloc, &p.contact)
        }
      },
    );
    store.save()?;
    Some(done)
  };
  print_link(&plan, group, outcome.as_ref(), backup.as_deref());
  Ok(exit_code(outcome.as_ref()))
}

fn print_link(plan: &LinkPlan, group: &str, outcome: Option<&Outcome>, backup: Option<&Path>) {
  let what = |p: &Pair| {
    let mut w = Vec::new();
    if p.marker_stale {
      w.push("pplr URL updated");
    } else if !p.has_marker {
      w.push("pplr URL");
    }
    if p.needs_group {
      w.push("group");
    }
    if !p.has_inetloc {
      w.push(".inetloc");
    }
    format!("[{}; {}]", p.how, w.join(" + "))
  };
  let by_person = |person: &str| plan.to_link.iter().find(|p| p.person == person);
  if let Some(outcome) = outcome {
    let ok: Vec<&(String, Option<String>)> = outcome.iter().filter(|(_, e)| e.is_none()).collect();
    let bad: Vec<&(String, Option<String>)> = outcome.iter().filter(|(_, e)| e.is_some()).collect();
    println!("pplr sync --link --apply\n");
    println!("  linked now              {}", ok.len());
    println!("  failed                  {}", bad.len());
    println!(
      "  still to link           {}",
      plan.to_link.len() - ok.len()
    );
    println!("  already linked          {}", plan.already_done);
    if !ok.is_empty() {
      println!(
        "\nLinked (the pplr URL on the card, the \"{group}\" group for iCloud cards; a (Contacts).inetloc in About):"
      );
      for (person, _) in &ok {
        if let Some(p) = by_person(person) {
          println!("  {person}  <->  {}  {}", p.contact_name, what(p));
        }
      }
    }
    print_failed(&bad);
  } else {
    println!("pplr sync --link (dry run: nothing written; --apply to write)\n");
    println!("  to link                 {}", plan.to_link.len());
    println!("  already linked          {}", plan.already_done);
    println!("  name only, not linked   {}", plan.names_skipped.len());
    println!("  ambiguous, not linked   {}", plan.ambiguous.len());
    if !plan.to_link.is_empty() {
      println!(
        "\nWould link (the pplr URL on the card, the \"{group}\" group for iCloud cards, nothing else; a (Contacts).inetloc in About):"
      );
      for p in &plan.to_link {
        println!("  {}  <->  {}  {}", p.person, p.contact_name, what(p));
      }
    }
  }
  if !plan.no_account.is_empty() {
    println!(
      "\nMatched, but the card is in no account (a directory or Other Known card?), so cannot be linked:"
    );
    for p in &plan.no_account {
      println!("  {p}");
    }
  }
  if !plan.unknown_names.is_empty() {
    println!("\nNot a name-only match, so ignored:");
    for n in &plan.unknown_names {
      println!("  {n}");
    }
  }
  if outcome.is_none() && !plan.names_skipped.is_empty() {
    println!(
      "\nName-only matches left alone (confirm with --name \"Surname, First\", or --all-names):"
    );
    for n in &plan.names_skipped {
      println!("  {n}");
    }
  }
  if let Some(backup) = backup {
    println!("\nBackup: {}", backup.display());
  }
}

// --photos

fn photos(store: &mut dyn Store, o: &Options, config: &Config) -> Result<ExitCode> {
  let everyone = people(&config.data)?;
  let keys = by_marker(&everyone);
  let cards = store.cards()?;
  let mut work: Vec<(&Person, &Card, &Path)> = cards
    .iter()
    .filter(|c| c.has_image != Some(true) && c.account.as_deref() != Some("none"))
    .filter_map(|c| {
      let p = *keys.get(&pplr_marker(c)?)?;
      Some((p, c, p.picture.as_deref()?))
    })
    .collect();
  work.sort_by(|a, b| a.0.key.cmp(&b.0.key));
  if o.apply && !work.is_empty() {
    let backup = store.backup(&config.backup_dir)?;
    let outcome = write_each(
      &work,
      o.limit,
      |(p, _, _)| p.key.clone(),
      |(_, c, picture)| store.set_photo(&c.id, picture),
    );
    store.save()?;
    let bad: Vec<&(String, Option<String>)> = outcome.iter().filter(|(_, e)| e.is_some()).collect();
    println!(
      "pplr sync --photos --apply\n\n  photos set              {}\n  failed                  {}",
      outcome.len() - bad.len(),
      bad.len()
    );
    for (person, error) in &bad {
      println!("  {person}: {}", error.as_deref().unwrap_or_default());
    }
    println!("\nBackup: {}", backup.display());
    return Ok(exit_code(Some(&outcome)));
  }
  println!(
    "pplr sync --photos (dry run: nothing written; --apply to write)\n\n  linked cards with no photo, where pplr has one  {}",
    work.len()
  );
  for (p, c, _) in &work {
    println!("  {}  <->  {}", p.key, display_name(c));
  }
  Ok(ExitCode::SUCCESS)
}

// --pull

struct PullWork<'a> {
  person: &'a Person,
  card_name: String,
  emails: Vec<String>,
  phones: Vec<String>,
  linkedin: Option<String>,
  company: Option<String>,
  role: Option<String>,
  /// Numbers left out: probably a foreign number saved without its country code.
  doubtful: Vec<String>,
}

impl PullWork<'_> {
  fn is_empty(&self) -> bool {
    self.emails.is_empty()
      && self.phones.is_empty()
      && self.linkedin.is_none()
      && self.company.is_none()
      && self.role.is_none()
  }

  fn summary(&self) -> Vec<String> {
    let mut w = Vec::new();
    if let Some(c) = &self.company {
      w.push(format!("company: {c}"));
    }
    if let Some(r) = &self.role {
      w.push(format!("role: {r}"));
    }
    w.extend(self.emails.iter().map(|e| format!("+ email {e}")));
    w.extend(self.phones.iter().map(|p| format!("+ phone {p}")));
    if let Some(l) = &self.linkedin {
      w.push(format!("+ LinkedIn {l}"));
    }
    w.extend(self.doubtful.iter().map(|d| {
      format!("left out {d}: not a UK number, so probably saved without its country code")
    }));
    w
  }
}

/// A number read as UK but not one: ten digits after +44, never starting with 4.
fn doubtful_uk(phone: &str) -> bool {
  let Some(nsn) = phone.strip_prefix("+44") else {
    return false;
  };
  nsn.chars().count() != 10 || nsn.starts_with('4')
}

/// What each strictly matched card knows that pplr lacks, by person.
fn pull_work<'a>(r: &Report, everyone: &'a [Person], cards: &[Card]) -> Vec<PullWork<'a>> {
  // A card linked across accounts comes back once per account under one id: read as one
  let mut by_id: Vec<(String, Card)> = Vec::new();
  for c in cards {
    match by_id.iter_mut().find(|(id, _)| *id == c.id) {
      Some((_, m)) => {
        m.emails.extend(c.emails.iter().cloned());
        m.phones.extend(c.phones.iter().cloned());
        m.linkedin = sorted_set(m.linkedin.iter().chain(&c.linkedin).cloned());
        if m.organization.is_empty() {
          m.organization = c.organization.clone();
        }
        if m.job_title.is_empty() {
          m.job_title = c.job_title.clone();
        }
      }
      None => by_id.push((c.id.clone(), c.clone())),
    }
  }
  let mut out = Vec::new();
  for pr in r.linked.iter().chain(&r.matched) {
    let (Some(p), Some((_, c))) = (
      everyone.iter().find(|p| p.key == pr.person),
      by_id.iter().find(|(id, _)| *id == pr.contact),
    ) else {
      continue;
    };
    let mine: HashSet<String> = p
      .header
      .emails
      .iter()
      .map(|e| decompose(&email_key(e)))
      .collect();
    let mut seen_email = HashSet::new();
    let emails = c
      .emails
      .iter()
      .filter(|e| {
        let k = decompose(&email_key(e));
        !mine.contains(&k) && seen_email.insert(k)
      })
      .cloned()
      .collect();
    let mut seen_phone: HashSet<String> = p.header.phones.iter().cloned().collect();
    let fresh: Vec<String> = c
      .phones
      .iter()
      .filter(|ph| ph.chars().count() >= MIN_PHONE && seen_phone.insert((*ph).clone()))
      .cloned()
      .collect();
    let w = PullWork {
      person: p,
      card_name: display_name(c),
      emails,
      phones: fresh
        .iter()
        .filter(|ph| !doubtful_uk(ph))
        .cloned()
        .collect(),
      linkedin: (p.header.linkedin.is_empty() && c.linkedin.len() == 1)
        .then(|| c.linkedin[0].clone()),
      company: (p.header.company.is_empty() && !c.organization.is_empty())
        .then(|| c.organization.clone()),
      role: (p.header.role.is_empty() && !c.job_title.is_empty()).then(|| c.job_title.clone()),
      doubtful: fresh.iter().filter(|ph| doubtful_uk(ph)).cloned().collect(),
    };
    if !w.is_empty() {
      out.push(w);
    }
  }
  out.sort_by(|a, b| a.person.key.cmp(&b.person.key));
  out
}

/// An empty header value: nothing, or a link with no text ("[](mailto:)").
fn is_blank(v: &str) -> bool {
  let t = v.trim_matches([' ', '\t']);
  t.is_empty() || t.starts_with("[](")
}

/// The About text with the pulled values in its header: appended to a field
/// with a value, set on a blank one, and a missing field added before the
/// Tags line (or after the last header line).
fn pulled_about(text: &str, w: &PullWork) -> String {
  let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
  let mut put = |field: &str, values: Vec<String>, append: bool| {
    if values.is_empty() {
      return;
    }
    let prefix = format!("- {field}:");
    let label: String = format!("{prefix:<LABEL_WIDTH$}")
      .chars()
      .take(LABEL_WIDTH)
      .collect();
    let joined = values.join(", ");
    if let Some(i) = lines.iter().position(|l| l.starts_with(&prefix)) {
      let current = lines[i][prefix.len()..].to_owned();
      lines[i] = if is_blank(&current) || !append {
        format!("{label}{joined}")
      } else {
        format!("{label}{}, {joined}", current.trim_matches([' ', '\t']))
      };
    } else if let Some(t) = lines.iter().position(|l| l.starts_with("- Tags:")) {
      lines.insert(t, format!("{label}{joined}"));
    } else if let Some(last) = lines
      .iter()
      .rposition(|l| l.starts_with("- ") && l.contains(':'))
    {
      lines.insert(last + 1, format!("{label}{joined}"));
    }
  };
  if let Some(c) = &w.company {
    put("Company", vec![c.clone()], false);
  }
  if let Some(r) = &w.role {
    put("Role", vec![r.clone()], false);
  }
  if let Some(s) = &w.linkedin {
    put(
      "LinkedIn",
      vec![format!("[in/{s}/]({})", linkedin_url(s))],
      false,
    );
  }
  put(
    "Email",
    w.emails
      .iter()
      .map(|e| format!("[{e}](mailto:{e})"))
      .collect(),
    true,
  );
  put("Phone", w.phones.clone(), true);
  lines.join("\n")
}

fn pull(store: &mut dyn Store, o: &Options, group: &str, config: &Config) -> Result<ExitCode> {
  let everyone = people(&config.data)?;
  let cards = store.cards()?;
  let report = matches(&everyone, &cards, group);
  let (only, skip) = (key_set(&o.names), key_set(&o.skips));
  let work: Vec<PullWork> = pull_work(&report, &everyone, &cards)
    .into_iter()
    .filter(|w| {
      let k = decompose(&w.person.key);
      (only.is_empty() || only.contains(&k)) && !skip.contains(&k)
    })
    .collect();
  if !(o.apply && !work.is_empty()) {
    print_pull(&work.iter().collect::<Vec<_>>(), None, None);
    return Ok(ExitCode::SUCCESS);
  }
  let dir = config
    .backup_dir
    .join(format!("{}-pull", contacts::stamp()));
  fs::create_dir_all(&dir)?;
  let chosen: Vec<&PullWork> = work.iter().take(o.limit.unwrap_or(usize::MAX)).collect();
  let mut outcome: Outcome = Vec::new();
  for w in &chosen {
    // The About is written back, so it is read as exact text or not at all
    let text = match w.person.about.as_deref().map_or(Ok(None), read_exact) {
      Ok(Some(text)) => text,
      Ok(None) => {
        outcome.push((w.person.key.clone(), Some("no About file".to_owned())));
        continue;
      }
      Err(e) => {
        outcome.push((w.person.key.clone(), Some(e.to_string())));
        continue;
      }
    };
    let saved = dir.join(format!(
      "{}.md",
      marker_path(&w.person.key).replace('/', "_")
    ));
    fs::write(saved, &text)?;
    if let Some(about) = &w.person.about {
      crate::fs::write_atomic(about, pulled_about(&text, w).as_bytes())?;
    }
    outcome.push((w.person.key.clone(), None));
  }
  print_pull(&chosen, Some(&outcome), Some(&dir));
  Ok(exit_code(Some(&outcome)))
}

fn print_pull(work: &[&PullWork], outcome: Option<&Outcome>, backup: Option<&Path>) {
  if let Some(outcome) = outcome {
    let bad: Vec<&(String, Option<String>)> = outcome.iter().filter(|(_, e)| e.is_some()).collect();
    println!("pplr sync --pull --apply\n");
    println!("  people updated in pplr  {}", outcome.len() - bad.len());
    println!("  failed                  {}", bad.len());
    for (person, error) in &bad {
      println!("  {person}: {}", error.as_deref().unwrap_or_default());
    }
  } else {
    println!("pplr sync --pull (dry run: nothing written; --apply to write)\n");
    println!("  people to update in pplr  {}", work.len());
  }
  for w in work {
    println!("\n  {}  <-  {}", w.person.key, w.card_name);
    for line in w.summary() {
      println!("      {line}");
    }
  }
  if let Some(backup) = backup {
    println!("\nBackup of each About file: {}", backup.display());
  }
}

// --plan and --apply-plan

fn plan_mode(store: &mut dyn Store, o: &Options, group: &str, config: &Config) -> Result<ExitCode> {
  let now = config.now()?;
  let plan = plan::plan(
    &people(&config.data)?,
    &store.cards()?,
    group,
    now.strftime("%Y-%m-%d %H:%M").to_string(),
  );
  if o.json {
    println!("{}", to_swift_json(&serde_json::to_value(&plan)?));
    return Ok(ExitCode::SUCCESS);
  }
  let shown = o.out.clone().unwrap_or_else(|| {
    format!(
      "{}/_out/contacts-plan-{}.xlsx",
      config.data.display(),
      now.strftime("%Y%m%d-%H%M")
    )
  });
  // A relative --out is under the data folder, where the legacy script ran
  let out = config.data.join(&shown);
  if let Some(dir) = out.parent() {
    fs::create_dir_all(dir)?;
  }
  workbook::write(&plan, &out)?;
  println!("pplr sync --plan (read-only)");
  println!();
  for ((confidence, decision), n) in plan.proposals() {
    println!("  {n}\t{confidence} / {decision}");
  }
  println!();
  println!("Workbook: {shown}");
  Ok(ExitCode::SUCCESS)
}

fn apply_plan(
  store: &mut dyn Store,
  o: &Options,
  group: &str,
  config: &Config,
) -> Result<ExitCode> {
  let path = std::env::current_dir()?.join(o.workbook.as_deref().unwrap_or_default());
  if !path.is_file() {
    println!("Error: no such workbook: {}", path.display());
    return Ok(ExitCode::FAILURE);
  }
  let decisions = match workbook::decisions(&path) {
    Err(e) if e.is::<workbook::Mistake>() => {
      eprintln!("{e}");
      return Ok(ExitCode::FAILURE);
    }
    read => read?,
  };
  // --name: only these people (eg to try one card first)
  let only = key_set(&o.names);
  let chosen: Vec<Reviewed> = decisions
    .into_iter()
    .filter(|d| only.is_empty() || only.contains(&decompose(&d.person)))
    .collect();
  let everyone = people(&config.data)?;
  let cards = store.cards()?;
  let (work, problems) = plan::plan_work(&chosen, &everyone, &cards, group);
  let todo: Vec<&Work> = work.iter().filter(|w| !w.is_empty()).collect();
  let mut backup = None;
  let outcome = if o.apply && !todo.is_empty() {
    backup = Some(store.backup(&config.backup_dir)?);
    let done = write_each(
      &todo,
      o.limit,
      |w| w.person().key.clone(),
      |w| match w {
        Work::Add { person, card } => store
          .create(card, group)
          .and_then(|id| write_inetloc(&inetloc_path(person), &id)),
        Work::Change {
          person,
          card,
          change,
          ..
        } => store
          .update(&card.id, change, group)
          .and_then(|()| write_inetloc(&inetloc_path(person), &card.id)),
      },
    );
    store.save()?;
    Some(done)
  } else {
    None
  };
  print_work(
    &work,
    &todo,
    &problems,
    outcome.as_ref(),
    backup.as_deref(),
    if o.apply { None } else { o.limit },
  );
  Ok(exit_code(outcome.as_ref()))
}

fn print_work(
  work: &[Work],
  todo: &[&Work],
  problems: &[String],
  outcome: Option<&Outcome>,
  backup: Option<&Path>,
  limit: Option<usize>,
) {
  let count = |d: Decision| todo.iter().filter(|w| w.decision() == d).count();
  println!(
    "{}",
    if outcome.is_some() {
      "pplr sync --apply-plan --apply"
    } else {
      "pplr sync --apply-plan (dry run: nothing written; --apply to write)"
    }
  );
  println!();
  println!("  add (new iCloud cards)  {}", count(Decision::Add));
  println!("  update                  {}", count(Decision::Update));
  println!("  link only               {}", count(Decision::LinkOnly));
  println!("  nothing to change       {}", work.len() - todo.len());
  if !problems.is_empty() {
    println!("  cannot be done          {}", problems.len());
  }
  if let Some(outcome) = outcome {
    let bad: Vec<_> = outcome.iter().filter(|(_, e)| e.is_some()).collect();
    let ok = outcome.len() - bad.len();
    println!("  written now             {ok}");
    println!("  failed                  {}", bad.len());
    println!("  still to write          {}", todo.len() - ok);
    print_failed(&bad);
  } else {
    println!(
      "\nWould write{}:",
      limit
        .map(|l| format!(" (the first {l})"))
        .unwrap_or_default()
    );
    for w in todo.iter().take(limit.unwrap_or(usize::MAX)) {
      let card = match w {
        Work::Change { card, .. } => format!("  <->  {}", display_name(card)),
        Work::Add { .. } => String::new(),
      };
      println!(
        "  {:<10} {}{card}  [{}]",
        w.decision(),
        w.person().key,
        w.summary()
      );
    }
  }
  if !problems.is_empty() {
    println!("\nCannot be done:");
    for p in problems {
      println!("  {p}");
    }
  }
  if let Some(b) = backup {
    println!("\nBackup: {}", b.display());
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;
  use crate::about::Header;
  use crate::contacts::LabelledValue;

  // The fixture's Zsófia Varga, her accent one character (as an About file
  // holds it) or an o and a combining acute (as a card can): one value to
  // Swift's String equality, two to Rust's (vc decision 22)
  pub(crate) const KEY: &str = "V/Varga, Zs\u{f3}fia";
  pub(crate) const KEY_DECOMPOSED: &str = "V/Varga, Zso\u{301}fia";
  pub(crate) const COMPOSED: &str = "zs\u{f3}fia-varga";
  pub(crate) const DECOMPOSED: &str = "zso\u{301}fia-varga";

  /// A person with this key and LinkedIn slug, and nothing else.
  pub(crate) fn person(key: &str, linkedin: &str) -> Person {
    let name = key.split_once('/').map_or(key, |(_, n)| n);
    let (family, given) = name.split_once(", ").unwrap_or((name, ""));
    Person {
      key: key.to_owned(),
      given: given.to_owned(),
      family: family.to_owned(),
      dir: PathBuf::from(key),
      about: None,
      picture: None,
      aliases: Vec::new(),
      header: Header {
        linkedin: linkedin.to_owned(),
        ..Header::default()
      },
    }
  }

  /// A card with this id and these slugs, and nothing else.
  pub(crate) fn card(id: &str, linkedin: &[&str]) -> Card {
    Card {
      id: id.to_owned(),
      given: String::new(),
      family: String::new(),
      organization: String::new(),
      job_title: String::new(),
      emails: Vec::new(),
      phones: Vec::new(),
      urls: Vec::new(),
      linkedin: linkedin.iter().map(|s| (*s).to_owned()).collect(),
      groups: Vec::new(),
      account: None,
      has_image: None,
    }
  }

  #[test]
  fn a_slug_in_either_unicode_form_is_one_value_in_a_field_diff() -> Result<()> {
    let p = person(KEY, COMPOSED);
    assert!(
      diffs(&p, &card("c1", &[DECOMPOSED]))
        .iter()
        .all(|d| d.field != "linkedin"),
      "the card's slug, decomposed, differed from the About file's"
    );
    // A slug that differs still does, each side shown as it holds it
    let other = format!("{DECOMPOSED}-2");
    let differs = diffs(&p, &card("c2", &[&other]));
    let linkedin = differs
      .iter()
      .find(|d| d.field == "linkedin")
      .ok_or("a different slug was not a difference")?;
    assert_eq!(linkedin.pplr, [COMPOSED]);
    assert_eq!(linkedin.contacts, [other]);
    Ok(())
  }

  #[test]
  fn a_card_is_matched_by_a_slug_in_either_unicode_form() {
    let p = person(KEY, COMPOSED);
    let c = card("c1", &[DECOMPOSED]);
    let r = matches(
      std::slice::from_ref(&p),
      std::slice::from_ref(&c),
      DEFAULT_GROUP,
    );
    assert!(
      r.matched
        .iter()
        .any(|m| m.person == p.key && m.contact == c.id && m.how == "linkedin"),
      "no match by the slug in its other form: {r:?}"
    );
  }

  #[test]
  fn a_group_named_in_either_unicode_form_is_the_group() {
    let mut c = card("c1", &[]);
    c.groups.push("\u{c9}quipe".to_owned());
    assert!(in_group(&c, "E\u{301}quipe"));
    assert!(!needs_group(&c, "E\u{301}quipe"));
  }

  #[test]
  fn a_pplr_url_naming_the_key_in_the_other_unicode_form_links_the_card() {
    // A pplr URL's first form names the key itself, percent-encoded
    let p = person(KEY, "");
    let mut c = card("c1", &[]);
    c.urls.push(LabelledValue {
      label: "pplr".to_owned(),
      value: "pplr://V/Varga,%20Zso%CC%81fia".to_owned(),
    });
    let r = matches(
      std::slice::from_ref(&p),
      std::slice::from_ref(&c),
      DEFAULT_GROUP,
    );
    assert!(
      r.linked
        .iter()
        .any(|l| l.person == p.key && l.contact == c.id),
      "the card was not linked: {r:?}"
    );
  }

  #[test]
  fn a_name_typed_in_the_other_unicode_form_confirms_a_name_only_match() {
    let p = person(KEY, "");
    let mut c = card("c1", &[]);
    c.given = "Zs\u{f3}fia".to_owned();
    c.family = "Varga".to_owned();
    let r = matches(
      std::slice::from_ref(&p),
      std::slice::from_ref(&c),
      DEFAULT_GROUP,
    );
    let typed = KEY_DECOMPOSED
      .split_once('/')
      .map_or(KEY_DECOMPOSED, |(_, n)| n);
    let plan = plan_link(&r, &[typed.to_owned()], false);
    assert!(
      plan.to_link.iter().any(|l| l.person == p.key),
      "the name, typed decomposed, confirmed nothing; unknown: {:?}",
      plan.unknown_names
    );
    assert!(plan.unknown_names.is_empty(), "{:?}", plan.unknown_names);
  }

  #[test]
  fn a_name_to_pull_or_skip_is_found_in_either_unicode_form() {
    let typed = KEY.split_once('/').map_or(KEY, |(_, n)| n);
    assert!(key_set(&[typed.to_owned()]).contains(&decompose(KEY_DECOMPOSED)));
  }

  #[test]
  fn a_set_keeps_one_spelling_of_each_value() {
    assert_eq!(
      sorted_set([DECOMPOSED, COMPOSED].map(str::to_owned)),
      [DECOMPOSED]
    );
  }

  #[test]
  fn a_uk_number_has_ten_digits_and_never_starts_with_four() {
    assert!(!doubtful_uk("+447700900123"));
    assert!(doubtful_uk("+44412345678"));
    assert!(doubtful_uk("+4420123"));
    assert!(!doubtful_uk("+491512345678"));
  }

  #[test]
  fn a_person_key_gains_its_letter() {
    assert_eq!(person_key("Kemp, Jon"), "K/Kemp, Jon");
    assert_eq!(person_key("de Vries, Annelies"), "D/de Vries, Annelies");
    assert_eq!(person_key("K/Kemp, Jon"), "K/Kemp, Jon");
  }

  #[test]
  fn blank_values_are_empty_or_a_textless_link() {
    assert!(is_blank("   "));
    assert!(is_blank(" [](mailto:)"));
    assert!(!is_blank(" [a@b.c](mailto:a@b.c)"));
  }
}
