//! A person's `pplr://` marker, and the URLs built on it (ST0001 recon
//! section 6): `pplr://<letter>/<slug>[/<path in their folder>]`, and
//! `pplr://tag/<tag>` for a tag's page.

use crate::Result;
use crate::config::Config;
use crate::normalise::{fold_untrimmed, percent_decode};
use crate::tree::{Person, people};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const SCHEME: &str = "pplr://";

/// The commands that take a `pplr://` URL as it is; every other command gets
/// a person's marker rewritten to their folder name ("Surname, First").
pub const TAKES_URLS: &[&str] = &["open", "resolve", "links", "handler"];

/// `resolve` finding nobody, as the engine exited.
const NOBODY: u8 = 4;

/// "B/Bray, Martin" -> "b/bray-martin": lower case, accents dropped, and every
/// run of other characters one hyphen, so it is easy to type.
pub fn marker_path(key: &str) -> String {
  let (letter, name) = match key.split_once('/') {
    Some((letter, name)) => (Some(letter), name),
    None => (None, key),
  };
  let folded: String = fold_untrimmed(name)
    .chars()
    .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
    .collect();
  let slug = folded
    .split('-')
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("-");
  let head = match letter {
    Some(letter) => letter.to_lowercase(),
    None => slug.chars().take(1).collect(),
  };
  format!("{head}/{slug}")
}

/// "pplr://b/bray-martin".
pub fn marker_url(key: &str) -> String {
  format!("{SCHEME}{}", marker_path(key))
}

/// The marker path of a person's key, then of each former key.
pub fn marker_paths(person: &Person) -> impl Iterator<Item = String> + '_ {
  person.keys().map(|key| marker_path(key))
}

/// Everyone by their marker path, and by the marker of each former name.
pub fn by_marker(people: &[Person]) -> HashMap<String, &Person> {
  let mut out = HashMap::new();
  for person in people {
    for path in marker_paths(person) {
      out.insert(path, person);
    }
  }
  out
}

/// What a person URL names: the person, and the file or folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target<'a> {
  pub person: &'a Person,
  /// "" for the person, else eg "Meetings/20260828 Catch-up/Notes.md".
  pub rest: String,
  pub path: PathBuf,
}

/// The person (and the path in their folder) a `pplr://` URL names.
pub fn resolve<'a>(url: &str, by_marker: &HashMap<String, &'a Person>) -> Option<Target<'a>> {
  let body = strip_scheme(url)?;
  let mut parts = body.splitn(3, '/');
  let (letter, slug) = (parts.next()?, parts.next()?);
  let person = *by_marker.get(&format!(
    "{}/{}",
    letter.to_lowercase(),
    slug.to_lowercase()
  ))?;
  let raw = parts.next().unwrap_or_default();
  let rest = percent_decode(raw).unwrap_or_else(|| raw.to_owned());
  let path = if rest.is_empty() {
    person.page().to_path_buf()
  } else {
    person.dir.join(rest.trim_end_matches('/'))
  };
  Some(Target { person, rest, path })
}

/// `pplr://tag/vc` -> `<root>/_tags/vc.md`; `pplr://tag` -> `<root>/_tags/index.md`.
/// A person URL starts with a single letter, so "tag" and "tags" cannot clash.
pub fn tag_page(url: &str, root: &Path) -> Option<PathBuf> {
  let mut parts = strip_scheme(url)?.split('/').filter(|p| !p.is_empty());
  let head = parts.next()?.to_lowercase();
  if head != "tag" && head != "tags" {
    return None;
  }
  let name = parts
    .next()
    .map_or_else(|| "index".to_owned(), |t| t.to_lowercase().replace('#', ""));
  Some(root.join("_tags").join(format!("{name}.md")))
}

fn strip_scheme(url: &str) -> Option<&str> {
  url
    .get(..SCHEME.len())
    .filter(|head| head.eq_ignore_ascii_case(SCHEME))
    .map(|_| &url[SCHEME.len()..])
}

/// `^pplr://[a-z]/[^/]+/?$`: a person's marker, with no path inside their folder.
fn is_person_marker(arg: &str) -> bool {
  let Some(rest) = arg.strip_prefix(SCHEME) else {
    return false;
  };
  let mut chars = rest.chars();
  let letter = chars.next().is_some_and(|c| c.is_ascii_lowercase());
  let key = chars
    .as_str()
    .strip_prefix('/')
    .map(|k| k.strip_suffix('/').unwrap_or(k));
  letter && key.is_some_and(|k| !k.is_empty() && !k.contains('/'))
}

/// A command's arguments with each person marker rewritten to the person's
/// folder name, as `bin/pplr` rewrote them; the commands in TAKES_URLS get
/// theirs as they are. A marker that names nobody stops the command.
pub fn rewrite_args(command: &str, args: &[String], config: &Config) -> Result<Vec<String>> {
  if TAKES_URLS.contains(&command) || !args.iter().any(|a| is_person_marker(a)) {
    return Ok(args.to_vec());
  }
  let everyone = people(&config.data)?;
  let by_marker = by_marker(&everyone);
  args
    .iter()
    .map(|arg| {
      if !is_person_marker(arg) {
        return Ok(arg.clone());
      }
      let target = resolve(arg, &by_marker).ok_or_else(|| format!("no one answers to {arg}"))?;
      Ok(
        target
          .person
          .dir
          .file_name()
          .map(|n| n.to_string_lossy().into_owned())
          .unwrap_or_default(),
      )
    })
    .collect()
}

/// `pplr resolve <url>`: the file or folder a `pplr://` URL names.
pub fn run_resolve(args: &[String], config: &Config) -> Result<ExitCode> {
  let [url] = args else {
    println!("Usage: pplr resolve pplr://<letter>/<surname-first>[/<path>]");
    return Ok(ExitCode::FAILURE);
  };
  match target_path(url, config)? {
    Some(path) => {
      println!("{}", path.display());
      Ok(ExitCode::SUCCESS)
    }
    None => {
      eprintln!("pplr: no one answers to {url}");
      Ok(ExitCode::from(NOBODY))
    }
  }
}

/// The file or folder a URL names: a tag's page, or a person's.
pub fn target_path(url: &str, config: &Config) -> Result<Option<PathBuf>> {
  if let Some(page) = tag_page(url, &config.data) {
    return Ok(Some(page));
  }
  let everyone = people(&config.data)?;
  Ok(resolve(url, &by_marker(&everyone)).map(|t| t.path))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::about::Header;

  fn person(key: &str, aliases: &[&str]) -> Person {
    Person {
      key: key.to_owned(),
      given: String::new(),
      family: String::new(),
      dir: PathBuf::from("/p").join(key),
      about: None,
      picture: None,
      aliases: aliases.iter().map(|a| (*a).to_owned()).collect(),
      header: Header::default(),
    }
  }

  #[test]
  fn a_marker_is_the_letter_and_a_typeable_slug() {
    assert_eq!(marker_path("B/Bray, Martin"), "b/bray-martin");
    assert_eq!(marker_path("V/Varga, Zsófia"), "v/varga-zsofia");
    assert_eq!(marker_path("D/de Vries, Annelies"), "d/de-vries-annelies");
    assert_eq!(
      marker_path("M/Moreau-Lindqvist, Henrik"),
      "m/moreau-lindqvist-henrik"
    );
    assert_eq!(marker_path("Ø/Ødegård, Ola"), "ø/degard-ola");
    assert_eq!(marker_url("K/Kemp, Jon"), "pplr://k/kemp-jon");
  }

  #[test]
  fn a_url_resolves_by_marker_alias_and_path() {
    let everyone = [person("T/Thistlewood, Ottoline", &["T/Thistle, Ottoline"])];
    let by = by_marker(&everyone);
    let whole = resolve("pplr://t/thistlewood-ottoline", &by).map(|t| t.path);
    assert_eq!(whole, Some(PathBuf::from("/p/T/Thistlewood, Ottoline")));
    let alias = resolve("PPLR://T/Thistle-Ottoline/", &by).map(|t| t.rest);
    assert_eq!(alias, Some(String::new()));
    let inside = resolve(
      "pplr://t/thistle-ottoline/Meetings/20260702%20Board%20prep",
      &by,
    );
    assert_eq!(
      inside.map(|t| t.path),
      Some(PathBuf::from(
        "/p/T/Thistlewood, Ottoline/Meetings/20260702 Board prep"
      ))
    );
    assert_eq!(resolve("pplr://T/Thistle,%20Ottoline", &by), None);
    assert_eq!(resolve("https://t/thistle-ottoline", &by), None);
  }

  #[test]
  fn a_tag_url_names_its_page_or_the_index() {
    let root = Path::new("/p");
    assert_eq!(
      tag_page("pplr://tag/VIP", root),
      Some(PathBuf::from("/p/_tags/vip.md"))
    );
    assert_eq!(
      tag_page("pplr://tags/#cto", root),
      Some(PathBuf::from("/p/_tags/cto.md"))
    );
    assert_eq!(
      tag_page("pplr://tag", root),
      Some(PathBuf::from("/p/_tags/index.md"))
    );
    assert_eq!(tag_page("pplr://t/thistle-ottoline", root), None);
  }

  #[test]
  fn only_a_bare_person_marker_is_rewritten() {
    assert!(is_person_marker("pplr://k/kemp-jon"));
    assert!(is_person_marker("pplr://k/kemp-jon/"));
    assert!(!is_person_marker("pplr://k/kemp-jon/About"));
    assert!(!is_person_marker("pplr://K/kemp-jon"));
    assert!(!is_person_marker("pplr://tag/vip/x"));
    for other in ["pplr://k/", "pplr://kk/x", "Kemp, Jon", "http://k/x"] {
      assert!(!is_person_marker(other), "{other}");
    }
  }
}
