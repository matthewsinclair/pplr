//! The command table, and dispatch: a ported command runs natively, and every
//! other command goes through the legacy fallback until its own WP.

use crate::Result;
use crate::config::Config;
use crate::{
  contact, doctor, email, fallback, grep, handler, icons, index, install, links, marker, meetings,
  new, open, pictures, refresh, rename, search, sync, tags,
};
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
    name: "contact",
    usage: "when you last spoke to each person (About/contact.yaml), and who to get in touch with next\nscan [--dry-run]          last contact from each person's newest meeting; next from the cadence\ndue [DAYS] [--all]        who is due for a note (next on or before today, plus DAYS)\nshow \"Surname, First\"     a person's last contact, next date and cadence\nlog [\"Surname, First\"] [DATE] [--via email|call|message|meeting] [--email N|MESSAGE-ID] [--link URL] [--note TEXT]\nnext \"Surname, First\" WHEN  set the next contact: a date, or +2w, +10d, +3m\ncontext \"Surname, First\"  what to write a note from: role, latest update, last meeting's notes\nrender                   About/<First Surname> (Contact).md, a read-only view of contact.yaml, for everyone",
    run: contact::run,
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
    name: "email",
    usage: email::HELP,
    run: email::run,
  },
  Command {
    name: "grep",
    usage: "Text search through people files or tags\n[options] <text> text to search for\n-t, --tag    search in tag files instead of markdown files",
    run: grep::run,
  },
  Command {
    name: "handler",
    usage: "register this tree's app for pplr:// links (macOS)\n--install    register it with Launch Services, by the tree's stable path\n--uninstall  unregister it\n--status     the app, and which app macOS uses for pplr://",
    run: handler::run,
  },
  Command {
    name: "help",
    usage: "provide help for a specific command (if provided) or all commands\n[cmd] specific command to get help for\n--details show comprehensive documentation",
    run: run_help,
  },
  Command {
    name: "index",
    usage: "generate index.md based on contents of $PPLR_DIR",
    run: index::run_index,
  },
  Command {
    name: "install",
    usage: "publish this dev build as an opt tree (maintainer)\n[--prefix <dir>]  the opt tree, else $PPLR_OPT_PREFIX\n[--force]         replace an opt tree pplr wrote\n[--managed-by brew]  for the Homebrew formula",
    run: install::install,
  },
  Command {
    name: "json",
    usage: "generate index.json based on contents of $PPLR_DIR",
    run: index::run_json,
  },
  Command {
    name: "linkedin",
    usage: "open a person's LinkedIn URL (if it exists)\n<first>        firstname of person to edit\n<surname>      surname of person to edit",
    run: open::run_linkedin,
  },
  Command {
    name: "links",
    usage: "rewrite Markdown links into the people tree as pplr:// URLs\n<file or dir>...  the Markdown to convert (directories are searched for .md)\n[--apply]         write the changes (a dry run otherwise)\n[--json]          every link found, converted or not, as JSON",
    run: links::run,
  },
  Command {
    name: "meetings",
    usage: "Find meetings within a date range and display just the filenames\n<date>  Date in YYYY, YYYYMM, or YYYYMMDD format\n<end_date>  Optional end date in YYYY, YYYYMM, or YYYYMMDD format (if omitted, only <date> is used)",
    run: meetings::run,
  },
  Command {
    name: "new",
    usage: "create a new person in $PPLR_DIR\n<first>        firstname of new person\n<surname>      surname of new person\n[linkedin_url] LinkedIn URL of the new person",
    run: new::run,
  },
  Command {
    name: "open",
    usage: "Open the location of a person's file, or a pplr:// URL\n[pplr://...]   a pplr:// URL instead of a name (--print to show where it goes)\n<first>        firstname of person to edit\n<surname>      surname of person to edit\n[--type | -t]  type of file (about | a, linkedin | l, profile | p)",
    run: open::run_open,
  },
  Command {
    name: "pictures",
    usage: "show each person's photo at the top of their About page\n[--dry-run]         list what would change\n[\"Surname, First\"]  just these people (default everyone)",
    run: pictures::run,
  },
  Command {
    name: "refresh",
    usage: "track when each person was last checked against their public profile\nnext [N] [--older-than DAYS]   the next N people to check (never checked first, then oldest)\nstamp \"Surname, First\" [SOURCE] record a check today (SOURCE eg linkedin; default linkedin)\nstatus                          how many are checked, and how recently",
    run: refresh::run,
  },
  Command {
    name: "reindex",
    usage: "(re)create the indexes in $PPLR_DIR/.index, the tag pages and each About's Tags line",
    run: index::run_reindex,
  },
  Command {
    name: "rename",
    usage: "rename a person: the folder, their files, and every path-style link to them\n<\"Old, First\">   the current folder name (\"Surname, First\", or \"L/Surname, First\")\n<\"New, First\">   the new folder name\n[--dry-run]      list what would change, and change nothing\n[--refs DIR]     where to repoint links (repeatable; default $PPLR_REFS_DIR, or ~/Dropbox)",
    run: rename::run,
  },
  Command {
    name: "resolve",
    usage: "print the file or folder a pplr:// URL names\n<pplr://l/surname-first[/path]>   a person (their About file), or a path in their folder",
    run: marker::run_resolve,
  },
  Command {
    name: "search",
    usage: "Natural language search across people using Claude AI\n<query> natural language query to search for\nUses tags_index.json for fast, intelligent search",
    run: search::run,
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
    name: "sync",
    usage: "Compare pplr with Apple Contacts, and link matched cards to pplr\n--check          report linked, matched, missing and differing people\n--link           mark matched cards as pplr's (a dry run without --apply)\n--plan           write a review workbook: pplr, Contacts, and a proposed merge\n--apply-plan FILE  write the reviewed plan workbook to Contacts (a dry run without --apply)\n--photos         give linked cards with no photo their pplr picture (a dry run without --apply)\n--pull           copy into pplr what linked cards know and pplr lacks (a dry run without --apply)\n--backup         save every Contacts card to a dated .vcf in $PPLR_BACKUP_DIR",
    run: sync::run,
  },
  Command {
    name: "tag",
    usage: "show or edit one person's tags (About/tags.yaml), checked against the vocabulary\n<\"Surname, First\">            show their tags, by facet\n<\"Surname, First\"> +tag -tag   add or remove tags (an added tag is yours: never overwritten)",
    run: tags::run_tag,
  },
  Command {
    name: "tags",
    usage: "each person's tags, from About/tags.yaml, against _pplr/vocabulary.yaml\ncheck                 every tags.yaml against the vocabulary\nshow \"Surname, First\" a person's tags, by facet\nrender [--dry-run]    the generated \"- Tags:\" line in each About\nindex                 .index/tags_index.json: everyone, with marker, role, company, picture, tags\napply FILE [--dry-run] write tags.yaml from a reviewed JSON list (hand tags kept)",
    run: tags::run_tags,
  },
  Command {
    name: "use",
    usage: "point ~/.local/bin/pplr at a tree, or report the one it serves\n[dev|opt|brew]  the tree; brew removes the link so Homebrew's pplr answers",
    run: install::use_tree,
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

/// The commands that run where there is no People tree: those about pplr
/// itself (ST0002 WP03), and doctor, which reports a missing tree as a row.
const WITHOUT_DATA: &[&str] = &["doctor", "help", "install", "use", "version"];

pub fn run(args: Vec<String>) -> Result<ExitCode> {
  let config = Config::from_env()?;
  if matches!(args.first().map(String::as_str), Some("-v" | "--version")) {
    return run_version(&[], &config);
  }
  let Some(command) = args.first().and_then(|name| native(name)) else {
    return fallback::run(&args, &config);
  };
  if !WITHOUT_DATA.contains(&command.name)
    && let Some(code) = data_missing(&config)
  {
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

/// `pplr version`, `-v`, `--version`: `pplr <version> (<tree>, <sha12>[+dirty])`, the
/// version from Cargo.toml, the one source (design D7), and the tree from its manifest (T4).
fn run_version(_args: &[String], _config: &Config) -> Result<ExitCode> {
  println!("{}", install::version_line());
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
