//! Finder icons through `fileicon`: `applyicons` (the About, Meetings and
//! meeting icons), `setpicfordir` (a person's picture on their folder) and
//! `setpicsfordirs` (everyone's). macOS only (design D9).

use crate::Result;
use crate::assets::ICONS;
use crate::config::Config;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// The icons applyicons needs, by file name.
const ABOUT: &str = "About.pdf";
const MEETING: &str = "Meeting.pdf";
const CALENDAR: &str = "Calendar.pdf";

/// Finder's own files in a folder: "Icon\r" holds the folder's icon.
const FINDER_FILES: [&str; 2] = ["Icon\r", ".DS_Store"];

pub fn applyicons(args: &[String], config: &Config) -> Result<ExitCode> {
  macos_with_fileicon()?;
  let [first, surname, ..] = args else {
    println!("Usage: pplr applyicons <FIRSTNAME> <SURNAME>");
    return Ok(ExitCode::FAILURE);
  };
  let person = person_dir(&config.data, first, surname);
  if !person.is_dir() {
    println!("Error: Directory for '{first} {surname}' not found.");
    return Ok(ExitCode::FAILURE);
  }
  println!("Applying icons to {}...", person.display());
  let icons = IconDir::find(config)?;
  if [ABOUT, MEETING, CALENDAR]
    .iter()
    .any(|name| !icons.path(name).is_file())
  {
    println!(
      "Error: Required icon files not found in {}.",
      icons.dir().display()
    );
    return Ok(ExitCode::FAILURE);
  }

  let about = person.join("About");
  if about.is_dir() {
    println!("Applying About icon to {}...", about.display());
    set_icon(&about, &icons.path(ABOUT))
      .unwrap_or_else(|_| println!("Error: Failed to apply About icon."));
  } else {
    println!("Warning: About directory not found for {first} {surname}.");
  }

  let meetings = person.join("Meetings");
  if meetings.is_dir() {
    println!("Applying Meetings icon to {}...", meetings.display());
    set_icon(&meetings, &icons.path(MEETING))
      .unwrap_or_else(|_| println!("Error: Failed to apply Meetings icon."));
    println!(
      "Applying Calendar icon to directories and files in {}...",
      meetings.display()
    );
    for item in sorted_entries(&meetings)? {
      let name = item
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
      if FINDER_FILES.contains(&name.as_str()) {
        continue;
      }
      set_icon(&item, &icons.path(CALENDAR)).unwrap_or_else(|_| {
        println!(
          "Error: Failed to apply Calendar icon to {}.",
          item.display()
        )
      });
    }
  } else {
    println!("Warning: Meetings directory not found for {first} {surname}.");
  }
  println!("Icons successfully applied for {first} {surname}.");
  Ok(ExitCode::SUCCESS)
}

pub fn setpicfordir(args: &[String], config: &Config) -> Result<ExitCode> {
  macos_with_fileicon()?;
  let mut args: Vec<&str> = args.iter().map(String::as_str).collect();
  let verbose = match args.first() {
    Some(&("-V" | "--verbose")) => {
      args.remove(0);
      true
    }
    Some(flag) if flag.starts_with('-') => {
      println!("Unknown option: {flag}");
      return Ok(ExitCode::FAILURE);
    }
    _ => false,
  };
  let [first, surname, ..] = args[..] else {
    println!("Usage: pplr setpicfordir [-V|--verbose] <FIRSTNAME> <SURNAME>");
    return Ok(ExitCode::FAILURE);
  };
  match set_picture(&config.data, first, surname, verbose) {
    Ok(()) => Ok(ExitCode::SUCCESS),
    Err(message) => {
      println!("{message}");
      Ok(ExitCode::FAILURE)
    }
  }
}

pub fn setpicsfordirs(_args: &[String], config: &Config) -> Result<ExitCode> {
  macos_with_fileicon()?;
  for letter in 'A'..='Z' {
    let letter_dir = config.data.join(letter.to_string());
    if !letter_dir.is_dir() {
      continue;
    }
    println!("Processing letter directory: {}", letter_dir.display());
    for person in sorted_entries(&letter_dir)?
      .into_iter()
      .filter(|p| p.is_dir())
    {
      let name = person
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
      println!("Setting icon for: {name}");
      // "Surname, First" split at its comma, so a multi-word name stays whole
      let (surname, first) = name
        .split_once(',')
        .map_or((name.as_str(), ""), |(s, f)| (s.trim(), f.trim()));
      if let Err(message) = set_picture(&config.data, first, surname, false) {
        println!("{message}");
        println!("Error: Failed to set icon for {name}");
      }
    }
  }
  println!("Done setting icons for all directories.");
  Ok(ExitCode::SUCCESS)
}

/// A person's About picture on their folder; the error is the message to print.
fn set_picture(data: &Path, first: &str, surname: &str, verbose: bool) -> Result<()> {
  let log = |line: String| {
    if verbose {
      println!("{line}");
    }
  };
  log(format!("Setting directory icon for {first} {surname}..."));
  let target = person_dir(data, first, surname);
  let about = target.join("About");
  log(format!("Target directory: \"{}\"", target.display()));
  log(format!("About directory: \"{}\"", about.display()));
  if !target.is_dir() {
    return Err(
      format!(
        "Error: Target directory '{}' does not exist.",
        target.display()
      )
      .into(),
    );
  }
  // By suffix, not name: an accented name typed one way need not match the bytes on disk
  let picture = sorted_entries(&about)
    .unwrap_or_default()
    .into_iter()
    .find(|p| {
      let name = p
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
      name.ends_with(" (picture).jpg") || name.ends_with(" (picture).png")
    });
  let Some(picture) = picture else {
    return Err(
      format!(
        "Error: No picture file found for '{first} {surname}' in '{}'.",
        about.display()
      )
      .into(),
    );
  };
  log(format!("Found picture file: \"{}\"", picture.display()));
  log("Setting folder icon...".to_owned());
  set_icon(&target, &picture).map_err(|_| "Error: Failed to set folder icon")?;
  log(format!(
    "Successfully set the picture for '{first} {surname}' as the directory icon."
  ));
  log("Note: You may need to restart Finder to see the changes.".to_owned());
  log("To restart Finder, hold the Option key and right-click on the Finder icon in the Dock, then select Relaunch.".to_owned());
  Ok(())
}

/// `<data>/<S>/<Surname>, <First>`, lettered by the surname's first character.
fn person_dir(data: &Path, first: &str, surname: &str) -> PathBuf {
  let letter: String = surname
    .chars()
    .take(1)
    .map(|c| c.to_ascii_uppercase())
    .collect();
  data.join(letter).join(format!("{surname}, {first}"))
}

fn set_icon(target: &Path, icon: &Path) -> Result<()> {
  let status = Command::new("fileicon")
    .arg("set")
    .arg(target)
    .arg(icon)
    .status()?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("fileicon set {} failed", target.display()).into())
  }
}

fn sorted_entries(dir: &Path) -> Result<Vec<PathBuf>> {
  let mut entries = std::fs::read_dir(dir)
    .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
    .map(|entry| entry.map(|e| e.path()))
    .collect::<std::io::Result<Vec<_>>>()?;
  entries.sort();
  Ok(entries)
}

/// Icons need Finder, and `fileicon` to reach it.
fn macos_with_fileicon() -> Result<()> {
  crate::tools::macos_only("Finder icons need macOS")?;
  if crate::tools::on_path("fileicon").is_none() {
    return Err(
      "'fileicon' command not found. Please install it using: brew install fileicon".into(),
    );
  }
  Ok(())
}

/// Where the icons come from: PPLR_ICON_DIR, else the data tree's
/// `_pplr/icons`, else the set inside the binary, written to a temporary
/// directory for `fileicon` and removed when dropped.
enum IconDir {
  Given(PathBuf),
  Embedded(PathBuf),
}

impl IconDir {
  fn find(config: &Config) -> Result<Self> {
    if let Some(dir) = &config.icon_dir {
      return Ok(Self::Given(dir.clone()));
    }
    let tree = config.data.join("_pplr/icons");
    if tree.is_dir() {
      return Ok(Self::Given(tree));
    }
    let dir = std::env::temp_dir().join(format!("pplr-icons-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    for (name, bytes) in ICONS {
      std::fs::write(dir.join(name), bytes)?;
    }
    Ok(Self::Embedded(dir))
  }

  fn dir(&self) -> &Path {
    match self {
      Self::Given(dir) | Self::Embedded(dir) => dir,
    }
  }

  fn path(&self, name: &str) -> PathBuf {
    self.dir().join(name)
  }
}

impl Drop for IconDir {
  fn drop(&mut self) {
    if let Self::Embedded(dir) = self {
      let _ = std::fs::remove_dir_all(dir);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_person_is_filed_under_their_surnames_first_letter() {
    let data = Path::new("/d");
    assert_eq!(
      person_dir(data, "Jon", "kemp"),
      PathBuf::from("/d/K/kemp, Jon")
    );
    assert_eq!(
      person_dir(data, "Ann", "Van der Berg"),
      PathBuf::from("/d/V/Van der Berg, Ann")
    );
  }

  #[test]
  fn every_icon_applyicons_needs_is_embedded() {
    for name in [ABOUT, MEETING, CALENDAR] {
      assert!(
        ICONS
          .iter()
          .any(|(n, bytes)| *n == name && !bytes.is_empty()),
        "{name}"
      );
    }
  }
}
