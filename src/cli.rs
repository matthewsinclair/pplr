//! The command table, and dispatch: a ported command runs natively, and every
//! other command goes through the legacy fallback until its own WP.

use crate::Result;
use crate::config::Config;
use crate::fallback;
use crate::{doctor, icons};
use std::process::ExitCode;

pub struct Command {
  pub name: &'static str,
  /// One line of usage, for `help` once it is ported (WP04).
  pub usage: &'static str,
  pub run: fn(&[String], &Config) -> Result<ExitCode>,
}

/// The commands ported to Rust. Each porting WP adds its rows here, and the
/// golden replay skips every command not in this table.
pub const COMMANDS: &[Command] = &[
  Command {
    name: "applyicons",
    usage: "applyicons <first> <surname>   Finder icons on the About and Meetings folders",
    run: icons::applyicons,
  },
  Command {
    name: "doctor",
    usage: "doctor [--fix]                 Check the data tree, its indexes, the settings and the tools",
    run: doctor::run,
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
];

/// The native command called `name`, if it has been ported.
pub fn native(name: &str) -> Option<&'static Command> {
  COMMANDS.iter().find(|command| command.name == name)
}

pub fn run(args: Vec<String>) -> Result<ExitCode> {
  let config = Config::from_env()?;
  match args.first().and_then(|name| native(name)) {
    Some(command) => (command.run)(&args[1..], &config),
    None => fallback::run(&args, &config),
  }
}
