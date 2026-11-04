//! The People tree: its walk rules, its people and the ways a command names
//! one (ST0001 recon sections 3 and 5).

use crate::Result;
use crate::about::{Header, header};
use crate::fs::read_text;
use std::fs;
use std::path::{Path, PathBuf};

/// The endings of a picture's name in About/.
const PICTURES: [&str; 3] = ["(Picture).jpg", "(Picture).png", "(Picture).jpeg"];

/// A person's picture among their About/ folder's names, sorted: the first
/// with a picture's ending, as the engine found it. The one rule: the engine,
/// `pplr pictures` and the index all take it.
pub fn picture(names: &[String]) -> Option<&String> {
  names
    .iter()
    .find(|n| PICTURES.iter().any(|ending| n.ends_with(ending)))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
  /// "K/Kemp, Jon": the letter folder and the person folder.
  pub key: String,
  /// The folder name after its first ", ".
  pub given: String,
  /// The folder name before its first ", ".
  pub family: String,
  /// The person's folder.
  pub dir: PathBuf,
  /// Their About file: the first `*(About).md` in About/, by name.
  pub about: Option<PathBuf>,
  /// Their picture: the first file in About/, by name, ending (Picture).jpg,
  /// .png or .jpeg.
  pub picture: Option<PathBuf>,
  /// Former keys ("H/Horely, Steve"), from .index/aliases, written by rename.
  pub aliases: Vec<String>,
  /// The About file's header.
  pub header: Header,
}

impl Person {
  /// What a bare link to the person opens: their About file, or their folder.
  pub fn page(&self) -> &Path {
    self.about.as_deref().unwrap_or(&self.dir)
  }

  /// Their key, then each former key.
  pub fn keys(&self) -> impl Iterator<Item = &String> {
    std::iter::once(&self.key).chain(&self.aliases)
  }
}

/// Every person under `root`, as the legacy engine loaded them: each letter
/// folder (a single upper-case letter) and each folder in it holding ", ",
/// both by name. An About or aliases file that is there but cannot be read
/// is the error, naming it.
pub fn people(root: &Path) -> Result<Vec<Person>> {
  let mut out = Vec::new();
  for letter in names(root).into_iter().filter(|n| is_letter(n)) {
    let letter_dir = root.join(&letter);
    for name in names(&letter_dir).into_iter().filter(|n| n.contains(", ")) {
      out.push(person(&letter_dir.join(&name), &letter, &name)?);
    }
  }
  Ok(out)
}

fn person(dir: &Path, letter: &str, name: &str) -> Result<Person> {
  let (family, given) = name.split_once(", ").unwrap_or((name, ""));
  let aliases = read_text(&dir.join(".index/aliases"))?
    .unwrap_or_default()
    .split(['\n', '\r'])
    .map(str::trim)
    .filter(|a| !a.is_empty())
    .map(str::to_owned)
    .collect();
  let about_dir = dir.join("About");
  let files = names(&about_dir);
  let about = files
    .iter()
    .find(|n| n.ends_with("(About).md"))
    .map(|n| about_dir.join(n));
  let picture = picture(&files).map(|n| about_dir.join(n));
  let text = match &about {
    Some(path) => read_text(path)?,
    None => None,
  };
  Ok(Person {
    key: format!("{letter}/{name}"),
    given: given.to_owned(),
    family: family.to_owned(),
    dir: dir.to_path_buf(),
    about,
    picture,
    aliases,
    header: text.map(|text| header(&text)).unwrap_or_default(),
  })
}

/// A letter folder's name: one upper-case letter.
pub fn is_letter(name: &str) -> bool {
  let mut chars = name.chars();
  chars.next().is_some_and(char::is_uppercase) && chars.next().is_none()
}

/// The names in a directory, sorted; none when it cannot be read.
pub fn names(dir: &Path) -> Vec<String> {
  let mut out: Vec<String> = fs::read_dir(dir)
    .map(|entries| {
      entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect()
    })
    .unwrap_or_default();
  out.sort();
  out
}

/// The letter folder for a surname: its first character, upper-cased, as
/// `cut -c1 | tr '[:lower:]' '[:upper:]'` gives it.
pub fn letter_of(surname: &str) -> String {
  surname
    .chars()
    .next()
    .map(|c| c.to_uppercase().collect())
    .unwrap_or_default()
}

/// The folder of a person named as `<first> <surname>`, whether it exists or not.
pub fn folder_of(root: &Path, first: &str, surname: &str) -> PathBuf {
  root
    .join(letter_of(surname))
    .join(format!("{surname}, {first}"))
}

/// The folder of a person named as "Surname, First" or "L/Surname, First"
/// (anything up to the first / is dropped), whether it exists or not.
pub fn folder_of_name(root: &Path, name: &str) -> (String, PathBuf) {
  let name = name.split_once('/').map_or(name, |(_, rest)| rest);
  (name.to_owned(), root.join(letter_of(name)).join(name))
}

/// A file in the About folder of a person named as `<first> <surname>`, eg
/// with the suffix "(About).md".
pub fn about_file(root: &Path, first: &str, surname: &str, suffix: &str) -> PathBuf {
  folder_of(root, first, surname)
    .join("About")
    .join(format!("{first} {surname} {suffix}"))
}

/// Every person folder as the legacy Python helpers globbed them,
/// `[A-Z]/*, *` (ASCII letters, no dot names), by path.
pub fn person_dirs(root: &Path) -> Vec<PathBuf> {
  let mut out: Vec<PathBuf> = names(root)
    .into_iter()
    .filter(|n| n.len() == 1 && n.bytes().all(|b| b.is_ascii_uppercase()))
    .flat_map(|letter| {
      names(&root.join(&letter))
        .into_iter()
        .filter(|n| n.contains(", ") && !n.starts_with('.'))
        .map(move |n| root.join(&letter).join(n))
    })
    .filter(|d| d.is_dir())
    .collect();
  out.sort_by_key(|p| p.to_string_lossy().into_owned());
  out
}

/// "Surname, First" as "First Surname": the stem of a person's About files.
pub fn display_name(dir: &Path) -> String {
  let name = dir
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_default();
  match name.split_once(", ") {
    Some((surname, first)) => format!("{first} {surname}"),
    None => name,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn letter_folders_are_one_upper_case_letter() {
    assert!(is_letter("K"));
    assert!(is_letter("\u{d6}"));
    assert!(!is_letter("k"));
    assert!(!is_letter("_tags"));
    assert!(!is_letter("bin"));
  }

  #[test]
  fn a_person_is_named_by_two_arguments_or_one_folder_name() {
    let root = Path::new("/p");
    assert_eq!(
      folder_of(root, "Annelies", "de Vries"),
      PathBuf::from("/p/D/de Vries, Annelies")
    );
    assert_eq!(
      folder_of_name(root, "V/Varga, Zsófia"),
      (
        "Varga, Zsófia".to_owned(),
        PathBuf::from("/p/V/Varga, Zsófia")
      )
    );
    assert_eq!(
      about_file(root, "Jon", "Kemp", "(About).md")
        .display()
        .to_string(),
      "/p/K/Kemp, Jon/About/Jon Kemp (About).md"
    );
  }

  /// AT-05.1 (ST0003 AC-05.1): the engine, pplr pictures and the index take a
  /// person's picture by one rule, the first picture in About/ by name: with
  /// two pictures, and with one not named after the person. With none, the
  /// index names "<First Surname> (Picture).jpg", as 2.0.0 did.
  #[test]
  fn the_engine_pictures_and_the_index_take_one_picture() -> Result<()> {
    let root = std::env::temp_dir().join(format!("pplr-picture-rule-{}", std::process::id()));
    // Each person's key, their About's stem, their pictures, and the picture
    // each command should take
    let cases = [
      (
        "L/Lovelace, Ada",
        "Ada Lovelace",
        &["A (Picture).png", "Ada Lovelace (Picture).jpg"][..],
        Some("A (Picture).png"),
      ),
      (
        "B/Babbage, Charles",
        "Charles Babbage",
        &["Old Name (Picture).jpg"][..],
        Some("Old Name (Picture).jpg"),
      ),
      ("S/Somerville, Mary", "Mary Somerville", &[][..], None),
    ];
    for (key, stem, pictures, _) in cases {
      let about = root.join(key).join("About");
      fs::create_dir_all(&about)?;
      fs::write(
        about.join(format!("{stem} (About).md")),
        format!("# {stem} (About)\n"),
      )?;
      for picture in pictures {
        fs::write(about.join(picture), "")?;
      }
    }
    let config = crate::config::Config::from_lookup(|name| match name {
      "HOME" | "PPLR_DATA" => Some(root.clone()),
      _ => None,
    })?;
    let people = people(&root)?;
    let index = crate::index::letters(&root);
    crate::pictures::run(&[], &config)?;
    for (key, stem, _, want) in cases {
      let engine = people
        .iter()
        .find(|p| p.key == key)
        .and_then(|p| p.picture.as_ref())
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned());
      assert_eq!(engine.as_deref(), want, "the engine's picture for {key}");
      let (letter, name) = key.split_once('/').ok_or("a key with no letter")?;
      let indexed = index
        .iter()
        .find(|l| l.name == letter)
        .and_then(|l| l.people.iter().find(|e| e.name == name))
        .map(|e| e.picture.clone())
        .ok_or_else(|| format!("the index has no {key}"))?;
      let file = want.map_or_else(|| format!("{stem} (Picture).jpg"), str::to_owned);
      assert_eq!(
        indexed,
        format!("./{key}/About/{file}"),
        "the index's picture for {key}"
      );
      let text = fs::read_to_string(
        root
          .join(key)
          .join("About")
          .join(format!("{stem} (About).md")),
      )?;
      let shown = text
        .lines()
        .find_map(|l| l.strip_prefix("<img src=\""))
        .and_then(|rest| rest.split_once('"'))
        .map(|(src, _)| src.to_owned());
      assert_eq!(shown.as_deref(), want, "pplr pictures' picture for {key}");
    }
    fs::remove_dir_all(&root)?;
    Ok(())
  }
}
