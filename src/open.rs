//! The commands that hand a person's file to something else: `about`, `cp`,
//! `open`, `linkedin` and `edit`, and the CMS probe `open` makes.

use crate::Result;
use crate::config::Config;
use crate::marker::target_path;
use crate::normalise::percent_encode;
use crate::tree::about_file;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{Command, ExitCode};
use std::time::Duration;

/// How long the CMS has to answer, as `curl -m 1` allowed.
const CMS_TIMEOUT: Duration = Duration::from_secs(1);

/// The suffix of each kind of file `cp` and `open` name.
fn suffix(kind: &str) -> Option<&'static str> {
  match kind {
    "about" | "a" => Some("(About).md"),
    "linkedin" | "l" => Some("(LinkedIn).webloc"),
    "profile" | "p" => Some("(Profile).pdf"),
    _ => None,
  }
}

/// A two-argument command's own usage, as each legacy script printed it.
fn two_names_usage(script: &str) -> ExitCode {
  println!("Usage: {script} <FIRSTNAME> <SURNAME>");
  ExitCode::FAILURE
}

/// `pplr about <first> <surname>`: the About file, printed.
pub fn run_about(args: &[String], config: &Config) -> Result<ExitCode> {
  let [first, surname, ..] = args else {
    return Ok(two_names_usage("pplr_about"));
  };
  if args.len() > 3 {
    return Ok(two_names_usage("pplr_about"));
  }
  let file = about_file(&config.data, first, surname, "(About).md");
  let text = std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
  std::io::stdout().write_all(&text)?;
  Ok(ExitCode::SUCCESS)
}

/// `-t KIND` and `-o` (with `allow_o`), then exactly two names, as getopts
/// read them; None prints the usage.
fn options(args: &[String], allow_o: bool) -> Option<(&'static str, bool, &str, &str)> {
  let (mut kind, mut to_stdout, mut i) = ("(About).md", false, 0);
  while let Some(arg) = args.get(i).filter(|a| a.starts_with('-') && a.len() > 1) {
    if arg == "--" {
      i += 1;
      break;
    }
    let mut flags = arg[1..].chars();
    while let Some(flag) = flags.next() {
      match flag {
        't' => {
          let rest: String = flags.by_ref().collect();
          let value = if rest.is_empty() {
            i += 1;
            args.get(i)?.clone()
          } else {
            rest
          };
          kind = suffix(&value)?;
        }
        'o' if allow_o => to_stdout = true,
        _ => return None,
      }
    }
    i += 1;
  }
  match &args[i..] {
    [first, surname] => Some((kind, to_stdout, first.as_str(), surname.as_str())),
    _ => None,
  }
}

/// `pplr cp [-t about|linkedin|profile] [-o] <first> <surname>`: the file's
/// path, quoted, on the clipboard (or with -o on stdout).
pub fn run_cp(args: &[String], config: &Config) -> Result<ExitCode> {
  let Some((kind, to_stdout, first, surname)) = options(args, true) else {
    eprintln!("Usage: pplr_cp [-t about|linkedin|profile] [-o] <FIRSTNAME> <SURNAME>");
    return Ok(ExitCode::FAILURE);
  };
  let file = about_file(&config.data, first, surname, kind);
  if !file.is_file() {
    eprintln!("Error: File not found: {}", file.display());
    return Ok(ExitCode::FAILURE);
  }
  let quoted = format!("\"{}\"", file.display());
  if to_stdout {
    println!("{quoted}");
    return Ok(ExitCode::SUCCESS);
  }
  macos_only("pplr cp (without -o) copies to the macOS clipboard with pbcopy")?;
  let mut copy = Command::new("pbcopy")
    .stdin(std::process::Stdio::piped())
    .spawn()?;
  if let Some(stdin) = copy.stdin.as_mut() {
    stdin.write_all(quoted.as_bytes())?;
  }
  exit_of(copy.wait()?)
}

/// `pplr open [-t about|linkedin|profile] <first> <surname>`, or
/// `pplr open [--print] pplr://...`: the CMS page when the CMS answers, else
/// the file; --print says where it would go.
pub fn run_open(args: &[String], config: &Config) -> Result<ExitCode> {
  let first = args.first().map(String::as_str);
  if first == Some("--print") || first.is_some_and(|a| a.starts_with("pplr://")) {
    let print_only = first == Some("--print");
    let url = args
      .get(usize::from(print_only))
      .map(String::as_str)
      .unwrap_or_default();
    let Some(path) = target_path(url, config)? else {
      eprintln!("pplr: no one answers to {url}");
      return Ok(ExitCode::FAILURE);
    };
    let target = match cms_page(&path, config) {
      Some(page) => page,
      None => path.display().to_string(),
    };
    if print_only {
      println!("{target}");
      return Ok(ExitCode::SUCCESS);
    }
    return open_target(&target);
  }
  let Some((kind, _, first, surname)) = options(args, false) else {
    eprintln!("Usage: pplr_open [-t about|linkedin|profile] <FIRSTNAME> <SURNAME>");
    return Ok(ExitCode::FAILURE);
  };
  let file = about_file(&config.data, first, surname, kind);
  if !file.is_file() {
    eprintln!("Error: File not found: {}", file.display());
    return Ok(ExitCode::FAILURE);
  }
  open_target(&file.display().to_string())
}

/// `pplr linkedin <first> <surname>`: the person's LinkedIn .webloc, opened.
pub fn run_linkedin(args: &[String], config: &Config) -> Result<ExitCode> {
  let [first, surname, ..] = args else {
    return Ok(two_names_usage("pplr_linkedin"));
  };
  if args.len() > 3 {
    return Ok(two_names_usage("pplr_linkedin"));
  }
  let webloc = about_file(&config.data, first, surname, "(LinkedIn).webloc");
  open_target(&webloc.display().to_string())
}

/// `pplr edit <first> <surname>`: the About file in $VISUAL, else $EDITOR,
/// in the foreground (design D10).
pub fn run_edit(args: &[String], config: &Config) -> Result<ExitCode> {
  let [first, surname, ..] = args else {
    return Ok(two_names_usage("pplr_edit"));
  };
  if args.len() > 3 {
    return Ok(two_names_usage("pplr_edit"));
  }
  let set = |name: &str| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
  let Some(editor) = set("VISUAL").or_else(|| set("EDITOR")) else {
    return Err("neither VISUAL nor EDITOR names an editor, so pplr edit has none to run".into());
  };
  let about = about_file(&config.data, first, surname, "(About).md");
  // An editor setting may carry arguments ("code --wait"), as a shell reads it
  let mut words = editor.split_whitespace();
  let program = words.next().unwrap_or_default();
  exit_of(Command::new(program).args(words).arg(&about).status()?)
}

/// The CMS page for a path in the tree, when the CMS answers: its URL plus
/// the path inside the tree, percent-encoded as Python's urllib.parse.quote.
fn cms_page(path: &Path, config: &Config) -> Option<String> {
  if config.cms_url.is_empty() || !answers(&config.cms_url) {
    return None;
  }
  let real = config.data.canonicalize().ok();
  let rel = real
    .as_deref()
    .and_then(|r| path.strip_prefix(r).ok())
    .or_else(|| path.strip_prefix(&config.data).ok())
    .unwrap_or(path);
  Some(format!(
    "{}/{}",
    config.cms_url,
    percent_encode(&rel.to_string_lossy(), b"/")
  ))
}

/// Whether anything answers HTTP at the CMS's status page within the
/// timeout: `curl -s -o /dev/null <base>/_status`, which any reply satisfies.
fn answers(cms_url: &str) -> bool {
  let base = cms_url.rsplit_once('/').map_or(cms_url, |(b, _)| b);
  let Some(rest) = base.strip_prefix("http://") else {
    return false;
  };
  let (host_port, path) = rest.split_once('/').unwrap_or((rest, ""));
  let path = if path.is_empty() {
    "/_status".to_owned()
  } else {
    format!("/{path}/_status")
  };
  let address = if host_port.contains(':') {
    host_port.to_owned()
  } else {
    format!("{host_port}:80")
  };
  let Some(addr) = address.to_socket_addrs().ok().and_then(|mut a| a.next()) else {
    return false;
  };
  let Ok(mut stream) = TcpStream::connect_timeout(&addr, CMS_TIMEOUT) else {
    return false;
  };
  let _ = stream.set_read_timeout(Some(CMS_TIMEOUT));
  let _ = stream.set_write_timeout(Some(CMS_TIMEOUT));
  let request = format!("GET {path} HTTP/1.0\r\nHost: {host_port}\r\nConnection: close\r\n\r\n");
  let mut first = [0_u8; 5];
  stream.write_all(request.as_bytes()).is_ok()
    && stream.read_exact(&mut first).is_ok()
    && &first == b"HTTP/"
}

/// Hands a file or URL to macOS's `open`.
fn open_target(target: &str) -> Result<ExitCode> {
  macos_only("pplr open hands files and URLs to macOS's open")?;
  exit_of(Command::new("open").arg(target).status()?)
}

/// Refuses, naming why, where the macOS service a command needs is absent (design D9).
fn macos_only(why: &str) -> Result<()> {
  if cfg!(target_os = "macos") {
    Ok(())
  } else {
    Err(format!("{why}, which this system does not have").into())
  }
}

fn exit_of(status: std::process::ExitStatus) -> Result<ExitCode> {
  Ok(match status.code() {
    Some(0) => ExitCode::SUCCESS,
    Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
    None => ExitCode::FAILURE,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(|a| (*a).to_owned()).collect()
  }

  #[test]
  fn options_read_like_getopts() {
    let read = |args: &[&str], o| {
      options(&strings(args), o).map(|(k, s, f, l)| (k, s, f.to_owned(), l.to_owned()))
    };
    assert_eq!(
      read(&["-t", "l", "-o", "Jon", "Kemp"], true),
      Some((
        "(LinkedIn).webloc",
        true,
        "Jon".to_owned(),
        "Kemp".to_owned()
      ))
    );
    assert_eq!(
      read(&["-tprofile", "Jon", "Kemp"], false),
      Some(("(Profile).pdf", false, "Jon".to_owned(), "Kemp".to_owned()))
    );
    assert_eq!(read(&["-o", "Jon", "Kemp"], false), None);
    assert_eq!(read(&["-t", "nope", "Jon", "Kemp"], true), None);
    assert_eq!(read(&["Jon"], true), None);
  }

  #[test]
  fn nothing_answers_on_a_closed_port() {
    assert!(!answers("http://127.0.0.1:9/people"));
    assert!(!answers("https://example.invalid/people"));
  }
}
