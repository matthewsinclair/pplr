---
verblock: "2026-11-03:v0.4: matts - WP13 landed, the scripts moved into devbin; the push and the closes next"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: port Pplr to one Rust binary; ST0002 ships it through Homebrew, signed**

- vc holds the pen and slot control (hv's words recorded on the take). Scope rule while it lasts: simpler or easier only, nothing new.
- Next: vc verifies main in a fresh clone, then hv pushes it. Main's CI gives WP13's Linux evidence.

## Active Steel Threads

- **ST0001** (17 of 28 criteria):
  - WP13 (cc): after the push and CI, cc records the AT-00 rows with their reds and greens, rewords AC-01.3, and withdraws AC-02.2 with AT-02.2 marked n/a; then wp gate and wp done. vc satisfies AC-00.5 once the live parity's sync rows are in, and AC-00.8 on the docs.
  - WP09 (dc): AC-09.1 needs its covering AT with a measured red; AC-09.2 needs hv's live Contacts run.
  - WP08 (dc): AC-08.1 is satisfied and nothing else holds it; dc gates and closes it.
- **ST0002** (6 of 17 criteria):
  - WP08 (dc): AT-08.1's rows and the package's body, then wp done once vc has verified main.
  - WP04: AC-04.1 waits on a devbin fix to its generated manifest header and on Developer ID signing; AC-04.2 needs hv's tap edit and push.
  - WP05: prepared up to `brew install`, held for hv. WP07: Developer ID and notarisation are hv's.
  - Not started: WP06, the 2.0.0 release and the move to brew, on hv's approval.

## Upcoming Work

- After the push: localfold and compact for vc, cc and dc.
- hv's held list is hold 2 on vc's board: the live parity's sync rows from iTerm and the live Contacts test (before the next reboot), the keychain entries, Developer ID signing, the Intent fixes, the brew install, and the Devbin issue with the tap's description and CI.
- Candidates after the cutover, each under the scope rule: todo 6 on vc's board.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- A porting WP is not done while a skip naming it fires against Rust (rule 8, refined).
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
