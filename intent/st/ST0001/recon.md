# ST0001 recon: Pplr as it stands, 2026-11-02

Recon by vc, 2026-11-02, before any port work. Sources: every file under `bin/`, `lib/`, `src/`, `tests/` and `templates/`, plus the README, CHANGELOG, `usage-rules.md`, `setup.sh`, `.env.example`, the CI workflow and its last run, and the live People tree, which was counted but never read. Two peer surveys fed it: dc's boot findings and cc's survey, both 2026-11-02 07:37Z and 07:38Z in vc's inbox. Where they found something this recon missed, they are credited.

## 1. Summary

- Pplr is 2,442 lines of bash (the `bin/pplr` dispatcher plus 30 `bin/pplr_*` commands), 1,681 lines of Swift (`src/pplr-contacts/main.swift`), and 1,030 lines of Python in four helpers run under `uv`. The bats suite has 99 tests in 10 files (1,229 lines) plus a 399-line helper.
- **The Swift program is Pplr's engine, not just its Contacts bridge.** Besides Contacts sync it holds the people model, the About header parser, the `pplr://` marker rules, `resolve` and `links`, and the `people` listing that the Python helpers read. `lib/engine.sh` compiles it with `swiftc` on first use, into `$PPLR_ROOT/.build`.
- **The Python is load-bearing:** tags (`tags.py`), contact cadence (`contact.py`), the sync plan workbook (`workbook.py`) and the email listing (`email_list.py`), plus a `python3` one-liner in `pplr_open`. hv's ruling of 2026-11-02 removes all of it.
- **CI is a false green.** `tests/run_tests.sh` ends with `rm -rf "$PPLR_TEST_DATA"`, so it exits with `rm`'s status, never bats'. The last run (2026-10-13, run 38407533112) reported success with 9 failing tests on macOS and 41 on Ubuntu.
- **The test suite can delete real data.** `setup()` honours an inherited `PPLR_TEST_DATA`, and both teardown and the runner `rm -rf` it (dc's finding).
- **The suite is a usable parity oracle once two seams are fixed.** Every test drives `"$PPLR_BIN_DIR/pplr" <cmd>` from outside, and the 42 sync tests read Contacts from a JSON fixture (`PPLR_CONTACTS_JSON`), never the live store. But `setup()` resets `PPLR_BIN_DIR` to the checkout (dc's finding), so the suite cannot yet be pointed at another binary.
- **Nothing here survives a Homebrew keg as is:** the run-time Swift build, `PPLR_ROOT`, the handler's baked path, `help --details` reading the checkout's README, and the version read from `VERSION.md`.
- Other code reads Pplr's data: **Gtools** reads the People tree through `PPLR_DIR`, then `PPLR_DATA`, and the CMS that `pplr open` targets (`localhost:4360`) reads the index files. The file formats and both variable names are a contract, not an implementation detail.

## 2. Inventory

| Component       | Files                                | Lines | Role                                                                                    |
| --------------- | ------------------------------------ | ----- | --------------------------------------------------------------------------------------- |
| Dispatcher      | `bin/pplr`                           | 86    | env defaults, `cd $PPLR_DATA`, marker rewriting, runs `bin/pplr_<cmd>`                  |
| Commands        | `bin/pplr_*` (30)                    | 2,356 | one script per command                                                                  |
| Engine build    | `lib/engine.sh`                      | 15    | `swiftc -O ... -framework Contacts` into `$PPLR_ROOT/.build/pplr-contacts` on first use |
| Swift engine    | `src/pplr-contacts/main.swift`       | 1,681 | people model, markers, resolve, links, Contacts read, sync logic, Contacts.app scripts  |
| Python          | `src/pplr-tags/tags.py`              | 380   | tags: vocabulary, `tags.yaml`, Tags line, `_tags/` pages, `tags_index.json` (PyYAML)    |
| Python          | `src/pplr-contact/contact.py`        | 422   | contact cadence: `contact.yaml`, `cadence.yaml`, `(Contact).md` (PyYAML)                |
| Python          | `src/pplr-contacts/workbook.py`      | 165   | the sync plan workbook, written and read back (openpyxl)                                |
| Python          | `src/pplr-email/email_list.py`       | 63    | numbered mail listing from `mail-recent` JSON, cached for `contact log --email N`       |
| Templates       | `templates/person/**`, `icons/*.pdf` | 3 md  | new-person skeleton (About, Meetings, Client), Finder icons                             |
| Tests           | `tests/*.bats`, `test_helper.bash`   | 1,628 | 99 black-box tests, mock `claude`, fixture data built per test                          |
| Install, config | `setup.sh`, `.env.example`           | 114   | writes `PPLR_ROOT`, `PPLR_DATA`, `PATH` into the shell rc file                          |
| CI              | `.github/workflows/test.yml`         | 52    | bats on ubuntu-latest and macos-latest; installs jq and bats, not uv                    |
| Version         | `VERSION.md`                         | 1     | `1.0.1`; the CHANGELOG has a large unreleased section since 2025-07-19                  |
| devbin          | `bin/devbin`, `bin/.devbin/`         | -     | devbin 0.1.19 vendored 2026-11-02, untracked, config declares only the project name     |

## 3. How a command runs today

`bin/pplr`:

1. Defaults `PPLR_DATA` to `$HOME/Dropbox/Career/People`, `PPLR_DIR` to `$PPLR_DATA`, `PPLR_ROOT` to the checkout (from its own resolved path) and `PPLR_BIN_DIR` to `$PPLR_ROOT/bin`.
2. `-v` and `--version` run `pplr_version`, which prints `VERSION.md`. No argument prints `Usage: pplr <cmd>` and exits 1. An unknown command prints `Error: Command 'x' not found.` to stdout and exits 1.
3. Exports `PPLR_CALLER_DIR=$PWD` and changes to `$PPLR_DATA`, so every command runs from the data root and resolves the caller's relative paths against `PPLR_CALLER_DIR`.
4. Rewrites any argument matching `^pplr://[a-z]/[^/]+/?$` to its folder name ("Surname, First") through `pplr_resolve`, for every command except `open`, `resolve`, `links` and `handler`, which take URLs as they are.
5. Runs `bin/pplr_<cmd>` and exits with its status.

Every command script then re-derives the same environment defaults itself: 25 copies of the same block.

**Two ways to name a person.** `about`, `edit`, `linkedin`, `cp`, `open`, `new`, `applyicons` and `setpicfordir` take `<first> <surname>` as two arguments. `tag`, `tags show`, `contact`, `refresh stamp`, `rename`, `pictures` and `sync --name/--skip` take one `"Surname, First"` (or `"L/Surname, First"`). The marker rewrite in step 4 helps only the second form.

## 4. Command catalogue

| Command          | Lines | Arguments                                                                         | Implemented by                       | Writes                                                                                | Needs                                               |
| ---------------- | ----- | --------------------------------------------------------------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------- | --------------------------------------------------- |
| `about`          | 34    | `<first> <surname>`                                                               | bash                                 | nothing                                                                               | -                                                   |
| `applyicons`     | 85    | `<first> <surname>`                                                               | bash                                 | Finder icons on About, Meetings and each meeting                                      | `fileicon`, `templates/icons`                       |
| `contact`        | 27    | `scan`, `due`, `show`, `log`, `next`, `context`, `render`                         | bash, `contact.py`, engine `people`  | `About/contact.yaml`, `About/<F S> (Contact).md`                                      | `uv`, PyYAML, `swiftc`                              |
| `count`          | 25    | none                                                                              | bash                                 | nothing                                                                               | -                                                   |
| `cp`             | 80    | `[-t about\|linkedin\|profile] [-o] <first> <surname>`                            | bash                                 | the clipboard                                                                         | `pbcopy`                                            |
| `doctor`         | 298   | `[--fix]`                                                                         | bash                                 | with `--fix`: A to Z dirs, `.index`, a reindex, removes old `*/Index` dirs            | `jq`, `bash`                                        |
| `edit`           | 38    | `<first> <surname>`                                                               | bash                                 | nothing                                                                               | `emacsclient`, hard-coded                           |
| `email`          | 38    | `[recent\|inbox] [--account T] [--hours N] [--sent]`                              | bash, `email_list.py`, engine        | `email-last.json` in the cache dir                                                    | `mail-recent`, `python3`                            |
| `grep`           | 112   | `[-t\|--tag] <text>`                                                              | bash                                 | nothing                                                                               | -                                                   |
| `handler`        | 61    | `--install`, `--uninstall`, `--status`                                            | bash                                 | `~/Applications/Pplr Links.app`, registered for `pplr://`                             | `osacompile`, PlistBuddy, `lsregister`              |
| `help`           | 62    | `[cmd]...`, `--details`                                                           | bash, harvesting `# USAGE:` comments | nothing                                                                               | `less`                                              |
| `index`          | 72    | none                                                                              | bash                                 | stdout: the `index.md` text                                                           | -                                                   |
| `json`           | 122   | none                                                                              | bash                                 | stdout: the `index.json` text                                                         | -                                                   |
| `linkedin`       | 34    | `<first> <surname>`                                                               | bash                                 | nothing                                                                               | `open`                                              |
| `links`          | 35    | `<file or dir>... [--apply] [--json]`                                             | bash, engine `links`                 | Markdown files anywhere it is pointed                                                 | `swiftc`                                            |
| `meetings`       | 133   | `[date [end]]` as YYYY, YYYYMM or YYYYMMDD                                        | bash                                 | nothing                                                                               | -                                                   |
| `new`            | 142   | `<first> <surname> [linkedin-url]`                                                | bash                                 | the person's folder from the template, About, `.webloc`                               | -                                                   |
| `open`           | 81    | `[-t type] <first> <surname>`, or `[--print] pplr://...`                          | bash, engine `resolve`               | nothing                                                                               | `open`, `curl`, `python3`                           |
| `pictures`       | 57    | `[--dry-run] ["Surname, First"]...`                                               | bash, `perl`                         | the `<img>` line under each About title                                               | `perl`                                              |
| `refresh`        | 49    | `next [N] [--older-than D]`, `stamp P [SOURCE]`, `status`                         | bash                                 | `<person>/.index/refreshed`                                                           | -                                                   |
| `reindex`        | 41    | none                                                                              | bash, runs `json`, `index`, `tags`   | `.index/index.json`, `.index/index.md`, `.index/tags_index.json`, `_tags/`, Tags line | `jq`, `uv`                                          |
| `rename`         | 159   | `"Old, F" "New, F" [--dry-run] [--refs DIR]...`                                   | bash, `perl`                         | moves the folder and files, About, links under the refs dirs, `.index/aliases`        | `perl`                                              |
| `resolve`        | 14    | `pplr://...`                                                                      | bash, engine `resolve`               | nothing                                                                               | `swiftc`                                            |
| `search`         | 221   | `<query>...`                                                                      | bash, `claude`, `jq`                 | nothing                                                                               | `claude`, `jq`                                      |
| `setpicfordir`   | 91    | `[-V] <first> <surname>`                                                          | bash                                 | the person folder's Finder icon                                                       | `fileicon`                                          |
| `setpicsfordirs` | 44    | none                                                                              | bash, runs `setpicfordir`            | every person folder's Finder icon                                                     | `fileicon`                                          |
| `sync`           | 136   | `--check`, `--link`, `--plan`, `--apply-plan F`, `--photos`, `--pull`, `--backup` | bash, engine, `workbook.py`          | Contacts cards, About (pull), `.inetloc`, backups, `_out/*.xlsx`                      | `swiftc`, `uv`, openpyxl, `jq`, `osascript`, `sips` |
| `tag`            | 19    | `"Surname, First" [+tag] [-tag]...`                                               | bash, runs `tags` and `contact`      | `About/tags.yaml`, `(Contact).md`                                                     | `uv`                                                |
| `tags`           | 31    | `check`, `show P`, `render`, `index`, `apply F` (and `edit`, used by `tag`)       | bash, `tags.py`, engine `people`     | `tags.yaml`, Tags line, `_tags/`, `.index/tags_index.json`                            | `uv`, PyYAML, `swiftc`                              |
| `version`        | 15    | none                                                                              | bash                                 | nothing                                                                               | `VERSION.md`                                        |

## 5. The data contract

The People tree is the user's live data: 368 people, 436 meeting folders, 366 `tags.yaml`, 262 `contact.yaml` and 7 alias files when counted on 2026-11-02. It syncs through Dropbox and is itself a git repository. **Anything the port writes into it must be byte-identical to what the legacy code writes for the same input**, or the first Rust run rewrites hundreds of files.

```
<PPLR_DATA>/
  <L>/<Surname, First>/
    About/<First Surname> (About).md       header list, Tags line, picture line
    About/<First Surname> (LinkedIn).webloc
    About/<First Surname> (Contacts).inetloc
    About/<First Surname> (Contact).md     generated view of contact.yaml
    About/<First Surname> (Picture).jpg|jpeg|png, (Profile).pdf
    About/tags.yaml, About/contact.yaml
    .index/aliases, .index/refreshed
    Meetings/<YYYYMMDD Title>/<YYYYMMDD Title>.md | Summary.md | Notes.md
    Client/
  .index/index.json, index.md, tags_index.json
  _pplr/vocabulary.yaml, cadence.yaml, templates/, icons/
  _tags/index.md, <tag>.md
  _out/contacts-plan-<stamp>.xlsx
```

Walk rules: letter folders are single upper-case letters; a person folder contains `", "`; every walk skips names starting with `_` or `.` (`index` and `json` also skip `bin`); meeting folders start with eight digits; `Icon\r` and `.DS_Store` are skipped where Finder leaves them.

| File                                | Written by today                                                                   | Format to keep                                                                                                                       | Read by                      |
| ----------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------- |
| `About/<F S> (About).md`            | `new`, `tags render`, `pictures`, `sync --pull`, `rename`                          | verblock front matter; `- Field: value` header list; the Tags line ends `<!-- from tags.yaml: pplr tags render -->`                  | Gtools, the CMS, Obsidian    |
| `About/tags.yaml`                   | `tags edit`, `tags apply`                                                          | `tags.py`'s own emitter: two comment lines, `tags:`, `  - word` or `  - {tag: t, source: s, ...}`, evidence JSON-quoted, facet order | `search`, the CMS            |
| `About/contact.yaml`                | `contact scan`, `log`, `next`                                                      | `contact.py`'s own emitter: two comment lines, then `last: {date: D, via: V, link: "...", note: "..."}`, `next: D`, `cadence: N`     | -                            |
| `About/<F S> (Contact).md`          | every `contact.yaml` write, `contact render`                                       | generated-note line, `# Contact: <name>`, bullet list                                                                                | the CMS                      |
| `About/<F S> (Contacts).inetloc`    | `sync --link`, `sync --apply-plan`                                                 | XML plist `{URL: addressbook://<card id>}` from `PropertyListSerialization`; writing one removes a `.webloc` beside it               | Finder                       |
| `About/<F S> (LinkedIn).webloc`     | `new`                                                                              | XML plist from a heredoc, two-space indent                                                                                           | Finder                       |
| `<person>/.index/aliases`           | `rename`                                                                           | one former `L/Surname, First` per line                                                                                               | the engine                   |
| `<person>/.index/refreshed`         | `refresh stamp`                                                                    | `YYYY-MM-DD <source>`                                                                                                                | `refresh`                    |
| `.index/index.json`                 | `reindex` (`json` through `jq .`)                                                  | `jq`'s pretty print                                                                                                                  | the CMS, `doctor`            |
| `.index/index.md`                   | `reindex` (`index`)                                                                | dated verblock, `# People Index`, `##### <L>`, one link per person                                                                   | `count`                      |
| `.index/tags_index.json`            | `tags index`, `tags render`                                                        | Python `json.dump(indent=1, ensure_ascii=False)`; `generated_at` in UTC                                                              | `search`, the CMS            |
| `_tags/index.md`, `_tags/<tag>.md`  | `tags render`                                                                      | dated generated-note line; rewritten in full; pages for unused tags removed                                                          | Obsidian, the CMS            |
| `_out/contacts-plan-<stamp>.xlsx`   | `sync --plan`                                                                      | four sheets (Merged, pplr, Contacts, How to review), shaded review columns, a Decision list, Low and Medium highlighted              | the reviewer, `--apply-plan` |
| backups, `PPLR_BACKUP_DIR`          | `sync --backup`, and before every Contacts write; `--pull` copies each About first | `<stamp>-contacts.vcf` (or `.json` against a fixture); `<stamp>-pull/`                                                               | the user                     |
| `email-last.json`, `PPLR_CACHE_DIR` | `email`                                                                            | Python `json.dump(indent=1)`                                                                                                         | `contact log --email N`      |

**Gtools reads this tree too** (`~/Devel/prj/Gtools`, `native/rust/crates/gtools-rs/src/engagement/mod.rs`). It finds the root through `PPLR_DIR`, then `PPLR_DATA`, then its estate's `people:`, and reads `L/Surname, First/About/First Surname (About).md`. So the layout and both variable names stay. Note that Gtools reads `PPLR_DIR` first while pplr defaults `PPLR_DIR` from `PPLR_DATA`; they agree only while the two are equal.

## 6. The Swift engine

`pplr-contacts <mode>`. Modes and what they are made of:

| Mode             | Pure logic                                                                                  | Contacts I/O                                                                   |
| ---------------- | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `people`         | walk the tree, aliases, marker, About and picture paths                                     | none                                                                           |
| `resolve`        | marker or alias to person, path inside the folder, `pplr://tag/<t>` to `_tags/<t>.md`       | none                                                                           |
| `links`          | rewrite `](...)` links into the tree (paths or CMS URLs) as `pplr://` URLs; follow symlinks | none                                                                           |
| `check`          | match people to cards: pplr URL, then email, LinkedIn, phone, then name alone; field diffs  | read every card, its groups and its account                                    |
| `link`           | which matched cards need the URL, the group or an `.inetloc`                                | backup; card writes through Contacts.app; group create via the API             |
| `plan`           | near-miss scoring (nicknames, Levenshtein, double-barrelled names, domains), confidence     | read                                                                           |
| `applyplan`      | which card gets what                                                                        | backup; card create and update through Contacts.app; photos as TIFF via `sips` |
| `photos`, `pull` | linked cards with no photo; what a card adds to pplr, never removing anything               | read; backup; photo writes (`photos`); About writes (`pull`)                   |
| `backup`, `dump` | none                                                                                        | every card to `.vcf`; every card as JSON                                       |

Facts the port must keep:

- **Card writes go through AppleScript to Contacts.app.** The framework refused card updates from an unentitled tool (Cocoa error 134092), though it can create the group. `osascript` and `sips` are subprocesses, so Rust can drive them with no framework binding.
- **Reads need the Contacts framework**: per-container enumeration to resolve a unified card's merged id back to a real, addressable card (iCloud first), group membership, and account kind (`home`, `other`, `none`). The `dump` mode emits exactly the JSON the test fixtures use.
- **Normalisation rules** (all pure): case and diacritic folding; phones as digits with `+`, UK assumed for a leading 0, the trunk 0 dropped after nine named country codes (Italy keeps it), several numbers split on `,;·/`; Gmail addresses folded for dots and `+suffix`; LinkedIn reduced to its slug; a Markdown link read as its text.
- **The marker**: `pplr://<letter>/<slug>`, lower case, accents dropped, every run of other characters one hyphen; the first form (`pplr://B/Bray,%20Martin`) is still recognised; aliases from `.index/aliases` resolve too.
- **Exit codes**: 3 for no Contacts access, 4 for `resolve` finding nobody, 2 for usage, 1 for any other failure; `link` and `applyplan` stop after three failed writes.

## 7. The Python helpers

- **`tags.py`**: vocabulary with seven facets in code (role, function, sector, org, relationship, place, cadence; the README says six), `also` folds and `split` maps. Its `tags.yaml` emitter sorts by facet, hand tags first. `render` inserts or replaces the generated Tags line after the first header list, rewrites `_tags/` in full, removes pages for unused tags, then writes `tags_index.json`, joined with the engine's people list. `check` exits 1 on any problem; an unknown subcommand exits 2.
- **`contact.py`**: `contact.yaml` per person; roster and cadence tags from `_pplr/cadence.yaml` (`roster:` defaults, `every:` overrides, the file's own `cadence` over both); a meeting folder dated after today is "booked". `scan` takes `last` from the newest past meeting, linked by marker. `log --email N` reads the cached listing. `render` writes `(Contact).md` and removes an old `contact.md`. Seams: `PPLR_TODAY` pins the date; `PPLR_CACHE_DIR` moves the cache (default `~/Library/Caches/pplr`).
- **`workbook.py`**: renders the engine's plan JSON as the four-sheet workbook, and with `--decisions` reads a reviewed workbook back into `[{person, decision, card}]`, refusing an invalid Decision or a missing Card ref.
- **`email_list.py`**: numbers `mail-recent` messages, marks the person by matching `mailto:` addresses in About Email lines, and caches the listing.
- **`pplr_open`** runs `python3 -c 'urllib.parse.quote(...)'` to percent-encode a CMS path.

## 8. Environment

| Variable             | Default                                               | Read by                                        | In the port                                                |
| -------------------- | ----------------------------------------------------- | ---------------------------------------------- | ---------------------------------------------------------- |
| `PPLR_DATA`          | `~/Dropbox/Career/People`                             | everything                                     | keeps its meaning                                          |
| `PPLR_DIR`           | `$PPLR_DATA`                                          | most scripts; Python and Gtools read it first  | legacy fallback; doctor warns when both are set and differ |
| `PPLR_ROOT`          | the checkout                                          | scripts, the engine build, `help`, `version`   | retired: the binary finds its own root (devbin's T1)       |
| `PPLR_BIN_DIR`       | `$PPLR_ROOT/bin`                                      | the dispatcher, `reindex`, `doctor`, the tests | retired; the tests get their own seam                      |
| `PPLR_TEMPLATE_DIR`  | `$PPLR_ROOT/templates`                                | `new`, `doctor`                                | retired: embedded templates, `_pplr/templates` overrides   |
| `PPLR_CALLER_DIR`    | set by the dispatcher                                 | `links`, `tags`, `sync`, `rename`              | retired: the binary never leaves the caller's directory    |
| `PPLR_CMS_URL`       | `http://localhost:4360/people` (empty: files only)    | `open`                                         | keeps                                                      |
| `PPLR_BACKUP_DIR`    | `~/Library/Application Support/pplr/contacts-backups` | `sync`                                         | keeps; default per ST0001 design D5                        |
| `PPLR_CACHE_DIR`     | `~/Library/Caches/pplr`                               | `contact`, `email`                             | keeps; default per ST0001 design D5                        |
| `PPLR_REFS_DIR`      | `~/Dropbox`                                           | `rename`                                       | keeps                                                      |
| `PPLR_PICTURE_WIDTH` | `160`                                                 | `pictures`                                     | keeps                                                      |
| `PPLR_ICON_DIR`      | `$PPLR_ROOT/templates/icons`                          | `applyicons`                                   | keeps; the default changes                                 |
| `PPLR_HANDLER_APP`   | `~/Applications/Pplr Links.app`                       | `handler`                                      | keeps                                                      |
| `PPLR_MAIL_RECENT`   | `mail-recent` on `PATH`                               | `email`                                        | keeps                                                      |
| `PPLR_TODAY`         | today                                                 | `contact` (test seam)                          | keeps, for every date-dependent command                    |
| `PPLR_CONTACTS_JSON` | none                                                  | `sync` (test seam)                             | keeps, in the `dump` format                                |
| `PPLR_PEOPLE_JSON`   | set by the scripts                                    | the Python helpers                             | gone: internal                                             |
| `PPLR_TEST_DATA`     | `tests/fixtures`                                      | the tests                                      | test-only, and never again a deletion target               |

## 9. External tools

| Tool                                               | Used by                                                              | In the port                                                       |
| -------------------------------------------------- | -------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `jq`                                               | `reindex`, `search`, the `sync --plan` summary, `doctor`             | gone at run time; the tests may keep it                           |
| `perl`                                             | `pictures`, `rename`                                                 | gone                                                              |
| `uv`, `python3`, PyYAML, openpyxl                  | `contact`, `tags`, `sync --plan` and `--apply-plan`, `email`, `open` | gone                                                              |
| `swiftc`                                           | the engine, built at run time                                        | gone from run time; the sidecar is built at build or install time |
| `claude`                                           | `search`                                                             | stays, as a subprocess                                            |
| `mail-recent`                                      | `email`                                                              | stays, as a subprocess; **not installed on rhadamanth**           |
| `fileicon`                                         | `applyicons`, `setpicfordir`, `setpicsfordirs`                       | stays                                                             |
| `osascript`, `sips`                                | Contacts.app writes and photos                                       | stay, driven from Rust                                            |
| `osacompile`, PlistBuddy, `lsregister`, `codesign` | `handler`                                                            | stay                                                              |
| `open`, `pbcopy`, `curl`, `emacsclient`, `less`    | `open` and `linkedin`, `cp`, the CMS probe, `edit`, `help --details` | stay, or are replaced where trivial (the CMS probe)               |

## 10. Tests and CI

| File                 | Tests | Notes                                                                                 |
| -------------------- | ----- | ------------------------------------------------------------------------------------- |
| `pplr_contact.bats`  | 7     | needs `uv` and fails without it rather than skipping                                  |
| `pplr_core.bats`     | 11    | includes checks of bash internals: `bin/pplr` executable, env defaults, README paging |
| `pplr_doctor.bats`   | 7     | checks bash internals: executable scripts, bash version                               |
| `pplr_meetings.bats` | 6     |                                                                                       |
| `pplr_new.bats`      | 7     |                                                                                       |
| `pplr_refresh.bats`  | 1     |                                                                                       |
| `pplr_search.bats`   | 9     | mock `claude` on `PATH`; one test skips without `uv`                                  |
| `pplr_sync.bats`     | 42    | Contacts from `PPLR_CONTACTS_JSON`; builds the Swift engine; four skip without `uv`   |
| `pplr_tag.bats`      | 5     | four skip without `uv`                                                                |
| `pplr_tags.bats`     | 4     | all skip without `uv`                                                                 |

The last CI run (2026-10-13, run 38407533112) reported success. What actually ran:

| Runner        | ok  | not ok | skipped | Failing tests                                                                                    |
| ------------- | --- | ------ | ------- | ------------------------------------------------------------------------------------------------ |
| macos-latest  | 77  | 9      | 13      | the 7 contact tests (no `uv` in CI), 21 (`doctor --fix`), 33 (`new` directory structure)         |
| ubuntu-latest | 45  | 41     | 13      | the 7 contact tests, 33, 41 (`search` by role), and every test that needs the Contacts framework |

Hazards, all fixed before anything else (ST0001 WP01):

- **False green**: `run_tests.sh` exits with `rm -rf`'s status.
- **Deletion**: `setup()` honours an inherited `PPLR_TEST_DATA`, teardown runs `rm -rf "$PPLR_TEST_DATA"/*` and the runner `rm -rf "$PPLR_TEST_DATA"`; run against a real path, it wipes it (dc).
- **No seam**: `setup()` sets `PPLR_BIN_DIR="$PPLR_ROOT/bin"`, so the suite always tests the checkout's bash (dc).
- **Tracked runtime files**: `tests/fixtures/.cache/email-last.json`, `.mock_bin/claude` and `.mock_home/.claude/local/claude` are generated by the tests but tracked, and the runner deletes them.

## 11. Defects and drift

| #   | Finding                                                                                                           | Where                              | Disposition                                     |
| --- | ----------------------------------------------------------------------------------------------------------------- | ---------------------------------- | ----------------------------------------------- |
| 1   | CI false green                                                                                                    | `run_tests.sh`, `test.yml`         | ST0001 WP01                                     |
| 2   | inherited `PPLR_TEST_DATA` is a deletion target                                                                   | `test_helper.bash`, `run_tests.sh` | ST0001 WP01                                     |
| 3   | the suite cannot target another binary                                                                            | `test_helper.bash`                 | ST0001 WP01                                     |
| 4   | generated fixtures tracked, and deleted by the runner                                                             | `tests/fixtures/`                  | ST0001 WP01                                     |
| 5   | the search query goes into `sed` unescaped: a `/` in a query breaks it                                            | `pplr_search`                      | gone with the port                              |
| 6   | `setpicsfordirs` word-splits names, so multi-word names fail                                                      | `pplr_setpicsfordirs`              | fixed in the port                               |
| 7   | `edit` ignores `$VISUAL` and `$EDITOR` (hard-coded `emacsclient`) though its comment says `$EDITOR`               | `pplr_edit`                        | kept for parity; flagged for hv                 |
| 8   | `meetings` with no argument lists today only; the README says "recent meetings"                                   | `pplr_meetings`, README            | behaviour kept; the README corrected at cutover |
| 9   | `new` writes "Matthew Sinclair" into every verblock                                                               | `pplr_new`                         | kept for parity; flagged for hv                 |
| 10  | `handler` bakes `$PPLR_ROOT/bin/pplr` into its app                                                                | `pplr_handler`                     | redesigned in the port                          |
| 11  | `help --details` pages the checkout's README                                                                      | `pplr_help`                        | the README is embedded in the binary            |
| 12  | two version sources once `Cargo.toml` exists                                                                      | `VERSION.md`                       | `VERSION.md` retired                            |
| 13  | 25 copies of the environment defaults                                                                             | `bin/*`                            | gone with the port                              |
| 14  | an unknown command's error goes to stdout                                                                         | `bin/pplr`                         | stderr in the port, unless a test pins stdout   |
| 15  | PyYAML reads YAML 1.1, so a bare `yes`, `no`, `on` or `off` in `tags.yaml` is a boolean                           | `tags.py`, `contact.py`            | pinned by a golden in WP06                      |
| 16  | `usage-rules.md` and the README describe the retired Claude tagger and `.index/tags.json`                         | docs                               | rewritten at cutover                            |
| 17  | the CHANGELOG has `## [Unreleased]` with `### Fixed` twice; devbin's release core reads `## [X.Y.Z] - unreleased` | `CHANGELOG.md`                     | ST0002                                          |
| 18  | `.gitignore` is an Elixir/Mix template; a Finder `Icon\r` is tracked at the root                                  | repo                               | ST0001 WP02                                     |
| 19  | `mail-recent` is not installed on rhadamanth, so `pplr email` cannot run here                                     | the machine                        | flagged for hv                                  |
| 20  | `setup.sh` and `.env.example` write and document `PPLR_ROOT` and a `PATH` entry                                   | repo                               | retired at cutover                              |

## 12. Installed today

- `~/.local/bin/pplr` is a symlink to `~/Devel/prj/Pplr/bin/pplr`: the user runs the checkout.
- `PPLR_DATA` is unset in the login shell, so the default applies.
- Toolchain on rhadamanth: Rust 1.99.1 from Homebrew (no rustup, so a `rust-toolchain.toml` would be ignored), Swift 6.4.2, bats 1.14, `jq`, `uv`, `fileicon`, `claude`; no `mail-recent` (dc, cc).
- The estate runs `cargo` and `bats` in single-slot heavy lanes shared with the Devbin and Utilz sessions, granted through `intent estate slots`.

## 13. Precedent for the port and the release

Summarised from Cldr, Ccusage and devbin 0.1.19; ST0002's design carries the release detail.

- **House style (Cldr is the newest and canonical; Ccusage ships Python, so it is not a model for tooling):** one crate at the repo root; edition 2024; `rustfmt.toml` with `tab_spaces = 2`; `unsafe_code = "forbid"`, `unwrap_used` and `expect_used` denied; a 9-line `main.rs` calling `cli::run`; `lib.rs` as the module list; a hand-rolled CLI with a `HELP` constant and small `take_value` and `take_flag` helpers (no clap, anyhow or thiserror); `Box<dyn Error + Send + Sync>` errors that name the file or setting; "pure core, thin shell", with writes through a temp file; each dependency commented with why; unit tests in each module returning `Result`; one integration file, `tests/cli.rs`, with a `Sandbox` that redirects HOME, XDG and PATH (devbin's `check autotests` refuses a second `tests/*.rs`); no hand-written counts in tests.
- **`forbid` cannot be relaxed** by an inner `#[allow]`, so framework bindings such as `objc2-contacts` would need the crate lint lowered or a second crate. Ccusage reaches macOS through the safe `security-framework` crate; nothing here uses objc2.
- **TCC**: Cldr's Full Disk Access grant lands on the terminal that runs it. A CLI's Contacts grant does the same today ("the app running pplr, eg iTerm or Terminal"), which is why the engine needs no signing.
- **Release**: Cldr renders a source-build formula into `matthewsinclair/homebrew-cldr`; Ccusage ships Developer ID signed tarballs (for Keychain access) and builds from source on Linux. devbin 0.1.19's release core and `formula: mode: source` cover what Pplr needs without either project's bespoke tools.
