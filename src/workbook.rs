//! sync --plan's review workbook (ST0001 WP11), as the legacy workbook.py
//! laid it out: Merged (one row per pplr person, the proposal first and the
//! reviewer's columns beside it), pplr, Contacts, and How to review. The
//! reviewer edits Decision, Card, Dupes and Notes on Merged; the card refs
//! (C0001...) resolve through the Contacts sheet of the same workbook.

use crate::Result;
use crate::plan::{Confidence, Decision, Plan, Reviewed};
use crate::yaml::Value as Py;
use calamine::{Data, Reader, Xlsx, open_workbook};
use rust_xlsxwriter::utility::column_number_to_name;
use rust_xlsxwriter::{
  Color, ConditionalFormatFormula, DataValidation, Format, FormatAlign, Workbook, Worksheet,
};
use std::collections::HashMap;
use std::ops::RangeInclusive;
use std::path::Path;

const REVIEW_FILL: u32 = 0xFFF4D6;
const LOW_FILL: u32 = 0xF9D9A8;
const MEDIUM_FILL: u32 = 0xFDF1C4;
const HOW_WIDTH: f64 = 140.0;
const TITLE_SIZE: u32 = 13;

/// The sheets --apply-plan reads.
const MERGED_TITLE: &str = "Merged";
const CONTACTS_TITLE: &str = "Contacts";

/// The headers --apply-plan reads, and those the writer places by name.
const PERSON: &str = "Person";
const DECISION: &str = "Decision";
const CARD: &str = "Card";
const NOTES: &str = "Notes";
const CONFIDENCE: &str = "Confidence";
const CARD_ID: &str = "Card id";

/// Merged's columns: header, width, wrapped. Decision to Notes are the
/// reviewer's.
const MERGED: [(&str, u16, bool); 21] = [
  (PERSON, 26, false),
  (DECISION, 12, false),
  (CARD, 9, false),
  ("Dupes", 12, false),
  (NOTES, 28, false),
  (CONFIDENCE, 11, false),
  ("Why", 34, true),
  ("Changes", 44, true),
  ("Card name", 22, false),
  ("Company (pplr)", 20, false),
  ("Company (card)", 20, false),
  ("Role (pplr)", 24, false),
  ("Role (card)", 24, false),
  ("Emails (pplr)", 28, true),
  ("Emails (card)", 28, true),
  ("Phones (pplr)", 16, true),
  ("Phones (card)", 16, true),
  ("LinkedIn (pplr)", 20, false),
  ("LinkedIn (card)", 20, true),
  ("Card account", 10, false),
  ("Other candidates", 44, true),
];

const PPLR: [(&str, u16, bool); 11] = [
  (PERSON, 26, false),
  ("Given", 14, false),
  ("Family", 16, false),
  ("Company", 22, false),
  ("Role", 30, false),
  ("Emails", 30, true),
  ("Phones", 16, true),
  ("LinkedIn", 22, false),
  ("pplr URL", 30, false),
  ("Photo", 7, false),
  ("Folder", 50, false),
];

const CONTACTS: [(&str, u16, bool); 12] = [
  (CARD, 8, false),
  ("Name", 24, false),
  ("Company", 22, false),
  ("Role", 24, false),
  ("Emails", 30, true),
  ("Phones", 16, true),
  ("LinkedIn", 20, true),
  ("Account", 9, false),
  ("Groups", 16, false),
  ("pplr URL", 24, false),
  ("Proposed for", 26, false),
  (CARD_ID, 44, false),
];

fn lines(values: &[String]) -> String {
  values
    .iter()
    .filter(|v| !v.is_empty())
    .map(String::as_str)
    .collect::<Vec<_>>()
    .join("\n")
}

/// The index of the column headed `header` in a sheet's table.
fn col(columns: &[(&str, u16, bool)], header: &str) -> Result<u16> {
  let at = columns
    .iter()
    .position(|(h, _, _)| *h == header)
    .ok_or_else(|| format!("the workbook has no {header} column"))?;
  Ok(u16::try_from(at)?)
}

/// One sheet: a bold header row, the rows top-aligned (wrapped where the
/// column says), the widths, the panes frozen at B2 and a filter over it all.
fn sheet<'a>(
  wb: &'a mut Workbook,
  title: &str,
  columns: &[(&str, u16, bool)],
  rows: &[Vec<String>],
  fill: Option<(RangeInclusive<u16>, u32)>,
) -> Result<&'a mut Worksheet> {
  let ws = wb.add_worksheet();
  ws.set_name(title)?;
  let bold = Format::new().set_bold();
  let mut formats = Vec::new();
  for (col, (header, width, wrap)) in (0u16..).zip(columns) {
    ws.write_string_with_format(0, col, *header, &bold)?;
    ws.set_column_width(col, *width)?;
    let mut format = Format::new().set_align(FormatAlign::Top);
    if *wrap {
      format = format.set_text_wrap();
    }
    if let Some((cols, color)) = &fill
      && cols.contains(&col)
    {
      format = format.set_background_color(Color::RGB(*color));
    }
    formats.push(format);
  }
  for (row, values) in (1u32..).zip(rows) {
    for ((col, value), format) in (0u16..).zip(values).zip(&formats) {
      if value.is_empty() {
        ws.write_blank(row, col, format)?;
      } else {
        ws.write_string_with_format(row, col, value, format)?;
      }
    }
  }
  let last_row = u32::try_from(rows.len())?;
  let last_col = u16::try_from(columns.len().saturating_sub(1))?;
  ws.set_freeze_panes(1, 1)?;
  ws.autofilter(0, 0, last_row, last_col)?;
  Ok(ws)
}

/// The plan as the review workbook, at `out`.
pub fn write(plan: &Plan, out: &Path) -> Result<()> {
  let people: HashMap<&str, _> = plan.people.iter().map(|p| (p.key.as_str(), p)).collect();
  let cards: HashMap<&str, _> = plan.cards.iter().map(|c| (c.r#ref.as_str(), c)).collect();
  let mut wb = Workbook::new();

  let merged: Vec<Vec<String>> = plan
    .rows
    .iter()
    .map(|r| {
      let p = people.get(r.person.as_str());
      let c = r.card.as_deref().and_then(|k| cards.get(k));
      let of_p =
        |f: &dyn Fn(&crate::plan::PlanPerson) -> String| p.map(|p| f(p)).unwrap_or_default();
      let of_c = |f: &dyn Fn(&crate::plan::PlanCard) -> String| c.map(|c| f(c)).unwrap_or_default();
      vec![
        r.person.clone(),
        r.decision.as_str().to_owned(),
        r.card.clone().unwrap_or_default(),
        r.dupes.join(", "),
        String::new(),
        r.confidence.as_str().to_owned(),
        r.why.clone(),
        lines(&r.changes),
        of_c(&|c| c.name.clone()),
        of_p(&|p| p.company.clone()),
        of_c(&|c| c.organization.clone()),
        of_p(&|p| p.role.clone()),
        of_c(&|c| c.job_title.clone()),
        of_p(&|p| lines(&p.emails)),
        of_c(&|c| lines(&c.emails)),
        of_p(&|p| lines(&p.phones)),
        of_c(&|c| lines(&c.phones)),
        of_p(&|p| p.linkedin.clone()),
        of_c(&|c| lines(&c.linkedin)),
        of_c(&|c| c.account.clone()),
        lines(
          &r.candidates
            .iter()
            .map(|k| format!("{} {} ({}: {})", k.r#ref, k.name, k.score, k.why))
            .collect::<Vec<_>>(),
        ),
      ]
    })
    .collect();
  let decision = col(&MERGED, DECISION)?;
  let confidence = col(&MERGED, CONFIDENCE)?;
  let review = decision..=col(&MERGED, NOTES)?;
  let ws = sheet(
    &mut wb,
    MERGED_TITLE,
    &MERGED,
    &merged,
    Some((review, REVIEW_FILL)),
  )?;
  if let Ok(last) = u32::try_from(merged.len())
    && last >= 1
  {
    let decisions = Decision::ALL.map(Decision::as_str);
    let dv = DataValidation::new()
      .allow_list_strings(&decisions)?
      .ignore_blank(false)
      // workbook.py's list sets no prompt and shows none
      .show_input_message(false)
      .set_error_title(DECISION)?
      .set_error_message(format!("One of: {}", decisions.join(", ")))?;
    ws.add_data_validation(1, decision, last, decision, &dv)?;
    // A rule names the range's first row, and Excel moves it down each row
    let letter = column_number_to_name(confidence);
    for (level, color) in [
      (Confidence::Low, LOW_FILL),
      (Confidence::Medium, MEDIUM_FILL),
    ] {
      let rule = ConditionalFormatFormula::new()
        .set_rule(format!("=${letter}2=\"{level}\"").as_str())
        .set_format(Format::new().set_background_color(Color::RGB(color)));
      ws.add_conditional_format(1, confidence, last, confidence, &rule)?;
    }
  }

  let pplr: Vec<Vec<String>> = plan
    .people
    .iter()
    .map(|p| {
      vec![
        p.key.clone(),
        p.given.clone(),
        p.family.clone(),
        p.company.clone(),
        p.role.clone(),
        lines(&p.emails),
        lines(&p.phones),
        p.linkedin.clone(),
        p.marker.clone(),
        if p.picture.is_some() { "yes" } else { "" }.to_owned(),
        p.dir.display().to_string(),
      ]
    })
    .collect();
  sheet(&mut wb, "pplr", &PPLR, &pplr, None)?;

  let contacts: Vec<Vec<String>> = plan
    .cards
    .iter()
    .map(|c| {
      vec![
        c.r#ref.clone(),
        c.name.clone(),
        c.organization.clone(),
        c.job_title.clone(),
        lines(&c.emails),
        lines(&c.phones),
        lines(&c.linkedin),
        c.account.clone(),
        c.groups.join(", "),
        c.marker.clone().unwrap_or_default(),
        c.proposed_for.clone().unwrap_or_default(),
        c.id.clone(),
      ]
    })
    .collect();
  sheet(&mut wb, CONTACTS_TITLE, &CONTACTS, &contacts, None)?;

  let how = wb.add_worksheet();
  how.set_name("How to review")?;
  how.set_column_width(0, HOW_WIDTH)?;
  let wrap = Format::new().set_text_wrap().set_align(FormatAlign::Top);
  let title = wrap.clone().set_bold().set_font_size(TITLE_SIZE);
  for (row, line) in (0u32..).zip(how_to_review(plan)) {
    let format = if row == 0 { &title } else { &wrap };
    if line.is_empty() {
      how.write_blank(row, 0, format)?;
    } else {
      how.write_string_with_format(row, 0, &line, format)?;
    }
  }
  wb.save(out)?;
  Ok(())
}

fn how_to_review(plan: &Plan) -> Vec<String> {
  let mut text: Vec<String> = [
    format!("pplr and Apple Contacts: sync plan, {}", plan.generated).as_str(),
    "",
    "Review the Merged sheet. Change only the four shaded columns; leave the rest as they are.",
    "  Decision: Add (a new iCloud card), Update (bring the card up to date from pplr), Link only (the pplr URL and group, no field changes), No change, or Skip (leave this person out of Contacts).",
    "  Card: the card to update or link, as a ref from the Contacts sheet (C0123). Change it if the plan picked the wrong card; clear it for Add.",
    "  Dupes: other cards that are the same person, comma-separated refs. They are listed for merging, never deleted without asking.",
    "  Notes: anything else, eg keep the card's name.",
    "",
    "Rows run least certain first: Low, then Medium, then High. Low and Medium rows are shaded in the Confidence column.",
    "Update never removes anything from a card. pplr wins on name, company and role; its emails, phones and LinkedIn are added beside the card's own. Addresses, birthdays, notes and photos on existing cards are left alone.",
    "Add makes an iCloud card from pplr with its photo, and every card touched gets the pplr URL and the PPLR group.",
    "Nothing is written until the reviewed workbook is applied, after a .vcf backup of every card.",
    "",
    "Proposals:",
  ]
  .iter()
  .map(|s| (*s).to_owned())
  .collect();
  for ((confidence, decision), n) in plan.proposals() {
    text.push(format!("  {n:4}  {confidence} / {decision}"));
  }
  text
}

/// What the reviewer got wrong, named as workbook.py named it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mistake {
  /// A sheet without a column --apply-plan reads.
  MissingHeader {
    sheet: &'static str,
    header: &'static str,
  },
  /// A Decision that is not one of Decision::ALL, as the reviewer wrote it.
  BadDecision { person: String, decision: String },
  /// Update or Link only without a card ref from the Contacts sheet.
  NoCard {
    person: String,
    decision: Decision,
    card: String,
  },
}

impl std::fmt::Display for Mistake {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Mistake::MissingHeader { sheet, header } => {
        write!(f, "The {sheet} sheet has no {header} column")
      }
      Mistake::BadDecision { person, decision } => write!(
        f,
        "{person}: Decision must be one of {}, not {}",
        Decision::ALL.map(Decision::as_str).join(", "),
        Py::Str(decision.clone()).py_repr()
      ),
      Mistake::NoCard {
        person,
        decision,
        card,
      } => write!(
        f,
        "{person}: {decision} needs a Card ref from the Contacts sheet, not {}",
        Py::Str(card.clone()).py_repr()
      ),
    }
  }
}

impl std::error::Error for Mistake {}

/// The reviewed workbook's decisions: Merged and Contacts read as text, then
/// `reviewed`. The reviewer's mistake is a Mistake, for the caller to report.
pub fn decisions(path: &Path) -> Result<Vec<Reviewed>> {
  let mut wb: Xlsx<_> = open_workbook(path)?;
  let text = |range: calamine::Range<Data>| -> Vec<Vec<String>> {
    range
      .rows()
      .map(|row| row.iter().map(ToString::to_string).collect())
      .collect()
  };
  let contacts = text(wb.worksheet_range(CONTACTS_TITLE)?);
  let merged = text(wb.worksheet_range(MERGED_TITLE)?);
  Ok(reviewed(&merged, &contacts)?)
}

/// A row's cell, or nothing past its end.
fn cell(row: &[String], col: usize) -> &str {
  row.get(col).map_or("", String::as_str)
}

/// The reviewed Merged rows as decisions, each card ref resolved through the
/// Contacts rows. The first row of each sheet is its header, and a column is
/// found by its header, wherever the reviewer has moved it.
pub fn reviewed(
  merged: &[Vec<String>],
  contacts: &[Vec<String>],
) -> std::result::Result<Vec<Reviewed>, Mistake> {
  let [person_at, decision_at, card_at] = columns(MERGED_TITLE, merged, [PERSON, DECISION, CARD])?;
  let [ref_at, id_at] = columns(CONTACTS_TITLE, contacts, [CARD, CARD_ID])?;
  let mut ids: HashMap<&str, &str> = HashMap::new();
  for row in contacts.iter().skip(1) {
    let r#ref = cell(row, ref_at);
    if !r#ref.is_empty() {
      ids.insert(r#ref, cell(row, id_at));
    }
  }
  let mut out = Vec::new();
  for row in merged.iter().skip(1) {
    let person = cell(row, person_at).to_owned();
    if person.is_empty() {
      continue;
    }
    let written = cell(row, decision_at).trim();
    let card = cell(row, card_at).trim();
    let Some(decision) = Decision::named(written) else {
      return Err(Mistake::BadDecision {
        person,
        decision: written.to_owned(),
      });
    };
    if matches!(decision, Decision::Update | Decision::LinkOnly) && !ids.contains_key(card) {
      return Err(Mistake::NoCard {
        person,
        decision,
        card: card.to_owned(),
      });
    }
    out.push(Reviewed {
      person,
      decision,
      card: if card.is_empty() {
        None
      } else {
        ids.get(card).map(|id| (*id).to_owned())
      },
    });
  }
  Ok(out)
}

/// Where each of `headers` is in the sheet's first row; the first one
/// missing is the mistake.
fn columns<const N: usize>(
  sheet: &'static str,
  rows: &[Vec<String>],
  headers: [&'static str; N],
) -> std::result::Result<[usize; N], Mistake> {
  let first = rows.first().map(Vec::as_slice).unwrap_or_default();
  let mut at = [0; N];
  for (slot, header) in at.iter_mut().zip(headers) {
    *slot = first
      .iter()
      .position(|h| h == header)
      .ok_or(Mistake::MissingHeader { sheet, header })?;
  }
  Ok(at)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn row(cells: &[&str]) -> Vec<String> {
    cells.iter().map(|c| (*c).to_owned()).collect()
  }

  /// Merged and Contacts as --plan writes them, one person reviewed.
  fn sheets() -> (Vec<Vec<String>>, Vec<Vec<String>>) {
    let header = |columns: &[(&str, u16, bool)]| columns.iter().map(|c| c.0.to_owned()).collect();
    let merged = vec![
      header(&MERGED),
      row(&["K/Kemp, Jon", "Update", "C0001", "", ""]),
    ];
    let contacts = vec![
      header(&CONTACTS),
      row(&[
        "C0001", "Jon Kemp", "", "", "", "", "", "home", "", "", "", "card-1",
      ]),
    ];
    (merged, contacts)
  }

  #[test]
  fn a_column_the_reviewer_inserted_is_read_by_its_header() {
    let (merged, contacts) = sheets();
    let want = reviewed(&merged, &contacts);
    assert_eq!(
      want,
      Ok(vec![Reviewed {
        person: "K/Kemp, Jon".to_owned(),
        decision: Decision::Update,
        card: Some("card-1".to_owned()),
      }])
    );
    let inserted = |rows: &[Vec<String>]| -> Vec<Vec<String>> {
      rows
        .iter()
        .enumerate()
        .map(|(i, r)| [vec![if i == 0 { "Mine" } else { "" }.to_owned()], r.clone()].concat())
        .collect()
    };
    assert_eq!(reviewed(&inserted(&merged), &inserted(&contacts)), want);
  }

  #[test]
  fn a_sheet_without_a_header_it_reads_is_refused_naming_it() {
    for (sheet, header) in [
      ("Merged", "Person"),
      ("Merged", "Decision"),
      ("Merged", "Card"),
      ("Contacts", "Card"),
      ("Contacts", "Card id"),
    ] {
      let (mut merged, mut contacts) = sheets();
      let rows = if sheet == "Merged" {
        &mut merged
      } else {
        &mut contacts
      };
      for h in rows.iter_mut().take(1).flatten().filter(|h| *h == header) {
        *h = format!("Not {header}");
      }
      assert_eq!(
        reviewed(&merged, &contacts).map_err(|e| e.to_string()),
        Err(format!("The {sheet} sheet has no {header} column")),
        "{sheet} without {header}"
      );
    }
  }
}
