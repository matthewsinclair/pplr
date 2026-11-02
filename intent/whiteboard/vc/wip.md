---
node: vc
name: vc
role: vc
session_id: 01f7e7ed-4beb-47d7-b826-1033c56a385d
heartbeat_at: 2026-11-02 17:36Z
status: paused
focus: "Closed for hv's reboot: CI red on ubuntu (see todo), db45555 unpushed, Dropbox push blocked on macOS file access"
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- Next session, first: CI is red on ubuntu since WP01. (1) 41491091725 at b663b95 fails 6 replay tests (contact, tag/tags, sync, every_golden, read cmds, index/new/pictures/search): new with WP10, cc to diagnose. (2) The rename order fix db45555 (dc) is local, unpushed, awaiting hv. (3) The Dropbox local remote: access() on ~/Dropbox files hangs for this terminal (TCC/File Provider); hv to check Privacy settings.

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
- vc's independent verification, measured: HEAD e40bdee cloned fresh into vc's scratchpad, slot N8Z5E. fmt, clippy -D warnings, cargo test (69 unit and 21 cli pass; 1 ignored, the live Contacts test), tools/build, bats on Rust 101/101 and on legacy 101/101: every gate exited 0. This is the evidence for every WP accepted so far, independent of the nodes' own slot results.
- ST0001 WP14 (dc), ruled 2026-11-02: keep mail-recent's real semantics; make the golden fake honest (zsh) and re-capture the two email goldens from legacy at 507b0f6 before the Rust; no PPLR_NOW, PPLR_TODAY takes an optional HH:MM:SS (one clock setting); no mail path settings, the replay builds both DBs under the sandbox HOME; rusqlite 0.32 bundled; deleting bin/pplr_email and email_list.py is WP13's (D3).

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
