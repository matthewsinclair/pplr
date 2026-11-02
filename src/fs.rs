//! File writes, always through a temporary file beside the target, so a
//! reader never sees half a file.

use crate::Result;
use std::path::Path;

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
  let name = path
    .file_name()
    .ok_or_else(|| format!("{} names no file", path.display()))?;
  let temp = path.with_file_name(format!(
    ".{}.pplr-{}",
    name.to_string_lossy(),
    std::process::id()
  ));
  std::fs::write(&temp, bytes).map_err(|e| format!("cannot write {}: {e}", temp.display()))?;
  std::fs::rename(&temp, path).map_err(|e| {
    let _ = std::fs::remove_file(&temp);
    format!("cannot replace {}: {e}", path.display()).into()
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_write_replaces_the_file_and_leaves_no_temp_behind() -> Result<()> {
    let dir = std::env::temp_dir().join(format!("pplr-fs-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let file = dir.join("contacts.json");
    std::fs::write(&file, "old")?;
    write_atomic(&file, b"new")?;
    assert_eq!(std::fs::read_to_string(&file)?, "new");
    let left: Vec<_> = std::fs::read_dir(&dir)?.collect::<std::io::Result<_>>()?;
    assert_eq!(left.len(), 1);
    std::fs::remove_dir_all(&dir)?;
    Ok(())
  }
}
