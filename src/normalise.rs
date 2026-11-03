//! The normalisation rules the legacy engine compared people and cards by
//! (ST0001 recon section 6), as pure functions.

use crate::fold_table::{DECOMPOSE, FOLD};

/// Case and diacritic folding as Foundation does it
/// (`folding(options: [.caseInsensitive, .diacriticInsensitive])`), then
/// trimmed of whitespace: "  Zsófia " and "zsofia" fold alike, while ø, ł and
/// æ stay letters of their own.
pub fn fold(s: &str) -> String {
  fold_untrimmed(s).trim().to_owned()
}

/// `fold` without the trim, as the marker folds a name.
pub fn fold_untrimmed(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for c in s.chars() {
    if ('\u{300}'..='\u{36F}').contains(&c) {
      continue; // a combining mark, as a decomposed accent arrives
    }
    match FOLD.binary_search_by_key(&c, |&(from, _)| from) {
      Ok(at) => out.push_str(FOLD[at].1),
      Err(_) => out.extend(c.to_lowercase()),
    }
  }
  out
}

/// Country codes whose trunk 0 is dropped after the code; Italy (+39) keeps it.
const TRUNK_ZERO: [&str; 9] = [
  "+44", "+61", "+33", "+49", "+32", "+31", "+353", "+64", "+262",
];

/// Digits with a leading +: a UK number without a country code is assumed
/// +44, and the trunk 0 after a country code goes, written "(0)" or not.
pub fn normalise_phone(raw: &str) -> String {
  let kept: String = raw
    .replace("(0)", "")
    .chars()
    .filter(|c| *c == '+' || c.is_ascii_digit())
    .collect();
  let mut s = if let Some(rest) = kept.strip_prefix("00") {
    format!("+{rest}")
  } else {
    kept
  };
  if let Some(rest) = s.strip_prefix('0') {
    s = format!("+44{rest}");
  }
  if !s.starts_with('+') && !s.is_empty() {
    s.insert(0, '+');
  }
  for code in TRUNK_ZERO {
    if let Some(rest) = s.strip_prefix(code).and_then(|r| r.strip_prefix('0')) {
      s = format!("{code}{rest}");
    }
  }
  s
}

/// The address as the mail server sees it: Gmail ignores dots, and anything
/// after a +, in the local part.
pub fn email_key(email: &str) -> String {
  let lower = email.to_lowercase();
  let Some(at) = lower.rfind('@') else {
    return lower;
  };
  let (local, domain) = (&lower[..at], &lower[at + 1..]);
  if domain != "gmail.com" && domain != "googlemail.com" {
    return lower;
  }
  let local = local.split('+').next().unwrap_or_default();
  format!("{}@gmail.com", local.replace('.', ""))
}

/// "https://www.linkedin.com/in/JonTolley/?x" -> "jontolley".
pub fn linkedin_slug(raw: &str) -> Option<String> {
  let lower = raw.to_lowercase();
  let at = lower.find("linkedin.com/in/")? + "linkedin.com/in/".len();
  let slug = lower[at..]
    .split(|c| "/?#)] ".contains(c))
    .find(|part| !part.is_empty())
    .unwrap_or_default();
  if slug.is_empty() {
    return None;
  }
  Some(percent_decode(slug).unwrap_or_else(|| slug.to_owned()))
}

/// A Markdown link's text, or the plain value: "[Co-Founder](https://x)" -> "Co-Founder".
pub fn link_text(value: &str) -> String {
  let t = value.trim_matches(is_blank);
  if let (Some(rest), Some(close)) = (t.strip_prefix('['), t.find("](")) {
    return rest[..close - 1].trim_matches(is_blank).to_owned();
  }
  t.to_owned()
}

/// Foundation's `.whitespaces`: spaces and tabs, not line breaks.
fn is_blank(c: char) -> bool {
  c.is_whitespace() && !matches!(c, '\n' | '\r' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

/// Percent-decoding as Foundation's `removingPercentEncoding`: None when an
/// escape is malformed or the bytes are not UTF-8.
pub fn percent_decode(s: &str) -> Option<String> {
  let bytes = s.as_bytes();
  let mut out = Vec::with_capacity(bytes.len());
  let mut i = 0;
  while i < bytes.len() {
    if bytes[i] == b'%' {
      let hex = std::str::from_utf8(bytes.get(i + 1..i + 3)?).ok()?;
      out.push(u8::from_str_radix(hex, 16).ok()?);
      i += 3;
    } else {
      out.push(bytes[i]);
      i += 1;
    }
  }
  String::from_utf8(out).ok()
}

/// Percent-encoding as Python's `urllib.parse.quote(s, safe)`: every byte but
/// letters, digits, `_.-~` and the bytes in `safe` becomes `%XX`.
pub fn percent_encode(s: &str, safe: &[u8]) -> String {
  let mut out = String::with_capacity(s.len());
  for b in s.bytes() {
    if b.is_ascii_alphanumeric() || b"_.-~".contains(&b) || safe.contains(&b) {
      out.push(char::from(b));
    } else {
      out.push_str(&format!("%{b:02X}"));
    }
  }
  out
}

/// Canonical decomposition of Latin letters as Foundation gives it, the form
/// the legacy engine's file names took on disk: "Zsófia" becomes
/// "Zso\u{301}fia". Anything outside the table passes through unchanged.
pub fn decompose(s: &str) -> String {
  let mut out = String::with_capacity(s.len());
  for c in s.chars() {
    match DECOMPOSE.binary_search_by_key(&c, |&(from, _)| from) {
      Ok(at) => out.push_str(DECOMPOSE[at].1),
      Err(_) => out.push(c),
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn fold_drops_case_and_accents_as_foundation_does() {
    assert_eq!(fold("  Zsófia "), "zsofia");
    assert_eq!(fold("Ze\u{301}phyr"), "zephyr");
    assert_eq!(fold("Straße"), "strasse");
    assert_eq!(fold("Ødegård"), "ødegard");
    assert_eq!(fold("Łukasz"), "łukasz");
  }

  #[test]
  fn decompose_splits_accents_as_foundation_does() {
    assert_eq!(decompose("Zsófia Varga"), "Zso\u{301}fia Varga");
    assert_eq!(decompose("\u{1EA4}"), "A\u{302}\u{301}");
    assert_eq!(decompose("Ødegård"), "Ødega\u{30A}rd");
  }

  #[test]
  fn phones_become_digits_with_a_country_code() {
    assert_eq!(normalise_phone("+44 (0) 7700 900001"), "+447700900001");
    assert_eq!(normalise_phone("07700 900456"), "+447700900456");
    assert_eq!(normalise_phone("0049 0151 2345678"), "+491512345678");
    assert_eq!(normalise_phone("+39 06 1234 5678"), "+390612345678");
    assert_eq!(normalise_phone("+353 0 1 234 5678"), "+35312345678");
    assert_eq!(normalise_phone("n/a"), "");
  }

  #[test]
  fn gmail_addresses_lose_dots_and_suffixes_and_others_keep_them() {
    assert_eq!(
      email_key("Zsofia.Varga+news@Gmail.com"),
      "zsofiavarga@gmail.com"
    );
    assert_eq!(email_key("a.b@googlemail.com"), "ab@gmail.com");
    assert_eq!(email_key("a.b+x@example.com"), "a.b+x@example.com");
    assert_eq!(email_key("no-at-sign"), "no-at-sign");
  }

  #[test]
  fn a_linkedin_url_becomes_its_slug() {
    assert_eq!(
      linkedin_slug("[in/chidi](https://www.linkedin.com/in/Chidi-Okafor-X/?trk=1)"),
      Some("chidi-okafor-x".to_owned())
    );
    assert_eq!(
      linkedin_slug("https://linkedin.com/in/zs%C3%B3fia"),
      Some("zsófia".to_owned())
    );
    assert_eq!(linkedin_slug("https://www.linkedin.com/in/"), None);
    assert_eq!(linkedin_slug("https://example.com"), None);
  }

  #[test]
  fn link_text_is_the_text_of_a_markdown_link_or_the_value() {
    assert_eq!(link_text(" [Co-Founder](https://x)"), "Co-Founder");
    assert_eq!(link_text("  Head of Engineering "), "Head of Engineering");
    assert_eq!(link_text("[](mailto:)"), "");
  }

  #[test]
  fn percent_decoding_refuses_a_malformed_escape() {
    assert_eq!(percent_decode("a%20b%2Cc"), Some("a b,c".to_owned()));
    assert_eq!(percent_decode("50%"), None);
    assert_eq!(percent_decode("%ZZ"), None);
    assert_eq!(percent_decode("a%2"), None);
  }

  #[test]
  fn percent_encoding_matches_urllib_quote_with_its_safe_bytes() {
    assert_eq!(
      percent_encode("V/Varga, Zsófia/About/Zsófia Varga (About).md", b"/"),
      "V/Varga%2C%20Zs%C3%B3fia/About/Zs%C3%B3fia%20Varga%20%28About%29.md"
    );
    assert_eq!(percent_encode("a_b.c-d~e/f", b"/"), "a_b.c-d~e/f");
    assert_eq!(
      percent_encode("CAL=x+y/z@mail.gmail.com", b"@="),
      "CAL=x%2By%2Fz@mail.gmail.com"
    );
  }
}
