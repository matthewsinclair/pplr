//! sync --plan's proposal and --apply-plan's work, as pure functions (ST0001
//! WP11): each pplr person scored against the cards the strict check left
//! free, a decision proposed for each, and a reviewed decision turned back
//! into the changes it asks of a card. The engine's rules, rule for rule.

use crate::contacts::{Card, CardChange, MarkerWrite, NewCard};
use crate::marker::{marker_paths, marker_url};
use crate::normalise::{decompose, email_key, fold};
use crate::sync::{
  How, card_name, display_name, is_home, marker_write, matches, needs_group, pplr_marker, same_text,
};
use crate::tree::Person;
use serde::{Serialize, Serializer};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::PathBuf;

/// What to do about one person, in the order rows of one confidence run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Decision {
  Update,
  LinkOnly,
  Add,
  NoChange,
  Skip,
}

impl Decision {
  /// Every decision, in the workbook's order.
  pub const ALL: [Decision; 5] = [
    Decision::Add,
    Decision::Update,
    Decision::LinkOnly,
    Decision::NoChange,
    Decision::Skip,
  ];

  /// As the workbook and the reports write it.
  pub fn as_str(self) -> &'static str {
    match self {
      Decision::Add => "Add",
      Decision::Update => "Update",
      Decision::LinkOnly => "Link only",
      Decision::NoChange => "No change",
      Decision::Skip => "Skip",
    }
  }

  /// The decision a reviewer wrote, if it is one.
  pub fn named(name: &str) -> Option<Decision> {
    Decision::ALL.into_iter().find(|d| d.as_str() == name)
  }
}

impl fmt::Display for Decision {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.pad(self.as_str())
  }
}

impl Serialize for Decision {
  fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(self.as_str())
  }
}

/// How sure the plan is of a row, least certain first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Confidence {
  Low,
  Medium,
  High,
}

impl Confidence {
  /// As the workbook and the reports write it.
  pub fn as_str(self) -> &'static str {
    match self {
      Confidence::Low => "Low",
      Confidence::Medium => "Medium",
      Confidence::High => "High",
    }
  }
}

impl fmt::Display for Confidence {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.pad(self.as_str())
  }
}

impl Serialize for Confidence {
  fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
    s.serialize_str(self.as_str())
  }
}

/// A score this good is a match; below it, a person is proposed as an Add.
const MATCH: i32 = 40;
/// A twin scores at least this, and within TWIN_GAP of the best.
const TWIN: i32 = 60;
const TWIN_GAP: i32 = 20;
/// High confidence needs this score and no rename.
const HIGH: i32 = 80;
/// A shared email is worth this, with or without a name.
const SHARED_EMAIL: i32 = 40;
/// How many other near cards a row lists.
const OTHERS: usize = 3;

/// Common English short forms, both ways. Enough to catch Bill for William;
/// anything subtler is for the reviewer.
const NICKNAMES: &[(&str, &[&str])] = &[
  ("william", &["bill", "billy", "will", "liam"]),
  ("robert", &["rob", "bob", "bobby", "robbie", "bert"]),
  ("richard", &["rich", "rick", "richie", "dick"]),
  ("james", &["jim", "jimmy", "jamie"]),
  ("alexander", &["alex", "sandy", "xander"]),
  ("alexandra", &["alex", "sandra", "lexi"]),
  ("alexis", &["alex"]),
  ("michael", &["mike", "mick", "mikey"]),
  ("christopher", &["chris", "kit"]),
  ("christine", &["chris", "tina"]),
  ("christina", &["chris", "tina"]),
  ("thomas", &["tom", "tommy"]),
  ("anthony", &["tony"]),
  ("andrew", &["andy", "drew"]),
  ("daniel", &["dan", "danny"]),
  ("david", &["dave", "davy"]),
  ("edward", &["ed", "eddie", "ted", "ned"]),
  ("elizabeth", &["liz", "beth", "lizzie", "betty", "eliza"]),
  ("katherine", &["kate", "kathy", "katie", "kat"]),
  ("catherine", &["cath", "cathy", "kate", "cat"]),
  ("kathryn", &["kate", "kathy", "katie"]),
  ("jennifer", &["jen", "jenny"]),
  ("joseph", &["joe", "joey"]),
  ("joshua", &["josh"]),
  ("jonathan", &["jon", "jonny", "john"]),
  ("john", &["jon", "johnny", "jack"]),
  ("matthew", &["matt", "matty"]),
  ("nicholas", &["nick", "nicky"]),
  ("patrick", &["pat", "paddy"]),
  ("peter", &["pete"]),
  ("samuel", &["sam", "sammy"]),
  ("samantha", &["sam", "sammy"]),
  ("stephen", &["steve", "stevie"]),
  ("steven", &["steve", "stevie"]),
  ("timothy", &["tim", "timmy"]),
  ("benjamin", &["ben", "benny"]),
  ("gregory", &["greg"]),
  ("jeffrey", &["jeff"]),
  ("geoffrey", &["geoff", "jeff"]),
  ("kenneth", &["ken", "kenny"]),
  ("margaret", &["maggie", "meg", "peggy", "margo"]),
  ("rebecca", &["becky", "bec"]),
  ("susan", &["sue", "suzy"]),
  ("suzanne", &["sue", "suzy"]),
  ("victoria", &["vicky", "tori"]),
  ("zachary", &["zach", "zac"]),
  ("frederick", &["fred", "freddie"]),
  ("gerald", &["gerry"]),
  ("lawrence", &["larry", "laurie"]),
  ("ronald", &["ron", "ronnie"]),
  ("donald", &["don", "donny"]),
  ("douglas", &["doug"]),
  ("philip", &["phil"]),
  ("phillip", &["phil"]),
  ("raymond", &["ray"]),
  ("charles", &["charlie", "chuck", "chas"]),
  ("henry", &["harry", "hank"]),
  ("jacob", &["jake"]),
  ("nathan", &["nate", "nat"]),
  ("nathaniel", &["nate", "nat"]),
  ("abigail", &["abby", "abi"]),
  ("deborah", &["deb", "debbie"]),
  ("pamela", &["pam"]),
  ("angela", &["angie"]),
  ("theodore", &["theo", "ted"]),
  ("oliver", &["ollie"]),
  ("alfred", &["alf", "alfie"]),
  ("augustus", &["gus"]),
  ("angus", &["gus"]),
  ("gustav", &["gus"]),
  ("vincent", &["vince", "vinny"]),
  ("leonard", &["leo", "len", "lenny"]),
  ("nicola", &["nicky", "nic"]),
  ("nicole", &["nicky", "nic"]),
  ("jessica", &["jess", "jessie"]),
  ("michelle", &["shelley", "chelle"]),
  ("francis", &["frank", "fran"]),
  ("frances", &["fran", "frankie"]),
  ("cameron", &["cam"]),
  ("maximilian", &["max"]),
  ("maxwell", &["max"]),
];

/// Mail providers anyone can use: a shared one says nothing.
const GENERIC_DOMAINS: [&str; 19] = [
  "gmail.com",
  "googlemail.com",
  "hotmail.com",
  "hotmail.co.uk",
  "outlook.com",
  "live.com",
  "live.co.uk",
  "yahoo.com",
  "yahoo.co.uk",
  "icloud.com",
  "me.com",
  "mac.com",
  "aol.com",
  "btinternet.com",
  "protonmail.com",
  "proton.me",
  "msn.com",
  "fastmail.com",
  "fastmail.fm",
];

// The plan, as the engine encoded it (keys camelCase; an absent value left out)

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
  pub r#ref: String,
  pub name: String,
  pub score: i32,
  pub why: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanRow {
  pub person: String,
  pub decision: Decision,
  pub confidence: Confidence,
  pub why: String,
  /// The card's ref, eg C0042.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub card: Option<String>,
  /// Other cards that look like the same person.
  pub dupes: Vec<String>,
  pub changes: Vec<String>,
  /// The other near cards, best first.
  pub candidates: Vec<Candidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanCard {
  pub r#ref: String,
  pub id: String,
  pub name: String,
  pub given: String,
  pub family: String,
  pub organization: String,
  pub job_title: String,
  pub emails: Vec<String>,
  pub phones: Vec<String>,
  pub linkedin: Vec<String>,
  pub account: String,
  pub groups: Vec<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub marker: Option<String>,
  /// The pplr person the plan would give this card to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub proposed_for: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanPerson {
  pub key: String,
  pub given: String,
  pub family: String,
  pub company: String,
  pub role: String,
  pub emails: Vec<String>,
  pub phones: Vec<String>,
  pub linkedin: String,
  pub marker: String,
  /// About/<First Surname> (Picture).jpg, if there is one.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub picture: Option<PathBuf>,
  pub dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plan {
  pub generated: String,
  pub group: String,
  pub rows: Vec<PlanRow>,
  pub people: Vec<PlanPerson>,
  pub cards: Vec<PlanCard>,
}

impl Plan {
  /// How many rows of each confidence and decision, least certain first and
  /// then by the decision's name, as the engine listed them:
  /// ((confidence, decision), count).
  pub fn proposals(&self) -> Vec<((Confidence, Decision), usize)> {
    let mut counts: HashMap<(Confidence, Decision), usize> = HashMap::new();
    for r in &self.rows {
      *counts.entry((r.confidence, r.decision)).or_default() += 1;
    }
    let mut out: Vec<_> = counts.into_iter().collect();
    out.sort_by_key(|((c, d), _)| (*c, d.as_str()));
    out
  }
}

// Scoring

/// Letters and digits only, folded: "O'Brien-Smith" -> "obriensmith".
fn squash(s: &str) -> String {
  fold(s)
    .to_lowercase()
    .chars()
    .filter(|c| c.is_alphanumeric())
    .collect()
}

/// Name parts, folded: "Dixon-Black" -> ["dixon", "black"].
fn tokens(s: &str) -> Vec<String> {
  fold(s)
    .to_lowercase()
    .split(|c: char| !c.is_alphanumeric())
    .filter(|t| !t.is_empty())
    .map(str::to_owned)
    .collect()
}

fn levenshtein(a: &str, b: &str) -> usize {
  let a: Vec<char> = a.chars().collect();
  let b: Vec<char> = b.chars().collect();
  let mut prev: Vec<usize> = (0..=b.len()).collect();
  for (i, x) in a.iter().enumerate() {
    let mut cur = vec![i + 1; b.len() + 1];
    for (j, y) in b.iter().enumerate() {
      cur[j + 1] = (prev[j + 1] + 1)
        .min(cur[j] + 1)
        .min(prev[j] + usize::from(x != y));
    }
    prev = cur;
  }
  prev[b.len()]
}

fn is_nickname(a: &str, b: &str) -> bool {
  NICKNAMES.iter().any(|(full, short)| {
    (*full == a && short.contains(&b))
      || (*full == b && short.contains(&a))
      || (short.contains(&a) && short.contains(&b))
  })
}

/// Bill and William, Alex and Alexander, Jon and John, Mark and Marc.
fn given_compatible(a: &str, b: &str) -> bool {
  let (Some(x), Some(y)) = (tokens(a).into_iter().next(), tokens(b).into_iter().next()) else {
    return false;
  };
  let shorter = x.chars().count().min(y.chars().count());
  x == y
    || is_nickname(&x, &y)
    || (shorter >= 3 && (x.starts_with(&y) || y.starts_with(&x)))
    || (shorter >= 4 && levenshtein(&x, &y) <= 1)
}

/// A shared part of a double-barrelled name, or a one-letter slip.
fn surname_compatible(a: &str, b: &str) -> bool {
  let long = |s: &str| -> HashSet<String> {
    tokens(s)
      .into_iter()
      .filter(|t| t.chars().count() >= 3)
      .collect()
  };
  if !long(a).is_disjoint(&long(b)) {
    return true;
  }
  let (x, y) = (squash(a), squash(b));
  x.chars().count().min(y.chars().count()) >= 5 && levenshtein(&x, &y) <= 1
}

/// The email domains, keyed as `same_text` compares, less the generic ones.
fn domains(emails: &[String]) -> HashSet<String> {
  emails
    .iter()
    .filter_map(|e| e.split('@').next_back())
    .filter(|d| !GENERIC_DOMAINS.contains(d))
    .map(decompose)
    .collect()
}

/// How near a card is to a person by name, company and email domain; None
/// when the names have nothing in common.
fn score(p: &Person, c: &Card) -> Option<(i32, String)> {
  let (pg, pf, cg, cf) = (
    squash(&p.given),
    squash(&p.family),
    squash(&c.given),
    squash(&c.family),
  );
  if cg.is_empty() && cf.is_empty() {
    return None;
  }
  let (mut s, mut why) = if pg == cg && pf == cf {
    (60, vec!["same name".to_owned()])
  } else if pg == cf && pf == cg {
    (45, vec!["name reversed".to_owned()])
  } else if pf == cf && given_compatible(&p.given, &c.given) {
    (
      45,
      vec![format!("same surname, {} for {}", c.given, p.given)],
    )
  } else if pg == cg && surname_compatible(&p.family, &c.family) {
    (
      40,
      vec![format!("same given name, {} for {}", c.family, p.family)],
    )
  } else if given_compatible(&p.given, &c.given) && surname_compatible(&p.family, &c.family) {
    (25, vec!["similar name".to_owned()])
  } else {
    return None;
  };
  let (pc, cc) = (squash(&p.header.company), squash(&c.organization));
  if pc.chars().count() >= 3 && cc.chars().count() >= 3 && (pc.contains(&cc) || cc.contains(&pc)) {
    s += 20;
    why.push("same company".to_owned());
  }
  if !domains(&p.header.emails).is_disjoint(&domains(&c.emails)) {
    s += 15;
    why.push("same email domain".to_owned());
  }
  Some((s, why.join(", ")))
}

// What a decision would change

fn or_empty(s: &str) -> &str {
  if s.is_empty() { "empty" } else { s }
}

/// What Update writes to the card: pplr wins on name, company and role; its
/// emails, phones and LinkedIn are added; the card is linked as --link links
/// it. Nothing on the card is removed.
fn card_change(p: &Person, c: &Card, group: &str) -> CardChange {
  let h = &p.header;
  let name = format!("{} {}", p.given, p.family);
  CardChange {
    name: (fold(&name) != fold(&card_name(c))).then(|| (p.given.clone(), p.family.clone())),
    organization: (!h.company.is_empty() && fold(&h.company) != fold(&c.organization))
      .then(|| h.company.clone()),
    job_title: (!h.role.is_empty() && fold(&h.role) != fold(&c.job_title)).then(|| h.role.clone()),
    add_emails: h
      .emails
      .iter()
      .filter(|e| {
        !c.emails
          .iter()
          .any(|x| same_text(&email_key(x), &email_key(e)))
      })
      .cloned()
      .collect(),
    add_phones: h
      .phones
      .iter()
      .filter(|ph| !c.phones.contains(ph))
      .cloned()
      .collect(),
    add_linkedin: (!h.linkedin.is_empty() && !c.linkedin.iter().any(|s| same_text(s, &h.linkedin)))
      .then(|| h.linkedin.clone()),
    marker: marker_write(c, &p.key),
    add_group: needs_group(c, group),
  }
}

/// What Link only writes of a change: the pplr URL and the group.
fn link_only(change: &CardChange) -> CardChange {
  CardChange {
    marker: change.marker.clone(),
    add_group: change.add_group,
    ..CardChange::default()
  }
}

/// Whether a change writes a field of the card, not only the link.
fn touches_fields(change: &CardChange) -> bool {
  *change != link_only(change)
}

/// A change as the Changes column lists it: each field set, beside what the
/// card has now, each value added, then the link.
fn change_lines(change: &CardChange, c: &Card, group: &str) -> Vec<String> {
  let mut out = Vec::new();
  if let Some((given, family)) = &change.name {
    out.push(format!(
      "name: {given} {family} (was {})",
      or_empty(&card_name(c))
    ));
  }
  if let Some(company) = &change.organization {
    out.push(format!(
      "company: {company} (was {})",
      or_empty(&c.organization)
    ));
  }
  if let Some(role) = &change.job_title {
    out.push(format!("role: {role} (was {})", or_empty(&c.job_title)));
  }
  out.extend(change.add_emails.iter().map(|e| format!("+ email {e}")));
  out.extend(change.add_phones.iter().map(|ph| format!("+ phone {ph}")));
  if let Some(slug) = &change.add_linkedin {
    out.push(format!("+ LinkedIn {slug}"));
  }
  match &change.marker {
    Some(MarkerWrite { replace: true, .. }) => out.push("pplr URL updated".to_owned()),
    Some(_) => out.push("+ pplr URL".to_owned()),
    None => {}
  }
  if change.add_group {
    out.push(format!("+ {group} group"));
  }
  out
}

fn count_of(n: usize, one: &str, many: &str) -> String {
  if n == 1 {
    one.to_owned()
  } else {
    format!("{n} {many}")
  }
}

/// The card Add makes from a person.
fn new_card(p: &Person) -> NewCard {
  let h = &p.header;
  NewCard {
    given: p.given.clone(),
    family: p.family.clone(),
    organization: h.company.clone(),
    job_title: h.role.clone(),
    emails: h.emails.clone(),
    phones: h.phones.clone(),
    linkedin: (!h.linkedin.is_empty()).then(|| h.linkedin.clone()),
    marker: marker_url(&p.key),
    picture: p.picture.clone(),
  }
}

/// An Add as the Changes column lists it.
fn add_changes(card: &NewCard, group: &str) -> Vec<String> {
  let mut parts = vec!["name".to_owned()];
  if !card.organization.is_empty() {
    parts.push("company".to_owned());
  }
  if !card.job_title.is_empty() {
    parts.push("role".to_owned());
  }
  if !card.emails.is_empty() {
    parts.push(count_of(card.emails.len(), "email", "emails"));
  }
  if !card.phones.is_empty() {
    parts.push(count_of(card.phones.len(), "phone", "phones"));
  }
  if card.linkedin.is_some() {
    parts.push("LinkedIn".to_owned());
  }
  if card.picture.is_some() {
    parts.push("photo".to_owned());
  }
  vec![
    format!("new iCloud card: {}", parts.join(", ")),
    "+ pplr URL".to_owned(),
    format!("+ {group} group"),
  ]
}

// The plan

/// A strict tier's words. plan() reads the linked and matched pairs alone,
/// so a name-only pair never comes here, and Name shares phone's words, as
/// it did under 2.0.0's catch-all.
fn tier(how: How) -> &'static str {
  match how {
    How::Linked => "linked by pplr URL",
    How::Email => "same email",
    How::Linkedin => "same LinkedIn",
    How::Phone | How::Name => "same phone",
  }
}

const NO_ACCOUNT: &str = "but that card is in no account (Other Known or a directory)";

struct Scored<'a> {
  card: &'a Card,
  score: i32,
  why: String,
}

/// A decision for every pplr person: the strict tiers first, then the best
/// of the free cards by score. `generated` is the plan's timestamp. Nothing
/// is read from disk: the people come with their pictures.
pub fn plan(people: &[Person], raw_cards: &[Card], group: &str, generated: String) -> Plan {
  // One entry per real card, each with a short ref for the reviewer
  let mut seen = HashSet::new();
  let mut cards: Vec<Card> = raw_cards
    .iter()
    .filter(|c| seen.insert(c.id.clone()))
    .cloned()
    .collect();
  cards.sort_by(|a, b| {
    (fold(&a.family), fold(&a.given), &a.organization, &a.id).cmp(&(
      fold(&b.family),
      fold(&b.given),
      &b.organization,
      &b.id,
    ))
  });
  let ref_of: HashMap<&str, String> = cards
    .iter()
    .enumerate()
    .map(|(i, c)| (c.id.as_str(), format!("C{:04}", i + 1)))
    .collect();
  let card_ref = |c: &Card| ref_of.get(c.id.as_str()).cloned().unwrap_or_default();
  let card_by_id: HashMap<&str, &Card> = cards.iter().map(|c| (c.id.as_str(), c)).collect();
  let by_key: HashMap<&str, &Person> = people.iter().map(|p| (p.key.as_str(), p)).collect();
  let report = matches(people, &cards, group);

  let mut rows: Vec<PlanRow> = Vec::new();
  let mut done: HashSet<&str> = HashSet::new();
  let strong: HashSet<&str> = report
    .linked
    .iter()
    .chain(&report.matched)
    .map(|pr| pr.contact.as_str())
    .collect();
  for pr in report.linked.iter().chain(&report.matched) {
    let (Some(p), Some(c)) = (
      by_key.get(pr.person.as_str()),
      card_by_id.get(pr.contact.as_str()),
    ) else {
      continue;
    };
    done.insert(p.key.as_str());
    let tier = tier(pr.how);
    if pr.no_account {
      rows.push(PlanRow {
        person: p.key.clone(),
        decision: Decision::Add,
        confidence: Confidence::Medium,
        why: format!("{tier}, {NO_ACCOUNT}"),
        card: None,
        dupes: Vec::new(),
        changes: add_changes(&new_card(p), group),
        candidates: vec![Candidate {
          r#ref: card_ref(c),
          name: display_name(c),
          score: 100,
          why: tier.to_owned(),
        }],
      });
      continue;
    }
    let change = card_change(p, c, group);
    let decision = if touches_fields(&change) {
      Decision::Update
    } else if change == CardChange::default() {
      Decision::NoChange
    } else {
      Decision::LinkOnly
    };
    // A rename is the reviewer's call: pplr's own spelling can be the wrong one
    let renames = change.name.is_some();
    rows.push(PlanRow {
      person: p.key.clone(),
      decision,
      confidence: if renames {
        Confidence::Medium
      } else {
        Confidence::High
      },
      why: format!("{tier}{}", if renames { ", names differ" } else { "" }),
      card: Some(card_ref(c)),
      dupes: Vec::new(),
      changes: change_lines(&change, c, group),
      candidates: Vec::new(),
    });
  }

  // Everyone else: score the cards the strict tiers left free. A shared email
  // (several cards carrying it) outranks any name.
  let free: Vec<&Card> = cards
    .iter()
    .filter(|c| !strong.contains(c.id.as_str()))
    .collect();
  for p in people.iter().filter(|p| !done.contains(p.key.as_str())) {
    let mine: HashSet<String> = p
      .header
      .emails
      .iter()
      .map(|e| decompose(&email_key(e)))
      .collect();
    let mut cands: Vec<Scored> = free
      .iter()
      .filter_map(|c| {
        let shares = c
          .emails
          .iter()
          .any(|e| mine.contains(&decompose(&email_key(e))));
        match score(p, c) {
          Some((s, why)) if shares => Some(Scored {
            card: c,
            score: s + SHARED_EMAIL,
            why: format!("{why}, same email"),
          }),
          Some((s, why)) => Some(Scored {
            card: c,
            score: s,
            why,
          }),
          None if shares => Some(Scored {
            card: c,
            score: SHARED_EMAIL,
            why: "same email, different name".to_owned(),
          }),
          None => None,
        }
      })
      .collect();
    // Best first; the sort is stable, so ties keep the cards' order
    cands.sort_by_key(|s| std::cmp::Reverse((s.score, is_home(s.card))));
    let as_cand = |s: &Scored| Candidate {
      r#ref: card_ref(s.card),
      name: display_name(s.card),
      score: s.score,
      why: s.why.clone(),
    };
    let Some(best) = cands.first().filter(|b| b.score >= MATCH) else {
      rows.push(PlanRow {
        person: p.key.clone(),
        decision: Decision::Add,
        confidence: if cands.is_empty() {
          Confidence::High
        } else {
          Confidence::Low
        },
        why: if cands.is_empty() {
          "no card found"
        } else {
          "only weak candidates"
        }
        .to_owned(),
        card: None,
        dupes: Vec::new(),
        changes: add_changes(&new_card(p), group),
        candidates: cands.iter().map(as_cand).collect(),
      });
      continue;
    };
    if best.card.account.as_deref() == Some("none") {
      rows.push(PlanRow {
        person: p.key.clone(),
        decision: Decision::Add,
        confidence: Confidence::Low,
        why: format!("{}, {NO_ACCOUNT}", best.why),
        card: None,
        dupes: Vec::new(),
        changes: add_changes(&new_card(p), group),
        candidates: cands.iter().map(as_cand).collect(),
      });
      continue;
    }
    // Cards nearly as good as the best look like duplicates of one person
    let twins: Vec<&Scored> = cands[1..]
      .iter()
      .filter(|s| {
        s.score >= TWIN
          && s.score >= best.score - TWIN_GAP
          && s.card.account.as_deref() != Some("none")
      })
      .collect();
    let twin_ids: HashSet<&str> = twins.iter().map(|s| s.card.id.as_str()).collect();
    let change = card_change(p, best.card, group);
    let renames = change.name.is_some();
    let confidence = if !twins.is_empty() {
      Confidence::Low
    } else if best.score >= HIGH && !renames {
      Confidence::High
    } else if best.score >= TWIN {
      Confidence::Medium
    } else {
      Confidence::Low
    };
    rows.push(PlanRow {
      person: p.key.clone(),
      decision: if touches_fields(&change) {
        Decision::Update
      } else {
        Decision::LinkOnly
      },
      confidence,
      why: if twins.is_empty() {
        best.why.clone()
      } else {
        format!(
          "{}; {} cards look like this person",
          best.why,
          twins.len() + 1
        )
      },
      card: Some(card_ref(best.card)),
      dupes: twins.iter().map(|s| card_ref(s.card)).collect(),
      changes: change_lines(&change, best.card, group),
      candidates: cands[1..]
        .iter()
        .filter(|s| !twin_ids.contains(s.card.id.as_str()))
        .take(OTHERS)
        .map(as_cand)
        .collect(),
    });
  }

  // One card proposed for two people: neither can be trusted
  let mut claims: HashMap<String, Vec<String>> = HashMap::new();
  for r in &rows {
    if let Some(c) = &r.card {
      claims.entry(c.clone()).or_default().push(r.person.clone());
    }
  }
  for r in &mut rows {
    let Some(others) = r.card.as_ref().and_then(|c| claims.get(c)) else {
      continue;
    };
    if others.len() > 1 {
      r.confidence = Confidence::Low;
      let rest: Vec<&str> = others
        .iter()
        .filter(|o| **o != r.person)
        .map(String::as_str)
        .collect();
      r.why = format!("{}; card also proposed for {}", r.why, rest.join(", "));
    }
  }
  rows.sort_by(|a, b| {
    (a.confidence, a.decision, &a.person).cmp(&(b.confidence, b.decision, &b.person))
  });

  let mut proposed: HashMap<String, String> = HashMap::new();
  for r in &rows {
    if let Some(c) = &r.card {
      proposed.insert(c.clone(), r.person.clone());
    }
    for d in &r.dupes {
      proposed.insert(d.clone(), format!("dupe? {}", r.person));
    }
  }
  Plan {
    generated,
    group: group.to_owned(),
    people: people
      .iter()
      .map(|p| PlanPerson {
        key: p.key.clone(),
        given: p.given.clone(),
        family: p.family.clone(),
        company: p.header.company.clone(),
        role: p.header.role.clone(),
        emails: p.header.emails.clone(),
        phones: p.header.phones.clone(),
        linkedin: p.header.linkedin.clone(),
        marker: marker_url(&p.key),
        picture: p.picture.clone(),
        dir: p.dir.clone(),
      })
      .collect(),
    cards: cards
      .iter()
      .map(|c| PlanCard {
        r#ref: card_ref(c),
        id: c.id.clone(),
        name: display_name(c),
        given: c.given.clone(),
        family: c.family.clone(),
        organization: c.organization.clone(),
        job_title: c.job_title.clone(),
        emails: c.emails.clone(),
        phones: c.phones.clone(),
        linkedin: c.linkedin.clone(),
        account: c.account.clone().unwrap_or_else(|| "home".to_owned()),
        groups: c.groups.clone(),
        marker: pplr_marker(c),
        proposed_for: proposed.get(&card_ref(c)).cloned(),
      })
      .collect(),
    rows,
  }
}

// A reviewed plan, applied

/// One reviewed row: the person, the decision, and the card's id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reviewed {
  pub person: String,
  pub decision: Decision,
  pub card: Option<String>,
}

/// What one decision will do to Contacts: a new card, or these changes to one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Work<'a> {
  Add {
    person: &'a Person,
    card: NewCard,
  },
  Change {
    person: &'a Person,
    /// Update or Link only.
    decision: Decision,
    card: &'a Card,
    change: CardChange,
  },
}

impl Work<'_> {
  pub fn person(&self) -> &Person {
    match self {
      Work::Add { person, .. } | Work::Change { person, .. } => person,
    }
  }

  pub fn decision(&self) -> Decision {
    match self {
      Work::Add { .. } => Decision::Add,
      Work::Change { decision, .. } => *decision,
    }
  }

  /// What it writes, in a few words.
  pub fn summary(&self) -> String {
    let w: Vec<String> = match self {
      Work::Add { card, .. } => {
        let mut w = vec!["new iCloud card".to_owned()];
        if !card.emails.is_empty() {
          w.push(format!("{} email", card.emails.len()));
        }
        if !card.phones.is_empty() {
          w.push(format!("{} phone", card.phones.len()));
        }
        if card.linkedin.is_some() {
          w.push("LinkedIn".to_owned());
        }
        if card.picture.is_some() {
          w.push("photo".to_owned());
        }
        return w.join(", ");
      }
      Work::Change { change: c, .. } => {
        let mut w = Vec::new();
        let mut put = |yes: bool, what: &str| {
          if yes {
            w.push(what.to_owned());
          }
        };
        put(c.name.is_some(), "name");
        put(c.organization.is_some(), "company");
        put(c.job_title.is_some(), "role");
        put(
          !c.add_emails.is_empty(),
          &format!("+{} email", c.add_emails.len()),
        );
        put(
          !c.add_phones.is_empty(),
          &format!("+{} phone", c.add_phones.len()),
        );
        put(c.add_linkedin.is_some(), "+LinkedIn");
        match &c.marker {
          Some(m) if m.replace => put(true, "pplr URL updated"),
          Some(_) => put(true, "+pplr URL"),
          None => {}
        }
        put(c.add_group, "+group");
        w
      }
    };
    if w.is_empty() {
      "nothing to change".to_owned()
    } else {
      w.join(", ")
    }
  }

  /// Nothing to write: a Change whose card already has it all.
  pub fn is_empty(&self) -> bool {
    matches!(self, Work::Change { change, .. } if *change == CardChange::default())
  }
}

/// The work a reviewed plan asks for, and what cannot be done.
pub fn plan_work<'a>(
  decisions: &[Reviewed],
  people: &'a [Person],
  cards: &'a [Card],
  group: &str,
) -> (Vec<Work<'a>>, Vec<String>) {
  // A reviewed row names its person as the workbook holds the name: found as
  // `same_text` compares
  let by_key: HashMap<String, &Person> = people.iter().map(|p| (decompose(&p.key), p)).collect();
  let mut by_id: HashMap<&str, &Card> = HashMap::new();
  for c in cards {
    by_id.entry(c.id.as_str()).or_insert(c);
  }
  // A card that already carries the person's pplr URL (or a former one): an
  // Add has been done, so doing it again would make a duplicate
  let marked_for: HashSet<String> = cards.iter().filter_map(pplr_marker).collect();
  let mut work = Vec::new();
  let mut problems = Vec::new();
  for d in decisions {
    let Some(p) = by_key.get(&decompose(&d.person)).copied() else {
      problems.push(format!("{}: no such person in pplr", d.person));
      continue;
    };
    match d.decision {
      Decision::Add => {
        if marker_paths(p).any(|m| marked_for.contains(&m)) {
          continue;
        }
        work.push(Work::Add {
          person: p,
          card: new_card(p),
        });
      }
      decision @ (Decision::Update | Decision::LinkOnly) => {
        let Some(c) = d.card.as_deref().and_then(|id| by_id.get(id)).copied() else {
          problems.push(format!("{}: the card is no longer in Contacts", d.person));
          continue;
        };
        if c.account.as_deref() == Some("none") {
          problems.push(format!(
            "{}: {} is in no account (Other Known or a directory), so cannot be written",
            d.person,
            display_name(c)
          ));
          continue;
        }
        let change = card_change(p, c, group);
        work.push(Work::Change {
          person: p,
          decision,
          card: c,
          change: if decision == Decision::Update {
            change
          } else {
            link_only(&change)
          },
        });
      }
      Decision::NoChange | Decision::Skip => {}
    }
  }
  (work, problems)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::sync::tests::{COMPOSED, DECOMPOSED, KEY, KEY_DECOMPOSED, card, person};

  #[test]
  fn names_compare_by_nickname_prefix_and_slip() {
    assert!(given_compatible("Bill", "William"));
    assert!(given_compatible("Alex", "Alexander"));
    assert!(given_compatible("Mark", "Marc"));
    assert!(!given_compatible("Ada", "Grace"));
    assert!(surname_compatible("Dixon-Black", "Black"));
    assert!(surname_compatible("Thistlewood", "Thistlewod"));
    assert!(!surname_compatible("Lovelace", "Hopper"));
  }

  #[test]
  fn levenshtein_counts_edits() {
    assert_eq!(levenshtein("kitten", "sitting"), 3);
    assert_eq!(levenshtein("", "abc"), 3);
    assert_eq!(levenshtein("same", "same"), 0);
  }

  #[test]
  fn generic_mail_domains_say_nothing() {
    let d = domains(&["a@gmail.com".to_owned(), "b@acme.example".to_owned()]);
    assert_eq!(d, HashSet::from(["acme.example".to_owned()]));
  }

  #[test]
  fn decisions_sort_update_link_add_then_the_rest() {
    let mut d = Decision::ALL;
    d.sort();
    assert_eq!(
      d.map(Decision::as_str),
      ["Update", "Link only", "Add", "No change", "Skip"]
    );
  }

  #[test]
  fn a_decision_is_read_back_by_the_name_it_is_written_with() {
    for d in Decision::ALL {
      assert_eq!(Decision::named(d.as_str()), Some(d));
    }
    assert_eq!(Decision::named("link only"), None);
  }

  #[test]
  fn update_adds_no_slug_the_card_holds_in_the_other_unicode_form() {
    let change = card_change(&person(KEY, COMPOSED), &card("c1", &[DECOMPOSED]), "PPLR");
    assert_eq!(change.add_linkedin, None);
  }

  #[test]
  fn a_reviewed_row_finds_its_person_named_in_the_other_unicode_form() {
    let people = [person(KEY, "")];
    let row = Reviewed {
      person: KEY_DECOMPOSED.to_owned(),
      decision: Decision::Add,
      card: None,
    };
    let (work, problems) = plan_work(&[row], &people, &[], "PPLR");
    assert!(problems.is_empty(), "{problems:?}");
    assert!(work.iter().any(|w| w.person().key == KEY));
  }
}
