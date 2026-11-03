//! Stamps the commit this binary is built from, and whether its build inputs
//! were dirty, for the version line (devbin's dev/opt/brew T4; ST0002 WP03).
//! Like Intent's, the inputs are a declared pathspec scope, read by git; a
//! change outside them (eg a whiteboard or docs write) never marks a build
//! dirty. When git cannot answer, the commit is `unknown`.

use std::process::Command;

/// The build inputs: the crate, what it embeds, and the sidecar's source.
const INPUTS: &[&str] = &[
  "Cargo.toml",
  "Cargo.lock",
  "build.rs",
  "src",
  "templates",
  "swift",
  "README.md",
];

fn git(args: &[&str]) -> Option<String> {
  let out = Command::new("git").args(args).output().ok()?;
  out
    .status
    .success()
    .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn main() {
  let commit = git(&["rev-parse", "HEAD"]);
  let mut status = vec!["status", "--porcelain", "--"];
  status.extend_from_slice(INPUTS);
  let (commit, dirty) = match (commit, git(&status)) {
    (Some(sha), Some(changes)) => (sha, if changes.is_empty() { "0" } else { "1" }),
    _ => ("unknown".to_owned(), "0"),
  };
  println!("cargo:rustc-env=PPLR_SOURCE_COMMIT={commit}");
  println!("cargo:rustc-env=PPLR_SOURCE_DIRTY={dirty}");

  // Rerun when an input or the commit moves; a commit changes HEAD and the index
  for input in INPUTS {
    println!("cargo:rerun-if-changed={input}");
  }
  if let Some(git_dir) = git(&["rev-parse", "--git-dir"]) {
    println!("cargo:rerun-if-changed={git_dir}/HEAD");
    println!("cargo:rerun-if-changed={git_dir}/index");
  }
}
