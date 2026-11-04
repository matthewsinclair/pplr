---
st_id: ST0003
title: Post-port simplifications, then 2.0.1 by one command
---

# ST0003: Post-port simplifications, then 2.0.1 by one command -- Acceptance

> **THIS FILE IS A GENERATED VIEW, AND A ROW AUTHORED HERE IS DISCARDED BY THE NEXT SYNC.** The acceptance contract is canon in the thread model; this file renders it. Acceptance Criteria (AC) are the ratified completeness boundary; Acceptance Tests (AT) are the small red-to-green tests that prove them.
>
> Done = every AC is covered by a GREEN AT, or (for a non-test AC) its named evidence is satisfied, AND the AC set is the ratified full boundary. Done is read from this map, never from a hand-ticked box.
>
> Test-backed satisfaction is COMPUTED from covering green ATs and never stored -- storing it would be double truth. An AC has four states, not two: beyond satisfied and unsatisfied, a requirement can be **descoped** to a named thread or **withdrawn** with its reason on the record. Both are non-blocking and both are reported separately, so a thread that descoped half its contract looks like one.

## Acceptance Criteria

### ST-level

- AC-00.1 Behaviour is preserved: every frozen golden replays unchanged, apart from a ruled divergence's, which changes in its own commit naming the ruling; check all and test all exit 0 at the thread's tip, measured in a fresh clone. -- satisfied: no (computed)

### WP-01 -- Pair: how as an enum, and has_inetloc's not-read state its own (status: Done)

- AC-01.1 Pair.how is an enum that names the five ways once, and the JSON and the report print 2.0.0's words; has_inetloc tells not read from absent. -- satisfied: yes (computed)

### WP-02 -- workbook::write fills Merged by header, as the reader and the styling do (status: Done)

- AC-02.1 workbook::write places each cell of the sheets the reader reads, Merged and Contacts, by its column's header, as the reader and the styling do, and a written workbook reads back as at 2.0.0. -- satisfied: yes (computed)

### WP-03 -- One macOS gate for every command that needs a macOS service (status: Done)

- AC-03.1 One helper gates every command that needs a macOS service, and each refusal keeps 2.0.0's message and exit code. -- satisfied: yes (computed)

### WP-04 -- One walker for grep, links and rename, or the record of why not (status: Done)

- AC-04.1 grep, links and rename walk through one walker that keeps each command's order, dot-name and symlink rules and names each folder it cannot read on stderr, one line each, carrying on with the exit code unchanged; or the package records why one walker is not simpler. -- satisfied: yes (computed)

### WP-05 -- One picture rule for the engine, pplr pictures and the index (hv's ruling) (status: Done)

- AC-05.1 The engine, pplr pictures and the index choose a person's picture by one rule, and CHANGELOG's divergences name it. -- satisfied: yes (computed)

### WP-06 -- pplr 2.0.1 by one command (status: Done)

- AC-06.1 (non-test) pplr 2.0.1 is released by bin/devbin release v2.0.1 alone: the signed and notarised asset attached, the formula published to the tap, and brew upgrade pplr reading 2.0.1. -- evidence: hv's one command, bin/devbin release v2.0.1, re-run at 97713b7 on 2026-11-04 after its first run stopped at test all (a version written by hand in tests/cli.rs, fixed by 97713b7): release commit 733e127, tag v2.0.1, candidate CI 41800550695 green on macOS and Linux. Its after:, bin/devbin post-cut, signed and notarised the asset and published the release https://github.com/matthewsinclair/pplr/releases/tag/v2.0.1 (2026-11-04T08:37:35Z) with pplr-2.0.1-aarch64-apple-darwin.tar.gz and SHA256SUMS 4f6593acc2cf149aced3c3798ab34e92c697f51739f70b9528e608a6f7170bb0. dc checked a fresh download: shasum -c OK; codesign --verify --strict and spctl accept pplr, pplr-contacts and Pplr Links.app (team 76BQL8L47U); no ._ entry. The formula went to matthewsinclair/homebrew-pplr as bf5b108 with that sha256, version 2.0.1 and source_commit 733e127, and this machine's tap was pulled. Then hv's brew upgrade pplr took 2.0.0 to 2.0.1, brew's codesign verify running at install for all three. dc re-read it: brew info reads installed 2.0.1; the keg /opt/homebrew/Cellar/pplr/2.0.1's pplr, pplr-contacts and Pplr Links.app pass codesign --verify --strict (team 76BQL8L47U); and pplr --version, pplr on PATH being brew's, prints pplr 2.0.1 (brew, 733e12742185). -- satisfied: yes

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Pair: how as an enum, and has_inetloc's not-read state its own (status: Done)

- AT-01.1 `src/sync.rs` -- covers AC-01.1 -- status: green -- sync::tests::each_way_has_its_word_and_an_inetloc_not_read_is_not_absent: five people, one matched each of the five ways, the JSON's (person, how, hasInetloc) rows against 2.0.0's words; matches() leaves every .inetloc None, check() reads a written .inetloc as Some(true) and the others as Some(false), and hasInetloc reads false for not read and absent alike; red at two mutants of b335c08 in slot 01M6JFTPZQKGNGPP0YDMJXYSGE: f3c8b2c (as_str gives Phone) exit 101, the JSON row for P/Phone, Pat reading Phone, and 9121e93 (matches() sets Some(false)) exit 101, matches() read an .inetloc; green at b335c08 (on main as e61e0ae) in the same slot: check all exit 0, test all exit 0 (cargo lib 108 passed 1 ignored, cli 39 passed 1 ignored with the golden replay, bats 106 ok 0 not ok 1 skip)

### WP-02 -- workbook::write fills Merged by header, as the reader and the styling do (status: Done)

- AT-02.1 `src/workbook.rs` -- covers AC-02.1 -- status: green -- workbook::tests::each_cell_is_written_under_its_header_whatever_the_table_order: Merged and Contacts written through sheet() with both column tables reversed; each cell read back under its own header, and the decision and its card through decisions(); red at mutant 2e16260 of 0bf1482 (by_header fills by position) in slot 01M6JGP5MNCGNGF5E6Z33RWT5N: exit 101, decisions() refusing BadDecision for Other candidates of Jon; green at 0bf1482 (on main as 1f710df) in the same slot: check all exit 0, test all exit 0 (cargo lib 110 passed 1 ignored, cli 39 passed 1 ignored with the golden replay, bats 106 ok 0 not ok 1 skip); sync --plan on the golden fixture against brew's 2.0.0: the same 13 zip parts, only docProps/core.xml's write stamps differ, xlsx-cells the same 364 lines

### WP-03 -- One macOS gate for every command that needs a macOS service (status: Done)

- AT-03.1 `src/tools.rs` -- covers AC-03.1 -- status: green -- tools::tests::the_gate_passes_on_macos_and_elsewhere_refuses_with_the_message_alone: tools::macos_only, the one gate, is Ok on macOS and elsewhere an Err displaying exactly the command's message, which main prints before exit 1; beside it AT-00.7 in tests/pplr_core.bats holds each gate's whole 2.0.0 stderr line and exit 1 on CI's Linux leg; red at mutant 1994dff of 34a130f (macos_only refuses on macOS too) in slot 01M6JHGVSWP34ZRH39C8VM3GMT: exit 101, Err against Ok; green at 34a130f (on main as 90919e6) in the same slot: check all exit 0, test all exit 0 (cargo lib 111 passed 1 ignored, cli 39 passed 1 ignored, bats 106 ok 0 not ok 1 skip); AT-00.7's rows proven on macOS at that mutant with its Darwin skip lifted, 1 of 1 ok; each of 2.0.0's five refusal texts verbatim once as macos_only's argument

### WP-04 -- One walker for grep, links and rename, or the record of why not (status: Done)

- AT-04.1 `src/fs.rs` -- covers AC-04.1 -- status: green -- fs::tests::each_walk_keeps_its_commands_rules: on one tree, grep's list in name order with dot names below the top, links' with no dot name and a symlinked note kept as its file, search's with dot names, no symlink and a skip folder passed over, named_after's by prefix, a symlink too, a folder before its contents; none enters a symlinked folder; red at two mutants of d1ee974 in slot 01M6JJ8NE9K011PKR0R11S2C1B: 9054310 (the walk enters a symlinked folder) exit 101, grep's list gaining A/link-dir/b.md, and 73f0083 (no name order) exit 101, grep's list in the folder's order; green at 69b17c0, 2.0.0's own four walks with the test, exit 0; green at d1ee974 (on main as d2f432f) in the same slot: check all exit 0, test all exit 0 (cargo lib 112 passed 1 ignored, cli 39 passed 1 ignored with the golden replay, bats 107 ok 0 not ok 1 skip)
- AT-04.2 `tests/pplr_search.bats` -- covers AC-04.1 -- status: green -- tests/pplr_search.bats, pplr grep names a folder it cannot read on stderr, and still searches the rest: a chmod 000 folder beside a person; exit 0, the person's match, and exactly pplr: cannot read <folder>: Permission denied (os error 13) on stderr; skipped under uid 0, which reads such a folder; red against brew's pplr 2.0.0 (dd6dc5b) in slot 01M6JJ8NE9K011PKR0R11S2C1B: not ok on the stderr line, exit 1; green at d1ee974 (on main as d2f432f) in the same slot, ok in test all's bats run (107 ok, 0 not ok); its id joined the test's comment in 585cb6d (on main as 2dab7ed), whose bats run in slot 01M6JKBDGVM723DT43DAE776NB was 107 ok, 0 not ok

### WP-05 -- One picture rule for the engine, pplr pictures and the index (hv's ruling) (status: Done)

- AT-05.1 `src/tree.rs` -- covers AC-05.1 -- status: green -- tree::tests::the_engine_pictures_and_the_index_take_one_picture: on one tree, two pictures, one named otherwise and none, the engine (tree::people), the index (index::letters) and pplr pictures (its img line) take the same file, the first picture in About/ by name; with none the index names <First Surname> (Picture).jpg, as 2.0.0 did; red at ea6ed7b, 2.0.0's pplr pictures and index with the test, in slot 01M6JKBDGVM723DT43DAE776NB: exit 101, the index naming ./L/Lovelace, Ada/About/Ada Lovelace (Picture).jpg where the engine takes A (Picture).png; green at 585cb6d (on main as 2dab7ed) in the same slot: check all exit 0, test all exit 0 (cargo lib 113 passed 1 ignored, cli 39 passed 1 ignored with the golden replay unchanged, bats 107 ok 0 not ok 1 skip)

### WP-06 -- pplr 2.0.1 by one command (status: Done)

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
