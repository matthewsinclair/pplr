---
st_id: ST0001
title: Port Pplr from shell scripts to Rust
---

# ST0001: Port Pplr from shell scripts to Rust -- Acceptance

> **THIS FILE IS A GENERATED VIEW, AND A ROW AUTHORED HERE IS DISCARDED BY THE NEXT SYNC.** The acceptance contract is canon in the thread model; this file renders it. Acceptance Criteria (AC) are the ratified completeness boundary; Acceptance Tests (AT) are the small red-to-green tests that prove them.
>
> Done = every AC is covered by a GREEN AT, or (for a non-test AC) its named evidence is satisfied, AND the AC set is the ratified full boundary. Done is read from this map, never from a hand-ticked box.
>
> Test-backed satisfaction is COMPUTED from covering green ATs and never stored -- storing it would be double truth. An AC has four states, not two: beyond satisfied and unsatisfied, a requirement can be **descoped** to a named thread or **withdrawn** with its reason on the record. Both are non-blocking and both are reported separately, so a thread that descoped half its contract looks like one.

## Acceptance Criteria

### ST-level

- AC-00.1 No Python: `git ls-files '*.py'` is empty, no committed file runs `python3` or `uv`, and the project's check gate fails on either. -- satisfied: no (computed)
- AC-00.2 One binary: every command the shell pplr offered (the 30 in recon section 4) is served by the Rust `pplr` with the same arguments and flags, and no `bin/pplr*` script, `lib/engine.sh` or run-time build remains. -- satisfied: no (computed)
- AC-00.3 Parity: the bats suite passes against the Rust binary: every test on macOS, and on Linux every test that needs no macOS service. A test retired as shell-specific is withdrawn on the record, with its reason. -- satisfied: no (computed)
- AC-00.4 Byte-identical data: over the synthetic fixture tree, every file the Rust binary writes under PPLR_DATA is byte-identical to the legacy golden, apart from the time-dependent fields listed beside the goldens (WP03). The plan workbook matches by content. -- satisfied: no (computed)
- AC-00.5 (non-test) Live read-only parity: on rhadamanth, over the live People tree, the read-only commands print the same output under the legacy code (checked out from history into a scratch worktree) and under the Rust binary. Nothing writes to the live tree. -- satisfied: no
- AC-00.6 Install-independence: the binary runs from any directory and any install location, with no PPLR_ROOT, PPLR_BIN_DIR, PPLR_TEMPLATE_DIR or checkout. PPLR_DATA, then PPLR_DIR, names the data. Templates, icons and the README come from the binary, or from the data tree's _pplr/ overrides. -- satisfied: no (computed)
- AC-00.7 Linux honesty: on Linux, every command that needs a macOS service exits non-zero with a message naming why, and every other command works. -- satisfied: no (computed)
- AC-00.8 (non-test) One version, current docs: Cargo.toml is the only version source, and README, usage-rules.md, CHANGELOG and intent/llm/ARCHITECTURE.md describe the Rust binary. -- satisfied: no
- AC-00.9 Code and data never mix: no name, marker, email or other content from the People tree is in Pplr's tree or history, tests use synthetic data alone, and a leak check that reads PPLR_DATA at run time fails the pre-commit gate on any match. -- satisfied: no (computed)

### WP-01 -- Test safety, the parity seam and an honest runner (status: Done)

- AC-01.1 The bats runner and CI exit non-zero when any test fails. -- satisfied: yes (computed)
- AC-01.2 No test or runner deletes outside its own temporary directory: an inherited PPLR_TEST_DATA, or any other path from the environment, is never a deletion target. -- satisfied: yes (computed)
- AC-01.3 One setting points the bats suite at either implementation, and the default is the legacy code. -- satisfied: yes (computed)
- AC-01.4 (non-test) The baseline against the legacy code is recorded on this thread for rhadamanth and both CI runners, with a one-line triage for each failing test. -- evidence: baseline.md on ST0001: rhadamanth slot 01M6EBWZSD9THBN1Q549VEK46T (99/99 legacy); CI run 41467789463 on 1a545a6 (macOS 93 ok / 9 not ok, Ubuntu 61 ok / 41 not ok, 13 skipped each), each failure triaged -- satisfied: yes

### WP-02 -- Rust scaffold under devbin, with the fallback (status: WIP)

- AC-02.1 `bin/devbin check all` and `bin/devbin test all` run the cargo gates (fmt; clippy with unsafe forbidden and unwrap and expect denied; test) and the bats suite against the Rust binary, and both pass on macOS and on Linux CI. -- satisfied: no (computed)
- AC-02.2 Every command not yet ported runs its legacy script through the Rust binary, with the environment bin/pplr sets, so the whole bats suite passes against the Rust binary. -- satisfied: no (computed)

### WP-03 -- Golden capture from the legacy code (status: Done)

- AC-03.1 Goldens exist for every writing and reporting command over the synthetic tree, captured from the legacy code, and a test in tests/cli.rs replays each against the Rust binary, skipping only commands still on the fallback. -- satisfied: yes (computed)

### WP-04 -- Core model and the read commands (status: Not Started)

- AC-04.1 resolve, count, about, cp, open, linkedin, edit, meetings, refresh, grep, help and version run natively in Rust, and their goldens and bats tests pass. -- satisfied: no (computed)

### WP-05 -- Indexes, new person, pictures and search (status: Not Started)

- AC-05.1 json, index, reindex, new, pictures and search run natively in Rust, and their goldens and bats tests pass; a search query is never interpolated into a command line. -- satisfied: no (computed)

### WP-06 -- Tags (status: Not Started)

- AC-06.1 tag and tags run natively in Rust, and their goldens and bats tests pass. -- satisfied: no (computed)

### WP-07 -- Contact cadence (status: Not Started)

- AC-07.1 contact runs natively in Rust, and the contact goldens and bats tests pass; `log --email` is covered by AC-14.1. -- satisfied: no (computed)

### WP-08 -- Rename, links and the pplr:// handler (status: Not Started)

- AC-08.1 rename, links and handler run natively in Rust; their goldens and bats tests pass, and handler --install registers the tree's prebuilt, signed Pplr Links.app by its stable path, never a Cellar path (ST0002 ruling 7). -- satisfied: no (computed)

### WP-09 -- The Contacts store and the Swift I/O sidecar (status: WIP)

- AC-09.1 Sync reaches Contacts only through the store interface. The JSON store reads and writes the dump format, and the Swift sidecar does only dump, backup and group, built ahead of time and never at run time. -- satisfied: no (computed)
- AC-09.2 (non-test) On rhadamanth, the sidecar reads the live Contacts store, and the Rust `pplr sync --check --json` matches the legacy engine's report (read-only). -- satisfied: no

### WP-10 -- Sync: check, link, pull, photos and backup (status: Not Started)

- AC-10.1 sync --check, --link, --pull, --photos and --backup run in Rust against the JSON store, and their goldens and bats tests pass. -- satisfied: no (computed)

### WP-11 -- The sync plan and its workbook (status: Not Started)

- AC-11.1 sync --plan and --apply-plan run in Rust. The workbook's four sheets, review columns, Decision list and highlighting match the legacy workbook by content. -- satisfied: no (computed)

### WP-12 -- Doctor and Finder icons (status: Not Started)

- AC-12.1 doctor, applyicons, setpicfordir and setpicsfordirs run natively in Rust. doctor reports one row per check, each saying what to fix, and its bats tests test behaviour, not bash internals. -- satisfied: no (computed)

### WP-14 -- pplr's own mail reader (status: Not Started)

- AC-14.1 pplr reads recent and inbox mail itself, read-only, from Mail's Envelope Index (no mail-recent, no Python). email's listing and cache and contact log --email N and --email <Message-ID> match their goldens, against a synthetic SQLite fixture. -- satisfied: no (computed)
- AC-14.2 pplr email --json is a versioned contract (a schema field, documented fields and exit codes): it exits non-zero naming the cause on no Full Disk Access or an unrecognised Envelope Index schema, and never returns rows from a schema it did not verify. -- satisfied: no (computed)

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Test safety, the parity seam and an honest runner (status: Done)

- AT-01.1 `tests/harness.bats` -- covers AC-01.1 -- status: green
- AT-01.2 `tests/harness.bats` -- covers AC-01.2 -- status: green
- AT-01.3 `tests/harness.bats` -- covers AC-01.3 -- status: green

### WP-02 -- Rust scaffold under devbin, with the fallback (status: WIP)

_(no tests in this group)_

### WP-03 -- Golden capture from the legacy code (status: Done)

- AT-03.1 `tests/cli.rs` -- covers AC-03.1 -- status: green -- every_golden_of_a_ported_command_replays: 54 cases captured (tests/golden); proved green over all 54 through the fallback from a clean checkout of deebdee, and red on a mutated copy naming both changed cases

### WP-04 -- Core model and the read commands (status: Not Started)

_(no tests in this group)_

### WP-05 -- Indexes, new person, pictures and search (status: Not Started)

_(no tests in this group)_

### WP-06 -- Tags (status: Not Started)

_(no tests in this group)_

### WP-07 -- Contact cadence (status: Not Started)

_(no tests in this group)_

### WP-08 -- Rename, links and the pplr:// handler (status: Not Started)

_(no tests in this group)_

### WP-09 -- The Contacts store and the Swift I/O sidecar (status: WIP)

_(no tests in this group)_

### WP-10 -- Sync: check, link, pull, photos and backup (status: Not Started)

_(no tests in this group)_

### WP-11 -- The sync plan and its workbook (status: Not Started)

_(no tests in this group)_

### WP-12 -- Doctor and Finder icons (status: Not Started)

_(no tests in this group)_

### WP-14 -- pplr's own mail reader (status: Not Started)

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
