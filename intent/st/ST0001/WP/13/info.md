---
wp_id: WP-13
title: Cutover
scope: M
status: WIP
---

# WP-13: Cutover

## Objective

Remove the fallback and every legacy file, prove parity read-only on the live tree, and bring the docs up to date. This leaves one Rust binary ready for ST0002's release.

**Scope**

- **Removals**: the fallback, `bin/pplr`, `lib/`, `setup.sh`, `.env.example`, `VERSION.md`, and any `src/` remnant. `templates/` stays as the embedding source.
  - From WP11, kept until now because hv's live pplr runs the legacy code (D3): `bin/pplr_sync`, `src/pplr-contacts/workbook.py`, and the plan and applyplan modes in `swift/pplr-contacts/main.swift` (the file stays: it is also the Contacts sidecar's source). The setup_file engine build in `tests/pplr_sync.bats` goes with them.
  - From WP14, for the same reason: `bin/pplr_email` and `src/pplr-email/email_list.py`.
- **The no-Python check** joins the gate (AC-00.1).
- **Live read-only parity** on rhadamanth (AC-00.5):
  - check the legacy code out from history into a scratch worktree;
  - run the read-only commands over the live People tree under both implementations;
  - compare the outputs and record them here.
  - Nothing writes to the live tree.
- **Docs**:
  - README: the commands, the brew install (ST0002), and a maintainer section for dev and opt;
  - `usage-rules.md`: the current tags model;
  - CHANGELOG: `## [2.0.0] - unreleased`;
  - `intent/llm/ARCHITECTURE.md`: the Rust architecture;
  - the project-specific section of `CLAUDE.md`: pointers to these.

**Criteria**: AC-00.1 to AC-00.8, the thread's own criteria, hold at the end of this WP.

**As built** (cc; the series C1 to C10, verified by vc on cc/wp13 in decision 16, to land after hv's cutover):

1. The legacy code and the fallback go (C1): `bin/pplr` and its command scripts, `lib/engine.sh`, `setup.sh`, `.env.example`, `VERSION.md`, the Python helpers under `src/pplr-*` and `src/fallback.rs`, with the tests that leaned on them. `PPLR_UNDER_TEST` defaults to the dev build (AC-01.3 reworded, vc decision 13).
2. The Contacts sidecar does only dump, backup and group (C2), and `IN-SW-CODE-006` is on again (rule 9).
3. The goldens are frozen as the legacy's record (C3): the capture, its scenarios and the fakes only the legacy reached go.
4. No Python, checked (C4): `tools/nopython` in `devbin check nopython`, the pre-commit guards and CI (AC-00.1).
5. The thread's own acceptance tests (C5): AT-00.2, AT-00.6, AT-00.7 (every macOS gate, on Linux) and AT-00.9.

**Fixes** (design D10: each changes no data file and no tested output; the CHANGELOG lists them):

- `pplr tags` and `pplr contact` with no command name themselves in their usage, where the port had kept the names `tags.py` and `contact.py` (vc decision 14; C6).
- `doctor` no longer asks for `jq`, which only the legacy scripts ran (C7).
- `grep` and `grep -t` list people in name order, the same on every filesystem, where the legacy's `grep -r` and `find` gave the filesystem's own directory order (vc decision 15; C9).
- `grep` prints a matched line as the file holds it, where the legacy's `IFS=: read` dropped a trailing colon that was the line's only one (vc decision 15; C9).

**Port bugs the live parity found, fixed to match the legacy**:

- `rename` tested its running-text skips against a hit's text alone, so it listed the person's own About as left unchanged; the legacy's `grep -vF` tested the whole hit, path and text (C8).
- `links` walked into symlinked directories, so a link out of the tree was searched and one back up it sent the walk round and round; the legacy's FileManager enumerator never descends one (C10).
- `sync` compared a card's values with pplr's as bytes, so `sync --check` listed a person as differing on a LinkedIn slug the card holds decomposed and the About file precomposed; the legacy engine held each value as a Swift string, whose equality is canonical. Every value it compared or keyed that way, in `src/sync.rs` and `src/plan.rs`, is now compared and keyed by its decomposed form and shown as written; the commit lists the sites (C11, f9798b3; vc decision 22).

**Live read-only parity** (AC-00.5), on rhadamanth, 2026-11-03. Verdicts only (design D11):

- The legacy: main d7ee43d, the code hv's live pplr ran, exported with `git archive` into a scratch folder, its Swift engine built there. The Rust: the series' tip a44f79b, whose code tree is the one that lands, published as an opt tree; for the sync rows' last run, C11's c57cf12 (f9798b3's tree), published the same way.
- Each row ran under both, from the same folder with the same settings, inside `sandbox-exec` with every write to the tree denied; caches and backups went to scratch. The tree's listing (path, size, time) was the same before and after.
- Compared: stdout, stderr and the exit code. No output is recorded here, and the outputs were deleted once each difference was classified. Two earlier runs gave nothing to record: the first through a slip in the runner, the second stopped at `links` on the port's symlink walk, fixed as C10.
- hv ran the rows that need a macOS grant, from hv's own terminal, which has Full Disk Access. hv's first run compared nothing: every sync row was refused Contacts access on both sides, and the legacy's `email` never found `mail-recent`, the external tool it reads Mail through, because the runner clears every `PPLR_` setting. Both runs now get that setting, which the port never reads, and hv's second run compared the email rows.
- The sync rows need Contacts access, which a terminal app can be granted only if it declares a use of Contacts: hv's does not, and iTerm does. hv's third run, from iTerm, whose grant holds inside `sandbox-exec`, compared the sync rows, and found `sync --check` and `--check --json` different by the port bug C11 fixes. hv's fourth run, from iTerm against C11's build, compared all five sync rows again.

| Rows                                                                                                                                      | Verdict                                                                                                                                              |
| ----------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `count`, `json`, `index`                                                                                                                  | same                                                                                                                                                 |
| `help --details`                                                                                                                          | same                                                                                                                                                 |
| `meetings`: a year, a month, a range                                                                                                      | same                                                                                                                                                 |
| `tags check`, `tags render --dry-run`                                                                                                     | same                                                                                                                                                 |
| `contact due`, `contact due --all`, `contact scan --dry-run`                                                                              | same                                                                                                                                                 |
| `refresh next`, `refresh next` with `--older-than`, `refresh status`                                                                      | same                                                                                                                                                 |
| `pictures --dry-run`                                                                                                                      | same                                                                                                                                                 |
| `links` and `links --json` over the tree (dry runs)                                                                                       | same                                                                                                                                                 |
| `rename --dry-run` (one person)                                                                                                           | same                                                                                                                                                 |
| For every person: `about`, `cp -o`, `cp -o -t linkedin`, `resolve`, `open --print`, `tag`, `tags show`, `contact show`, `contact context` | same                                                                                                                                                 |
| `email`, `email inbox` (hv's terminal)                                                                                                    | same                                                                                                                                                 |
| `sync --check`, `--check --json`, and `--link`, `--pull`, `--photos` (dry runs), from iTerm against C11's build                           | same                                                                                                                                                 |
| `help`                                                                                                                                    | different by design: generated from the Rust command table (WP04), which carries ST0002 WP03's `install` and `use` and each WP's usage text          |
| `version`                                                                                                                                 | different by design: the version line (design D7, ST0002 WP03), for `VERSION.md`'s number                                                            |
| `doctor`                                                                                                                                  | different by design: WP12's checks, with ST0002's install section and design D5's legacy-folder warnings; neither doctor reports a fault in the tree |
| `grep`, `grep -t`                                                                                                                         | different by design: vc decision 15 (name order; the line as the file holds it)                                                                      |

Unexplained differences: none.

## Acceptance

Acceptance Criteria for this work package are RENDERED into `ST0001/acceptance.md`, under the `WP-13` heading. THAT FILE IS A GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in the thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
