//! `pplr meetings [date [end]]`: the meeting folders dated in a range.

use crate::Result;
use crate::config::Config;
use std::path::Path;
use std::process::ExitCode;

const DATE_DIGITS: usize = 8;

fn usage() -> ExitCode {
  println!("Usage: pplr_meetings <date> [<end_date>]");
  println!("  <date>: Date in YYYY, YYYYMM, or YYYYMMDD format");
  println!("  <end_date>: Optional end date in YYYY, YYYYMM, or YYYYMMDD format");
  ExitCode::FAILURE
}

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let today = config.today().strftime("%Y%m%d").to_string();
  let (start, end) = match args {
    [] => (today.clone(), today),
    [start] => (start.clone(), start.clone()),
    [start, end, ..] => (start.clone(), end.clone()),
  };
  let (start, end) = (start.replace('-', ""), end.replace('-', ""));
  if !is_digits(&start) {
    println!("Error: Invalid date format");
    return Ok(usage());
  }
  if !is_digits(&end) {
    println!("Error: End date must contain only digits (after removing hyphens)");
    return Ok(usage());
  }
  let lines = meetings(&config.data, &first_day(&start), last_day(&end).as_deref());
  if lines.is_empty() {
    println!("No meetings found");
  } else {
    println!("{}", lines.join("\n"));
  }
  Ok(ExitCode::SUCCESS)
}

fn is_digits(s: &str) -> bool {
  !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// YYYY and YYYYMM widen to their first day; anything else is taken as it is.
fn first_day(date: &str) -> String {
  match date.len() {
    4 => format!("{date}0101"),
    6 => format!("{date}01"),
    _ => date.to_owned(),
  }
}

/// YYYY and YYYYMM widen to their last day; a month past 12 has none.
fn last_day(date: &str) -> Option<String> {
  match date.len() {
    4 => Some(format!("{date}1231")),
    6 => {
      let year: u32 = date[..4].parse().ok()?;
      let last = match &date[4..] {
        "01" | "03" | "05" | "07" | "08" | "10" | "12" => "31",
        "04" | "06" | "09" | "11" => "30",
        "02"
          if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) =>
        {
          "29"
        }
        "02" => "28",
        _ => return None,
      };
      Some(format!("{date}{last}"))
    }
    _ => Some(date.to_owned()),
  }
}

/// "YYYYMMDD Title (First Surname)" for each folder under a Meetings folder,
/// at any depth, whose name starts with a date in the range; sorted.
fn meetings(root: &Path, start: &str, end: Option<&str>) -> Vec<String> {
  let (Ok(start), Some(Ok(end))) = (start.parse::<u64>(), end.map(str::parse::<u64>)) else {
    return Vec::new();
  };
  let mut out = Vec::new();
  let mut stack = vec![root.to_path_buf()];
  while let Some(dir) = stack.pop() {
    let Ok(entries) = std::fs::read_dir(&dir) else {
      continue;
    };
    for entry in entries.flatten() {
      let path = entry.path();
      if !entry.file_type().is_ok_and(|t| t.is_dir()) {
        continue;
      }
      stack.push(path.clone());
      let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
      if !format!("./{rel}").contains("/Meetings/") || rel.contains("Icon") {
        continue;
      }
      let name = entry.file_name().to_string_lossy().into_owned();
      let Some(date) = name
        .get(..DATE_DIGITS)
        .filter(|d| d.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|d| d.parse::<u64>().ok())
      else {
        continue;
      };
      if date < start || date > end {
        continue;
      }
      let person = path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
      out.push(format!("{name} ({})", display(&person)));
    }
  }
  out.sort();
  out
}

/// "Last, First" as "First Last", as `cut -d, -f1` and `-f2` split it.
fn display(folder: &str) -> String {
  if !folder.contains(", ") {
    return folder.to_owned();
  }
  let mut fields = folder.split(',');
  let last = fields.next().unwrap_or_default();
  let first = fields.next().unwrap_or_default().trim_start_matches(' ');
  format!("{first} {last}")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_year_or_a_month_widens_to_its_days() {
    assert_eq!(first_day("2026"), "20260101");
    assert_eq!(last_day("2026").as_deref(), Some("20261231"));
    assert_eq!(first_day("202602"), "20260201");
    assert_eq!(last_day("202402").as_deref(), Some("20240229"));
    assert_eq!(last_day("190002").as_deref(), Some("19000228"));
    assert_eq!(last_day("202613"), None);
    assert_eq!(last_day("20260315").as_deref(), Some("20260315"));
  }

  #[test]
  fn a_folder_name_reads_first_name_first() {
    assert_eq!(display("Okafor, Chidi"), "Chidi Okafor");
    assert_eq!(display("Meetings"), "Meetings");
  }
}
