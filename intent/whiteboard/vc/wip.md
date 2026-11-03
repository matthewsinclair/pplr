---
node: vc
name: vc
role: vc
session_id: 60a887be-5fa2-4cf5-8de1-310286047151
heartbeat_at: 2026-11-03 07:47Z
status: active
focus: "localfold for hv's compact, holding. In flight: vc's clean-clone run of main a3ab0fb (the one About reader), slot 01M6G5YWE8VHKZYS3KDCHXWHTY, background task writing vc's scratchpad verify-summary.txt (clone verify/, logs verify-logs/). On its result: release the slot with it, edit decision 6, archive todo 7 and decision 10, tell cc, ping hv to push. Then: dc closes ST0001 WP02 and drafts WP13's docs for vc's review (decision 11); cc starts WP13 (removals, no-Python gate, live read-only parity); keep the slot desk"
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- After the WP13 cutover, candidates under hv's simpler-or-easier rule, none in WP11 commit 3 (fe4b677; critic-rust's second read, via cc): (a) unify the three picture rules, the engine's, pplr_pictures' and the index's, which differ as their legacy scripts did; that changes behaviour on real data, so it is a divergence and needs hv's ruling (a comment on tree::PICTURES records the difference); (b) Pair.how's vocabulary as an enum, behaviour-preserving; (c) matches() leaves has_inetloc false to mean 'not read', a plain bool documented on the field; (d) workbook write() fills Merged's rows by position while the reader and the styling go by header. Left as is by vc's ruling: an option taken as another's value (sync --link --group --apply) falls to a dry run, which is safe. (edited)
- At cc's About-reader commit (decision 10), check: one reader in src/fs.rs serves every About read, NotFound as None and any other error named and returned to the coordinator; read_text (lossy) at tree.rs's header and aliases reads, grep.rs, tags.rs's two reads and contact.rs:792; read_exact (strict, naming a non-UTF-8 file) at every write-back: sync --pull, rename.rs, tags render, pictures.rs; --pull names the real cause per person; the three tests run red then green; no golden changed. Then amend WP14's body to drop its fourth edge case, and vc's clean-clone run. It lands before WP13. (edited)

## Holds

- Each item needs hv, and each is released as hv does it. (1) Contacts access for this terminal, or hv runs 'cargo test --test cli live -- --ignored' (ST0001 AC-09.2). (2) Create the geodica.apple.{APPLE_ID,APPLE_APP_PASSWORD,APPLE_TEAM_ID} keychain entries. (3) Allow Developer ID signing, the identity check and the first notarytool submission (ST0002 WP07, AC-07.1 and 07.2). (4) Intent fixes outside pplr: the pre-commit hook's word-splitting, and AGENTS.md's swift lines (watchouts 1 and 2). (5) WP13 cutover moves ~/.local/bin/pplr to the Rust binary. (6) ST0002 WP05: the real brew install from a scratch tap; dc prepares everything up to it (it could shadow ~/.local/bin/pplr). (7) ST0002 WP04: the tap's description and CI need gh repo edit and a tap push (AC-04.2); if dc's rubocop wrap fails, a Devbin issue about the formula manifest line over brew's 118-column limit (lib/cmd/formula:424). Item (8), the divergences, was released by hv on 2026-11-03 (decision 9). Each push is hv's, from the terminal: the auto-mode classifier refuses one from vc's session. (edited)

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).

## Decisions

- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- vc's independent verification, measured: main 3121fb9 (ST0001 WP11 commit 3 fe4b677 and the ubuntu bats fixes, on WP14 and the golden replay's decomposed-name lookup) cloned fresh into vc's scratchpad, slot 01M6G5153R6DD6SZG3NA95TKJQ. format, clippy, autotests, targetdir, critic, cargo test (96 unit and 30 cli pass; 1 ignored in each, the live Mail and Contacts tests), the replay 54 of 54 cases native, bats on Rust and on legacy 102 ok each with 1 skip each, the AC-00.7 Linux refusal test by design on macOS: every gate exited 0. It supersedes the 64be72f run and is the evidence for every WP accepted so far, independent of the nodes' own slot results; cc's measurement of commit 3 (slot 01M6G4Q1MKP3S5271NFW66AKDK) and dc's of the bats fixes (slot 01M6G4WQ43E8DD0RQFSV6QB5D3) agree. (edited)
- hv, 2026-11-03, in vc's session: 'yes, the divergences stand'. All four stand as recorded in WP11's and WP14's bodies, for WP13's CHANGELOG: WP11, sync refuses a bad or missing option value, a non-numeric --limit and a trailing --group, and --apply-plan reads its sheets by header name; WP14, email finds people through the shared About header reader (four edge cases differ), and contact log refuses a second --email. Item (8) of hold 2 is released.
- Todo 5 ruled, 2026-11-03, now WP11 commit 3 is in, refined at cc's ask: one reader in src/fs.rs beside write_atomic gives an About's bytes, None on NotFound ('no About'), and names any other error ('cannot read <path>: <e>'); the error goes back to the coordinator, never printed by the library (PFIC), and fails the command. Two decodings sit on it. read_text decodes lossily, for the reads, so a non-UTF-8 About keeps its header as the legacy's byte-wise reads did and WP14's fourth edge case goes away (WP14's body is amended when it lands): tree.rs's header and .index/aliases reads, grep.rs:90, tags.rs:681 and :906, contact.rs:792. read_exact refuses a file that is not UTF-8, naming it, for every command that writes the About back, so pplr never rewrites a byte it could not decode: sync --pull (sync.rs:1177, which then names the real cause per person instead of 'no About file'), rename.rs:239, tags.rs:604 (render), pictures.rs:62. tree::people() returns a Result. Tests red then green: a non-UTF-8 About's header reads; an unreadable About fails naming its path; --pull leaves a non-UTF-8 About byte for byte and names it. No golden changes expected. Owner: cc, after WP11 (done) and before WP13; measured once with cargo and bats. (edited)
- CI is green on both runners: run 41598011297 at 6464721, ubuntu cargo 96 unit and 30 cli, and ubuntu's first bats run 102 ok with the one skip AT-12.2 by design; the AC-00.7 Linux refusal test ran and passed. ST0001 AC-02.1 and AC-02.2 are dc's to satisfy (WP02), citing this run and vc's decision 6; AC-00.3 and AC-00.7 hold on this evidence and are satisfied at WP13's end with the thread's other criteria. WP13 split, to run it in parallel: cc owns WP13 and claims ST0001/13 (the removals, the no-Python gate, the live read-only parity); dc drafts WP13's docs under it as their own commits: the README, usage-rules.md, CHANGELOG 2.0.0 unreleased with the divergences (WP11's two, WP14's two, decision 10's reads), intent/llm/ARCHITECTURE.md and CLAUDE.md's pointers.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
