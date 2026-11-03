//! sync --plan's review workbook (ST0001 WP11), as the legacy workbook.py
//! laid it out: Merged (one row per pplr person, the proposal first and the
//! reviewer's columns beside it), pplr, Contacts, and How to review. The
//! reviewer edits Decision, Card, Dupes and Notes on Merged; the card refs
//! (C0001...) resolve through the Contacts sheet of the same workbook.

use crate::Result;
use crate::plan::{DECISIONS, Decision, Plan};
use crate::yaml::Value as Py;
use calamine::{Data, Reader, Xlsx, open_workbook};
use rust_xlsxwriter::{
  Color, ConditionalFormatFormula, DataValidation, Format, FormatAlign, Workbook, Worksheet,
};
use std::collections::HashMap;
use std::path::Path;

const REVIEW_FILL: u32 = 0xFFF4D6;
const LOW_FILL: u32 = 0xF9D9A8;
const MEDIUM_FILL: u32 = 0xFDF1C4;
const HOW_WIDTH: f64 = 140.0;
const TITLE_SIZE: u32 = 13;

/// Merged's columns: header, width, wrapped. B to E are the reviewer's.
const MERGED: [(&str, u16, bool); 21] = [
  ("Person", 26, false),
  ("Decision", 12, false),
  ("Card", 9, false),
  ("Dupes", 12, false),
  ("Notes", 28, false),
  ("Confidence", 11, false),
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
/// The reviewer's columns on Merged, by index: Decision, Card, Dupes, Notes.
const REVIEW: std::ops::RangeInclusive<u16> = 1..=4;
const DECISION_COL: u16 = 1;
const CONFIDENCE_COL: u16 = 5;

const PPLR: [(&str, u16, bool); 11] = [
  ("Person", 26, false),
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
  ("Card", 8, false),
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
  ("Card id", 44, false),
];
/// On the Contacts sheet: the card's ref and its id.
const REF_COL: usize = 0;
const ID_COL: usize = 11;

fn lines(values: &[String]) -> String {
  values
    .iter()
    .filter(|v| !v.is_empty())
    .cloned()
    .collect::<Vec<_>>()
    .join("\n")
}

/// One sheet: a bold header row, the rows top-aligned (wrapped where the
/// column says), the widths, the panes frozen at B2 and a filter over it all.
fn sheet<'a>(
  wb: &'a mut Workbook,
  title: &str,
  columns: &[(&str, u16, bool)],
  rows: &[Vec<String>],
  fill: Option<(std::ops::RangeInclusive<u16>, u32)>,
) -> Result<&'a mut Worksheet> {
  let ws = wb.add_worksheet();
  ws.set_name(title)?;
  let bold = Format::new().set_bold();
  let top = Format::new().set_align(FormatAlign::Top);
  for (col, (header, width, _)) in (0u16..).zip(columns) {
    ws.write_string_with_format(0, col, *header, &bold)?;
    ws.set_column_width(col, *width)?;
  }
  for (row, values) in (1u32..).zip(rows) {
    for (col, (value, (_, _, wrap))) in (0u16..).zip(values.iter().zip(columns)) {
      let mut format = if *wrap {
        top.clone().set_text_wrap()
      } else {
        top.clone()
      };
      if let Some((cols, color)) = &fill
        && cols.contains(&col)
      {
        format = format.set_background_color(Color::RGB(*color));
      }
      if value.is_empty() {
        ws.write_blank(row, col, &format)?;
      } else {
        ws.write_string_with_format(row, col, value, &format)?;
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
        r.decision.to_owned(),
        r.card.clone().unwrap_or_default(),
        r.dupes.join(", "),
        String::new(),
        r.confidence.to_owned(),
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
  let ws = sheet(
    &mut wb,
    "Merged",
    &MERGED,
    &merged,
    Some((REVIEW, REVIEW_FILL)),
  )?;
  if let Ok(last) = u32::try_from(merged.len())
    && last >= 1
  {
    let dv = DataValidation::new()
      .allow_list_strings(&DECISIONS)?
      .ignore_blank(false)
      // workbook.py's list sets no prompt and shows none
      .show_input_message(false)
      .set_error_title("Decision")?
      .set_error_message(format!("One of: {}", DECISIONS.join(", ")))?;
    ws.add_data_validation(1, DECISION_COL, last, DECISION_COL, &dv)?;
    for (level, color) in [("Low", LOW_FILL), ("Medium", MEDIUM_FILL)] {
      let rule = ConditionalFormatFormula::new()
        .set_rule(format!("=$F2=\"{level}\"").as_str())
        .set_format(Format::new().set_background_color(Color::RGB(color)));
      ws.add_conditional_format(1, CONFIDENCE_COL, last, CONFIDENCE_COL, &rule)?;
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
  sheet(&mut wb, "Contacts", &CONTACTS, &contacts, None)?;

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

fn text(cell: Option<&Data>) -> String {
  cell.map(ToString::to_string).unwrap_or_default()
}

/// The reviewed Merged sheet as decisions, card refs resolved through the
/// same workbook's Contacts sheet. The inner Err is the reviewer's mistake,
/// named, as workbook.py named it.
pub fn decisions(path: &Path) -> Result<std::result::Result<Vec<Decision>, String>> {
  let mut wb: Xlsx<_> = open_workbook(path)?;
  let mut ids: HashMap<String, String> = HashMap::new();
  for row in wb.worksheet_range("Contacts")?.rows().skip(1) {
    let r#ref = text(row.get(REF_COL));
    if !r#ref.is_empty() {
      ids.insert(r#ref, text(row.get(ID_COL)));
    }
  }
  let mut out = Vec::new();
  for row in wb.worksheet_range("Merged")?.rows().skip(1) {
    let person = text(row.first());
    if person.is_empty() {
      continue;
    }
    let decision = text(row.get(1)).trim().to_owned();
    let card = text(row.get(2)).trim().to_owned();
    if !DECISIONS.contains(&decision.as_str()) {
      return Ok(Err(format!(
        "{person}: Decision must be one of {}, not {}",
        DECISIONS.join(", "),
        Py::Str(decision).py_repr()
      )));
    }
    if (decision == "Update" || decision == "Link only") && !ids.contains_key(&card) {
      return Ok(Err(format!(
        "{person}: {decision} needs a Card ref from the Contacts sheet, not {}",
        Py::Str(card).py_repr()
      )));
    }
    out.push(Decision {
      person,
      decision,
      card: if card.is_empty() {
        None
      } else {
        ids.get(&card).cloned()
      },
    });
  }
  Ok(Ok(out))
}
