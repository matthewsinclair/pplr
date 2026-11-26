pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;

/// Never let a path or a setting inject terminal control sequences.
pub fn clean(text: &str) -> String {
  text.chars().filter(|c| !c.is_control()).collect()
}

/// A refusal, or a warning a command gives before it carries on, in devbin's
/// shape: `pplr: <message>` on stderr, as `main` prints an error (pplr#0002,
/// hv's ruling A of 2026-11-26). The caller gives the exit code.
pub fn complain(message: &str) {
  eprintln!("pplr: {}", clean(message));
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn clean_drops_control_characters_and_keeps_the_rest() {
    assert_eq!(clean("Kemp\u{1b}[31m, Jon\n"), "Kemp[31m, Jon");
    assert_eq!(clean("Kemp, Jon"), "Kemp, Jon");
  }
}
