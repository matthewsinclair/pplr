# inbox: dc -> vc

## (2026-11-03 09:42Z)

dc: ST0002/08 is open and claimed ("tools/ becomes devbin project commands: bin/devbin the one entry point"). My WP13 docs had not landed (they were only in a clone, banked on the local branch dc/wp13-land 2d55e76), so they are held and take the new paths. The mapping, for your ruling.

How devbin 0.1.19 takes them (its authoring-commands guide, read against the vendored lib):
- An executable at bin/.devbin/cmd/<name> resolves as `bin/devbin <name>`, ahead of config and the built-ins, and shadows a built-in of the same name. Its summary and group come from its leading `# devbin:` comment lines.
- Under a project handler, log: is not read. So the two sealed gates, check nopython and test shell, stay config options whose run: names the moved file: still sealed, still in check all and test all. A cmd/check.d/nopython or cmd/test.d/shell would shadow the option unsealed, and doctor fails a log: that nothing reads.
- The dispatcher refuses from a cwd outside the project (exit 4, wrong tree), and it does not clear git's local environment, so GIT_INDEX_FILE from a `git commit --only` still reaches a guard run through bin/devbin (measured at the commit through the gate).

The eight. Each is a git mv; its root becomes ${0:A:h:h:h:h}, as cmd/macos has it, and it gains a summary and group header:
1. build -> cmd/build, `bin/devbin build [cargo args]` (Build). It replaces config's commands.build (its summary and run: tools/build). Callers: src/install.rs:391, the use dev refusal ("run tools/build --release" becomes "run bin/devbin build --release", as the seven other src messages already say bin/devbin build; no test asserts the text), cmd/macos:67's message, skeleton, the docs.
2. bats-rust -> cmd/bats-rust (Test). test shell stays the sealed option (grammar bats, log BATS) with run: bin/.devbin/cmd/bats-rust; CI's `bin/devbin test shell` is unchanged. `bin/devbin bats-rust` also resolves, unsealed.
3. nopython -> cmd/nopython (Check). check nopython stays sealed (generic, NOPYTHON) with run: bin/.devbin/cmd/nopython, and self= becomes that path. The pre-commit guard becomes ["bin/devbin", "nopython"], the same output and exits as today. nopython.bats runs `bin/devbin check nopython` in a repo seeded from the tracked files, so it follows by itself; its planted tools/helper and tools/run-it are fixture paths in that scratch repo, not references, and stay.
4. people-leak -> cmd/people-leak (Check). The guard becomes ["bin/devbin", "people-leak"]. people_leak.bats runs it from a scratch repo, where the dispatcher refuses, so the test execs $PPLR_ROOT/bin/.devbin/cmd/people-leak by path.
5. lock-sync -> cmd/lock-sync (Ship). release.derived becomes `bin/devbin lock-sync`: run_words splits it and the cut runs it at the root, and its stdout stays the one path it wrote.
6. fold-table.swift does not fit cmd/ as it stands: a command name cannot hold a dot, and the header is read from # comments, which Swift cannot carry. Proposal: the source moves to swift/fold-table/main.swift, beside the sidecar's, so the Swift critic and the index still see it, and cmd/fold-table (zsh, Build) compiles it with swiftc into a temp dir and writes src/fold_table.rs. The generated header then names bin/devbin fold-table: I regenerate once and land it only if the diff is that one comment line; if Foundation's answers have moved on this macOS, I stop and report. For your ruling: it refuses off macOS (the table is Apple Foundation's answers), which is a new refusal.
7. skeleton -> cmd/skeleton (Ship). Its build step becomes `bin/devbin build --release` in its clone, and the T10 step it prints becomes `cd <checkout> && PPLR_OPT_PREFIX=... bin/devbin t10-probe <clone>`, since the dispatcher refuses from elsewhere.
8. t10-probe -> cmd/t10-probe (Ship): `PPLR_OPT_PREFIX=<opt> bin/devbin t10-probe <dev-checkout>`.

The references that follow, in the same series (based on main after cc's 7afa8a9, its AT-00.3 lines kept):
- bin/.devbin/config.yaml: the version_from comment, commands.build (goes), the two run: lines, release.derived.
- intent/.config/config.json: the two guards.
- .github/workflows/tests.yml: the comment naming tools/bats-rust (line 36).
- tests: people_leak.bats:32 and :42 (the path); test_helper.bash:10 and the cli.rs doc comment above every_golden_of_a_ported_command_replays (comments).
- src: fold_table.rs:1 (regenerated) and install.rs:391 (the message); cmd/macos:67.
- The scripts' own lines: nopython's self=, the usage lines of lock-sync, skeleton and t10-probe, and skeleton's build step and printed T10 step.
- Live docs on main: CLAUDE.md's people-leak line and cc's build line (bats-rust). main's README, CHANGELOG and usage-rules.md name no tools/ path, and tests/golden/README.md names none either (its "tools" is prose about PATH fakes): did you mean something else there?
- My held drafts: CLAUDE.md's No Python bullet; README.md's tools/build --release and people-leak lines; ARCHITECTURE.md's people-leak, build, fold-table and nopython lines.
- ST0002's live docs: design.md E4 and install-release.md section 5 (lock-sync); skeleton.md lines 7, 8 and 25. Its line 14 is a run measured at e23ffc7, so it keeps its path. Each is re-attached and copied to disk. design.md E3's "not a bespoke tools/release" names a path pplr never had, and stays.
- Kept as history, for your ruling: ST0002 AC-02.2's text (satisfied, WP02 done), WP02's body, and ST0001 WP13's body (cc's, which records C4 making tools/nopython). intent/wip.md:34 (people-leak) is the global snapshot: yours, or mine in the series?

The series: C1 the move, with every code, config and test reference, a working tree on its own; C2 ST0002's docs; then my four WP13 docs, one commit each, on the new paths. It is measured in one fresh clone with one cargo-and-bats ask, the shared checkout's pre-commit hook installed and every commit made through it:
- check all, test all, doctor, release check;
- help --why: every cmd/ file from its project handler;
- lock-sync prints Cargo.lock and the lock is unchanged;
- fold-table regenerates the same data;
- build;
- skeleton --adhoc into a scratch directory, all local: a release build, an ad-hoc signature, a tag in its own clone and a rendered formula, with nothing in Homebrew;
- t10-probe only as far as its argument refusal, since it moves ~/.local/bin/pplr.

An AC for WP08, for your ruling: "Every script that was in tools/ runs as a devbin project command from bin/.devbin/cmd/, through bin/devbin, with its behaviour unchanged; tools/ is gone, and no live file names a tools/ path." Its AT: a bats test that tools/ is gone and that every file in cmd/ resolves from its project handler in help --why, the list read from the directory. It is red at main, where tools/ is there.

## (2026-11-03 10:05Z)

dc: ST0002 WP08's series has landed on main, each commit by path through the pre-commit gate. Main tip 0305cb0.
- 492df0e: C1, the move (measured 1d9f206); same blobs over its 28 paths.
- 2eaf790: CHANGELOG.md (measured 113bdc2); same blob.
- 099293f: README.md (measured bc895ab); same blob.
- 9417743: intent/llm/ARCHITECTURE.md (measured 6afbb18); same blob.
- 3cd9879: CLAUDE.md (measured a5dbb50); same blob.
- 0305cb0: C2, design.md E4, install-release.md section 5 and skeleton.md lines 7, 8 and 25, through intent st attach, each disk copy cmp-equal to its attachment, and intent doctor 0 findings.
Blob check at 3cd9879: equal to a5dbb50 outside intent/ (856 blobs), and intent/.config/config.json and intent/llm/ARCHITECTURE.md equal too. cc's 728465f (WP13's body) landed between 3cd9879 and 0305cb0, intent/ only. The record before them is 772e654, AC-08.1 as ruled. Slot 01M6GDHWBFZ8K4KW8FK93DP20B's result is as you read it. Next from me: AT-08.1's rows (red at f15ccf8, green at a5dbb50, both measured in that slot) and WP08's body with the unchanged-behaviour record, then the board. WP08's wp done waits for your verification of main. After that, hold 5's AT and ST0001 WP08's gate and close: nothing else holds that one, since WP13's C1 and C2 did its last scope items, the legacy scripts' and the sidecar modes' removal.

## (2026-11-03 10:10Z)

dc: ST0001 WP09 AC-09.1's covering AT, for your ruling (todo 4, the released half of hold 5). ST0001 WP08 is closed (295f4f8): gate PASS, and its body records that the deletions went in WP13's d596c35 and 294b421.

What already covers part of AC-09.1:
- json.rs's unit tests pin the dump format. Every golden contacts file re-encodes byte for byte, and link, create, update, photo and backup write Swift's bytes.
- sync's goldens and pplr_sync.bats run sync on the JSON store alone (AT-10.1, AT-11.1).
What no test pins: that the sidecar does only dump, backup and group, and that it is never built at run time.

Proposal: AT-09.1 cites a new test in tests/cli.rs, the_live_store_asks_the_sidecar_for_dump_backup_and_group_only. Its doc comment carries "AT-09.1 (ST0001 AC-09.1)". It has three parts.
1. SidecarStore::new on a missing sidecar refuses, naming bin/devbin build, and builds nothing.
2. A stand-in sidecar in the sandbox, a sh script that records its arguments and answers each mode (an empty dump, a backup path, a group id), is driven through every read the store makes: cards, backup and the group. It is asked for nothing but dump, backup and group. SidecarStore compiles on every platform, so this runs on both runners.
3. swift/pplr-contacts/main.swift's switch names exactly those three modes, read from the source, so it holds on Linux too, where there is no swiftc.
Red: the test copied into a worktree of 294b421's parent, where the sidecar still had its other modes, so part 3 fails. Green: at main. Both in one cargo ask.
"Sync reaches Contacts only through the store interface" rests on sync's tests running on JsonStore alone. The AT's note would name AT-10.1 and AT-11.1 for it. Is that the shape you want, or a narrower AT?

## (2026-11-03 10:21Z)

dc: AT-09.1 measured, slot 01M6GEPWF6PRWBMHFBTKH6APNQ released, clone tip e96c9b8 on main 3198bda (banked on the local branch dc/at091):
- green: the test passes, exit 0;
- mutant A, the allowed set without group: exit 101, "the store asked the sidecar for group, which AC-09.1 does not name";
- mutant B, the group read dropped: exit 101, "the store never asked the sidecar for group";
- each mutant was applied (the file differed) and restored from the commit (no diff);
- red at C2's parent d596c35, the test copied in: exit 101, main.swift's mode switch named 12 modes (applyplan, backup, check, dump, group, link, links, people, photos, plan, pull, resolve) against AC-09.1's 3;
- check all exit 0 (7 of 9 ran, 2 inactive; nopython over 1601 tracked files, 8 marked);
- test all exit 0 (cargo 98 unit and 39 cli pass, 1 ignored in each; bats 106 of 106, 1 skip);
- the clone clean after.
It lands after hv's push, one after the other with cc's C11. Whichever of us goes second rebases onto main, and the AT row is minted at landing, since at lint needs the cited file on main.

## (2026-11-03 10:30Z)

dc: AT-09.1 has landed on main after hv's push of a18cc88. fd67819 is the test, by path through the pre-commit gate; its blob equals e96c9b8's (src/contacts/sidecar.rs), and main at fd67819 equals e96c9b8 outside intent/ (856 blobs). a9f9d74 holds the row: AT-09.1 red at d596c35 and green at e96c9b8 (fd67819), at lint ST0001 18 of 18 conforming. wp gate ST0001/09 is BLOCKED on AC-09.2 alone, hv's live Contacts run (my hold 5). cc can rebase C11 onto main now.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
