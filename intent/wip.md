---
verblock: "2026-11-03:v0.6: matts - ST0001 closed; ST0002 cut to 2.0.0 on macOS, hv's release sitting next"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0002: ship pplr 2.0.0 through Homebrew, signed and notarised, macOS first**

- vc holds the pen and slot control. Scope rule: simpler or easier only. hv, 2026-11-03: be aggressive and ship asap (vc decisions 24 and 25 record what that cut).
- Next: hv's one release sitting, from dc's runbook (hold 2 on vc's board).

## Active Steel Threads

- **ST0002** (7 of 16 criteria; AC-05.1 withdrawn, WP05 cancelled):
  - 2.0.0 ships the signed macOS asset alone, and the formula declares macOS alone; the Linux tarballs and their workflow come in a later release.
  - WP07: AT-07.1's green is hv's `bin/devbin macos prepare` at the tag (sign, verify, notarytool Accepted), after the three `geodica.apple.*` keychain entries.
  - WP06: `bin/devbin release v2.0.0`, then the attach from `dist/release`, then `bin/devbin formula publish` by hand; then rhadamanth moves to brew (AC-00.1 to AC-00.3, AC-06.1, AC-06.2).
  - WP04: AT-04.1's green is the render plus hv's brew install and `brew test` from the tap; AC-04.2 is the tap's description.

## Upcoming Work

- After the release: devbin's manifest-header fix brings back `brew style`, `brew audit --strict` and the tap's CI (the Devbin issue, hold 2 item 7); the Linux assets with their workflow; todo 6's behaviour-preserving cleanups, in a thread of their own.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
