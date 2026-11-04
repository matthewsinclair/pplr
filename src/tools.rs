//! External tools pplr runs, found as the shell finds them, and the one gate
//! for the commands whose tool or service only macOS has.

use crate::Result;
use std::path::PathBuf;

/// The executable `name` resolves to on PATH, as `command -v` finds it.
pub fn on_path(name: &str) -> Option<PathBuf> {
  use std::os::unix::fs::PermissionsExt;
  let path = std::env::var_os("PATH")?;
  std::env::split_paths(&path)
    .map(|dir| dir.join(name))
    .find(|candidate| {
      candidate
        .metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    })
}

/// Refuses with `refusal` where the macOS service a command needs is absent
/// (design D9). Every command that needs one passes this gate first, so each
/// refuses the same way: its own message, and exit 1 from main.
pub fn macos_only(refusal: &str) -> Result<()> {
  if cfg!(target_os = "macos") {
    Ok(())
  } else {
    Err(refusal.into())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// AT-03.1 (ST0003 AC-03.1): the one gate lets a command through on macOS,
  /// and elsewhere refuses with exactly the command's own message, which main
  /// prints as its stderr line before it exits 1. CI runs it on both legs, and
  /// AT-00.7 in tests/pplr_core.bats holds each gate's line on Linux.
  #[test]
  fn the_gate_passes_on_macos_and_elsewhere_refuses_with_the_message_alone() {
    let refusal = "the service this test names needs macOS";
    let want = if cfg!(target_os = "macos") {
      Ok(())
    } else {
      Err(refusal.to_owned())
    };
    assert_eq!(macos_only(refusal).map_err(|e| e.to_string()), want);
  }
}
