//! `pplr tags` and `pplr tag`: each person's tags, in About/tags.yaml, against
//! the vocabulary in _pplr/vocabulary.yaml (the port of tags.py).
//!
//! A tag in tags.yaml is a bare word (a hand tag, yours: never rewritten) or
//! `{tag: t, source: auto|inferred, ...}`; its facet comes from the vocabulary.

use crate::Result;
use crate::config::Config;
use crate::fs::{read_exact, read_text};
use crate::marker::marker_url;
use crate::tree::{display_name, letter_of, names, people, person_dirs};
use crate::yaml::{self, Value};
use regex::Regex;
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The facets, in the order everything is shown and sorted by.
pub const FACETS: [&str; 7] = [
  "role",
  "function",
  "sector",
  "org",
  "relationship",
  "place",
  "cadence",
];
/// The sort place of a tag outside the vocabulary: after every facet.
const UNKNOWN_FACET: usize = 9;
const RENDER_NOTE: &str = "<!-- from tags.yaml: pplr tags render -->";
const TAGS_DIR: &str = "_tags";
/// The most tags a page lists under "Seen with".
const SEEN_WITH: usize = 20;
/// The width `show` pads a facet's name to.
const FACET_WIDTH: usize = 13;
const SOURCES: [&str; 3] = ["hand", "auto", "inferred"];
/// What `pplr tags` with no command prints: tags.py's usage, laid out as it
/// was, naming `pplr tags` (ST0001 WP13, vc decision 14).
const USAGE: &str = "
    pplr tags check                 every tags.yaml against the vocabulary
    pplr tags show PERSON           a person's tags, by facet
    pplr tags render [--dry-run]    the \"- Tags:\" line in each About, linking each tag to
                                    its page, and the tag pages in $PPLR_DATA/_tags/
    pplr tags index                 .index/tags_index.json: everyone with name, marker,
                                    role, company, picture and tags by facet
    pplr tags edit PERSON +tag -tag add or remove hand tags, checked against the vocabulary
    pplr tags apply FILE [--dry-run]
                                    write tags.yaml from a JSON list of
                                    {person, auto: [tag], inferred: [{tag, evidence}], hand: [tag]};
                                    hand tags already in the file are kept
";

/// The vocabulary: each tag's facet, what each old or folded word becomes,
/// and the file as read (for each tag's note).
pub struct Vocabulary {
  pub facet_of: HashMap<String, &'static str>,
  pub fold: HashMap<String, Vec<String>>,
  raw: Value,
}

impl Vocabulary {
  pub fn load(root: &Path) -> Result<Self> {
    let file = root.join("_pplr/vocabulary.yaml");
    // No vocabulary yet (a new pplr, or a test): no tag is known, and indexes still build
    let raw = match fs::read_to_string(&file) {
      Ok(text) => yaml::parse(&text).map_err(|e| format!("{}: {e}", file.display()))?,
      Err(_) => Value::Null,
    };
    let (mut facet_of, mut fold) = (HashMap::new(), HashMap::new());
    for facet in FACETS {
      for (tag, d) in raw.get(facet).map(Value::pairs).unwrap_or_default() {
        let tag = tag.py_str();
        facet_of.insert(tag.clone(), facet);
        fold.insert(tag.clone(), vec![tag.clone()]);
        for also in d.get("also").map(Value::as_seq).unwrap_or_default() {
          fold.insert(also.py_str(), vec![tag.clone()]);
        }
      }
    }
    for (old, tags) in raw.get("split").map(Value::pairs).unwrap_or_default() {
      fold.insert(
        old.py_str(),
        tags.as_seq().iter().map(Value::py_str).collect(),
      );
    }
    Ok(Self {
      facet_of,
      fold,
      raw,
    })
  }

  fn order(&self, tag: &str) -> usize {
    self
      .facet_of
      .get(tag)
      .and_then(|f| FACETS.iter().position(|x| x == f))
      .unwrap_or(UNKNOWN_FACET)
  }

  /// A tag's note in the vocabulary, if it has one.
  fn note(&self, facet: &str, tag: &str) -> Option<String> {
    let entry = self.raw.get(facet)?.get(tag)?;
    entry
      .get("note")
      .filter(|n| n.is_truthy())
      .map(Value::py_str)
  }
}

/// One entry of a tags.yaml.
#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
  pub tag: String,
  /// None when a mapping names no source.
  pub source: Option<String>,
  /// The mapping's other keys, in the file's order.
  pub extra: Vec<(String, Value)>,
}

impl Tag {
  fn hand(tag: &str) -> Self {
    Self {
      tag: tag.to_owned(),
      source: Some("hand".to_owned()),
      extra: Vec::new(),
    }
  }

  fn is_hand(&self) -> bool {
    self.source.as_deref() == Some("hand")
  }

  fn source(&self, file: &Path) -> Result<&str> {
    self
      .source
      .as_deref()
      .ok_or_else(|| format!("{}: the tag {} has no source", file.display(), self.tag).into())
  }
}

fn key_of(root: &Path, dir: &Path) -> String {
  dir.strip_prefix(root).unwrap_or(dir).display().to_string()
}

fn tags_file(dir: &Path) -> PathBuf {
  dir.join("About/tags.yaml")
}

/// A person's tags, a bare word read as a hand tag.
pub fn read_tags(dir: &Path) -> Result<Vec<Tag>> {
  let file = tags_file(dir);
  let Ok(text) = fs::read_to_string(&file) else {
    return Ok(Vec::new());
  };
  let doc = yaml::parse(&text).map_err(|e| format!("{}: {e}", file.display()))?;
  let tags = match &doc {
    Value::Null => Value::Null,
    Value::Map(_) => doc.get("tags").cloned().unwrap_or(Value::Null),
    _ => return Err(format!("{}: not a mapping with a tags list", file.display()).into()),
  };
  Ok(
    tags
      .as_seq()
      .iter()
      .map(|t| match t {
        Value::Map(pairs) => Tag {
          tag: t.get("tag").map(Value::py_str).unwrap_or_default(),
          source: t.get("source").map(Value::py_str),
          extra: pairs
            .iter()
            .map(|(k, v)| (k.py_str(), v.clone()))
            .filter(|(k, _)| k != "tag" && k != "source")
            .collect(),
        },
        other => Tag::hand(&other.py_str()),
      })
      .collect(),
  )
}

/// tags.yaml from tags.py's own emitter: by facet, hand tags first in each.
fn write_tags(dir: &Path, tags: &[Tag], vocab: &Vocabulary) -> Result<()> {
  let file = tags_file(dir);
  let mut sorted: Vec<&Tag> = tags.iter().collect();
  sorted.sort_by_key(|t| (vocab.order(&t.tag), !t.is_hand()));
  let mut lines = vec![
    format!(
      "# Tags for {}. A bare word is your own tag; auto and inferred tags",
      display_name(dir)
    ),
    "# are rewritten by pplr, hand tags never are. Vocabulary: _pplr/vocabulary.yaml".to_owned(),
    "tags:".to_owned(),
  ];
  for t in sorted {
    let source = t.source(&file)?;
    if source == "hand" {
      lines.push(format!("  - {}", t.tag));
      continue;
    }
    let extra: String = t
      .extra
      .iter()
      .map(|(k, v)| {
        let shown = if k == "evidence" {
          json_dumps(v)
        } else {
          v.py_str()
        };
        format!(", {k}: {shown}")
      })
      .collect();
    lines.push(format!("  - {{tag: {}, source: {source}{extra}}}", t.tag));
  }
  fs::write(&file, lines.join("\n") + "\n")?;
  Ok(())
}

/// `json.dumps(v, ensure_ascii=False)` for the values tags.yaml holds.
fn json_dumps(v: &Value) -> String {
  match v {
    Value::Str(s) | Value::Date(s) if matches!(v, Value::Str(_)) => {
      serde_json::to_string(s).unwrap_or_default()
    }
    Value::Null => "null".to_owned(),
    Value::Bool(b) => b.to_string(),
    other => serde_json::to_string(&other.py_str()).unwrap_or_default(),
  }
}

/// The person a name means; a missing one stops the command, as sys.exit did.
fn find_person(root: &Path, name: &str) -> std::result::Result<PathBuf, ExitCode> {
  let name = name.split_once('/').map_or(name, |(_, rest)| rest);
  let dir = root.join(letter_of(name)).join(name);
  if dir.is_dir() {
    Ok(dir)
  } else {
    eprintln!("pplr tags: no such person: {name}");
    Err(ExitCode::FAILURE)
  }
}

/// Python's repr of a str, for the messages tags.py wrote with `!r`.
fn repr(s: &str) -> String {
  Value::Str(s.to_owned()).py_repr()
}

pub fn run_tags(args: &[String], config: &Config) -> Result<ExitCode> {
  let Some((command, rest)) = args
    .split_first()
    .filter(|(c, _)| !matches!(c.as_str(), "-h" | "--help"))
  else {
    println!("{USAGE}");
    return Ok(ExitCode::SUCCESS);
  };
  let dry = rest.iter().any(|a| a == "--dry-run");
  let rest: Vec<&String> = rest.iter().filter(|a| *a != "--dry-run").collect();
  let root = &config.data;
  match (command.as_str(), rest.as_slice()) {
    ("check", _) => check(root),
    ("show", [name, ..]) => show(root, name),
    ("render", _) => render(root, dry, config),
    ("index", _) => index(root),
    ("edit", [name, changes @ ..]) if !changes.is_empty() => {
      let changes: Vec<String> = changes.iter().map(|c| (*c).clone()).collect();
      edit(root, name, &changes)
    }
    ("apply", [file, ..]) => apply(root, Path::new(file), dry, config),
    ("show", _) | ("edit", _) | ("apply", _) => {
      let needs = match command.as_str() {
        "show" => "show PERSON",
        "edit" => "edit PERSON +tag -tag",
        _ => "apply FILE [--dry-run]",
      };
      println!("pplr tags: {command} needs more: pplr tags {needs}");
      Ok(ExitCode::from(2))
    }
    _ => {
      println!("pplr tags: unknown command {}", repr(command));
      Ok(ExitCode::from(2))
    }
  }
}

/// `pplr tag "Surname, First" [+tag] [-tag]...`: show, or edit then show; an
/// edit re-renders the (Contact).md pages, since a cadence tag changes them.
pub fn run_tag(args: &[String], config: &Config) -> Result<ExitCode> {
  let Some((person, changes)) = args
    .split_first()
    .filter(|(p, _)| !matches!(p.as_str(), "-h" | "--help"))
  else {
    println!("Usage: pplr tag \"Surname, First\" [+tag] [-tag]...");
    println!("  With no changes, shows their tags. Tags must be in _pplr/vocabulary.yaml.");
    println!("  Run pplr reindex (or pplr tags render) afterwards to refresh the tag pages.");
    return Ok(ExitCode::FAILURE);
  };
  if changes.is_empty() {
    return show(&config.data, person);
  }
  let code = edit(&config.data, person, changes)?;
  if code != ExitCode::SUCCESS {
    return Ok(code);
  }
  // pplr_tag ran `pplr_contact render >/dev/null`: the count goes unshown
  crate::contact::render_all(&config.data, config.today())?;
  Ok(ExitCode::SUCCESS)
}

fn check(root: &Path) -> Result<ExitCode> {
  let vocab = Vocabulary::load(root)?;
  let (mut count, mut problems) = (0, 0);
  for dir in person_dirs(root) {
    if !tags_file(&dir).exists() {
      continue;
    }
    count += 1;
    let key = key_of(root, &dir);
    let tags = match read_tags(&dir) {
      Ok(tags) => tags,
      Err(e) => {
        let first = e.to_string().lines().next().unwrap_or_default().to_owned();
        println!("{key}: unreadable tags.yaml: {first}");
        problems += 1;
        continue;
      }
    };
    let mut seen: Vec<&str> = Vec::new();
    for t in &tags {
      let tag = t.tag.as_str();
      if seen.contains(&tag) {
        println!("{key}: {tag} twice");
        problems += 1;
      }
      seen.push(tag);
      if tag.is_empty()
        || !tag
          .bytes()
          .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
      {
        println!("{key}: {} is not a single lowercase word", repr(tag));
        problems += 1;
      } else if !vocab.facet_of.contains_key(tag) {
        let hint = vocab
          .fold
          .get(tag)
          .map(|f| format!(" (the vocabulary folds it into {})", f.join(", ")))
          .unwrap_or_default();
        println!("{key}: {tag} is not in the vocabulary{hint}");
        problems += 1;
      }
      if !t.source.as_deref().is_some_and(|s| SOURCES.contains(&s)) {
        let shown = t.source.as_deref().map_or_else(|| "None".to_owned(), repr);
        println!("{key}: {tag} has source {shown}");
        problems += 1;
      }
      if t.source.as_deref() == Some("inferred") && vocab.facet_of.get(tag) != Some(&"relationship")
      {
        println!("{key}: {tag} is inferred but not a relationship tag");
        problems += 1;
      }
    }
  }
  println!("pplr tags check: {count} tags.yaml, {problems} problem(s)");
  Ok(if problems > 0 {
    ExitCode::FAILURE
  } else {
    ExitCode::SUCCESS
  })
}

fn show(root: &Path, name: &str) -> Result<ExitCode> {
  let vocab = Vocabulary::load(root)?;
  let dir = match find_person(root, name) {
    Ok(dir) => dir,
    Err(code) => return Ok(code),
  };
  let file = tags_file(&dir);
  let tags = read_tags(&dir)?;
  for facet in FACETS {
    let here: Vec<&Tag> = tags
      .iter()
      .filter(|t| vocab.facet_of.get(&t.tag) == Some(&facet))
      .collect();
    if here.is_empty() {
      continue;
    }
    let shown: Vec<String> = here
      .iter()
      .map(|t| {
        let source = t.source(&file)?;
        Ok(if source == "hand" {
          t.tag.clone()
        } else {
          format!("{} ({source})", t.tag)
        })
      })
      .collect::<Result<_>>()?;
    println!("{facet:<FACET_WIDTH$} {}", shown.join("  "));
  }
  let other: Vec<&str> = tags
    .iter()
    .filter(|t| !vocab.facet_of.contains_key(&t.tag))
    .map(|t| t.tag.as_str())
    .collect();
  if !other.is_empty() {
    println!("{:<FACET_WIDTH$} {}", "not in vocab", other.join("  "));
  }
  Ok(ExitCode::SUCCESS)
}

fn edit(root: &Path, name: &str, changes: &[String]) -> Result<ExitCode> {
  let vocab = Vocabulary::load(root)?;
  let dir = match find_person(root, name) {
    Ok(dir) => dir,
    Err(code) => return Ok(code),
  };
  let mut tags = read_tags(&dir)?;
  for change in changes {
    let mut chars = change.chars();
    let op = chars.next();
    let tag = chars.as_str().to_lowercase();
    if !matches!(op, Some('+' | '-')) || tag.is_empty() {
      eprintln!("pplr tag: say +tag or -tag, not {}", repr(change));
      return Ok(ExitCode::FAILURE);
    }
    if op == Some('+') {
      if !vocab.facet_of.contains_key(&tag) {
        let hint = vocab
          .fold
          .get(&tag)
          .map(|f| format!("; the vocabulary folds it into {}", f.join(", ")))
          .unwrap_or_default();
        eprintln!("pplr tag: {tag} is not in the vocabulary{hint}");
        return Ok(ExitCode::FAILURE);
      }
      tags.retain(|t| t.tag != tag);
      tags.push(Tag::hand(&tag));
    } else {
      tags.retain(|t| t.tag != tag);
    }
  }
  write_tags(&dir, &tags, &vocab)?;
  show(root, name)
}

fn apply(root: &Path, file: &Path, dry: bool, config: &Config) -> Result<ExitCode> {
  let vocab = Vocabulary::load(root)?;
  let today = config.today().to_string();
  let text = fs::read_to_string(file).map_err(|e| format!("{}: {e}", file.display()))?;
  let rows: serde_json::Value =
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))?;
  let strings = |v: Option<&serde_json::Value>| -> Vec<String> {
    v.and_then(serde_json::Value::as_array)
      .into_iter()
      .flatten()
      .map(|t| t.as_str().map_or_else(|| t.to_string(), str::to_owned))
      .collect()
  };
  let mut written = 0;
  for row in rows.as_array().into_iter().flatten() {
    let person = row
      .get("person")
      .and_then(serde_json::Value::as_str)
      .unwrap_or_default();
    let dir = match find_person(root, person) {
      Ok(dir) => dir,
      Err(code) => return Ok(code),
    };
    let mut hand: Vec<String> = read_tags(&dir)?
      .into_iter()
      .filter(Tag::is_hand)
      .map(|t| t.tag)
      .chain(strings(row.get("hand")))
      .collect();
    hand.sort();
    hand.dedup();
    let mut tags: Vec<Tag> = hand.iter().map(|t| Tag::hand(t)).collect();
    for tag in strings(row.get("auto"))
      .into_iter()
      .filter(|t| !hand.contains(t))
    {
      tags.push(Tag {
        tag,
        source: Some("auto".to_owned()),
        extra: vec![("at".to_owned(), Value::Date(today.clone()))],
      });
    }
    for inferred in row
      .get("inferred")
      .and_then(serde_json::Value::as_array)
      .into_iter()
      .flatten()
    {
      let tag = inferred
        .get("tag")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
      if hand.contains(&tag) {
        continue;
      }
      let evidence = inferred
        .get("evidence")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
      tags.push(Tag {
        tag,
        source: Some("inferred".to_owned()),
        extra: vec![("evidence".to_owned(), Value::Str(evidence.to_owned()))],
      });
    }
    let bad: Vec<&str> = tags
      .iter()
      .filter(|t| !vocab.facet_of.contains_key(&t.tag))
      .map(|t| t.tag.as_str())
      .collect();
    if !bad.is_empty() {
      println!(
        "{person}: not in the vocabulary, left out: {}",
        bad.join(", ")
      );
      tags.retain(|t| vocab.facet_of.contains_key(&t.tag));
    }
    if !dry {
      write_tags(&dir, &tags, &vocab)?;
    }
    written += 1;
  }
  println!(
    "pplr tags apply{}: {written} tags.yaml {}",
    if dry { " (dry run)" } else { "" },
    if dry { "to write" } else { "written" }
  );
  Ok(ExitCode::SUCCESS)
}

/// The first `*(About).md` in a person's About folder.
fn about_of(dir: &Path) -> Option<PathBuf> {
  names(&dir.join("About"))
    .into_iter()
    .find(|n| n.ends_with("(About).md"))
    .map(|n| dir.join("About").join(n))
}

/// The generated Tags line, each tag linking to its page.
fn tags_line(tags: &[Tag], vocab: &Vocabulary) -> Option<String> {
  if tags.is_empty() {
    return None;
  }
  let mut sorted: Vec<&Tag> = tags.iter().collect();
  sorted.sort_by_key(|t| vocab.order(&t.tag));
  let links: Vec<String> = sorted
    .iter()
    .map(|t| format!("[#{}](../../../{TAGS_DIR}/{}.md)", t.tag, t.tag))
    .collect();
  Some(format!("- Tags:     {} {RENDER_NOTE}", links.join(" ")))
}

/// The About text with its Tags line written, replaced or removed: an old
/// generated line is replaced in place, a new one goes after the last
/// `- Field:` line of the first header list.
fn with_tags_line(text: &str, line: Option<&str>) -> Result<String> {
  let old = Regex::new(&format!(
    r"(?m)^- Tags:.*{}[ \t]*\n",
    regex::escape(RENDER_NOTE)
  ))?;
  if let Some(m) = old.find(text) {
    let new_line = line.map(|l| format!("{l}\n")).unwrap_or_default();
    return Ok(format!(
      "{}{new_line}{}",
      &text[..m.start()],
      &text[m.end()..]
    ));
  }
  let Some(line) = line else {
    return Ok(text.to_owned());
  };
  let header = Regex::new(r"(?m)^- [A-Za-z]+:.*\n")?;
  let mut matches = header.find_iter(text);
  let Some(mut last) = matches.next() else {
    return Ok(text.to_owned());
  };
  for m in matches {
    if m.start() != last.end() {
      break;
    }
    last = m;
  }
  Ok(format!(
    "{}{line}\n{}",
    &text[..last.end()],
    &text[last.end()..]
  ))
}

fn render(root: &Path, dry: bool, config: &Config) -> Result<ExitCode> {
  let vocab = Vocabulary::load(root)?;
  let (mut changed, mut same) = (0, 0);
  for dir in person_dirs(root) {
    let Some(about) = about_of(&dir) else {
      continue;
    };
    let Some(text) = read_exact(&about)? else {
      continue;
    };
    let line = tags_line(&read_tags(&dir)?, &vocab);
    let new = with_tags_line(&text, line.as_deref())?;
    if new == text {
      same += 1;
      continue;
    }
    changed += 1;
    if !dry {
      fs::write(&about, new)?;
    }
  }
  println!(
    "pplr tags render{}: {changed} About file(s) {}, {same} already right",
    if dry { " (dry run)" } else { "" },
    if dry { "to change" } else { "changed" }
  );
  if !dry {
    pages(root, &vocab, &config.today().to_string())?;
    index(root)?;
  }
  Ok(ExitCode::SUCCESS)
}

/// `^- <Field>:` value, its link text when it starts with one, as tags.py read it.
fn header_field(text: &str, field: &str) -> String {
  let Ok(re) = Regex::new(&format!(r"(?m)^- {}:[ \t]*(.*)$", regex::escape(field))) else {
    return String::new();
  };
  let Some(value) = re
    .captures(text)
    .and_then(|c| c.get(1))
    .map(|m| m.as_str().trim())
  else {
    return String::new();
  };
  let link = value
    .strip_prefix('[')
    .and_then(|rest| rest.find("](").map(|close| &rest[..close]));
  link.unwrap_or(value).trim().to_owned()
}

fn md_link_target(path: &str) -> String {
  if path.contains([' ', '(', ')', '<', '>']) {
    format!("<{path}>")
  } else {
    path.to_owned()
  }
}

struct PagePerson {
  name: String,
  link: String,
  what: String,
  places: Vec<String>,
  tags: Vec<String>,
}

/// _tags/index.md, every tag by facet with counts, and _tags/<tag>.md for
/// each, everyone with the tag and the tags seen with it; rewritten in full,
/// and pages of tags no longer used removed.
fn pages(root: &Path, vocab: &Vocabulary, day: &str) -> Result<()> {
  let out = root.join(TAGS_DIR);
  fs::create_dir_all(&out)?;
  let mut people_by_key: Vec<(String, PagePerson)> = Vec::new();
  for dir in person_dirs(root) {
    let tags: Vec<String> = read_tags(&dir)?
      .into_iter()
      .map(|t| t.tag)
      .filter(|t| vocab.facet_of.contains_key(t))
      .collect();
    if tags.is_empty() {
      continue;
    }
    let about = about_of(&dir);
    let text = match &about {
      Some(a) => read_text(a)?.unwrap_or_default(),
      None => String::new(),
    };
    let (role, company) = (header_field(&text, "Role"), header_field(&text, "Company"));
    let what = [role, company]
      .into_iter()
      .filter(|x| !x.is_empty())
      .collect::<Vec<_>>()
      .join(" at ");
    let target = about.as_deref().unwrap_or(&dir);
    let rel = format!(
      "../{}",
      target.strip_prefix(root).unwrap_or(target).display()
    );
    let places = tags
      .iter()
      .filter(|t| vocab.facet_of.get(*t) == Some(&"place"))
      .cloned()
      .collect();
    people_by_key.push((
      key_of(root, &dir),
      PagePerson {
        name: display_name(&dir),
        link: md_link_target(&rel),
        what,
        places,
        tags,
      },
    ));
  }
  let person = |key: &str| people_by_key.iter().find(|(k, _)| k == key).map(|(_, p)| p);
  let mut by_tag: Vec<(String, Vec<String>)> = Vec::new();
  for (key, p) in &people_by_key {
    for tag in &p.tags {
      match by_tag.iter_mut().find(|(t, _)| t == tag) {
        Some((_, keys)) => keys.push(key.clone()),
        None => by_tag.push((tag.clone(), vec![key.clone()])),
      }
    }
  }
  let note = format!(
    "<!-- generated by pplr tags render, {day}: edit tags in each person's About/tags.yaml -->"
  );
  let mut written = vec!["index.md".to_owned()];
  for (tag, keys) in &mut by_tag {
    let sort_key = |k: &String| {
      let name = person(k).map(|p| p.name.clone()).unwrap_or_default();
      let last = name
        .split(' ')
        .next_back()
        .unwrap_or_default()
        .to_lowercase();
      format!("{last}{}", name.to_lowercase())
    };
    keys.sort_by_key(sort_key);
    let facet = vocab
      .facet_of
      .get(tag.as_str())
      .copied()
      .unwrap_or_default();
    let note_text = vocab
      .note(facet, tag)
      .map(|n| format!(": {n}"))
      .unwrap_or_default();
    let count = format!(
      "{} {}",
      keys.len(),
      if keys.len() == 1 { "person" } else { "people" }
    );
    let mut lines = vec![
      note.clone(),
      format!("# #{tag}"),
      String::new(),
      format!(
        "{}{note_text}. {count}. [All tags](index.md)",
        capitalize(facet)
      ),
      String::new(),
    ];
    let mut with_it: Vec<(String, usize)> = Vec::new();
    for key in keys.iter() {
      let Some(p) = person(key) else { continue };
      let places = p
        .places
        .iter()
        .map(|t| format!("#{t}"))
        .collect::<Vec<_>>()
        .join(" ");
      let extra = [p.what.clone(), places]
        .into_iter()
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>()
        .join(" \u{b7} ");
      let tail = if extra.is_empty() {
        String::new()
      } else {
        format!(": {extra}")
      };
      lines.push(format!("- [{}]({}){tail}", p.name, p.link));
      for other in p.tags.iter().filter(|t| *t != tag) {
        match with_it.iter_mut().find(|(t, _)| t == other) {
          Some((_, n)) => *n += 1,
          None => with_it.push((other.clone(), 1)),
        }
      }
    }
    with_it.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    with_it.truncate(SEEN_WITH);
    if !with_it.is_empty() {
      let seen = with_it
        .iter()
        .map(|(t, n)| format!("[#{t}]({t}.md) {n}"))
        .collect::<Vec<_>>()
        .join(" \u{b7} ");
      lines.extend([
        String::new(),
        "## Seen with".to_owned(),
        String::new(),
        seen,
      ]);
    }
    fs::write(out.join(format!("{tag}.md")), lines.join("\n") + "\n")?;
    written.push(format!("{tag}.md"));
  }
  let mut index = vec![
    note,
    "# Tags".to_owned(),
    String::new(),
    format!(
      "{} tags across {} people. Each tag lists everyone who has it.",
      by_tag.len(),
      people_by_key.len()
    ),
    String::new(),
  ];
  for facet in FACETS {
    let mut here: Vec<&(String, Vec<String>)> = by_tag
      .iter()
      .filter(|(t, _)| vocab.facet_of.get(t.as_str()) == Some(&facet))
      .collect();
    if here.is_empty() {
      continue;
    }
    here.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then_with(|| a.0.cmp(&b.0)));
    let links = here
      .iter()
      .map(|(t, k)| format!("[#{t}]({t}.md) {}", k.len()))
      .collect::<Vec<_>>()
      .join(" \u{b7} ");
    index.extend([
      format!("## {}", capitalize(facet)),
      String::new(),
      links,
      String::new(),
    ]);
  }
  fs::write(out.join("index.md"), index.join("\n"))?;
  for old in names(&out) {
    if old.ends_with(".md") && !written.contains(&old) {
      fs::remove_file(out.join(&old))?;
    }
  }
  println!(
    "pplr tags render: {} tag pages and an index in {}",
    by_tag.len(),
    out.display()
  );
  Ok(())
}

/// Python's str.capitalize for a facet name.
fn capitalize(s: &str) -> String {
  let mut chars = s.chars();
  chars
    .next()
    .map(|c| {
      c.to_uppercase()
        .chain(chars.flat_map(char::to_lowercase))
        .collect()
    })
    .unwrap_or_default()
}

/// The index's facets, in FACETS order, as a JSON object.
struct Facets(Vec<(&'static str, Vec<String>)>);

impl Serialize for Facets {
  fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for (facet, tags) in &self.0 {
      map.serialize_entry(facet, tags)?;
    }
    map.end()
  }
}

#[derive(Serialize)]
struct IndexPerson {
  name: String,
  display: String,
  path: String,
  about: String,
  marker: String,
  role: String,
  company: String,
  picture: Option<String>,
  tags: Vec<String>,
  facets: Facets,
}

#[derive(Serialize)]
struct IndexDoc {
  generated_at: String,
  people_count: usize,
  people: Vec<IndexPerson>,
}

/// .index/tags_index.json: everyone, with marker, role, company, picture and
/// tags by facet, in Python's `json.dump(indent=1, ensure_ascii=False)` form.
fn index(root: &Path) -> Result<ExitCode> {
  let vocab = Vocabulary::load(root)?;
  let rel = |p: &Path| p.strip_prefix(root).unwrap_or(p).display().to_string();
  let mut out = Vec::new();
  for p in people(root)? {
    let about = p.about.as_deref();
    let text = match about {
      Some(a) => read_text(a)?.unwrap_or_default(),
      None => String::new(),
    };
    let tags: Vec<String> = read_tags(&p.dir)?
      .into_iter()
      .map(|t| t.tag)
      .filter(|t| vocab.facet_of.contains_key(t))
      .collect();
    let facets = FACETS
      .iter()
      .map(|f| {
        (
          *f,
          tags
            .iter()
            .filter(|t| vocab.facet_of.get(*t) == Some(f))
            .cloned()
            .collect::<Vec<_>>(),
        )
      })
      .filter(|(_, here)| !here.is_empty())
      .collect();
    out.push(IndexPerson {
      name: p
        .dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default(),
      display: display_name(&p.dir),
      path: p.key.clone(),
      about: about.map(rel).unwrap_or_default(),
      marker: marker_url(&p.key),
      role: header_field(&text, "Role"),
      company: header_field(&text, "Company"),
      picture: p.picture.as_deref().map(rel),
      tags,
      facets: Facets(facets),
    });
  }
  let doc = IndexDoc {
    generated_at: jiff::Timestamp::now()
      .strftime("%Y-%m-%dT%H:%M:%SZ")
      .to_string(),
    people_count: out.len(),
    people: out,
  };
  let mut bytes = Vec::new();
  let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
  doc.serialize(&mut serde_json::Serializer::with_formatter(
    &mut bytes, formatter,
  ))?;
  fs::create_dir_all(root.join(".index"))?;
  fs::write(root.join(".index/tags_index.json"), bytes)?;
  println!(
    "pplr tags index: {} people in .index/tags_index.json",
    doc.people_count
  );
  Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_tags_line_goes_after_the_first_header_list_or_replaces_the_old_one() -> Result<()> {
    let about = "# A\n\n_A_\n- Role: x\n- Company: y\n\n_About_\n- Not: header\n";
    let line = format!("- Tags:     [#cto](../../../_tags/cto.md) {RENDER_NOTE}");
    let added = with_tags_line(about, Some(&line))?;
    assert_eq!(
      added,
      format!("# A\n\n_A_\n- Role: x\n- Company: y\n{line}\n\n_About_\n- Not: header\n")
    );
    assert_eq!(with_tags_line(&added, Some(&line))?, added);
    assert_eq!(with_tags_line(&added, None)?, about);
    Ok(())
  }

  #[test]
  fn header_fields_read_link_text_as_tags_py_did() {
    let text = "- Role:     [CTO](https://x)\n- Company:  Larkspur Ledger \n";
    assert_eq!(header_field(text, "Role"), "CTO");
    assert_eq!(header_field(text, "Company"), "Larkspur Ledger");
    assert_eq!(header_field(text, "Email"), "");
  }

  #[test]
  fn repr_quotes_as_python_does() {
    assert_eq!(repr("Fin-Tech"), "'Fin-Tech'");
    assert_eq!(repr("it's"), "\"it's\"");
  }
}
