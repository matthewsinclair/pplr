---
verblock: "2026-11-02:v0.3: matts - localfold after hv's afk stretch"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: port Pplr to one Rust binary; ST0002 ships it through Homebrew, signed**

- vc holds the pen and slot control (hv's words recorded on the take). Scope rule while it lasts: simpler or easier only, nothing new.
- vc verified HEAD e40bdee in a fresh clone, every gate exit 0 (vc decision 6). GitHub has 0013d4c. The Dropbox `local` remote push hung in git-receive-pack (no hooks, nothing dataless); it needs a look on hv's side.

## Active Steel Threads

- **ST0001** (10 of 28 criteria):
  - Done: WP01, WP03, WP04, WP05, WP06, WP07, WP12. The replay is 41 of 54 cases native, all matching; bats 101/101 on Rust and legacy.
  - WIP: WP02 (AC-02.1's CI half, now pushed), WP08 (dc: the signed app and one `handler --install`, both hv's), WP09 (AC-09.1 at WP13, AC-09.2 needs a Contacts grant), WP10 (cc).
  - Not started: WP11 (cc), WP14 (offered to dc if cc agrees), WP13 (cc, last).
- **ST0002** (5 of 16 criteria): Done WP01, WP03. WIP WP02 (CI half), WP04 (AC-04.1 blocked on devbin's generated manifest line, over brew's 118 columns; AC-04.2 needs hv's tap edit and push), WP05 (prepared up to `brew install`, held for hv), WP07 (ad hoc only; Developer ID and notarisation are hv's). WP06 is the release, hv's approval.

## Upcoming Work

- After the compact: cc on WP10 then WP11; dc on WP14 (or as vc directs); vc reviews and grants slots.
- hv's held list is hold 2 on vc's board (push, Contacts grant, keychain entries, signing and notarisation, brew install, tap edit, a Devbin issue, two Intent fixes).

## Notes

- D3 as amended: no `bin/` or `lib/` file is deleted before WP13. hv's live `~/.local/bin/pplr` runs the checkout's `bin/pplr`.
- Rule 8 (refined): a porting WP is not done while a skip naming it fires against Rust. Rule 9: WP13 re-enables IN-SW-CODE-006.
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `tools/people-leak` guards every commit.
