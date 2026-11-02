//! The install layout (ST0001 design 3.4, ST0002 install-release.md section 2):
//! where the running binary's tree is, and what sits beside it.

use crate::Result;
use std::path::{Path, PathBuf};

/// The Contacts sidecar's file name; it always sits beside `pplr`.
const SIDECAR: &str = "pplr-contacts";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
  /// The directory holding `pplr`: `<root>/bin`, or `target/<profile>` in dev.
  pub bin_dir: PathBuf,
  /// The tree's root: the parent of `bin/`, or the checkout in dev.
  pub root: PathBuf,
}

impl Tree {
  /// The tree of the running binary, from its resolved path.
  pub fn running() -> Result<Self> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot find the running pplr: {e}"))?;
    let exe = exe
      .canonicalize()
      .map_err(|e| format!("cannot resolve {}: {e}", exe.display()))?;
    Self::from_exe(&exe)
      .ok_or_else(|| format!("{} has no directory above it", exe.display()).into())
  }

  /// The tree a binary at `exe` belongs to. An installed tree keeps it in
  /// `<root>/bin/`; a dev build keeps it in `<checkout>/target/<profile>/`.
  pub fn from_exe(exe: &Path) -> Option<Self> {
    let bin_dir = exe.parent()?.to_path_buf();
    let in_target = bin_dir
      .parent()
      .filter(|up| up.file_name().is_some_and(|n| n == "target"));
    let root = match in_target {
      Some(target) => target.parent()?.to_path_buf(),
      None if bin_dir.file_name().is_some_and(|n| n == "bin") => bin_dir.parent()?.to_path_buf(),
      None => bin_dir.clone(),
    };
    Some(Self { bin_dir, root })
  }

  /// The Contacts sidecar, beside the binary in every tree.
  pub fn sidecar(&self) -> PathBuf {
    self.bin_dir.join(SIDECAR)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tree(exe: &str) -> Option<Tree> {
    Tree::from_exe(Path::new(exe))
  }

  #[test]
  fn an_installed_tree_is_the_parent_of_bin() {
    let brew = tree("/opt/homebrew/opt/pplr/libexec/bin/pplr");
    assert_eq!(
      brew.as_ref().map(|t| t.root.as_path()),
      Some(Path::new("/opt/homebrew/opt/pplr/libexec"))
    );
    assert_eq!(
      brew.map(|t| t.sidecar()),
      Some(PathBuf::from(
        "/opt/homebrew/opt/pplr/libexec/bin/pplr-contacts"
      ))
    );
  }

  #[test]
  fn a_dev_build_belongs_to_the_checkout_above_target() {
    let dev = tree("/src/Pplr/target/debug/pplr");
    assert_eq!(
      dev.as_ref().map(|t| t.root.as_path()),
      Some(Path::new("/src/Pplr"))
    );
    assert_eq!(
      dev.map(|t| t.sidecar()),
      Some(PathBuf::from("/src/Pplr/target/debug/pplr-contacts"))
    );
  }

  #[test]
  fn a_loose_binary_is_its_own_tree() {
    let loose = tree("/tmp/x/pplr");
    assert_eq!(loose.map(|t| t.root), Some(PathBuf::from("/tmp/x")));
    assert_eq!(tree("/"), None);
  }
}
