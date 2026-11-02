//! The About file's header list, read as the legacy engine read it: each
//! `- Field: value` line, the first of each field winning.

use crate::normalise::{link_text, linkedin_slug, normalise_phone};
use regex::Regex;
use std::sync::LazyLock;

/// The engine's email pattern for an Email line.
static EMAIL: LazyLock<Option<Regex>> =
  LazyLock::new(|| Regex::new(r"[A-Za-z0-9._%+'-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").ok());

/// A number shorter than this, once normalised, is not a phone number.
const MIN_PHONE: usize = 7;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
  /// The Role line's link text.
  pub role: String,
  /// The Company line's link text.
  pub company: String,
  /// Every address on the Email line, lower case, sorted, once each.
  pub emails: Vec<String>,
  /// Every number on the Phone line, normalised: numbers apart by a comma,
  /// semicolon, middle dot or slash; a Markdown link read as its text.
  pub phones: Vec<String>,
  /// The LinkedIn line's slug, or empty.
  pub linkedin: String,
}

/// The header of an About file's text.
pub fn header(text: &str) -> Header {
  let mut h = Header::default();
  let mut seen: Vec<&str> = Vec::new();
  for line in text.split(['\n', '\r']) {
    let Some(rest) = line.strip_prefix("- ") else {
      continue;
    };
    let Some(colon) = rest.find(':') else {
      continue;
    };
    let field = rest[..colon].trim();
    let value = &rest[colon + 1..];
    if seen.contains(&field) {
      continue;
    }
    match field {
      "Role" => h.role = link_text(value),
      "Company" => h.company = link_text(value),
      "Email" => h.emails = emails(value),
      "Phone" => {
        h.phones = value
          .split([',', ';', '\u{b7}', '/'])
          .map(|part| normalise_phone(&link_text(part)))
          .filter(|phone| phone.chars().count() >= MIN_PHONE)
          .collect();
      }
      "LinkedIn" => h.linkedin = linkedin_slug(value).unwrap_or_default(),
      _ => continue,
    }
    seen.push(field);
  }
  h
}

fn emails(value: &str) -> Vec<String> {
  let Some(re) = EMAIL.as_ref() else {
    return Vec::new();
  };
  let mut out: Vec<String> = re
    .find_iter(value)
    .map(|m| m.as_str().to_lowercase())
    .collect();
  out.sort();
  out.dedup();
  out
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_header_reads_link_text_addresses_numbers_and_the_slug() {
    let text = "# Ada Lovelace (About)\n\n_Ada Lovelace_\n\
      - Role:     [Engineer](https://x)\n\
      - Company:  Analytical\n\
      - LinkedIn: [in/ada](https://www.linkedin.com/in/Ada/)\n\
      - Email:    [Ada@Example.com](mailto:ada@example.com), b@x.org\n\
      - Phone:    [+44 (0) 7700 900001](tel:+447700900001) \u{b7} +49 0151 2345678 / 12\n\
      - Role:     Ignored, the first Role wins\n";
    let h = header(text);
    assert_eq!(h.role, "Engineer");
    assert_eq!(h.company, "Analytical");
    assert_eq!(h.linkedin, "ada");
    assert_eq!(h.emails, ["ada@example.com", "b@x.org"]);
    assert_eq!(h.phones, ["+447700900001", "+491512345678"]);
  }

  #[test]
  fn an_empty_header_reads_as_empty() {
    let h = header("- Role:\n- Company:\n- LinkedIn: []()\n- Email:    [](mailto:)\n- Phone:\n");
    assert_eq!(h, Header::default());
  }
}
