# ARCHITECTURE.md

How Pplr is shaped, and why.

The decisions are recorded where they were made: ST0001's `design.md` (D1 to D11, the module map, the Contacts boundary) for the port, and ST0002's `design.md` with `install-release.md` for the install and the release. This page is the shape they produced, the reason for each part, and what breaks if someone quietly undoes it. The source says what exists and where: `intent search` and each module's doc comment answer that.

## One binary, and the data outside it

`pplr` is one Rust crate at the repo root, built in Cldr's house style (D5): edition 2024, `unsafe_code` forbidden, `unwrap` and `expect` denied, a hand-rolled CLI, boxed errors that name the file or setting. It replaces the bash scripts, the four Python helpers and the Swift engine the shell version compiled at run time (D1).

- **Code and data never mix** (D11). The binary is the code. The People tree, named by `PPLR_DATA` and then `PPLR_DIR` (D8), is the data. The person template, the Finder icons and the README are compiled in (`assets.rs`, and `cli.rs` for the README), and the tree's `_pplr/` overrides them (D6). Tests use synthetic data alone, and `bin/devbin people-leak` refuses a staged file that matches the live tree.
- **Nothing is built or installed at run time.** `bin/devbin build` compiles the Contacts sidecar and `Pplr Links.app` ahead of time, beside the binary.
- **No checkout is needed** (D6, AC-00.6). `layout.rs` resolves the running binary's own path to its tree, and every path pplr needs comes from there or from the data tree. PPLR_ROOT and the variables that hung off it are retired.

Why: a Homebrew keg has no checkout (ST0002), and the shell version could not run without one. Undo it and the brew tree breaks: any path into the build checkout, or any compile on first use, fails where pplr is installed.

## The command table

`cli.rs` holds `COMMANDS`, one row per command (its name, its usage lines, its handler), sorted, and a unit test keeps it sorted and unique. `help` lists the table, and `help --details` prints the compiled-in README. `main.rs` only calls `cli::run`. An error comes back to it and is printed once, as `pplr: <message>` on stderr, with exit 1. The commands about pplr itself, and doctor, run where there is no tree: `doctor`, `help`, `install`, `use` and `version`.

Every argument passes through `marker::rewrite_args` first, so a `pplr://` marker works wherever a command takes "Surname, First". The parsing is hand-rolled, not clap's, because each command's flags, refusals and usage text had to match the legacy's to the byte (D4, D10).

## Pure core, impure edges

PFIC holds across the crate. The rules are pure functions:

- `about.rs`, the About header;
- `normalise.rs`, folding, phones, email keys and LinkedIn slugs;
- `marker.rs`, markers and URLs;
- `plan.rs`, sync's scoring and decisions;
- `yaml.rs`, the reader;
- the emitters for `index.json`, `tags_index.json`, `tags.yaml`, `contact.yaml` and the workbook's rows.

The I/O sits at the edges:

- `fs.rs`, the one file reader and writer;
- `tree.rs`, the walks;
- `contacts/`, the stores;
- `mail.rs`, Mail's SQLite;
- the command modules, which coordinate: parse, call, render.

A library function returns its error to the coordinator and never prints it (No Silent Errors).

- **One About reader** (`fs.rs`; a3ab0fb, vc decision 10). `read_text` decodes lossily, for the reads, so an About that is not UTF-8 keeps its header, as the shell's byte-wise reads did. `read_exact` refuses a file that is not UTF-8, for every command that writes the About back (`sync --pull`, `rename`, `tags render`, `pictures`), so pplr never rewrites a byte it could not decode. A file that is missing is `None`. Any other failure names the file. Every write goes through a temporary file beside the target (`write_atomic`), so a reader never sees half a file.

## Byte for byte with the legacy

The People tree is live, synced and versioned, so a format drift would rewrite hundreds of files on the first run (D4). Every file the binary writes under `PPLR_DATA` matches what the shell version wrote for the same input. The proof is the goldens in `tests/golden/cases/`, one case per scenario, captured from the legacy code over a synthetic tree, which `tests/cli.rs` replays against the binary: stdout, stderr, the exit code and the resulting tree. The goldens are frozen as the legacy's record, and a golden changes only in the commit of a ruled divergence, which names the ruling. The divergences are listed in the CHANGELOG's 2.0.0 section.

That is why the writers are hand-written rather than a library's defaults:

- `index.json` is printed as `jq .` prints it, and `tags_index.json` as Python's `json.dump(indent=1, ensure_ascii=False)`.
- `tags.yaml` and `contact.yaml` come from ported emitters. `yaml.rs` reads them as PyYAML's `safe_load` did: YAML 1.1 booleans and dates. The maintained crates follow YAML 1.2, and `serde_yaml` is deprecated.
- The `.webloc` and `.inetloc` plists are written from fixed templates.
- `fold_table.rs` is generated from Foundation by `bin/devbin fold-table` (from `swift/fold-table/main.swift`), so case and accent folding (markers, matching) and canonical decomposition (the `.inetloc` names, decomposed on disk as Foundation wrote them) are Foundation's own answers, not a Rust approximation of them.
- `PPLR_TODAY` pins the date for every date-dependent command, so the goldens hold on any day.

Undo any of these and the replay fails, or the live tree is rewritten.

## Contacts behind one store

```text
sync.rs, plan.rs (pure)  ->  Store  ->  JsonStore     PPLR_CONTACTS_JSON: the tests, Linux
                                    ->  SidecarStore  macOS: pplr-contacts dump | backup | group,
                                                      and Contacts.app scripts for every write
```

`contacts/mod.rs` defines `Store`, and sync reaches cards through it alone (D2). The live store reads through `pplr-contacts`, a Swift sidecar cut down to I/O: it dumps the cards with the cross-account id resolution, writes the `.vcf` backup and finds the `PPLR` group. Card and photo writes go to Contacts.app through `osascript` and `sips` (`contacts/scripts.rs`). The JSON store applies the same writes to a dump file, so every sync test runs without Contacts, on Linux too.

Why Swift for the sidecar: the house style forbids `unsafe`, and Rust bindings to Contacts would need it or a second crate. The Swift reads already worked. And the framework refuses card writes anyway (Cocoa error 134092), so the writes are AppleScript whatever reads the cards. The sidecar is compiled with `swiftc` from `swift/pplr-contacts/main.swift` alone, with its `Info.plist` linked in so macOS can name it in the access prompt; there is no `Package.swift`. In a release it is signed with the addressbook entitlement.

## Mail

`mail.rs` reads Mail's own SQLite index and the system's accounts store with `rusqlite`, read-only (WP14). Mail offers no public framework for its mailboxes, and the old route, `mail-recent`, was private Python tooling that was not on `PATH`. Both schemas are Apple's and unversioned, so the tables and columns pplr reads are checked first, and a store that lacks one is refused by name rather than read for wrong rows. `email --json` is a versioned document (`schema: 1`) for other tools, and changes its schema number when a field changes.

## The install layout

pplr follows devbin's dev/opt/brew contract (T1 to T10, ST0002 `install-release.md`):

- **`layout.rs`** resolves the running binary to its tree: the root, the kind (dev, opt, brew or unknown), and the sidecar and app beside it. Dev is a root whose `Cargo.toml` names `pplr`, with the binaries in `target/<profile>/`.
- **`install.rs`** owns the runtime set (`bin/pplr`, and on macOS `bin/pplr-contacts` and `Pplr Links.app`), the manifest `.pplr-install` (T3), `pplr install` (dev to an opt tree), `pplr use` (the one link, `~/.local/bin/pplr`) and the version line, `pplr <version> (<tree>, <sha12>[+dirty])` (T4). `Cargo.toml` is the one version source (D7).
- **`handler.rs`** registers the tree's prebuilt `Pplr Links.app` with `lsregister`, and builds nothing. The app runs the `bin/pplr` beside it, found through `path to me`, so one signed app works in any tree (ST0002 ruling 6.3).
- **The release** is devbin's release core, with `packaging/homebrew/pplr.rb` rendered into the tap in assets mode, and the signer in `bin/.devbin/cmd/macos`.

## Platforms

macOS is the target. Linux builds, tests and runs everything except what needs a macOS service (D9): the live Contacts store, `handler`, the Finder icon commands (through `fileicon`), `open`, `linkedin` and `cp` without `-o`. Each of those exits 1, naming why, rather than half-working. CI runs the cargo gates and the bats suite on both.

## Tests and gates

- **Unit tests** sit in each module, for its pure functions. Their expected values derive from their inputs, never from a count written down by hand.
- **`tests/cli.rs`** is the one integration target. It has a `Sandbox` (a private `HOME`, a data root and a clean environment), the golden replay and the acceptance tests.
- **The bats suite** (`tests/*.bats`) tests black-box against any pplr named by `PPLR_UNDER_TEST`, and its default is the checkout's dev build, `target/debug/pplr`. Each test gets its own data directory under `$BATS_TEST_TMPDIR`, and no path from the environment is ever a deletion target.
- **The gates** run through devbin. `check all` runs every read-only gate (`bin/devbin help check` names them), and `test all` runs `cargo test`, then the bats suite. Every project command, the gates' scripts among them, is a file in `bin/.devbin/cmd/` that `bin/devbin` runs, and `bin/devbin help` lists them. The pre-commit gate runs `bin/devbin people-leak`, `bin/devbin nopython`, `intent critic` on the staged files and `intent doctor`. CI (`tests.yml`) runs on `main` and on `candidate/**`, which devbin's release core pushes before it tags.
- **No Python** (ST0001 design ruling 1, AC-00.1). `bin/.devbin/cmd/nopython` refuses Python in the tree (its header says what it matches and how a line is marked). It runs in `check all`, the pre-commit gate and CI.
