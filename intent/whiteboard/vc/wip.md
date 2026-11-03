---
node: vc
name: vc
role: vc
session_id: 60a887be-5fa2-4cf5-8de1-310286047151
heartbeat_at: 2026-11-03 05:41Z
status: active
focus: "hv back, plan approved: dc lands config.rs; cc fixes the CMS port race, then WP11; dc finishes WP14; vc checks each in its own clone and holds slot control; hv pushes once CI fixes are in"
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- CI: both fixes are on main, and vc verified main at 25f85e7 (decision 6): 3d6dec4 serialises the stub CMS port, and 95a75e8 (WP11 commit 1) drops the replay's Swift warm-up and makes plan native. Next: hv pushes main to upstream and local; then vc watches the first run. ubuntu's bats step runs for the first time under tests.yml, and 515ee42's xlsx-cells styling pass (zsh and awk) has only run on macOS, so a Linux layer may surface. dc's rename fix db45555 goes with this push. (edited)
- At cc's two WP11 landings, check. Commit 1 (code, content replay): the exit-99 mutant proves no replay reaches legacy code; with uv hidden, no skip naming WP11 fires against Rust (rule 8); WP11 stays WIP and AT-11.1 not green, the reason in the commit; the brief's deletions stay deferred to WP13 (WP13's body lists them). Then vc's clean-clone run, then ping hv to push. Commit 2 (styling): xlsx-cells adds fills, list validations, conditional formats, frozen panes and filters; the re-capture of sync-plan and sync-apply-plan from 507b0f6 only adds lines; a planted styling defect fails the replay; tests/golden/README.md says widths, fonts and wrapping are not compared, and why. Then AT-11.1 green and WP11 done. (edited)

## Holds

- For hv when back (each needs hv): (1) push main to local and upstream (all work is local commits); CI then closes ST0001 AC-02.1 and gives the first tests.yml run. (2) Contacts access for this terminal, or hv runs 'cargo test --test cli live -- --ignored' (ST0001 AC-09.2). (3) Create the geodica.apple.{APPLE_ID,APPLE_APP_PASSWORD,APPLE_TEAM_ID} keychain entries. (4) Allow Developer ID signing, the identity check and the first notarytool submission (ST0002 WP07, AC-07.1 and 07.2). (5) Intent fixes outside pplr: the pre-commit hook's word-splitting, and AGENTS.md's swift lines (watchouts 1 and 2). (6) WP13 cutover moves ~/.local/bin/pplr to the Rust binary. (7) ST0002 WP05: the real brew install from a scratch tap; dc prepares everything up to it (it could shadow ~/.local/bin/pplr). (8) ST0002 WP04: the tap's description and CI need gh repo edit and a tap push (AC-04.2); if dc's rubocop wrap fails, a Devbin issue about the formula manifest line over brew's 118-column limit (lib/cmd/formula:424). (edited)

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).

## Decisions

- ST0001 and ST0002 are elaborated and on main (dc123cd): 14 + 7 WPs, 28 + 16 criteria; hv's rulings of 2026-11-02 are in each thread's design.md section 1 and section 7.
- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- For hv, about the live tool: dc's WP12 (de97d29) deleted four legacy scripts, which broke hv's live ~/.local/bin/pplr (a symlink to this checkout's bin/pplr) for doctor and the icon commands for a few minutes. Restored in a4401b7 and verified (version and help, with no People access). vc amended D3, rule 3, rule 9 and eight ACs (a7a2493): no bin/ or lib/ file is deleted before WP13, when the live link moves to Rust. One deliberate change to live behaviour, forced by the critic gate on the restore: legacy setpicsfordirs now splits 'Surname, First' at the comma, so multi-word names work, as in Rust; single-word names are unchanged.
- ST0001 WP08 (rename, links and the pplr:// handler) moves from cc to dc: its prebuilt, signed Pplr Links.app sits with dc's ST0002 WP07 signer, and dc's ST0002 lane is held at WP05 for hv. cc keeps WP07, WP10, WP11, WP14 and WP13.
- Slot-result integrity, for hv: dc found that some of its slot release results were written before the run finished, in ';'-chained commands. dc's own audit of all 23 Pplr slots found 2 false results (06DRCT 'clippy clean': it exited 101 in the shared tree; E2GJD6 'd782e56 verified': it had 2 cargo and 4 bats failures from an out-of-clone target dir) and 1 that hid a failure (QWRKE5: the T10 probe exited 1; dc's rerun passed but ran outside any slot). Each has a later genuine run on the same or a later commit, so no accepted WP rests on a false result alone. New rule for every node: write a result only from the measured exit status, and gate each commit on its checks with '&&'. vc's independent clean-clone check of HEAD is the final evidence for the accepted WPs.
- vc's independent verification, measured: main 25f85e7 (WP11 commits 1 and 2, 95a75e8 and 515ee42, on top of config.rs 6dfc81c and the port-race fix 3d6dec4) cloned fresh into vc's scratchpad, slot 01M6G1TVTJ7TS6PD9TV7ZGPAB4. format, clippy, autotests, targetdir, critic, cargo test (83 unit and 26 cli pass; 1 ignored, the live Contacts test), bats on Rust 101/101 and on legacy 101/101, none skipped: every gate exited 0. This is the evidence for every WP accepted so far, independent of the nodes' own slot results; it supersedes the 95a75e8 run. cc's own measurements add the exit-99 mutant (no replay reaches legacy code) and bats on Rust with uv hidden (2 skips, both WP14). (edited)
- ST0001 WP14 (dc), ruled 2026-11-02: keep mail-recent's real semantics; make the golden fake honest (zsh) and re-capture the two email goldens from legacy at 507b0f6 before the Rust; no PPLR_NOW, PPLR_TODAY takes an optional HH:MM:SS (one clock setting); no mail path settings, the replay builds both DBs under the sandbox HOME; rusqlite 0.32 bundled; deleting bin/pplr_email and email_list.py is WP13's (D3).

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
