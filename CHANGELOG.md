# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `pplr sync --check`: a read-only comparison of pplr with Apple Contacts (linked, matched by email or LinkedIn, name-only, ambiguous, only in pplr, group orphans, and differing fields), with `--json`, `--verbose` and `--group`. The Contacts side is a Swift engine in `src/pplr-contacts/`, built on first use.
- `pplr sync --link`: marks matched Contacts cards as pplr's with a `pplr` URL and the `PPLR` group, and writes `About/<First Surname> (Contacts).webloc` opening the card in Contacts; a dry run unless `--apply`, with `--name` and `--all-names` for name-only matches.
- `pplr sync --plan`: a review workbook (Merged, pplr, Contacts, How to review) proposing Add, Update, Link only, No change or Skip for every pplr person, with near-miss name scoring (short forms, double-barrelled surnames, one-letter slips, reversed names, shared company or email domain); `--out`, `--json`. The workbook step runs under `uv` with openpyxl.
- `pplr sync --apply-plan WORKBOOK`: carries out a reviewed plan workbook (Add as new iCloud cards with photo, Update without removing anything, Link only); a dry run unless `--apply`, with `--limit`, a `.vcf` backup first, and never a card added twice.
- `pplr sync --check` also matches on a shared phone.
- Gmail addresses match with or without dots and `+suffix` (Gmail ignores both), so an update never adds the same address twice.
- `pplr rename`: moves a person's folder and files, updates their About, records the old name in `.index/aliases`, repoints path-style links under `--refs` (default `$PPLR_REFS_DIR` or `~/Dropbox`), lists the old name in running text, and reindexes; `--dry-run`.
- `pplr resolve`, `pplr open pplr://...` (the CMS page, or the file; `--print`), `pplr links` (rewrites Markdown links into People as `pplr://` URLs; dry run unless `--apply`) and `pplr handler` (a macOS app registered for `pplr://`). The engine build moved to `lib/engine.sh`.
- `pplr tags` (`check`, `show`, `render`, `apply`): tags in one `About/tags.yaml` per person, against `_pplr/vocabulary.yaml`, with hand, auto and inferred sources; `render` links each tag to a generated page in `_tags/` (everyone with the tag, and the tags seen with it) and writes an index; runs under `uv` with PyYAML.
- `pplr tags index`: `.index/tags_index.json` from each `tags.yaml` and the engine's people list (marker, role, company, picture, tags by facet); `render` rebuilds it. The engine gains `people` (everyone without Contacts) and resolves `pplr://tag/<tag>` (or `tags/`) to the tag's page.
- `pplr pictures`: the person's photo at the top of their About page, idempotent, `--dry-run`.
- pplr config and templates live in `$PPLR_DATA/_pplr/` (`_pplr/templates` was `_Templates`); every walk of the people tree skips folders starting with `_` or `.`.
- Commands take paths relative to where `pplr` was run (`PPLR_CALLER_DIR`), although they run from `$PPLR_DATA`.
- `pplr sync` resolves a card's old `pplr://` URL through a renamed person's aliases, and `--link` replaces it.
- `pplr sync --backup`: every Contacts card to a dated `.vcf` in `$PPLR_BACKUP_DIR`; `--link --apply` runs it first.

### Fixed

- Phones written as a Markdown link (`[+49 …](tel:…)`) are read from the link text, a trunk 0 after a country code (`+44 07…`) is dropped, and numbers apart by a middle dot or slash are read as separate numbers.
- A LinkedIn URL inside a Markdown link no longer keeps the closing bracket in its slug.

### Fixed

- Tests: teardown clears `.index`, so an index one test writes no longer leaks into the next (the intermittent `pplr count` failures).

### Removed

- The Claude tagger: `pplr reindex --tags` (and `--stale-only`, `--max-age`) now explains where tags live, and `pplr tag` shows or edits one person's `tags.yaml` (`+tag -tag`) instead of generating tags. `search` and `grep -t` read `tags.yaml` and the index, not `.index/tags.json`.

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
- Configurable staleness threshold with `--max-age` option (e.g., 30d, 7days, 2weeks)
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
