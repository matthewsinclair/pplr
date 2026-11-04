//! `pplr handler --install | --uninstall | --status` (ST0001 WP08): the
//! pplr:// handler app ships prebuilt and signed in the tree (hv's ruling,
//! ST0002 install-release.md section 4), so install only registers it with
//! Launch Services, by the tree's stable path: under brew the opt link, never a
//! Cellar path, which `brew upgrade` deletes.

use crate::Result;
use crate::config::Config;
use crate::layout::Layout;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "Usage: pplr handler --install | --uninstall | --status
  A small app, registered for pplr:// URLs, that runs 'pplr open <url>': the
  person's CMS page when the CMS is up, else their file. Links then open from
  Obsidian, Contacts, Mail and the browser.";

const LSREGISTER: &str = "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister";

pub fn run(args: &[String], _config: &Config) -> Result<ExitCode> {
  let mode = args.first().map(String::as_str);
  if matches!(mode, None | Some("-h" | "--help")) {
    println!("{USAGE}");
    return Ok(if mode.is_some() {
      ExitCode::SUCCESS
    } else {
      ExitCode::FAILURE
    });
  }
  crate::tools::macos_only("the pplr:// handler is a macOS app, so it needs macOS")?;
  let app = stable(&Layout::running()?.app());
  match mode {
    Some("--install") => {
      if !app.is_dir() {
        return Err(
          format!(
            "no pplr:// handler app at {}: build it with bin/devbin build",
            app.display()
          )
          .into(),
        );
      }
      lsregister(&["-f"], &app)?;
      println!(
        "Installed: {} (opens pplr:// with the pplr in its own tree)",
        app.display()
      );
    }
    Some("--uninstall") => {
      if app.is_dir() {
        lsregister(&["-u"], &app)?;
      }
      println!("Removed: {}", app.display());
    }
    Some("--status") => {
      println!(
        "{}: {}",
        if app.is_dir() {
          "This tree's app"
        } else {
          "No app in this tree at"
        },
        app.display()
      );
      let dump = Command::new(LSREGISTER)
        .args(["-dump", "URLSchemeBinding"])
        .output()?;
      let text = String::from_utf8_lossy(&dump.stdout);
      let lines: Vec<&str> = text.lines().collect();
      let bound = lines
        .iter()
        .position(|l| l.to_lowercase().contains("pplr:"));
      for line in bound.map_or(&[][..], |i| &lines[i..(i + 2).min(lines.len())]) {
        println!("{line}");
      }
    }
    _ => {
      println!("{USAGE}");
      return Ok(ExitCode::FAILURE);
    }
  }
  Ok(ExitCode::SUCCESS)
}

fn lsregister(flags: &[&str], app: &Path) -> Result<()> {
  let status = Command::new(LSREGISTER).args(flags).arg(app).status()?;
  if status.success() {
    Ok(())
  } else {
    Err(format!("lsregister {} {} failed", flags.join(" "), app.display()).into())
  }
}

/// A path through a keg, `<prefix>/Cellar/pplr/<version>/...`, named by its
/// stable opt link, `<prefix>/opt/pplr/...`; any other path as it is.
pub fn stable(path: &Path) -> PathBuf {
  let parts: Vec<_> = path.components().collect();
  let cellar = parts
    .windows(2)
    .position(|w| w[0].as_os_str() == "Cellar" && w[1].as_os_str() == "pplr");
  match cellar {
    Some(i) if parts.len() > i + 2 => {
      let mut out: PathBuf = parts[..i].iter().collect();
      out.push("opt/pplr");
      out.extend(&parts[i + 3..]);
      out
    }
    _ => path.to_path_buf(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_keg_path_is_named_by_its_opt_link() {
    assert_eq!(
      stable(Path::new(
        "/opt/homebrew/Cellar/pplr/2.0.0/libexec/Pplr Links.app"
      )),
      PathBuf::from("/opt/homebrew/opt/pplr/libexec/Pplr Links.app")
    );
    assert_eq!(
      stable(Path::new("/src/Pplr/target/debug/Pplr Links.app")),
      PathBuf::from("/src/Pplr/target/debug/Pplr Links.app")
    );
  }
}
