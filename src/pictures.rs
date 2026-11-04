//! `pplr pictures [--dry-run] ["Surname, First"]...`: the person's photo at
//! the top of their About page, as `<img src="First Surname (Picture).jpg"
//! alt="First Surname" width="160" align="right">` under the title.

use crate::Result;
use crate::config::Config;
use crate::fs::read_exact;
use crate::tree::{folder_of_name, names};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub fn run(args: &[String], config: &Config) -> Result<ExitCode> {
  let mut dry = false;
  let mut wanted = Vec::new();
  for arg in args {
    match arg.as_str() {
      "--dry-run" => dry = true,
      "-h" | "--help" => {
        println!("Usage: pplr pictures [--dry-run] [\"Surname, First\"]...");
        println!(
          "  Puts <img src=\"First Surname (Picture).jpg\"> under the About file's title, floated"
        );
        println!("  right, for everyone with a picture in About/. Re-running changes nothing, or");
        println!(
          "  updates the line if the picture changed (eg .png to .jpg). No picture, no line."
        );
        return Ok(ExitCode::SUCCESS);
      }
      _ => wanted.push(arg.clone()),
    }
  }
  let root = &config.data;
  let dirs: Vec<PathBuf> = if wanted.is_empty() {
    everyone(root)
  } else {
    wanted.iter().map(|n| folder_of_name(root, n).1).collect()
  };
  let (mut added, mut updated, mut same, mut none) = (0, 0, 0, 0);
  for dir in dirs {
    let rel = |p: &Path| p.strip_prefix(root).unwrap_or(p).display().to_string();
    if !dir.is_dir() {
      println!("No such person: {}", rel(&dir));
      continue;
    }
    let about_dir = dir.join("About");
    let files = names(&about_dir);
    let Some(about) = files.iter().find(|n| n.ends_with("(About).md")) else {
      continue;
    };
    let about = about_dir.join(about);
    // The one picture rule, the engine's and the index's too
    let Some(picture) = crate::tree::picture(&files).cloned() else {
      none += 1;
      continue;
    };
    let line = img_line(&picture, &config.picture_width);
    let Some(text) = read_exact(&about)? else {
      continue;
    };
    if text.split('\n').any(|l| l == line) {
      same += 1;
      continue;
    }
    let update = text.split('\n').any(is_picture_line);
    if update {
      updated += 1;
    } else {
      added += 1;
    }
    if dry {
      println!(
        "would {}: {}",
        if update { "update" } else { "add" },
        rel(&about)
      );
      continue;
    }
    crate::fs::write_atomic(&about, with_picture(&text, &line).as_bytes())?;
  }
  println!(
    "pplr pictures{}: {added} added, {updated} updated, {same} already there, {none} with no picture",
    if dry { " (dry run)" } else { "" }
  );
  Ok(ExitCode::SUCCESS)
}

/// Every two-deep folder named "*, *", by path (`find -mindepth 2 -maxdepth 2`).
fn everyone(root: &Path) -> Vec<PathBuf> {
  let mut out: Vec<PathBuf> = names(root)
    .into_iter()
    .flat_map(|top| {
      names(&root.join(&top))
        .into_iter()
        .filter(|n| n.contains(", "))
        .map(move |n| root.join(&top).join(n))
    })
    .filter(|d| fs::symlink_metadata(d).is_ok_and(|m| m.is_dir()))
    .collect();
  out.sort_by_key(|p| p.to_string_lossy().into_owned());
  out
}

pub fn img_line(picture: &str, width: &str) -> String {
  let alt = picture
    .rsplit_once(" (Picture).")
    .map_or(picture, |(name, _)| name);
  format!("<img src=\"{picture}\" alt=\"{alt}\" width=\"{width}\" align=\"right\">")
}

/// `^<img src="[^"]*\(Picture\)\.[a-z]+"`.
fn is_picture_line(line: &str) -> bool {
  let Some(rest) = line.strip_prefix("<img src=\"") else {
    return false;
  };
  let Some(close) = rest.find('"') else {
    return false;
  };
  let src = &rest[..close];
  src.rfind("(Picture).").is_some_and(|at| {
    let ext = &src[at + "(Picture).".len()..];
    !ext.is_empty() && ext.bytes().all(|b| b.is_ascii_lowercase())
  })
}

/// The text with its first picture line replaced, or with one added after a
/// blank line under the first `# ` title.
pub fn with_picture(text: &str, line: &str) -> String {
  let mut lines: Vec<&str> = text.split('\n').collect();
  if let Some(at) = lines.iter().position(|l| is_picture_line(l)) {
    lines[at] = line;
    return lines.join("\n");
  }
  if let Some(at) = lines
    .iter()
    .position(|l| l.starts_with("# "))
    .filter(|&at| at + 1 < lines.len())
  {
    lines.splice(at + 1..at + 1, ["", line]);
  }
  lines.join("\n")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_picture_goes_under_the_title_or_replaces_the_old_one() {
    let line = img_line("Ada Lovelace (Picture).jpg", "160");
    assert_eq!(
      line,
      "<img src=\"Ada Lovelace (Picture).jpg\" alt=\"Ada Lovelace\" width=\"160\" align=\"right\">"
    );
    let added = with_picture("---\nv\n---\n# Ada Lovelace (About)\n\n_Ada_\n", &line);
    assert_eq!(
      added,
      format!("---\nv\n---\n# Ada Lovelace (About)\n\n{line}\n\n_Ada_\n")
    );
    let old = "# A\n\n<img src=\"A (Picture).png\" alt=\"A\" width=\"90\" align=\"right\">\n";
    assert_eq!(with_picture(old, &line), format!("# A\n\n{line}\n"));
  }
}
