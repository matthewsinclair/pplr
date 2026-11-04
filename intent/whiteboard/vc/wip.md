---
node: vc
name: vc
role: vc
session_id: 9417a4f6-ca66-4fc9-8f2e-e1c88bf255dd
heartbeat_at: 2026-11-04 05:04Z
status: active
focus: "Driving pplr, cldr and ccusage to one release shape this morning (vc decision 28): signed and notarised, one devbin command each. ST0003 open for pplr-dc (todo 6, then 2.0.1); Cldr v0.2.1 prep with cldr-dc and cldr-vc; ccusage v0.1.7 post-cut with ccusage-dc and ccusage-vc. vc holds slot control."
claims: []
---

# vc (vc)

## DOING

- ST0003 (vc decision 28): todo 6's candidates as WP01 to WP05, pplr-dc implementing and vc reviewing each against its criterion, then WP06, pplr 2.0.1 by bin/devbin release v2.0.1, the first real run of post-cut. WP05 (one picture rule) and the stderr note in WP04 wait for hv's rulings.
- The three one-command releases (vc decision 28): Cldr v0.2.1, its first signed one (cldr-dc's prep commit: version_from, lock-sync, doctor gate, CHANGELOG; then its signer proof; cldr-vc validates and dry-runs). ccusage v0.1.7 (ccusage-dc: post-cut over macos finalize and formula publish, notarised always with geodica.apple.*; ccusage-vc validates and dry-runs). pplr v2.0.1 after ST0003. Each cut is hv's one command. Cldr's CI on f382994 is green (run 41694524688). Lamplight's move to Geodica/Certificates' APPLE_APP_PASSWORD, then its own revoked, stays hv's.

## TODO

- After the three cuts (vc decision 29, item 4): pplr's, Cldr's and ccusage's signers move to one notarytool keychain profile (--keychain-profile), which hv creates once with xcrun notarytool store-credentials, and which an App Store Connect API key can back later with no code change. The same commit takes cldr-vc's nit (b) in all three: fact()'s failures are lost under local and inside a command's arguments, so a missing signing fact surfaces later under a misleading message (it cannot ship: verify's requirement or Apple refuses).

## Holds

_(none)_

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).
- For hv, dc's finding: the estate page draws every slot hold as orphaned, because intentd runs under launchd without CLAUDE_CONFIG_DIR and so finds no Claude session files. The CLI is right: read slot state from intent estate status and intent estate slots watch, not the page. The fix is Intent's, out of pplr's scope (hold 2 item 4).
- For hv, cldr-vc's finding 2026-11-04: Intent's generated AGENTS.md has no user block and calls intent/llm/RULES.md and ARCHITECTURE.md 'seeded empty by intent init' even once a project has written them. An agent that reads AGENTS.md alone (Codex, Cursor) misses the project's contract, including Cldr's public-repo rule against personal data. Claude Code loads it through CLAUDE.md's imports, and pplr is in the same state. The fix is Intent's, out of scope here.
- For hv, cldr-vc's F8, 2026-11-04: devbin's formula finish_ruby writes the install manifest as one line of 131 columns (with a 40-hex commit), over brew style's 118. So a tap whose CI runs brew style or brew audit goes red at publish. Cldr's tap carries --except-cops Layout/LineLength until devbin wraps the line. pplr 2.0.0's formula has the same line, unseen because homebrew-pplr has no CI, and ccusage's tap has style and audit off for devbin's formula:424. The fix is devbin's.

## Decisions

- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- hv, 2026-11-03 after ST0002's close: a release is one command, and pplr and cldr both release signed and notarised the same way. Utilz already does (its after: runs tools/post-cut), so this is pplr's config, not devbin's: release.after: is one line, bin/devbin post-cut, which in a detached worktree at the tag builds, runs macos prepare, attaches with release attach --from dist/release, then formula publish, and pulls this machine's tap, stopping at the first failure; a re-run skips what is done. Reverses decision 25's by-hand formula publish and supersedes decision 26's devbin changes. Next: Cldr gets the same (devbin installed, a signer and a post-cut), in Cldr's repo.
- hv, 2026-11-04, in vc's session: 'you have the pen here with pplr and cldr... I want both properly finished this morning', then ccusage-vc and ccusage-dc added: 'I want all three of these rust cli utils to operate in the same way (ie signed and notarised, and be able to be built from a single devbin command)'. vc drives pplr-dc, cldr-vc, cldr-dc, ccusage-vc and ccusage-dc. The shape, decision 27's, for all three: bin/devbin release vX.Y.Z is the whole release, project.version_from: Cargo.toml with derived: bin/devbin lock-sync, a CHANGELOG section the cut dates, and one after: line, bin/devbin post-cut, that signs and notarises with the neutral credentials geodica.apple.*, attaches, publishes the formula and pulls the tap, stopping at the first failure. ccusage keeps CI's four archives, so its post-cut runs macos finalize on CI's draft. pplr's todo 6 becomes ST0003, released as 2.0.1. Cuts, tags and pushes stay hv's.
- hv, 2026-11-04, four rulings asked in vc's session, each the recommended option: (1) one picture rule, first by name: pplr pictures and the index take what tree::PICTURES finds, a ruled divergence in CHANGELOG 2.0.1 (ST0003 WP05; none of the 358 live About folders holds two pictures); (2) the one walker names each directory it cannot read, one stderr line each, the walk carrying on and the exit code unchanged, a ruled divergence (ST0003 WP04; it goes if WP04's walker is cancelled); (3) patch versions: Cldr v0.2.1, ccusage v0.1.7, pplr v2.0.1; (4) the notary credentials move to one notarytool keychain profile (--keychain-profile) in all three signers AFTER this morning's three cuts, which use the geodica.apple.* items as pplr 2.0.0 did; an App Store Connect API key can later back the profile with no code change.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
