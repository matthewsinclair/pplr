---
node: vc
name: vc
role: vc
session_id: 60a887be-5fa2-4cf5-8de1-310286047151
heartbeat_at: 2026-11-03 06:50Z
status: active
focus: "Slot desk re-armed after the compact, nothing pending. Waiting on dc's WP14 (critic's findings, then one cargo+bats measurement) and cc's WP11 commit 3: first ready lands first, the other rebases. WP14 pre-read at 5accda2 against todo 4, no blocker so far. Each landing then gets vc's fresh-clone run, then hv's push"
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- CI: run 41591508539 at 92105af, the first with both fixes: macOS all green, bats included; ubuntu's Cargo tests fail 3 replay cases (sync-apply-plan, sync-link-all-names, sync-link-apply: 'No such file or directory' from the harness, since the Contacts sidecar is built only on macOS). Routed to dc (harness and sidecar owner), as its own commit under AC-00.7. ubuntu's bats step has still not run under tests.yml, so a further layer may follow. When both runners are green, ST0001 AC-02.1 and ST0002 WP02's CI half can close. (edited)
- At cc's WP11 commit 3 (critic-rust's findings, as ruled), check: one card_change() serves the Changes column and --apply-plan; plan() reads no files; sync refuses a missing value, a non-numeric --limit and a trailing --group (exit 1, with the usage); --apply-plan reads Merged and Contacts by header name and refuses a missing header; every warning fixed; each behaviour change has a test, red then green; no golden changed, unless cc first reported which one the marker-path helper changed. Then vc's clean-clone run, and WP11 done (AT-11.1 is green since 515ee42). (edited)
- At dc's WP14 landing (rebased onto main as it stands then, measured once with cargo and bats together), check: WP14's body as ruled (usage exits 1; the Full Disk Access row warns and never fails; no OS gate; the legacy deletions are WP13's); the home field reads HOME through home(); AT-14.1 and AT-14.2 green; the replay 54 of 54 native; no bats skip naming WP14 fires against Rust; rusqlite bundled. Then vc's clean-clone run. Order with cc's WP11 commit 3: the first ready lands first, and the other rebases. (edited)

## Holds

- Each item needs hv, and each is released as hv does it. (1) Contacts access for this terminal, or hv runs 'cargo test --test cli live -- --ignored' (ST0001 AC-09.2). (2) Create the geodica.apple.{APPLE_ID,APPLE_APP_PASSWORD,APPLE_TEAM_ID} keychain entries. (3) Allow Developer ID signing, the identity check and the first notarytool submission (ST0002 WP07, AC-07.1 and 07.2). (4) Intent fixes outside pplr: the pre-commit hook's word-splitting, and AGENTS.md's swift lines (watchouts 1 and 2). (5) WP13 cutover moves ~/.local/bin/pplr to the Rust binary. (6) ST0002 WP05: the real brew install from a scratch tap; dc prepares everything up to it (it could shadow ~/.local/bin/pplr). (7) ST0002 WP04: the tap's description and CI need gh repo edit and a tap push (AC-04.2); if dc's rubocop wrap fails, a Devbin issue about the formula manifest line over brew's 118-column limit (lib/cmd/formula:424). (8) Let stand or overrule vc's two WP11 divergences from the legacy, recorded in WP11's body: sync refuses a bad or missing --limit and a trailing --group; --apply-plan reads its sheets by header name. Each push is hv's, from the terminal: the auto-mode classifier refuses one from vc's session. (edited)

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).

## Decisions

- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- vc's independent verification, measured: main 25f85e7 (WP11 commits 1 and 2, 95a75e8 and 515ee42, on top of config.rs 6dfc81c and the port-race fix 3d6dec4) cloned fresh into vc's scratchpad, slot 01M6G1TVTJ7TS6PD9TV7ZGPAB4. format, clippy, autotests, targetdir, critic, cargo test (83 unit and 26 cli pass; 1 ignored, the live Contacts test), bats on Rust 101/101 and on legacy 101/101, none skipped: every gate exited 0. This is the evidence for every WP accepted so far, independent of the nodes' own slot results; it supersedes the 95a75e8 run. cc's own measurements add the exit-99 mutant (no replay reaches legacy code) and bats on Rust with uv hidden (2 skips, both WP14). (edited)
- ST0001 WP14 (dc), ruled 2026-11-02: keep mail-recent's real semantics; make the golden fake honest (zsh) and re-capture the two email goldens from legacy at 507b0f6 before the Rust; no PPLR_NOW, PPLR_TODAY takes an optional HH:MM:SS (one clock setting); no mail path settings, the replay builds both DBs under the sandbox HOME; rusqlite 0.32 bundled; deleting bin/pplr_email and email_list.py is WP13's (D3).

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
