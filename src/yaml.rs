//! The YAML pplr's own files are written in, read as PyYAML's `safe_load` read
//! them (ST0001 WP06). The legacy helpers used every value through Python's
//! `str()` and `json.dumps`, so a bare `no` is the boolean False and prints
//! "False", and `at: 2026-09-25` is a date that prints as itself.
//!
//! Why not a crate: the maintained Rust YAML readers follow YAML 1.2, where
//! `no` is the string "no", and none has a date; serde_yaml is deprecated.
//! The files hold block mappings and sequences, flow mappings and sequences,
//! plain and quoted scalars, and comments, which this reads; anything else
//! (block scalars, anchors, tags, documents) is refused by name.

use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Null,
  Bool(bool),
  Int(i64),
  Float(f64),
  /// A YAML 1.1 timestamp with a date only, kept as written (YYYY-MM-DD).
  Date(String),
  Str(String),
  Seq(Vec<Value>),
  /// Keys in the file's order.
  Map(Vec<(Value, Value)>),
}

impl Value {
  /// The value as Python's `str()` prints it.
  pub fn py_str(&self) -> String {
    match self {
      Value::Null => "None".to_owned(),
      Value::Bool(true) => "True".to_owned(),
      Value::Bool(false) => "False".to_owned(),
      Value::Int(i) => i.to_string(),
      Value::Float(f) => py_float(*f),
      Value::Date(d) => d.clone(),
      Value::Str(s) => s.clone(),
      Value::Seq(items) => format!(
        "[{}]",
        items
          .iter()
          .map(Value::py_repr)
          .collect::<Vec<_>>()
          .join(", ")
      ),
      Value::Map(pairs) => format!(
        "{{{}}}",
        pairs
          .iter()
          .map(|(k, v)| format!("{}: {}", k.py_repr(), v.py_repr()))
          .collect::<Vec<_>>()
          .join(", ")
      ),
    }
  }

  /// The value as Python's `repr()` prints it.
  pub fn py_repr(&self) -> String {
    match self {
      Value::Str(s) => py_repr_str(s),
      Value::Date(d) => format!(
        "datetime.date({})",
        d.split('-')
          .map(|p| p.trim_start_matches('0'))
          .collect::<Vec<_>>()
          .join(", ")
      ),
      other => other.py_str(),
    }
  }

  /// The value under `key` in a mapping (keys compared as Python's `str()`).
  pub fn get(&self, key: &str) -> Option<&Value> {
    match self {
      Value::Map(pairs) => pairs
        .iter()
        .find(|(k, _)| k.py_str() == key)
        .map(|(_, v)| v),
      _ => None,
    }
  }

  pub fn as_seq(&self) -> &[Value] {
    match self {
      Value::Seq(items) => items,
      _ => &[],
    }
  }

  pub fn pairs(&self) -> &[(Value, Value)] {
    match self {
      Value::Map(pairs) => pairs,
      _ => &[],
    }
  }

  /// Python's truthiness, for `x or {}`.
  pub fn is_truthy(&self) -> bool {
    match self {
      Value::Null => false,
      Value::Bool(b) => *b,
      Value::Int(i) => *i != 0,
      Value::Float(f) => *f != 0.0,
      Value::Str(s) => !s.is_empty(),
      Value::Date(_) => true,
      Value::Seq(items) => !items.is_empty(),
      Value::Map(pairs) => !pairs.is_empty(),
    }
  }
}

/// Python's repr of a str: single quotes unless it holds one and no double quote.
fn py_repr_str(s: &str) -> String {
  let quote = if s.contains('\'') && !s.contains('"') {
    '"'
  } else {
    '\''
  };
  let mut out = String::from(quote);
  for c in s.chars() {
    match c {
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if c == quote => {
        out.push('\\');
        out.push(c);
      }
      c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
      c => out.push(c),
    }
  }
  out.push(quote);
  out
}

/// Python's repr of a float: the shortest round-trip form, always with a point or exponent.
fn py_float(f: f64) -> String {
  if f.is_nan() {
    return "nan".to_owned();
  }
  if f.is_infinite() {
    return if f > 0.0 { "inf" } else { "-inf" }.to_owned();
  }
  let s = format!("{f:?}");
  if s.contains(['.', 'e', 'E']) {
    s
  } else {
    format!("{s}.0")
  }
}

/// The document in `text`, or a message naming the line it cannot read.
pub fn parse(text: &str) -> Result<Value> {
  let lines = logical_lines(text)?;
  if lines.is_empty() {
    return Ok(Value::Null);
  }
  let mut at = 0;
  let value = block(&lines, &mut at, lines[0].indent)?;
  if at < lines.len() {
    return Err(
      format!(
        "line {}: unexpected text: {}",
        lines[at].number, lines[at].text
      )
      .into(),
    );
  }
  Ok(value)
}

struct Line<'a> {
  number: usize,
  indent: usize,
  text: &'a str,
}

/// The lines that hold something, comments removed, with their indentation.
fn logical_lines(text: &str) -> Result<Vec<Line<'_>>> {
  let mut out = Vec::new();
  for (i, raw) in text.lines().enumerate() {
    let number = i + 1;
    let raw = raw.strip_suffix('\r').unwrap_or(raw);
    if raw.contains('\t') && raw.trim_start_matches(' ').starts_with('\t') {
      return Err(format!("line {number}: a tab indents it, which YAML does not allow").into());
    }
    let body = strip_comment(raw);
    let trimmed = body.trim_end();
    let content = trimmed.trim_start_matches(' ');
    if content.is_empty() {
      continue;
    }
    if content == "---" || content == "..." || content.starts_with("%") {
      if number == 1 && content == "---" {
        continue;
      }
      return Err(format!("line {number}: more than one document is not read here").into());
    }
    if content.starts_with(['|', '>', '&', '*', '!']) {
      return Err(
        format!("line {number}: block scalars, anchors and tags are not read here").into(),
      );
    }
    out.push(Line {
      number,
      indent: trimmed.len() - content.len(),
      text: content,
    });
  }
  Ok(out)
}

/// A line without its comment: a # that starts the line or follows a space,
/// outside quotes.
fn strip_comment(line: &str) -> &str {
  let (mut single, mut double, mut prev_space) = (false, false, true);
  let mut chars = line.char_indices().peekable();
  while let Some((i, c)) = chars.next() {
    match c {
      '\'' if !double => single = !single,
      '"' if !single => double = !double,
      '\\' if double => {
        chars.next();
      }
      '#' if !single && !double && prev_space => return &line[..i],
      _ => {}
    }
    prev_space = c == ' ';
  }
  line
}

/// A block node at `indent`: a sequence when it starts with "- ", else a mapping.
fn block(lines: &[Line], at: &mut usize, indent: usize) -> Result<Value> {
  let first = &lines[*at];
  if first.text == "-" || first.text.starts_with("- ") {
    let mut items = Vec::new();
    while *at < lines.len() && lines[*at].indent == indent {
      let line = &lines[*at];
      let Some(rest) = line.text.strip_prefix('-') else {
        break;
      };
      let rest = rest.trim_start_matches(' ');
      *at += 1;
      if rest.is_empty() {
        items.push(nested(lines, at, indent)?);
      } else if let Some((key, value)) = mapping_pair(rest, line.number)? {
        // "- key: value": a mapping whose first pair shares the dash's line
        let inner = indent + (line.text.len() - rest.len());
        let mut pairs = vec![(key, inline_or_nested(value, lines, at, inner, line.number)?)];
        if *at < lines.len()
          && lines[*at].indent == inner
          && let Value::Map(more) = block(lines, at, inner)?
        {
          pairs.extend(more);
        }
        items.push(Value::Map(pairs));
      } else {
        items.push(scalar_or_flow(rest, line.number)?);
      }
    }
    return Ok(Value::Seq(items));
  }
  let mut pairs = Vec::new();
  while *at < lines.len() && lines[*at].indent == indent {
    let line = &lines[*at];
    let Some((key, value)) = mapping_pair(line.text, line.number)? else {
      return Err(
        format!(
          "line {}: expected 'key: value', found: {}",
          line.number, line.text
        )
        .into(),
      );
    };
    *at += 1;
    pairs.push((
      key,
      inline_or_nested(value, lines, at, indent, line.number)?,
    ));
  }
  Ok(Value::Map(pairs))
}

fn inline_or_nested(
  value: &str,
  lines: &[Line],
  at: &mut usize,
  indent: usize,
  number: usize,
) -> Result<Value> {
  if value.is_empty() {
    nested(lines, at, indent)
  } else {
    scalar_or_flow(value, number)
  }
}

/// The node on the following lines when they are indented further (or a
/// sequence at the same indent, as PyYAML allows under a key), else null.
fn nested(lines: &[Line], at: &mut usize, indent: usize) -> Result<Value> {
  match lines.get(*at) {
    Some(next) if next.indent > indent => block(lines, at, next.indent),
    Some(next) if next.indent == indent && (next.text == "-" || next.text.starts_with("- ")) => {
      block(lines, at, indent)
    }
    _ => Ok(Value::Null),
  }
}

/// "key: value" (value possibly empty), the key resolved like any scalar.
fn mapping_pair(text: &str, number: usize) -> Result<Option<(Value, &str)>> {
  if text.starts_with(['{', '[']) {
    return Ok(None);
  }
  let (key_text, rest) = if let Some(quote @ ('"' | '\'')) = text.chars().next() {
    let end = quoted_end(text, quote).ok_or_else(|| format!("line {number}: an unclosed quote"))?;
    (&text[..end], &text[end..])
  } else {
    match find_colon(text) {
      Some(at) => (&text[..at], &text[at..]),
      None => return Ok(None),
    }
  };
  let Some(value) = rest.strip_prefix(':') else {
    return Ok(None);
  };
  if !value.is_empty() && !value.starts_with(' ') {
    return Ok(None);
  }
  Ok(Some((
    scalar(key_text.trim_end(), number)?,
    value.trim_start_matches(' '),
  )))
}

/// The first ": " (or a final ":") in plain text.
fn find_colon(text: &str) -> Option<usize> {
  let bytes = text.as_bytes();
  (0..bytes.len()).find(|&i| bytes[i] == b':' && (i + 1 == bytes.len() || bytes[i + 1] == b' '))
}

/// The byte just past the closing quote of a quoted scalar starting at 0.
fn quoted_end(text: &str, quote: char) -> Option<usize> {
  let mut chars = text.char_indices().skip(1);
  while let Some((i, c)) = chars.next() {
    if quote == '"' && c == '\\' {
      chars.next();
    } else if c == quote {
      if quote == '\'' && text[i + 1..].starts_with('\'') {
        chars.next();
        continue;
      }
      return Some(i + 1);
    }
  }
  None
}

fn scalar_or_flow(text: &str, number: usize) -> Result<Value> {
  if text.starts_with(['{', '[']) {
    let mut flow = Flow {
      text,
      at: 0,
      number,
    };
    let value = flow.node()?;
    flow.skip_spaces();
    if flow.at != text.len() {
      return Err(
        format!(
          "line {number}: text after a flow collection: {}",
          &text[flow.at..]
        )
        .into(),
      );
    }
    return Ok(value);
  }
  scalar(text, number)
}

/// A scalar on its own: quoted, or plain and resolved by PyYAML's rules.
fn scalar(text: &str, number: usize) -> Result<Value> {
  match text.chars().next() {
    Some('"') => Ok(Value::Str(unquote_double(text, number)?)),
    Some('\'') => Ok(Value::Str(unquote_single(text, number)?)),
    Some('|' | '>' | '&' | '*' | '!' | '%' | '@' | '`') => Err(
      format!("line {number}: block scalars, anchors, tags and reserved indicators are not read here: {text}").into(),
    ),
    _ => Ok(resolve(text.trim())),
  }
}

fn unquote_double(text: &str, number: usize) -> Result<String> {
  let inner = text
    .strip_prefix('"')
    .and_then(|t| t.strip_suffix('"'))
    .ok_or_else(|| format!("line {number}: an unclosed double quote"))?;
  let mut out = String::new();
  let mut chars = inner.chars();
  while let Some(c) = chars.next() {
    if c != '\\' {
      out.push(c);
      continue;
    }
    match chars.next() {
      Some('n') => out.push('\n'),
      Some('t') => out.push('\t'),
      Some('r') => out.push('\r'),
      Some('0') => out.push('\0'),
      Some('"') => out.push('"'),
      Some('\\') => out.push('\\'),
      Some('/') => out.push('/'),
      Some(' ') => out.push(' '),
      Some(kind @ ('x' | 'u' | 'U')) => {
        let width = match kind {
          'x' => 2,
          'u' => 4,
          _ => 8,
        };
        let hex: String = chars.by_ref().take(width).collect();
        let code = u32::from_str_radix(&hex, 16)
          .ok()
          .and_then(char::from_u32)
          .ok_or_else(|| format!("line {number}: a bad \\{kind} escape"))?;
        out.push(code);
      }
      other => {
        return Err(
          format!(
            "line {number}: an unknown escape \\{}",
            other.unwrap_or(' ')
          )
          .into(),
        );
      }
    }
  }
  Ok(out)
}

fn unquote_single(text: &str, number: usize) -> Result<String> {
  let inner = text
    .strip_prefix('\'')
    .and_then(|t| t.strip_suffix('\''))
    .ok_or_else(|| format!("line {number}: an unclosed single quote"))?;
  Ok(inner.replace("''", "'"))
}

/// A plain scalar by PyYAML's implicit resolvers (YAML 1.1): null, bool, int,
/// float and a date; anything else is a string.
pub fn resolve(text: &str) -> Value {
  match text {
    "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
    "yes" | "Yes" | "YES" | "true" | "True" | "TRUE" | "on" | "On" | "ON" => {
      return Value::Bool(true);
    }
    "no" | "No" | "NO" | "false" | "False" | "FALSE" | "off" | "Off" | "OFF" => {
      return Value::Bool(false);
    }
    ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => return Value::Float(f64::INFINITY),
    "-.inf" | "-.Inf" | "-.INF" => return Value::Float(f64::NEG_INFINITY),
    ".nan" | ".NaN" | ".NAN" => return Value::Float(f64::NAN),
    _ => {}
  }
  if let Some(i) = int(text) {
    return Value::Int(i);
  }
  if let Some(f) = float(text) {
    return Value::Float(f);
  }
  if is_date(text) {
    return Value::Date(text.to_owned());
  }
  Value::Str(text.to_owned())
}

fn int(text: &str) -> Option<i64> {
  let (sign, body) = match text.strip_prefix('-') {
    Some(rest) => (-1, rest),
    None => (1, text.strip_prefix('+').unwrap_or(text)),
  };
  let digits = |s: &str, ok: fn(u8) -> bool| !s.is_empty() && s.bytes().all(|b| ok(b) || b == b'_');
  let clean = |s: &str| s.replace('_', "");
  let value = if let Some(bin) = body.strip_prefix("0b") {
    digits(bin, |b| b == b'0' || b == b'1').then(|| i64::from_str_radix(&clean(bin), 2).ok())??
  } else if let Some(hex) = body.strip_prefix("0x") {
    digits(hex, |b| b.is_ascii_hexdigit()).then(|| i64::from_str_radix(&clean(hex), 16).ok())??
  } else if body.len() > 1 && body.starts_with('0') {
    digits(&body[1..], |b| (b'0'..=b'7').contains(&b))
      .then(|| i64::from_str_radix(&clean(&body[1..]), 8).ok())??
  } else if body == "0" {
    0
  } else if body.starts_with(|c: char| c.is_ascii_digit() && c != '0')
    && digits(body, |b| b.is_ascii_digit())
  {
    clean(body).parse().ok()?
  } else {
    return None;
  };
  Some(sign * value)
}

fn float(text: &str) -> Option<f64> {
  let body = text.strip_prefix(['-', '+']).unwrap_or(text);
  let (mantissa, exponent) = match body.find(['e', 'E']) {
    Some(at) => (&body[..at], Some(&body[at + 1..])),
    None => (body, None),
  };
  let point = mantissa.find('.')?;
  let (whole, frac) = (&mantissa[..point], &mantissa[point + 1..]);
  let ok_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit() || b == b'_');
  if !ok_digits(whole)
    || !frac
      .bytes()
      .all(|b| b.is_ascii_digit() || b == b'.' || b == b'_')
  {
    return None;
  }
  if whole.is_empty() && frac.is_empty() {
    return None;
  }
  if let Some(e) = exponent {
    let digits = e.strip_prefix(['-', '+'])?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
      return None;
    }
  }
  text.replace('_', "").parse().ok()
}

/// YYYY-MM-DD, as PyYAML's timestamp resolver takes a date (1 or 2 digit month and day).
fn is_date(text: &str) -> bool {
  let parts: Vec<&str> = text.split('-').collect();
  matches!(parts.as_slice(), [y, m, d]
    if y.len() == 4 && (1..=2).contains(&m.len()) && (1..=2).contains(&d.len())
      && [y, m, d].iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())))
}

/// A flow collection (`{...}` or `[...]`) on one line.
struct Flow<'a> {
  text: &'a str,
  at: usize,
  number: usize,
}

impl<'a> Flow<'a> {
  fn rest(&self) -> &'a str {
    &self.text[self.at..]
  }

  fn skip_spaces(&mut self) {
    while self.rest().starts_with(' ') {
      self.at += 1;
    }
  }

  fn error(&self, what: &str) -> crate::Error {
    format!("line {}: {what} at '{}'", self.number, self.rest()).into()
  }

  fn node(&mut self) -> Result<Value> {
    self.skip_spaces();
    if self.rest().starts_with('{') {
      self.at += 1;
      let mut pairs = Vec::new();
      loop {
        self.skip_spaces();
        if self.rest().starts_with('}') {
          self.at += 1;
          return Ok(Value::Map(pairs));
        }
        let key = self.scalar(true)?;
        self.skip_spaces();
        let value = if self.rest().starts_with(':') {
          self.at += 1;
          self.skip_spaces();
          if self.rest().starts_with([',', '}']) {
            Value::Null
          } else {
            self.node()?
          }
        } else {
          Value::Null
        };
        pairs.push((key, value));
        self.skip_spaces();
        if self.rest().starts_with(',') {
          self.at += 1;
        } else if !self.rest().starts_with('}') {
          return Err(self.error("expected ',' or '}'"));
        }
      }
    }
    if self.rest().starts_with('[') {
      self.at += 1;
      let mut items = Vec::new();
      loop {
        self.skip_spaces();
        if self.rest().starts_with(']') {
          self.at += 1;
          return Ok(Value::Seq(items));
        }
        items.push(self.node()?);
        self.skip_spaces();
        if self.rest().starts_with(',') {
          self.at += 1;
        } else if !self.rest().starts_with(']') {
          return Err(self.error("expected ',' or ']'"));
        }
      }
    }
    self.scalar(false)
  }

  /// A scalar inside a flow collection; a key stops at ':', any at , ] }.
  fn scalar(&mut self, is_key: bool) -> Result<Value> {
    self.skip_spaces();
    let rest = self.rest();
    if let Some(quote @ ('"' | '\'')) = rest.chars().next() {
      let end = quoted_end(rest, quote).ok_or_else(|| self.error("an unclosed quote"))?;
      let text = &rest[..end];
      self.at += end;
      return scalar(text, self.number);
    }
    let bytes = rest.as_bytes();
    let mut end = 0;
    while end < bytes.len() {
      let b = bytes[end];
      let colon = b == b':'
        && (end + 1 == bytes.len() || matches!(bytes[end + 1], b' ' | b',' | b'}' | b']'));
      if b == b',' || b == b'}' || b == b']' || (colon && (is_key || end > 0)) {
        break;
      }
      end += 1;
    }
    let text = rest[..end].trim_end();
    self.at += end;
    Ok(resolve(text))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn str_of(v: &Value) -> String {
    v.py_str()
  }

  #[test]
  fn a_tags_file_reads_as_pyyaml_reads_it() -> Result<()> {
    let doc = parse(
      "# Tags for Chidi Okafor. A bare word is your own tag; auto and inferred tags\n\
       # are rewritten by pplr, hand tags never are. Vocabulary: _pplr/vocabulary.yaml\n\
       tags:\n\
       \x20 - vip\n\
       \x20 - no\n\
       \x20 - {tag: cto, source: auto, at: 2026-09-25}\n\
       \x20 - {tag: mentor, source: inferred, evidence: \"Mentored the \\\"Ledger\\\" cohort, 2024\"}\n",
    )?;
    let tags = doc.get("tags").map(Value::as_seq).unwrap_or_default();
    assert_eq!(tags.len(), 4);
    assert_eq!(str_of(&tags[0]), "vip");
    assert_eq!(str_of(&tags[1]), "False");
    assert_eq!(
      tags[2].get("at"),
      Some(&Value::Date("2026-09-25".to_owned()))
    );
    assert_eq!(
      tags[3].get("evidence"),
      Some(&Value::Str(
        "Mentored the \"Ledger\" cohort, 2024".to_owned()
      ))
    );
    Ok(())
  }

  #[test]
  fn a_vocabulary_reads_flow_and_block_shapes() -> Result<()> {
    let doc = parse(
      "version: 1\nrole:\n  cto: {also: [chief-technology-officer]}\n  founder: {}\n  investor:\n    also: [vc-partner]\n    note: 'People who invest'\nsplit:\n  podcast-guest: [podcast]\nempty:\n",
    )?;
    assert_eq!(doc.get("version"), Some(&Value::Int(1)));
    let role = doc.get("role").map(Value::pairs).unwrap_or_default();
    assert_eq!(role.len(), 3);
    assert_eq!(
      str_of(
        &role[0]
          .1
          .get("also")
          .map(|a| a.as_seq()[0].clone())
          .unwrap_or(Value::Null)
      ),
      "chief-technology-officer"
    );
    assert_eq!(role[1].1, Value::Map(Vec::new()));
    assert_eq!(
      role[2].1.get("note").map(str_of).as_deref(),
      Some("People who invest")
    );
    assert_eq!(doc.get("empty"), Some(&Value::Null));
    Ok(())
  }

  #[test]
  fn plain_scalars_resolve_by_yaml_1_1() {
    assert_eq!(resolve("12month"), Value::Str("12month".to_owned()));
    assert_eq!(resolve("180"), Value::Int(180));
    assert_eq!(resolve("0x1F"), Value::Int(31));
    assert_eq!(resolve("1_000"), Value::Int(1000));
    assert_eq!(resolve("010"), Value::Int(8));
    assert_eq!(resolve("On"), Value::Bool(true));
    assert_eq!(resolve("~"), Value::Null);
    assert_eq!(resolve("1.5"), Value::Float(1.5));
    assert_eq!(resolve("2026-6-5"), Value::Date("2026-6-5".to_owned()));
    assert_eq!(Value::Float(1.0).py_str(), "1.0");
    assert_eq!(Value::Bool(false).py_str(), "False");
  }

  #[test]
  fn unread_shapes_are_refused_by_name() {
    for text in [
      "a: |\n  block\n",
      "a: &x 1\n",
      "a: 1\n---\nb: 2\n",
      "a: {b: 1\n",
    ] {
      assert!(parse(text).is_err(), "{text}");
    }
  }

  #[test]
  fn a_list_under_a_key_may_share_its_indent() -> Result<()> {
    let doc = parse("tags:\n- a\n- b\n")?;
    assert_eq!(doc.get("tags").map(|t| t.as_seq().len()), Some(2));
    Ok(())
  }

  #[test]
  fn comments_and_quotes_are_told_apart() -> Result<()> {
    let doc = parse("a: 'x # not a comment' # a comment\nb: c#d\n")?;
    assert_eq!(
      doc.get("a").map(str_of).as_deref(),
      Some("x # not a comment")
    );
    assert_eq!(doc.get("b").map(str_of).as_deref(), Some("c#d"));
    Ok(())
  }
}
