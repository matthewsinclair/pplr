//! File reads, each one a file or none and any other failure named, and file
//! writes, always through a temporary file beside the target, so a reader
//! never sees half a file.

use crate::Result;
use std::path::Path;

/// A file's bytes, or None when there is no such file. Any other failure to
/// read it is the error, naming the file, for the command to report.
pub fn read(path: &Path) -> Result<Option<Vec<u8>>> {
  match std::fs::read(path) {
    Ok(bytes) => Ok(Some(bytes)),
    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
    Err(e) => Err(format!("cannot read {}: {e}", path.display()).into()),
  }
}

/// A file as text for reading, as the legacy's byte-wise reads took it:
/// what is not UTF-8 is decoded lossily.
pub fn read_text(path: &Path) -> Result<Option<String>> {
  Ok(read(path)?.map(|bytes| {
    String::from_utf8(bytes).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
  }))
}

/// A file as text to rewrite: one that is not UTF-8 is the error, naming it,
/// as a rewrite would lose the bytes it could not decode.
pub fn read_exact(path: &Path) -> Result<Option<String>> {
  read(path)?
    .map(|bytes| {
      String::from_utf8(bytes)
        .map_err(|_| format!("{} is not UTF-8, so it is left as it is", path.display()).into())
    })
    .transpose()
}

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

  #[test]
  fn a_read_is_the_text_or_no_file_and_a_rewrite_refuses_what_it_cannot_decode() -> Result<()> {
    let dir = std::env::temp_dir().join(format!("pplr-fs-read-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let latin = dir.join("latin.md");
    std::fs::write(&latin, b"caf\xe9\n")?;
    assert_eq!(read_text(&latin)?, Some("caf\u{fffd}\n".to_owned()));
    let refused = read_exact(&latin).map_err(|e| e.to_string());
    assert_eq!(
      refused,
      Err(format!(
        "{} is not UTF-8, so it is left as it is",
        latin.display()
      ))
    );
    let missing = dir.join("missing.md");
    assert_eq!(read_text(&missing)?, None);
    assert_eq!(read_exact(&missing)?, None);
    let folder = read(&dir).map_err(|e| e.to_string());
    assert!(
      folder
        .as_ref()
        .is_err_and(|e| e.starts_with(&format!("cannot read {}: ", dir.display()))),
      "{folder:?}"
    );
    std::fs::remove_dir_all(&dir)?;
    Ok(())
  }
}
