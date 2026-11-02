# ST0001 design: Pplr in Rust

Design record by vc, 2026-11-02, from the recon attached to this thread (`recon.md`), dc's boot findings and cc's survey. It fixes what the port must preserve, how the work is divided, and the rules every work package follows. ST0002 ships the result through Homebrew; the two threads meet at the install layout (section 3.4).

## 1. Rulings (hv, 2026-11-02)

1. **The Python goes, all of it**: `tags.py`, `contact.py`, `workbook.py`, `email_list.py`, and the `python3` one-liner in `pplr_open`. `uv` leaves the run time with them.
2. **The core is Rust.** Swift is acceptable where Contacts needs it, because it is already written and works. Rust and Swift are both acceptable languages; Python is not.
3. **The test suite is to be cleaned up**, not trusted as it stands.

## 2. Decisions

| ID  | Decision                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Why                                                                                                                                                                                                                                                       |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| D1  | **One binary, `pplr`, owns all the logic**: the people model, markers, matching, planning, tags, cadence, indexes, rename, links, search and every renderer.                                                                                                                                                                                                                                                                                                                                                                                                                                                                         | Ruling 2; today the logic is split across bash, Python and Swift with no single owner.                                                                                                                                                                    |
| D2  | **Live Contacts sit behind one store interface.** The JSON store reads and writes a file in the engine's `dump` format (the `PPLR_CONTACTS_JSON` seam every sync test uses). The macOS store is a **thin Swift sidecar, `pplr-contacts`, cut down to I/O**: `dump` (cards as JSON, including the cross-account id resolution), `backup` (every card to `.vcf`) and `group` (ensure the PPLR group, print its id). Card and photo writes stay AppleScript to Contacts.app, run by Rust through `osascript` and `sips`.                                                                                                                | House style forbids `unsafe` and `forbid` cannot be relaxed inside a module, so `objc2-contacts` would cost the lint or a second crate. The Swift reads already work. The framework refuses card writes (Cocoa error 134092) anyway. cc's recommendation. |
| D3  | **Port behind a fallback.** From WP02 the Rust `pplr` serves the commands ported so far and runs `bin/pplr_<cmd>` for the rest, with exactly the environment `bin/pplr` sets today. Each command has one live implementation at a time: the WP that ports a command rewires its legacy callers and deletes its bash and Python. WP13 removes the fallback.                                                                                                                                                                                                                                                                           | The bats suite stays green against the Rust binary at every WP boundary (cc's recommendation 1), and nothing is implemented twice.                                                                                                                        |
| D4  | **Data files are byte-identical.** Every file the port writes under `PPLR_DATA` matches what the legacy code writes for the same input, proven against goldens captured from the legacy code over a synthetic tree (WP03). Exceptions are time-dependent fields, listed on this thread. The workbook is compared by content, and stdout JSON semantically.                                                                                                                                                                                                                                                                           | The People tree is live, synced and versioned: a format drift rewrites hundreds of files on the first run (cc's recommendation 3).                                                                                                                        |
| D5  | **House style is Cldr's** (recon section 13): one crate at the root, edition 2024, `rust-version` in `Cargo.toml`, `tab_spaces = 2`, `unsafe_code = "forbid"`, `unwrap_used` and `expect_used` denied, a thin `main.rs`, a hand-rolled CLI (no clap), boxed errors that name the file or setting, writes through a temp file, a why-comment on each dependency, unit tests in the module, one `tests/cli.rs` with a `Sandbox`. Paths follow XDG: `PPLR_BACKUP_DIR` defaults to `$XDG_DATA_HOME/pplr/contacts-backups` and `PPLR_CACHE_DIR` to `$XDG_CACHE_HOME/pplr`, and doctor names any legacy `~/Library` directory left behind. | One style across the user's Rust tools. The XDG move is a deliberate default change (hv, 2026-11-02); the backups already written stay where they are.                                                                                                    |
| D6  | **The binary finds everything from its own resolved path** (devbin T1). Templates, icons and the README are embedded at build time from the repo's `templates/` and `README.md`, the one source. The data tree's own `_pplr/templates/person` and `_pplr/icons` override them when present. The sidecar sits beside the binary in the same tree. `PPLR_ROOT`, `PPLR_BIN_DIR`, `PPLR_TEMPLATE_DIR` and `PPLR_CALLER_DIR` are retired.                                                                                                                                                                                                 | A keg has no checkout. The CHANGELOG already names `_pplr/` as the home of templates.                                                                                                                                                                     |
| D7  | **`Cargo.toml` is the one version source**; `VERSION.md` goes. The first release is 2.0.0 (hv, 2026-11-02; this is pplr's own version, independent of the Rust toolchain's 1.x), with the CHANGELOG section headed `## [2.0.0] - unreleased`. ST0002 adds the full version line, `pplr <version> (<tree>, <sha12>)`.                                                                                                                                                                                                                                                                                                                 | Two sources diverge; the install model changes, so a major version.                                                                                                                                                                                       |
| D8  | **`PPLR_DATA` names the data, and `PPLR_DIR` is its legacy fallback.** Doctor warns when both are set and differ, because Gtools reads `PPLR_DIR` first.                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Gtools reads the same tree through both names.                                                                                                                                                                                                            |
| D9  | **Linux builds and tests.** Everything runs there except what needs a macOS service: the live Contacts store, `handler`, Finder icons, `open`, `linkedin`, `cp` and `edit`, which refuse with a message naming why and a non-zero exit. Sync against the JSON store runs on Linux.                                                                                                                                                                                                                                                                                                                                                   | CI runs Ubuntu; kovacs is the Linux release leg; No Silent Errors.                                                                                                                                                                                        |
| D10 | **Parity first, fixes listed.** Behaviour matches the legacy code. A legacy defect is fixed only where the fix changes no data file and no tested output, and each fix is listed in its WP: the search query injection and the `setpicsfordirs` word splitting are fixed. By hv's ruling (2026-11-02), `edit` honours `$VISUAL`, then `$EDITOR`, and refuses naming both when neither is set; `new` writes the git author (`git config user.name`) into its verblock, and the goldens pin it with a sandboxed git config. Errors go to stderr; bats `run` captures both streams.                                                     | A port that also redesigns cannot be checked against its oracle.                                                                                                                                                                                          |
| D11 | **Code and data never mix.** `~/Devel/prj/Pplr` holds code and synthetic fixtures only; `~/Dropbox/Career/People` holds the data, and nothing from it (names, markers, emails, notes, generated files or command output) is ever committed to Pplr. Goldens and tests use synthetic data alone. WP13's live parity records only whether the outputs matched, never the outputs. A leak check reads the names, markers and About emails from `PPLR_DATA` at run time, so they never enter the repo, and fails on any match in what is staged; it runs in the pre-commit gate.                                                         | hv, 2026-11-02. Checked on 2026-11-02: no People name, marker or email appears in Pplr's tree or history.                                                                                                                                                 |

## 3. Architecture

### 3.1 Module map

One crate at the repo root. Module names are proposals; the owning WP may refine them, never merge two concerns into one.

| Module                                                                                  | Owns                                                                                                                | WP     |
| --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------ |
| `main.rs`, `cli.rs`                                                                     | the command table (name, one-line usage, handler); argument helpers; the fallback; `help` generated from the table  | 02     |
| `error.rs`                                                                              | `Error`, `Result`, message cleaning                                                                                 | 02     |
| `config.rs`                                                                             | the data root (D8), the XDG defaults (D5), every `PPLR_*` setting read once                                         | 02     |
| `assets.rs`                                                                             | embedded templates, icons and README; `_pplr/` overrides                                                            | 02, 05 |
| `tree.rs`                                                                               | walk rules, letters, person keys, `PersonRef` (two arguments, `Surname, First`, `L/Surname, First`, `pplr://`)      | 04     |
| `about.rs`                                                                              | the About header: read the fields; edit the Tags line, the picture line and pulled values; rename text              | 04     |
| `normalise.rs`                                                                          | folding, phones, email keys, LinkedIn slugs, link text                                                              | 04     |
| `marker.rs`                                                                             | marker path and URL, aliases, `resolve`, tag pages                                                                  | 04     |
| `index.rs`                                                                              | `index.json` (in `jq`'s format), `index.md`, `count`, `reindex`                                                     | 05     |
| `meetings.rs`, `refresh.rs`, `grep.rs`, `search.rs`, `new.rs`, `pictures.rs`, `open.rs` | one command group each (`open.rs`: `open`, `cp`, `linkedin`, `edit`, `about`, the CMS probe)                        | 04, 05 |
| `yaml.rs`, `tags.rs`                                                                    | YAML reading matched to PyYAML; vocabulary, `tags.yaml`, check, show, edit, apply, render, pages, `tags_index.json` | 06     |
| `contact.rs`, `email.rs`                                                                | `contact.yaml`, cadence, scan, due, show, log, next, context, render; the mail listing and its cache                | 07     |
| `rename.rs`, `links.rs`, `handler.rs`                                                   | rename, link rewriting, the `pplr://` app                                                                           | 08     |
| `contacts/`                                                                             | the `Card` model, the store interface, the JSON store, the sidecar store, the Contacts.app scripts                  | 09     |
| `sync/`                                                                                 | `check`, `link`, `pull`, `photos`, `backup` (WP10); `plan`, `workbook`, `applyplan` (WP11)                          | 10, 11 |
| `doctor.rs`, `icons.rs`                                                                 | doctor in Cldr's check pattern; `applyicons`, `setpicfordir`, `setpicsfordirs`                                      | 12     |
| `swift/pplr-contacts/main.swift`                                                        | the sidecar, I/O only (D2); moved out of `src/`, which becomes the crate's                                          | 09     |

Dependencies are few and each is justified where it is added. Expected: `serde` and `serde_json` (with formatters matching `jq` and Python's `indent=1`), a YAML reader chosen in WP06 against PyYAML's output (`serde_yaml` is deprecated), `rust_xlsxwriter` and `calamine` for the workbook (WP11), `jiff` for dates (as Cldr), and `regex` only where a hand-written scanner would be worse. Plists are small fixed XML, written from templates so they stay byte-identical.

### 3.2 The Contacts boundary

```
sync logic (Rust, pure)  ->  Store trait  ->  JsonStore      (PPLR_CONTACTS_JSON; tests, Linux)
                                          ->  SidecarStore   (macOS: pplr-contacts dump|backup|group,
                                                              plus osascript and sips for writes)
```

- The sidecar keeps today's read code: per-container enumeration, the unified-id resolution (iCloud first), groups, account kinds, `CNContactVCardSerialization`, and the access prompt with exit code 3. Everything else leaves `main.swift`.
- The JSON store applies writes to its file as the engine's fixture path does today, so the 42 sync tests keep their meaning.
- The sidecar is built ahead of time (devbin's build in dev, the formula in brew) and found beside the binary. A missing sidecar is an error naming the step that builds it. Nothing compiles at run time.

### 3.3 The fallback

- The Rust `pplr` looks its command up in its table. A command not yet ported runs `bin/pplr_<cmd>` from the checkout the binary was built in, with `PPLR_ROOT`, `PPLR_BIN_DIR`, `PPLR_DATA`, `PPLR_DIR` and `PPLR_CALLER_DIR` set and the working directory changed as `bin/pplr` does, and the `pplr://` arguments rewritten as `bin/pplr` does.
- A bash script that calls another command directly (eg `pplr_reindex` runs `pplr_json`, `pplr_open` runs `pplr cp`, `pplr_tag` runs `pplr_tags` and `pplr_contact`) must be rewired to `pplr <cmd>` before the script it calls is deleted. The porting WP greps for callers first.
- The fallback is dev-only scaffolding. It is never released: WP13 removes it before ST0002 cuts 2.0.0.

### 3.4 The install layout (the seam with ST0002)

Proposed here and settled by dc in ST0001 WP02 and ST0002 WP01, which dc owns both of:

- A tree is `<root>/bin/pplr`, plus `<root>/bin/pplr-contacts` and `<root>/Pplr Links.app` on macOS, plus ST0002's manifest. Brew's root is `$(brew --prefix)/opt/pplr/libexec`, named by its opt link, never a Cellar path.
- The binary resolves its own path, and one function returns the root, the tree kind and the sidecar and app paths.
- Templates, icons and the README are inside the binary (D6), so the runtime set is the binaries, the app and the manifest.
- **`Pplr Links.app` ships prebuilt and signed in the tree** (hv, 2026-11-02): signing happens only on rhadamanth, so nothing is built or ad-hoc signed on a user's machine. The app finds `bin/pplr` beside itself, and `pplr handler --install` only registers it with `lsregister`. This removes the app build from `handler`, which is simpler than the legacy flow.

### 3.5 Data compatibility

- Every file in recon section 5 is byte-identical against its golden.
- `index.json`: `jq .`'s print. `tags_index.json`: Python's `json.dump(indent=1, ensure_ascii=False)`, which `serde_json`'s pretty formatter reproduces with a one-space indent. Both are proved by golden, not by inspection.
- `tags.yaml` and `contact.yaml`: ported emitters, not a YAML library's output. Reading follows PyYAML where the files can tell the difference (dates, YAML 1.1 booleans).
- `.inetloc` and `.webloc`: written from fixed templates matching Apple's and the heredoc's bytes.
- Time-dependent output is pinned by `PPLR_TODAY` (extended from `contact` to every date-dependent command), and `generated_at` is the one listed exception.

### 3.6 Platforms

macOS is the target. Linux builds, tests and runs everything except the macOS services in D9. CI runs the cargo gates and the bats suite against the Rust binary on both.

## 4. Testing

1. **An honest, safe oracle (WP01).** Each test gets its own data directory under `$BATS_TEST_TMPDIR`, and no path from the environment is ever a deletion target. One setting points the suite at either implementation. The runner and CI fail when a test fails. Generated fixtures are no longer tracked. The true baseline is recorded on macOS (through the bats lane) and on CI, and each failing test is triaged on this thread before any port begins.
2. **Goldens from the legacy code (WP03).** A synthetic People tree covers the recon's edge cases: accents, multi-word and double-barrelled surnames, all three meeting-note styles, jpg, jpeg and png pictures, aliases, a vocabulary with `also` and `split`, cadence `roster` and `every`, `contact.yaml`, and a Contacts dump with `home`, `other` and `none` accounts and a card linked across accounts. Every writing and reporting command runs under the legacy code with `PPLR_TODAY` pinned. Its stdout, exit code and resulting tree are stored as goldens, and a test in `tests/cli.rs` replays each one against the Rust binary. **No legacy code is deleted before its goldens exist.** The capture uses the legacy Python as an oracle, once; no Python is committed.
3. **Unit tests** in each module, for every pure function: the normalisation rules, the marker, header parsing, the matching tiers, scoring, link conversion, the emitters and the cadence arithmetic. Expected values derive from their inputs: no hand-written counts.
4. **The bats suite against the Rust binary** at every WP boundary, on macOS and Linux. A test that checks bash internals (eg "scripts are executable", "bash 4") is rewritten to check behaviour, or withdrawn on the record with its reason.
5. **Live read-only parity (WP13).** On rhadamanth, with the legacy code checked out from history into a scratch worktree, the read-only commands run over the live People tree under both implementations and their output is compared. Nothing writes to the live tree.

## 5. Work packages

| WP  | Title                                             | Owner | Size | Depends on | Serves                     |
| --- | ------------------------------------------------- | ----- | ---- | ---------- | -------------------------- |
| 01  | Test safety, the parity seam and an honest runner | dc    | S    | -          | AC-03, AC-10               |
| 02  | Rust scaffold under devbin, with the fallback     | dc    | M    | 01         | AC-04, AC-08, AC-09        |
| 03  | Golden capture from the legacy code               | cc    | M    | -          | AC-05                      |
| 04  | Core model and the read commands                  | cc    | M    | 02, 03     | AC-02, AC-04, AC-05        |
| 05  | Indexes, new person, pictures and search          | cc    | M    | 04         | AC-02, AC-05               |
| 06  | Tags                                              | cc    | M    | 04         | AC-01, AC-02, AC-05        |
| 07  | Contact cadence                                   | cc    | M    | 06         | AC-01, AC-02, AC-05        |
| 08  | Rename, links and the `pplr://` handler           | cc    | M    | 05         | AC-02, AC-05, AC-08        |
| 09  | The Contacts store and the Swift I/O sidecar      | dc    | M    | 02         | AC-07, AC-11               |
| 10  | Sync: check, link, pull, photos and backup        | cc    | L    | 04, 09     | AC-02, AC-05, AC-07        |
| 11  | The sync plan and its workbook                    | cc    | L    | 10         | AC-01, AC-02, AC-05        |
| 12  | Doctor and Finder icons                           | dc    | S    | 04         | AC-02, AC-11               |
| 13  | Cutover                                           | cc    | M    | 01 to 12   | AC-01, AC-02, AC-06, AC-12 |
| 14  | pplr's own mail reader                            | cc    | M    | 04, 07     | AC-14.1, AC-14.2           |

The two lanes run side by side:

- **dc**: WP01, then WP02, then WP09 and WP12, interleaved with ST0002.
- **cc**: WP03 now (it needs no Rust), then WP04 once WP02 lands, then WP05 to WP08 in any order the dependencies allow, then WP10 and WP11, then WP14 once WP07 is done, then WP13.

ST0002's release waits on WP13.

## 6. Rules for every work package

1. Claim it (`intent wb claim ST0001/NN --node <you>`), then `intent wp start ST0001/NN`. Release the claim at done.
2. Heavy runs (`cargo`, `bats`) go through `intent estate slots`. Read `intent estate status` first.
3. One live implementation per command: port it, rewire its legacy callers, delete its bash and Python, all in the same WP. Never delete a command whose goldens do not yet exist.
4. Nothing writes to the live People tree, and nothing from it enters Pplr (D11). Read-only commands against it are allowed in WP13 only, and only the verdict is recorded.
5. Commit by path exactly what you wrote. Commit messages end with `(C) hello@matthewsinclair.com`, with no AI attribution. Tags and pushes are hv's.
6. A WP is done when its criteria hold: run `intent wp gate` and the cargo and bats gates. vc then reviews it against this design and the thread's criteria.
7. Anything this design gets wrong goes to vc as an `intent wb ask`, not a quiet deviation. A changed decision is recorded here.
8. **A porting WP is not done while a bats skip names it.** WP02 guards the legacy-engine and `uv` tests with skips that name the WP porting each one (`needs_legacy_engine WPnn`, and the `uv` guards naming WP07). Porting the command deletes its skips in the same WP. Before done, `grep -rn "WPnn" tests/*.bats` over the skip helpers finds none, and vc checks it at review.
9. **WP11 is not done while `IN-SW-CODE-006` is disabled.** WP09 disables the Swift force-unwrap rule in `.intent_critic.yml` because the legacy engine, the only Swift in the repo, carries force unwraps in check, plan, link and pull code that WP08, WP10 and WP11 delete. The sidecar I/O it keeps is clean. When WP11 cuts the engine to the sidecar, it re-enables the rule, and the sidecar passes it.

## 7. Answers and open questions

hv's answers, 2026-11-02:

1. **2.0.0** is the first Rust release (D7).
2. **XDG defaults** for backups and the cache (D5).
3. **`edit` honours `$VISUAL` and `$EDITOR`; `new` uses the git author** (D10).
4. **Live read-only parity** over the live People tree is agreed (section 4, step 5), recording verdicts only (D11).
5. **pplr reads mail itself** (WP14). `mail-recent` is private Geodica tooling (`~/Devel/cfg/Gtools-geodica/bin/mail-recent`, Python, not on `PATH`), so pplr must not depend on it. pplr gets its own read-only reader of Mail's Envelope Index in Rust, which reads SQLite as Cldr does; Swift has no public mail framework to offer. `pplr email --json` becomes a versioned contract that Gtools can call once it lands.

No questions are open.
