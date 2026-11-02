//! `pplr new <first> <surname> [linkedin-url]`: a person's folder from the
//! template, and with a URL their LinkedIn .webloc and a fresh About.

use crate::Result;
use crate::assets::{PERSON_DIRS, PERSON_FILES, PERSON_OVERRIDE};
use crate::config::Config;
use crate::tree::{folder_of, names};
use std::fs;
use std::path::Path;
use std::process::{Command, ExitCode};

/// The template's About file, renamed for the person.
const TEMPLATE_ABOUT: &str = "About/Firstname Surname (About).md";

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let (first, surname, url) = match args {
    [first, surname] => (first, surname, None),
    [first, surname, url] => (first, surname, Some(url.as_str()).filter(|u| !u.is_empty())),
    _ => {
      println!("Usage: pplr_new <FIRSTNAME> <SURNAME> [URL]");
      return Ok(ExitCode::FAILURE);
    }
  };
  let dir = folder_of(&config.data, first, surname);
  if dir.is_dir() {
    println!(
      "Error: Person '{first} {surname}' already exists at '{}'.",
      dir.display()
    );
    return Ok(ExitCode::FAILURE);
  }
  // The author is read before anything is written, so a refusal leaves no folder
  let author = match url {
    Some(_) => Some(git_author()?),
    None => None,
  };
  fs::create_dir_all(&dir)?;
  copy_template(&config.data, &dir)?;
  // The template's Client/ is empty, and git does not keep an empty folder
  fs::create_dir_all(dir.join("Client"))?;

  let about = dir.join(format!("About/{first} {surname} (About).md"));
  if dir.join(TEMPLATE_ABOUT).is_file() {
    fs::rename(dir.join(TEMPLATE_ABOUT), &about)?;
  } else {
    println!("Warning: '(About)' file does not exist in the template directory.");
  }
  match (url, author) {
    (Some(url), Some(author)) => {
      let webloc = dir.join(format!("About/{first} {surname} (LinkedIn).webloc"));
      fs::write(webloc, webloc_xml(url))?;
      let day = config.today().to_string();
      fs::write(&about, about_md(first, surname, url, &author, &day))?;
    }
    _ => println!("Warning: URL not supplied, About file not updated and .webloc not created"),
  }
  println!("{}", about.display());
  Ok(ExitCode::SUCCESS)
}

/// The data tree's own template when it has one (`cp -r <dir>/*`, top-level
/// dot files left out), else the one inside the binary.
fn copy_template(root: &Path, dir: &Path) -> Result<()> {
  let custom = root.join(PERSON_OVERRIDE);
  if custom.is_dir() {
    for name in names(&custom).into_iter().filter(|n| !n.starts_with('.')) {
      copy_all(&custom.join(&name), &dir.join(&name))?;
    }
    return Ok(());
  }
  for sub in PERSON_DIRS {
    fs::create_dir_all(dir.join(sub))?;
  }
  for (path, bytes) in PERSON_FILES {
    fs::write(dir.join(path), bytes)?;
  }
  Ok(())
}

fn copy_all(from: &Path, to: &Path) -> Result<()> {
  if fs::symlink_metadata(from)?.is_dir() {
    fs::create_dir_all(to)?;
    for name in names(from) {
      copy_all(&from.join(&name), &to.join(&name))?;
    }
  } else {
    fs::copy(from, to)?;
  }
  Ok(())
}

/// The author for a verblock: `git config user.name` (design D10).
fn git_author() -> Result<String> {
  let out = Command::new("git")
    .args(["config", "user.name"])
    .output()
    .map_err(|e| format!("cannot run git for the author's name: {e}"))?;
  let name = String::from_utf8_lossy(&out.stdout).trim().to_owned();
  if name.is_empty() {
    return Err(
      "git config user.name is not set, so pplr new has no author for the About's verblock".into(),
    );
  }
  Ok(name)
}

/// The LinkedIn shortcut, as the legacy heredoc wrote it.
pub fn webloc_xml(url: &str) -> String {
  format!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n  <key>URL</key>\n  <string>{url}</string>\n</dict>\n</plist>\n"
  )
}

/// A new About for a person with a LinkedIn URL; the slug is everything after
/// the last "/in/", as `sed 's#.*/in/\(.*\)#\1#'` took it.
pub fn about_md(first: &str, surname: &str, url: &str, author: &str, day: &str) -> String {
  let slug = url.rfind("/in/").map_or("", |at| &url[at + "/in/".len()..]);
  format!(
    "---\nverblock: \"{day}:v0.1: {author} - Initial version\"\n---\n# {first} {surname} (About)\n\n_{first} {surname}_\n- Role:     []()\n- Company:  []()\n- LinkedIn: [in/{slug}]({url})\n- Email:    [](mailto:)\n- Phone:\n\n_About_\n"
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_about_takes_the_slug_after_the_last_in() {
    let about = about_md(
      "Wren",
      "Ashdown",
      "https://www.linkedin.com/in/wren-ashdown/",
      "A Author",
      "2026-06-15",
    );
    assert!(
      about.contains("- LinkedIn: [in/wren-ashdown/](https://www.linkedin.com/in/wren-ashdown/)\n")
    );
    assert!(
      about.starts_with("---\nverblock: \"2026-06-15:v0.1: A Author - Initial version\"\n---\n")
    );
    let no_slug = about_md("A", "B", "https://example.com/x", "A Author", "2026-06-15");
    assert!(no_slug.contains("- LinkedIn: [in/](https://example.com/x)\n"));
  }
}
