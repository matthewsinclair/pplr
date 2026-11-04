//! File reads, each one a file or none and any other failure named, file
//! writes, always through a temporary file beside the target, so a reader
//! never sees half a file, and the one walk down a folder's tree.

use crate::Result;
use std::fs::DirEntry;
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

/// The order a walk meets a folder's entries in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
  /// By name, as `tree::names` sorts them.
  Name,
  /// As the folder gives them, the order `find` meets them in.
  Folder,
}

/// Every entry under `dir`, met in `order` and shown to `visit`, which keeps
/// what its caller wants and says whether to enter the entry. The walk enters
/// a real folder alone, never a symlink to one. A folder it cannot read is
/// named on stderr, one line each, and the walk carries on (hv's ruling,
/// CHANGELOG 2.0.1); an entry that fails mid-read is passed over, as 2.0.0's
/// walks passed it.
pub fn walk(dir: &Path, order: Order, visit: &mut dyn FnMut(&DirEntry) -> bool) {
  let mut entries: Vec<DirEntry> = match std::fs::read_dir(dir) {
    Ok(entries) => entries.flatten().collect(),
    Err(e) => {
      eprintln!(
        "pplr: {}",
        crate::clean(&format!("cannot read {}: {e}", dir.display()))
      );
      return;
    }
  };
  if order == Order::Name {
    entries.sort_by_cached_key(|e| e.file_name().to_string_lossy().into_owned());
  }
  for entry in entries {
    if visit(&entry) && entry.file_type().is_ok_and(|t| t.is_dir()) {
      walk(&entry.path(), order, visit);
    }
  }
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

  /// AT-04.1 (ST0003 AC-04.1): grep, links and rename walk through `walk`,
  /// and on one tree each keeps its rules from 2.0.0: grep's name order and
  /// dot names below the top, links' dot rule and symlinked notes, search's
  /// extensions and skip folders, named_after's prefix with a folder before
  /// its contents. None of them enters a symlinked folder.
  #[test]
  fn each_walk_keeps_its_commands_rules() -> Result<()> {
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    let root = std::env::temp_dir().join(format!("pplr-walk-rules-{}", std::process::id()));
    // Made in reverse name order, so a folder's own order is unlikely to be its names'
    for dir in ["A/sub", "A/skipme", "A/Old Name (y)", "A/.dot", "B", ".top"] {
      std::fs::create_dir_all(root.join(dir))?;
    }
    for file in [
      "A/sub/s.txt",
      "A/sub/s.md",
      "A/sub/Old Name (x).md",
      "A/skipme/k.md",
      "A/Old Name (y)/Old Name (z).md",
      "A/a.md",
      "A/.hidden.md",
      "A/.dot/d.md",
      "B/b.md",
      ".top/t.md",
    ] {
      std::fs::write(root.join(file), "x\n")?;
    }
    symlink(root.join("B"), root.join("A/link-dir"))?;
    symlink(root.join("A/sub/s.md"), root.join("A/link.md"))?;
    symlink(
      root.join("A/sub/s.md"),
      root.join("A/sub/Old Name (link).md"),
    )?;
    let under = |base: &Path, paths: &[&str]| -> Vec<PathBuf> {
      paths.iter().map(|p| base.join(p)).collect()
    };
    let sorted = |mut paths: Vec<PathBuf>| {
      paths.sort();
      paths
    };

    // grep: past dot-named tops, in name order, every regular .md below the top
    assert_eq!(
      crate::grep::markdown_files(&root),
      under(
        &root,
        &[
          "A/.dot/d.md",
          "A/.hidden.md",
          "A/Old Name (y)/Old Name (z).md",
          "A/a.md",
          "A/skipme/k.md",
          "A/sub/Old Name (x).md",
          "A/sub/s.md",
          "B/b.md",
        ]
      )
    );
    // links: no dot name at any depth, and a symlinked note kept as its file, once
    assert_eq!(
      crate::links::markdown_files(std::slice::from_ref(&root)),
      under(
        &root.canonicalize()?,
        &[
          "A/Old Name (y)/Old Name (z).md",
          "A/a.md",
          "A/skipme/k.md",
          "A/sub/Old Name (x).md",
          "A/sub/s.md",
          "B/b.md",
        ]
      )
    );
    // rename's search: dot names kept, symlinks never, a skip folder not entered
    assert_eq!(
      sorted(crate::rename::search(
        std::slice::from_ref(&root),
        &["md"],
        &["skipme"]
      )),
      sorted(under(
        &root,
        &[
          ".top/t.md",
          "A/.dot/d.md",
          "A/.hidden.md",
          "A/Old Name (y)/Old Name (z).md",
          "A/a.md",
          "A/sub/Old Name (x).md",
          "A/sub/s.md",
          "B/b.md",
        ]
      ))
    );
    // rename's named_after: any entry by its prefix, a symlink too, a folder before its contents
    let named = crate::rename::named_after(&root, "Old Name (");
    assert_eq!(
      sorted(named.clone()),
      sorted(under(
        &root,
        &[
          "A/Old Name (y)",
          "A/Old Name (y)/Old Name (z).md",
          "A/sub/Old Name (link).md",
          "A/sub/Old Name (x).md",
        ]
      ))
    );
    let at = |p: &str| named.iter().position(|n| *n == root.join(p));
    assert!(
      at("A/Old Name (y)") < at("A/Old Name (y)/Old Name (z).md"),
      "{named:?}"
    );
    std::fs::remove_dir_all(&root)?;
    Ok(())
  }
}
