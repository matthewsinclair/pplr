//! External tools pplr runs, found as the shell finds them.

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
