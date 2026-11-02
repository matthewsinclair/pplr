//! The command table, and dispatch: a ported command runs natively, and every
//! other command goes through the legacy fallback until its own WP.

use crate::Result;
use crate::config::Config;
use crate::{doctor, fallback, grep, icons, index, marker, meetings, open, refresh};
use std::io::{IsTerminal, Write};
use std::process::{Command as Process, ExitCode, Stdio};

pub struct Command {
  pub name: &'static str,
  /// What `help` prints for the command: its first line beside the name, and
  /// any further lines under it.
  pub usage: &'static str,
  pub run: fn(&[String], &Config) -> Result<ExitCode>,
}

/// The commands ported to Rust, by name. Each porting WP adds its rows here,
/// and the golden replay skips every command not in this table.
pub const COMMANDS: &[Command] = &[
  Command {
    name: "about",
    usage: "display a person's about file\n<first>        firstname of person to edit\n<surname>      surname of person to edit",
    run: open::run_about,
  },
  Command {
    name: "applyicons",
    usage: "applyicons <first> <surname>   Finder icons on the About and Meetings folders",
    run: icons::applyicons,
  },
  Command {
    name: "count",
    usage: "counts the number of people in the pplr db",
    run: index::run_count,
  },
  Command {
    name: "cp",
    usage: "Copy or output the location of a person's file to the clipboard or stdout\n<first>        firstname of person to edit\n<surname>      surname of person to edit\n[--type | -t]  type of file (about | a, linkedin | l, profile | p)\n[-o]           output the result to stdout instead of copying to clipboard",
    run: open::run_cp,
  },
  Command {
    name: "doctor",
    usage: "doctor [--fix]                 Check the data tree, its indexes, the settings and the tools",
    run: doctor::run,
  },
  Command {
    name: "edit",
    usage: "edit a person's about file in $VISUAL, else $EDITOR\n<first>        firstname of person to edit\n<surname>      surname of person to edit",
    run: open::run_edit,
  },
  Command {
    name: "grep",
    usage: "Text search through people files or tags\n[options] <text> text to search for\n-t, --tag    search in tag files instead of markdown files",
    run: grep::run,
  },
  Command {
    name: "help",
    usage: "provide help for a specific command (if provided) or all commands\n[cmd] specific command to get help for\n--details show comprehensive documentation",
    run: run_help,
  },
  Command {
    name: "linkedin",
    usage: "open a person's LinkedIn URL (if it exists)\n<first>        firstname of person to edit\n<surname>      surname of person to edit",
    run: open::run_linkedin,
  },
  Command {
    name: "meetings",
    usage: "Find meetings within a date range and display just the filenames\n<date>  Date in YYYY, YYYYMM, or YYYYMMDD format\n<end_date>  Optional end date in YYYY, YYYYMM, or YYYYMMDD format (if omitted, only <date> is used)",
    run: meetings::run,
  },
  Command {
    name: "open",
    usage: "Open the location of a person's file, or a pplr:// URL\n[pplr://...]   a pplr:// URL instead of a name (--print to show where it goes)\n<first>        firstname of person to edit\n<surname>      surname of person to edit\n[--type | -t]  type of file (about | a, linkedin | l, profile | p)",
    run: open::run_open,
  },
  Command {
    name: "refresh",
    usage: "track when each person was last checked against their public profile\nnext [N] [--older-than DAYS]   the next N people to check (never checked first, then oldest)\nstamp \"Surname, First\" [SOURCE] record a check today (SOURCE eg linkedin; default linkedin)\nstatus                          how many are checked, and how recently",
    run: refresh::run,
  },
  Command {
    name: "resolve",
    usage: "print the file or folder a pplr:// URL names\n<pplr://l/surname-first[/path]>   a person (their About file), or a path in their folder",
    run: marker::run_resolve,
  },
  Command {
    name: "setpicfordir",
    usage: "setpicfordir [-V] <first> <surname>   The person's picture as their folder's icon",
    run: icons::setpicfordir,
  },
  Command {
    name: "setpicsfordirs",
    usage: "setpicsfordirs                 Everyone's picture as their folder's icon",
    run: icons::setpicsfordirs,
  },
  Command {
    name: "version",
    usage: "display the pplr version",
    run: run_version,
  },
];

/// The README, the one source of `help --details` (design D6).
const README: &str = include_str!("../README.md");

/// The native command called `name`, if it has been ported.
pub fn native(name: &str) -> Option<&'static Command> {
  COMMANDS.iter().find(|command| command.name == name)
}

pub fn run(args: Vec<String>) -> Result<ExitCode> {
  let config = Config::from_env()?;
  if matches!(args.first().map(String::as_str), Some("-v" | "--version")) {
    return run_version(&[], &config);
  }
  let Some(command) = args.first().and_then(|name| native(name)) else {
    return fallback::run(&args, &config);
  };
  if let Some(code) = data_missing(&config) {
    return Ok(code);
  }
  let rest = marker::rewrite_args(command.name, &args[1..], &config)?;
  (command.run)(&rest, &config)
}

/// Every command runs from the data root, so a missing one stops it, as `bin/pplr` stopped.
pub fn data_missing(config: &Config) -> Option<ExitCode> {
  (!config.data.is_dir()).then(|| {
    println!(
      "Error: Could not change to directory {}.",
      config.data.display()
    );
    ExitCode::FAILURE
  })
}

/// `pplr version`, `-v`, `--version`: the version in Cargo.toml, the one source (design D7).
fn run_version(_args: &[String], _config: &Config) -> Result<ExitCode> {
  println!("{}", env!("CARGO_PKG_VERSION"));
  Ok(ExitCode::SUCCESS)
}

/// `pplr help [cmd]... | --details`.
fn run_help(args: &[String], _config: &Config) -> Result<ExitCode> {
  if args.first().map(String::as_str) == Some("--details") {
    return page(README);
  }
  let all = commands();
  let width = all
    .iter()
    .map(|(name, _)| name.chars().count())
    .max()
    .unwrap_or(0);
  // A name with no command has no usage lines, which the listing says
  let wanted: Vec<(String, Vec<String>)> = if args.is_empty() {
    all
  } else {
    args
      .iter()
      .map(|name| {
        all
          .iter()
          .find(|(n, _)| n == name)
          .cloned()
          .unwrap_or_else(|| (name.clone(), Vec::new()))
      })
      .collect()
  };
  for (name, lines) in &wanted {
    match lines.split_first() {
      Some((first, rest)) => {
        println!("{name:<width$}: {first}");
        for line in rest {
          println!("{:width$}  {line}", "");
        }
      }
      None => println!("{name:<width$}: No usage information found."),
    }
  }
  if args.is_empty() {
    println!("\nFor comprehensive documentation, use: pplr help --details");
  }
  Ok(ExitCode::SUCCESS)
}

/// Every command, native and still on the fallback, by name, with its usage lines.
fn commands() -> Vec<(String, Vec<String>)> {
  let mut all: Vec<(String, Vec<String>)> = COMMANDS
    .iter()
    .map(|c| {
      (
        c.name.to_owned(),
        c.usage.lines().map(str::to_owned).collect(),
      )
    })
    .collect();
  for (name, lines) in fallback::usages() {
    if native(&name).is_none() {
      all.push((name, lines));
    }
  }
  all.sort();
  all
}

/// Text through `less` at a terminal, else straight to stdout.
fn page(text: &str) -> Result<ExitCode> {
  if std::io::stdout().is_terminal()
    && let Ok(mut less) = Process::new("less").stdin(Stdio::piped()).spawn()
  {
    if let Some(stdin) = less.stdin.as_mut() {
      stdin.write_all(text.as_bytes())?;
    }
    less.wait()?;
    return Ok(ExitCode::SUCCESS);
  }
  std::io::stdout().write_all(text.as_bytes())?;
  Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn the_table_is_sorted_and_names_each_command_once() {
    let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(names, sorted);
  }
}
