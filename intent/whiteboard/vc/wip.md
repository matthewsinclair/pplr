---
node: vc
name: vc
role: vc
session_id: 9417a4f6-ca66-4fc9-8f2e-e1c88bf255dd
heartbeat_at: 2026-11-04 09:37Z
status: active
focus: "pplr 2.0.2, Cldr 0.2.2 and ccusage 0.1.8 are out, end to end by one command each, and everything is pushed. Next, todo 12: Molt 1.4.1 (pplr-dc's fix 8384180 measuring), hv's hand cut, then the molt run on rhadamanth, gyges and kovacs."
claims: []
---

# vc (vc)

## DOING

_(none)_

## TODO

- After the three cuts (vc decision 29, item 4): pplr's, Cldr's and ccusage's signers move to one notarytool keychain profile (--keychain-profile), which hv creates once with xcrun notarytool store-credentials, and which an App Store Connect API key can back later with no code change. The same commit takes the diagnostics the reviews left in all three: fact()'s failures lost under local and inside a command's arguments (cldr-vc's nit (b)), and require_identity reading a failure of security itself (eg a locked keychain) as 'not in this keychain'. Each fails closed, under a misleading message, and cannot ship a wrong release. (edited)
- Molt 1.4.1, hv 2026-11-04. pplr-dc's fix for Molt issue 0086 (8384180 in its scratch clone molt-pplr) puts the pplr liberator in cldr.sh's Homebrew shape. A dev or opt tree that pplr use chose is info, not a fault (hv's 27 Oct rule in 0071, which stays open and deferred). Install removes only Molt's old checkout link, and the upgrade hook, pplr_repo and MOLT_PPLR_HOME are gone. vc reviewed it. At 09:26Z it is measuring in slot 01M6JXYPJVWNKJEB4HZ5200V72: pplr.bats red at 5c7aa30 and green at 8384180, then the whole suite under bash 5 and /bin/bash 3.2. Next: it lands on Molt main by path. Then the 'Release v1.4.1' commit, VERSION 1.4.1 and CHANGELOG's Unreleased closed as 1.4.1 with the clock's date, shipping 0083 to 0085 too (hv's yes). Then vc verifies Molt's tip in a fresh clone. Then hv's hand cut in ~/Devel/prj/Molt: git tag -a v1.4.1 -m 'v1.4.1: <issues, as v1.4.0's message lists them>' at the release commit; git push upstream main v1.4.1 && git push local main v1.4.1; tests.yml's run on the tag green; tools/formula publish. Then molt upgrade --self && molt upgrade && molt maintain on rhadamanth, gyges and kovacs (--self brew-upgrades molt first). (edited)
- For hv's call: cldr and ccusage have no version subcommand. hv typed 'cldr version' and 'ccusage version' on 2026-11-04, and both answered 'unknown command'; only pplr has one. Both answer --version. A version subcommand, and -v, in each is a small patch, so the three answer alike.
- The molt run's pre-flight, read-only, for todo 12. rhadamanth: Molt 1.4.1's own doctor and dry-run resleeve (a scratch clone of 2fab7ac) pass, pplr and cldr included; one blocker, hv's: Molt-matts has config/arngz/desks/zeopit.yaml modified and uncommitted, and molt upgrade (with --self too) refuses a user config repo with uncommitted changes. gyges and kovacs (molt 1.4.0's dry runs over ssh): every repo the run pulls is clean. gyges has no brew pplr and still has Molt's old link ~/.local/bin/pplr -> ~/Devel/prj/Pplr/bin/pplr, whose checkout holds .git, so 1.4.1's resleeve installs pplr from the tap and removes the link. maintain's brew upgrade takes gyges' cldr 0.2.0 and ccusage 0.1.5 to 0.2.2 and 0.1.8 (both trusted there). kovacs enables neither pplr nor cldr, and has no ccusage.

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
