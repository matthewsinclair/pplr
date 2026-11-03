# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - unreleased

pplr is now one Rust binary. It replaces the bash scripts, the Python helpers and the Swift engine that was compiled at run time. Every command keeps its arguments and flags, and the files pplr writes in the People tree are byte for byte the ones the shell version wrote, apart from time stamps and the changes listed below.

### Added

- `pplr sync --check`: a read-only comparison of pplr with Apple Contacts (linked, matched by email or LinkedIn, name-only, ambiguous, only in pplr, group orphans, and differing fields), with `--json`, `--verbose` and `--group`.
- `pplr sync --link`: marks matched Contacts cards as pplr's with a `pplr` URL and the `PPLR` group, and writes `About/<First Surname> (Contacts).inetloc` opening the card in Contacts; a dry run unless `--apply`, with `--name` and `--all-names` for name-only matches.
- `pplr sync --plan`: a review workbook (Merged, pplr, Contacts, How to review) proposing Add, Update, Link only, No change or Skip for every pplr person, with near-miss name scoring (short forms, double-barrelled surnames, one-letter slips, reversed names, shared company or email domain); `--out`, `--json`.
- `pplr sync --apply-plan WORKBOOK`: carries out a reviewed plan workbook (Add as new iCloud cards with photo, Update without removing anything, Link only); a dry run unless `--apply`, with `--limit`, a `.vcf` backup first, and never a card added twice.
- `pplr refresh` (`next`, `stamp`, `status`): when each person was last checked against their profile, for batch and age-based refreshes.
- `pplr contact` (`scan`, `due`, `show`, `log`, `next`, `context`): last contact and next due in `About/contact.yaml`, with cadences by tag from `_pplr/cadence.yaml`.
- `pplr email` (`recent`, `inbox`, `--hours`, `--account`, `--sent`, `--limit`): mail with each Message-ID, senders in pplr marked, read from Mail's own store; `--json` prints the messages as a versioned document for other tools. `pplr contact log --email N` (or a Message-ID) links the message; `pplr contact render` writes `About/<First Surname> (Contact).md`, a read-only view of `contact.yaml`, also written whenever `contact.yaml` is.
- A person's `pplr://` marker (eg `pplr://k/kemp-jon`, or an old one kept as an alias) works wherever a command takes "Surname, First".
- The Contacts shortcut is `About/<First Surname> (Contacts).inetloc`: macOS would not open a `.webloc` holding an `addressbook://` URL. Writing one removes the old `.webloc`.
- `pplr sync --photos`: the pplr picture on linked cards that have no photo, never replacing one.
- `pplr sync --pull`: copies into pplr the emails, phones and LinkedIn a linked card has and pplr lacks, and a company or role where pplr has none; pplr wins, nothing is removed, a card linked across accounts is read as one, a misread `+44` number is left out and listed; `--name`, `--skip`, `--limit`, a copy of each About file first.
- `pplr sync --check` counts linked cards in an account other than iCloud on their own line, since the `PPLR` group cannot hold them; `otherAccount` in the JSON.
- `pplr sync --check` also matches on a shared phone.
- Gmail addresses match with or without dots and `+suffix` (Gmail ignores both), so an update never adds the same address twice.
- `pplr rename`: moves a person's folder and files, updates their About, records the old name in `.index/aliases`, repoints path-style links under `--refs` (default `$PPLR_REFS_DIR` or `~/Dropbox`), lists the old name in running text, and reindexes; `--dry-run`.
- `pplr resolve`, `pplr open pplr://...` (the CMS page, or the file; `--print`), `pplr links` (rewrites Markdown links into People as `pplr://` URLs; dry run unless `--apply`) and `pplr handler` (registers the app beside the binary that opens `pplr://` links, on macOS).
- `pplr tags` (`check`, `show`, `render`, `apply`): tags in one `About/tags.yaml` per person, against `_pplr/vocabulary.yaml`, with hand, auto and inferred sources; `render` links each tag to a generated page in `_tags/` (everyone with the tag, and the tags seen with it) and writes an index.
- `pplr tags index`: `.index/tags_index.json` from each `tags.yaml` and the people list (marker, role, company, picture, tags by facet); `render` rebuilds it. `pplr resolve` and `pplr open` take `pplr://tag/<tag>` (or `tags/`) to the tag's page.
- `pplr pictures`: the person's photo at the top of their About page, idempotent, `--dry-run`.
- pplr config and templates live in `$PPLR_DATA/_pplr/` (`_pplr/templates` was `_Templates`); every walk of the people tree skips folders starting with `_` or `.`.
- Commands take paths relative to where `pplr` was run.
- `pplr sync` resolves a card's old `pplr://` URL through a renamed person's aliases, and `--link` replaces it.
- `pplr sync --backup`: every Contacts card to a dated `.vcf` in `$PPLR_BACKUP_DIR`; `--link --apply` runs it first.
- `pplr version`, `-v` and `--version` print `pplr <version> (<tree>, <commit>)`: the tree the binary runs from (dev, opt or brew) and the commit it was built from, with `+dirty` on a dev build made from uncommitted changes.
- `pplr install` and `pplr use`, for the maintainer: `install` publishes a dev build as an opt tree, and `use dev|opt|brew` points `~/.local/bin/pplr` at one tree, or removes the link so that Homebrew's `pplr` answers.

### Changed

- Every command runs natively in one binary, `pplr`. Nothing is compiled at run time, and nothing needs Python or `uv`: pplr reads the tags' YAML, writes the `sync --plan` workbook and reads it back for `sync --apply-plan` itself.
- Live Contacts go through `pplr-contacts`, a Swift sidecar built ahead of time and installed beside `pplr`. It only reads the cards, writes the `.vcf` backup and finds the `PPLR` group. Card and photo writes still go to Contacts.app through `osascript` and `sips`.
- `pplr email` reads Mail's own message index and accounts store, read-only, instead of running `mail-recent`. The terminal needs Full Disk Access to read them.
- `Cargo.toml` is the one version source.
- The person template, the Finder icons and the README (`pplr help --details`) are inside the binary. The data tree's `_pplr/templates/person` and `_pplr/icons` override them, and `PPLR_ICON_DIR` overrides both icon sets. `PPLR_ROOT`, `PPLR_BIN_DIR`, `PPLR_TEMPLATE_DIR` and `PPLR_CALLER_DIR` are retired: pplr runs from any directory and any install, with no checkout.
- `PPLR_DATA` names the data, and `PPLR_DIR` is its fallback. `pplr doctor` warns when both are set and differ.
- `pplr doctor` no longer looks for `jq`, which nothing pplr runs needs.
- Backups and the cache follow XDG: `PPLR_BACKUP_DIR` defaults to `$XDG_DATA_HOME/pplr/contacts-backups`, and `PPLR_CACHE_DIR` to `$XDG_CACHE_HOME/pplr`. Backups already written stay where they are, and `pplr doctor` names any old `~/Library` directory left behind.
- `PPLR_TODAY`, a date or a date and time, pins today for every command that depends on it, not only `contact`.
- `pplr edit` opens `$VISUAL`, else `$EDITOR`, and refuses, naming both, when neither is set.
- `pplr new` with a LinkedIn URL writes the git author (`git config user.name`) into the new About's verblock, where the shell version wrote a fixed name. It refuses, before it writes anything, when no author is set.
- `pplr handler --install` registers the signed `Pplr Links.app` that ships beside the binary, and builds nothing.
- On Linux, a command that needs a macOS service exits 1 and says why: the live Contacts store, `handler`, `applyicons`, `setpicfordir`, `setpicsfordirs`, `open`, `linkedin`, and `cp` without `-o`. Everything else works there, `sync` included, against a Contacts dump named by `PPLR_CONTACTS_JSON`.
- Where the Rust binary deliberately differs from the shell version:
  - `pplr sync` refuses a missing option value, a non-numeric `--limit` and a trailing `--group`, with the error and the usage, exit 1. The shell version took them as no limit or an empty group, so `--apply-plan wb.xlsx --apply --limit l` wrote every change.
  - `pplr sync --apply-plan` reads the Merged and Contacts sheets by header name, and refuses a sheet that lacks one, naming it. The shell version read them by column position, so a column the reviewer inserted made it read nothing and exit 0.
  - `pplr email` finds each message's person through the About header's Email line, read by the one reader every command shares. A plain address on the Email line now tags, `mailto:x?subject=...` now tags x, and only the first `- Email:` line counts.
  - `pplr contact log` refuses a second `--email`, exit 1, naming why. The shell version logged the date, person and note of the first message with the link of the second.
  - Every command reads an About through one reader. A read decodes it lossily, so an About that is not UTF-8 still gives its header, as the shell version's byte-wise reads did. A command that writes the About back (`sync --pull`, `rename`, `tags render` and `pictures`) refuses one that is not UTF-8, naming it, so pplr never rewrites a byte it could not decode. An About that cannot be read stops the command, naming the file.

### Removed

- The shell version: the `bin/pplr` dispatcher and its `bin/pplr_*` commands, `lib/engine.sh`, the Python helpers (`tags.py`, `contact.py`, `workbook.py` and `email_list.py`) and `VERSION.md`.
- The Claude tagger: `pplr reindex --tags` (and `--stale-only`, `--max-age`) now explains where tags live, and `pplr tag` shows or edits one person's `tags.yaml` (`+tag -tag`) instead of generating tags. `search` and `grep -t` read `tags.yaml` and the index, not `.index/tags.json`.

### Fixed

- `pplr search` puts the query into its prompt as plain text. The shell version spliced it into a `sed` expression, so a `/`, `&` or backslash in the query garbled or broke the search.
- `pplr setpicsfordirs` passes each "Surname, First" whole, so a name with spaces no longer breaks it.
- `pplr tags` and `pplr contact` with no command name themselves in their usage, not `tags.py` and `contact.py`.
- `pplr grep` and `pplr grep -t` list people in name order, the same on every filesystem. The shell version listed them in the directory's own order.
- `pplr grep` prints a matched line as the file holds it. The shell version dropped a trailing colon, so an empty `- Role:` printed as `- Role`.
- Phones written as a Markdown link (`[+49 ...](tel:...)`) are read from the link text, a trunk 0 after a country code (`+44 07...`) is dropped, and numbers apart by a middle dot or slash are read as separate numbers.
- A LinkedIn URL inside a Markdown link no longer keeps the closing bracket in its slug.
- Tests: teardown clears `.index`, so an index one test writes no longer leaks into the next (the intermittent `pplr count` failures).

## [1.0.1] - 2025-07-19

### Added

- `pplr version` command to display current version
- `-v` and `--version` flags to main pplr command for quick version checking
- `-t|--tag` option to `pplr grep` for searching within tag files

### Changed

- Updated `pplr grep` output format to match `pplr search` format with person links and role/company info
- Improved markdown URL formatting in `pplr search` and `pplr grep` to use angle brackets and escape parentheses

### Fixed

- Markdown links now properly render in all markdown viewers by escaping special characters
- Test suite updated to dynamically read version from VERSION.md instead of hardcoding

## [1.0.0] - 2025-07-19

### Added

- Initial release of pplr (Personal Relationship Manager)
- Core commands: new, search, open, meetings, edit, tag, reindex, count, cp, linkedin, about, index
- AI-powered tagging system using Claude
- Natural language search functionality powered by Claude AI
- Smart tag regeneration with `--stale-only` flag for `pplr reindex`
- Configurable staleness threshold with `--max-age` option (eg 30d, 7days, 2weeks)
- Comprehensive help system with --details flag
- BATS test suite with 31 tests
- GitHub Actions CI/CD for automated testing
- Separated code and data directories
- Environment variable configuration
- Search results with clickable markdown links to About files

### Architecture

- Individual tag files stored in `.index/tags.json` for better organization
- Optimized search context file (`tags_index.json`) ~116KB for efficient Claude queries
- Test infrastructure uses absolute paths for better reliability
- Improved mock Claude detection to prevent false positives in production

### Fixed

- All parameters now use consistent firstname-surname order
- Mock Claude detection bug that prevented tag generation
- Test helper path resolution issues
- Search fallback behavior when Claude is unavailable
- Spinner termination message ("Terminated: 15") in search output

### Performance

- Smart tag regeneration reduces unnecessary API calls
- Improved search response times with better fallback handling
- Optimized directory structure for faster file access
