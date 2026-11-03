---
node: vc
name: vc
role: vc
session_id: 60a887be-5fa2-4cf5-8de1-310286047151
heartbeat_at: 2026-11-03 14:07Z
status: active
focus: "pplr 2.0.0 is out through brew (v2.0.0 at dd6dc5b; ST0002's evidence c244a76, 15 of 16). Open: hv's step 8, the Contacts check across a reinstall (AC-06.2, hold 2); then dc's design.md and WP04, WP06 and WP07 closes (dc's doing 7); then vc's st done ST0002. Next job: devbin's one-command release (decision 26, todo 8). Main's post-release commits are local until hv pushes."
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- After the WP13 cutover, candidates under hv's simpler-or-easier rule, none in WP11 commit 3 (fe4b677; critic-rust's second read, via cc): (a) unify the three picture rules, the engine's, pplr_pictures' and the index's, which differ as their legacy scripts did; that changes behaviour on real data, so it is a divergence and needs hv's ruling (a comment on tree::PICTURES records the difference); (b) Pair.how's vocabulary as an enum, behaviour-preserving; (c) matches() leaves has_inetloc false to mean 'not read', a plain bool documented on the field; (d) workbook write() fills Merged's rows by position while the reader and the styling go by header. (e) One macOS gate: contacts/mod.rs, handler.rs and icons.rs each test cfg!(target_os) with their own message, beside open.rs's macos_only(why); one helper for every gate, behaviour-preserving if the messages stay. (f) One walker: grep.rs, links.rs and rename.rs each recurse on their own (C10 fixed links' to pass over symlinked directories, as grep's and rename's already did); one walk for all, behaviour-preserving. Left as is by vc's ruling: an option taken as another's value (sync --link --group --apply) falls to a dry run, which is safe. (edited)
- Decision 26's one-command devbin release, the first job after 2.0.0: bin/devbin release vX runs the gates, CI, the tag, the Mac's signing and notarisation, attach and formula publish. Fold in: (a) a cut step for an asset built and signed locally, before attach and before after:; (b) bare-binary signing in devbin's built-in macos (Intent's signer does it); (c) one keychain prefix for the estate (pplr reads geodica.apple.*, devbin's built-in devbin.apple.*); (d) deterministic asset tarballs, as ccusage-dc packs them (COPYFILE_DISABLE=1, --no-mac-metadata, --no-xattrs, --uid 0 --gid 0 --uname '' --gname ''); pplr's plain tar -czf asset, extracted the way brew does, has no AppleDouble and verifies strictly, so 2.0.0 is not blocked; (e) require_identity captures security's output before matching, since grep -q under pipe_fail can lose to SIGPIPE; (f) lock-sync as a devbin built-in for Cargo projects (it is in Gtools, Pplr and Ccusage); (g) the cut's closing 'use brew' note only where a project has the dev/opt/brew switch. Outside devbin: Lamplight moves to Geodica/Certificates' APPLE_APP_PASSWORD (Conflab's, which Apple accepts), then its own is revoked.

## Holds

- hv's step 8 from iTerm, the last step of the 2.0.0 sitting (ST0002 AC-06.2): pplr sync --check; brew reinstall pplr; pplr sync --check, reporting only whether each prompted, the prompt's words and the two exit codes (the check's body names people). Released when hv reports it: vc satisfies AC-06.2, dc closes WP06 beside WP04 and WP07 (dc's doing 7), and vc runs st done ST0002. Also left from step 7: one pplr:// link clicked, which should open in brew's pplr. (edited)

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).
- For hv, dc's finding: the estate page draws every slot hold as orphaned, because intentd runs under launchd without CLAUDE_CONFIG_DIR and so finds no Claude session files. The CLI is right: read slot state from intent estate status and intent estate slots watch, not the page. The fix is Intent's, out of pplr's scope (hold 2 item 4).

## Decisions

- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- hv, 2026-11-03 during the 2.0.0 sitting, after the release runbook's manual steps and a dead notary password (Intent's app-specific password, 401; Conflab's, which Apple accepts, is now Geodica/Certificates' APPLE_APP_PASSWORD and the keychain's): 'agreed, finish 2.0.0 then devbin one-command release'. 2.0.0 finishes on dc's runbook. Then, first job after: one command, bin/devbin release vX, runs the gates, CI, the tag, the Mac's signing and notarisation, attach and formula publish. That is two devbin changes: a cut step for an asset built and signed locally, before attach and before after:; and bare-binary signing in devbin's built-in macos (Intent's signer already does it). Devbin is un-parked for it, and every Geodica project releases the same way.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
