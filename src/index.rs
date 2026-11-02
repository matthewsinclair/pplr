//! The tree's indexes (ST0001 design 3.1); WP04 ports `count`, WP05 the rest.

use crate::Result;
use crate::config::Config;
use crate::tree::{is_letter, names};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// `pplr count`: the people in `.index/index.md` when there is one (each line
/// holding a `[...]`), else each "Surname, First" folder under a letter.
pub fn run_count(_args: &[String], config: &Config) -> Result<ExitCode> {
  println!("{}", count(&config.data));
  Ok(ExitCode::SUCCESS)
}

pub fn count(root: &Path) -> usize {
  if let Ok(index) = fs::read_to_string(root.join(".index/index.md")) {
    // grep -o '\[.*\]\(.*\)': in a basic regular expression \( \) only groups,
    // so a line counts when a [ has a ] after it
    return index
      .lines()
      .filter(|line| {
        line
          .find('[')
          .is_some_and(|open| line[open..].contains(']'))
      })
      .count();
  }
  names(root)
    .into_iter()
    .filter(|letter| is_letter(letter) && root.join(letter).is_dir())
    .map(|letter| {
      names(&root.join(&letter))
        .into_iter()
        .filter(|name| name.contains(", ") && root.join(&letter).join(name).is_dir())
        .count()
    })
    .sum()
}
