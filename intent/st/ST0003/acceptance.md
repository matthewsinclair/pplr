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

### WP-03 -- One macOS gate for every command that needs a macOS service (status: WIP)

- AC-03.1 One helper gates every command that needs a macOS service, and each refusal keeps 2.0.0's message and exit code. -- satisfied: no (computed)

### WP-04 -- One walker for grep, links and rename, or the record of why not (status: Not Started)

- AC-04.1 grep, links and rename walk through one walker that keeps each command's order, dot-name and symlink rules, or the package records why one walker is not simpler. -- satisfied: no (computed)

### WP-05 -- One picture rule for the engine, pplr pictures and the index (hv's ruling) (status: Not Started)

- AC-05.1 The engine, pplr pictures and the index choose a person's picture by one rule, and CHANGELOG's divergences name it. -- satisfied: no (computed)

### WP-06 -- pplr 2.0.1 by one command (status: Not Started)

- AC-06.1 (non-test) pplr 2.0.1 is released by bin/devbin release v2.0.1 alone: the signed and notarised asset attached, the formula published to the tap, and brew upgrade pplr reading 2.0.1. -- satisfied: no

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Pair: how as an enum, and has_inetloc's not-read state its own (status: Done)

- AT-01.1 `src/sync.rs` -- covers AC-01.1 -- status: green -- sync::tests::each_way_has_its_word_and_an_inetloc_not_read_is_not_absent: five people, one matched each of the five ways, the JSON's (person, how, hasInetloc) rows against 2.0.0's words; matches() leaves every .inetloc None, check() reads a written .inetloc as Some(true) and the others as Some(false), and hasInetloc reads false for not read and absent alike; red at two mutants of b335c08 in slot 01M6JFTPZQKGNGPP0YDMJXYSGE: f3c8b2c (as_str gives Phone) exit 101, the JSON row for P/Phone, Pat reading Phone, and 9121e93 (matches() sets Some(false)) exit 101, matches() read an .inetloc; green at b335c08 (on main as e61e0ae) in the same slot: check all exit 0, test all exit 0 (cargo lib 108 passed 1 ignored, cli 39 passed 1 ignored with the golden replay, bats 106 ok 0 not ok 1 skip)

### WP-02 -- workbook::write fills Merged by header, as the reader and the styling do (status: Done)

- AT-02.1 `src/workbook.rs` -- covers AC-02.1 -- status: green -- workbook::tests::each_cell_is_written_under_its_header_whatever_the_table_order: Merged and Contacts written through sheet() with both column tables reversed; each cell read back under its own header, and the decision and its card through decisions(); red at mutant 2e16260 of 0bf1482 (by_header fills by position) in slot 01M6JGP5MNCGNGF5E6Z33RWT5N: exit 101, decisions() refusing BadDecision for Other candidates of Jon; green at 0bf1482 (on main as 1f710df) in the same slot: check all exit 0, test all exit 0 (cargo lib 110 passed 1 ignored, cli 39 passed 1 ignored with the golden replay, bats 106 ok 0 not ok 1 skip); sync --plan on the golden fixture against brew's 2.0.0: the same 13 zip parts, only docProps/core.xml's write stamps differ, xlsx-cells the same 364 lines

### WP-03 -- One macOS gate for every command that needs a macOS service (status: WIP)

_(no tests in this group)_

### WP-04 -- One walker for grep, links and rename, or the record of why not (status: Not Started)

_(no tests in this group)_

### WP-05 -- One picture rule for the engine, pplr pictures and the index (hv's ruling) (status: Not Started)

_(no tests in this group)_

### WP-06 -- pplr 2.0.1 by one command (status: Not Started)

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
