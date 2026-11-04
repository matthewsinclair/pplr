---
node: vc
name: vc
role: vc
session_id: 9417a4f6-ca66-4fc9-8f2e-e1c88bf255dd
heartbeat_at: 2026-11-04 09:02Z
status: active
focus: "Driving hv's end-to-end test (doing 3): a patch of pplr, Cldr and ccusage by bin/devbin release --patch, one after another, then Molt 1.4.1 and the molt run on rhadamanth, gyges and kovacs. vc holds slot control."
claims: []
---

# vc (vc)

## DOING

- hv's end-to-end test, 2026-11-04: bin/devbin release --patch in pplr (2.0.2), Cldr (0.2.2) and ccusage (0.1.8), one after another, then Molt 1.4.1 (todo 12) and molt upgrade --self && molt upgrade && molt maintain on rhadamanth, gyges and kovacs. Before each go: the tip verified in a fresh clone with the stamped run (vc's scratchpad verify-main.zsh <tag> for pplr; cldr-vc and ccusage-vc for theirs); a dry run in the shared checkout itself (porcelain empty, and only the two NOT REACHED lines: a third names dirt the stamp would refuse); that project's nodes frozen (no wb verb or pickup there); a slot booked for hv. After each: hv's brew upgrade and --version, the slot released, the freeze lifted, hv pushes the record commits. At 09:02Z: pplr 4b521af verifying in slot 01M6JWM8AJ4VEQT4DVNGCGDSMA; ccusage 27709bc verified by ccusage-vc, which is frozen, while ccusage-dc commits its dirty board and freezes; Cldr waits for cldr-dc's 0.2.2 notes (with cldr-vc's R1 and R2) and its board commit, then cldr-vc's stamped verify. pplr-dc does Molt's liberator after hv's compact. (edited)

## TODO

- After the three cuts (vc decision 29, item 4): pplr's, Cldr's and ccusage's signers move to one notarytool keychain profile (--keychain-profile), which hv creates once with xcrun notarytool store-credentials, and which an App Store Connect API key can back later with no code change. The same commit takes the diagnostics the reviews left in all three: fact()'s failures lost under local and inside a command's arguments (cldr-vc's nit (b)), and require_identity reading a failure of security itself (eg a locked keychain) as 'not in this keychain'. Each fails closed, under a misleading message, and cannot ship a wrong release. (edited)
- Molt 1.4.1, hv 2026-11-04: pplr-dc rewrites Molt's pplr liberator in cldr.sh's Homebrew shape (pplr 2.x has no bin/pplr, so molt upgrade fails on rhadamanth and gyges), then prepares the 'Release v1.4.1' commit. vc verifies it, and hv cuts by Molt's documented hand steps (tag, push, CI on the tag, tools/formula publish). Then hv runs molt upgrade --self && molt upgrade && molt maintain on rhadamanth, gyges and kovacs. Moving Molt onto devbin's one-command release is a later job, for Molt's own nodes.

## Holds

_(none)_

## Watch-outs

- For hv: Intent's pre-commit hook template word-splits staged Markdown paths (for f in $MD_FILES; prettier --list-different $MD_WHOLE), so it breaks on any .md with a space in its name. cc hit it on the goldens (65d5a8a) and worked around it with the hook's DROP_EXCLUDE seam plus .prettierignore. The fix is Intent's, out of pplr's scope (hv: just Pplr for now).
- For hv: Intent's AGENTS.md generator writes 'swift build' and 'swift test' lines for any project declaring swift. pplr has no Package.swift (its sidecar is built with swiftc), but undeclaring swift would disarm the Swift critic, so the fix is Intent's, out of pplr's scope. dc noted the swiftc caveat in CLAUDE.md's project section (815210a).
- For hv, dc's finding: the estate page draws every slot hold as orphaned, because intentd runs under launchd without CLAUDE_CONFIG_DIR and so finds no Claude session files. The CLI is right: read slot state from intent estate status and intent estate slots watch, not the page. The fix is Intent's, out of pplr's scope (hold 2 item 4).
- For hv, cldr-vc's finding 2026-11-04: Intent's generated AGENTS.md has no user block and calls intent/llm/RULES.md and ARCHITECTURE.md 'seeded empty by intent init' even once a project has written them. An agent that reads AGENTS.md alone (Codex, Cursor) misses the project's contract, including Cldr's public-repo rule against personal data. Claude Code loads it through CLAUDE.md's imports, and pplr is in the same state. The fix is Intent's, out of scope here.
- For hv, cldr-vc's F8, 2026-11-04: devbin's formula finish_ruby writes the install manifest as one line of 131 columns (with a 40-hex commit), over brew style's 118. So a tap whose CI runs brew style or brew audit goes red at publish. Cldr's tap carries --except-cops Layout/LineLength until devbin wraps the line. pplr 2.0.0's formula has the same line, unseen because homebrew-pplr has no CI, and ccusage's tap has style and audit off for devbin's formula:424. The fix is devbin's.
- For hv, cldr-vc's finding 2026-11-04: declaring shell makes Intent's generated AGENTS.md list 'Bats testing framework' and 'bats -r tests/', and say shell scripts are 'POSIX-compliant where practical'. In Cldr, which has no bats suite and zsh scripts by design, the command exits 1 ('Found no tests'). Cldr's intent/llm/RULES.md, imported by CLAUDE.md, says the lines do not apply, as pplr's CLAUDE.md does for its generated swift lines. The template fix is Intent's.
- For hv, another project's: Lamplight moves to Geodica/Certificates' APPLE_APP_PASSWORD (the one Conflab's CI already uses), and then its own password is revoked.

## Decisions

- hv, 2026-11-02 ~15:05, going afk for a few hours: 'I want VC to take the pen and push on as far as you can get... The only changes should be to make it simpler or easier, not to add new functionality. DC and [cc] are to follow instruction.' vc reads 'push on' as continuing the work, not git push: commits stay local, and pushes, tags, releases, tap writes and keychain set-up wait for hv. Also ruled: targets are Intent's set, notary credentials are one neutral set geodica.apple.*, and Pplr Links.app ships prebuilt and signed (5574e38).
- hv, 2026-11-04, four rulings asked in vc's session, each the recommended option: (1) one picture rule, first by name: pplr pictures and the index take what tree::PICTURES finds, a ruled divergence in CHANGELOG 2.0.1 (ST0003 WP05). Re-confirmed by hv the same morning on corrected facts: none of the 357 live people with a picture holds two, but the tags index changes for 2 of them, whose picture is named other than '<First Surname> (Picture)'. It now names their real picture where 2.0.0 named a missing .jpg. pplr pictures and the golden fixture do not change, and the index's no-picture fallback stays as 2.0.0's. (2) The one walker names each directory it cannot read, one stderr line each, the walk carrying on and the exit code unchanged, a ruled divergence (ST0003 WP04). (3) Patch versions: Cldr v0.2.1, ccusage v0.1.7, pplr v2.0.1. (4) The notary credentials move to one notarytool keychain profile (--keychain-profile) in all three signers AFTER this morning's three cuts, which use the geodica.apple.* items as pplr 2.0.0 did; an App Store Connect API key can later back the profile with no code change. (edited)

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
