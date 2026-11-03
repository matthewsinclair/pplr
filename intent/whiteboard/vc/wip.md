---
node: vc
name: vc
role: vc
session_id: 60a887be-5fa2-4cf5-8de1-310286047151
heartbeat_at: 2026-11-03 07:22Z
status: active
focus: "main 64be72f (todo 2's harness fix) verified in a fresh clone, every gate 0; hv pinged to push for the Linux proof. Next: watch that CI run (ubuntu's cargo should go green and its bats step run for the first time); cc's WP11 commit 3 measures once, then vc's review against todo 3 and a clean-clone run; dc pre-audits the bats suite for macOS-only assumptions; todo 5 after commit 3"
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- CI: run 41595332795 at acec3ca (WP14 on main): macOS all green, bats included; ubuntu's Cargo tests fail the same 3 replay cases as run 41591508539 (sync-apply-plan, sync-link-all-names, sync-link-apply), 51 of 54 cases ran, and WP14's email and contact log --email tests and goldens pass on ubuntu. Cause, found by dc: all three write Zsofia Varga's About/(Contacts).inetloc, which the manifest names with the o decomposed (pplr writes .inetloc names decomposed, as Foundation did), while git on macOS (core.precomposeunicode) stored the golden precomposed; APFS equates the two forms and ext4 does not, so the replay's fallback read failed with a bare ENOENT. Fix approved (dc, tests/cli.rs only): find stored goldens by matching each path component's decomposed form, and name the path in the harness's io errors; no refusal and no skip, since PPLR_CONTACTS_JSON needs no macOS service (AC-00.7). Linux proof: CI after hv's next push (hv did not take up kovacs). ubuntu's bats step has still not run, so a further layer may follow. When both runners are green, ST0001 AC-02.1 and ST0002 WP02's CI half can close. (edited)
- At cc's WP11 commit 3 (critic-rust's findings, as ruled), check: one card_change() serves the Changes column and --apply-plan; plan() reads no files; sync refuses a missing value, a non-numeric --limit and a trailing --group (exit 1, with the usage); --apply-plan reads Merged and Contacts by header name and refuses a missing header; every warning fixed; each behaviour change has a test, red then green; no golden changed, unless cc first reported which one the marker-path helper changed. Then vc's clean-clone run, and WP11 done (AT-11.1 is green since 515ee42). (edited)
- tree.rs:63 reads an unreadable or non-UTF-8 About as an empty header, silently (No Silent Errors); the same pattern is at grep.rs:90, tags.rs:681, tags.rs:906 and sync.rs:1107 (critic-rust on WP14, via dc). It predates WP14 and spans the crate, so it is its own item: rule and route it once WP11 commit 3 lands, since commit 3 changes tree.rs.
- After the WP13 cutover, candidates under hv's simpler-or-easier rule, none in commit 3 (cc, critic-rust's second read of WP11 commit 3): (a) unify the three picture rules, the engine's, pplr_pictures' and the index's, which differ as their legacy scripts did; that changes behaviour on real data, so it is a divergence and needs hv's ruling (a comment on tree::PICTURES records the difference); (b) Pair.how's vocabulary as an enum, behaviour-preserving. Left as is by vc's ruling: an option taken as another's value (sync --link --group --apply) falls to a dry run, which is safe.

## Holds

- Each item needs hv, and each is released as hv does it. (1) Contacts access for this terminal, or hv runs 'cargo test --test cli live -- --ignored' (ST0001 AC-09.2). (2) Create the geodica.apple.{APPLE_ID,APPLE_APP_PASSWORD,APPLE_TEAM_ID} keychain entries. (3) Allow Developer ID signing, the identity check and the first notarytool submission (ST0002 WP07, AC-07.1 and 07.2). (4) Intent fixes outside pplr: the pre-commit hook's word-splitting, and AGENTS.md's swift lines (watchouts 1 and 2). (5) WP13 cutover moves ~/.local/bin/pplr to the Rust binary. (6) ST0002 WP05: the real brew install from a scratch tap; dc prepares everything up to it (it could shadow ~/.local/bin/pplr). (7) ST0002 WP04: the tap's description and CI need gh repo edit and a tap push (AC-04.2); if dc's rubocop wrap fails, a Devbin issue about the formula manifest line over brew's 118-column limit (lib/cmd/formula:424). Item (8), the divergences, was released by hv on 2026-11-03 (decision 9). Each push is hv's, from the terminal: the auto-mode classifier refuses one from vc's session. (edited)

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).

## Decisions

- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- vc's independent verification, measured: main 64be72f (ST0001 WP14, then dc's decomposed-name lookup in the golden replay) cloned fresh into vc's scratchpad, slot 01M6G4FCG0M6F769Q7MHJY1X0B. format, clippy, autotests, targetdir, critic, cargo test (93 unit and 29 cli pass; 1 ignored in each, the live Mail and Contacts tests), the replay 54 of 54 cases native, bats on Rust 101/101 and on legacy 101/101, none skipped: every gate exited 0. It supersedes the f726f34 run and is the evidence for every WP accepted so far, independent of the nodes' own slot results; dc's measurement (slot 01M6G4AZ391EPDF1JDQZQAR9R9 at af37bb8, the same tree outside intent/) agrees. (edited)
- ST0001 WP14 (dc), ruled 2026-11-02: keep mail-recent's real semantics; make the golden fake honest (zsh) and re-capture the two email goldens from legacy at 507b0f6 before the Rust; no PPLR_NOW, PPLR_TODAY takes an optional HH:MM:SS (one clock setting); no mail path settings, the replay builds both DBs under the sandbox HOME; rusqlite 0.32 bundled; deleting bin/pplr_email and email_list.py is WP13's (D3).
- ST0001 WP14, critic-rust's two calls, ruled 2026-11-03 at dc's ask. (A) Highlander: pplr email builds its address map from person.header.emails, the one About reader every command shares, not a second read of each About with the legacy listing's mailto pattern. No golden changes. The four cases where the two readers disagree are recorded in WP14's body for WP13's CHANGELOG: a plain address now tags, mailto:x?subject tags x, a second '- Email:' line no longer tags, a non-UTF-8 About no longer tags. (B) contact log refuses a second --email, exit 1 naming why, with a test red then green; the legacy logged a mix of two messages, and the port's last-wins dropped the first silently. Recorded in WP14's body. The silent skip of an unreadable About (tree.rs:63, and the same pattern across the crate) predates WP14 and is its own item (todo 5).
- hv, 2026-11-03, in vc's session: 'yes, the divergences stand'. All four stand as recorded in WP11's and WP14's bodies, for WP13's CHANGELOG: WP11, sync refuses a bad or missing option value, a non-numeric --limit and a trailing --group, and --apply-plan reads its sheets by header name; WP14, email finds people through the shared About header reader (four edge cases differ), and contact log refuses a second --email. Item (8) of hold 2 is released.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
