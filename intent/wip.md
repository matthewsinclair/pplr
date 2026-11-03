---
verblock: "2026-11-03:v0.5: matts - pushed at eb15640; WP13's ATs and close, then ST0002's held steps"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: port Pplr to one Rust binary; ST0002 ships it through Homebrew, signed**

- vc holds the pen and slot control (hv's words recorded on the take). Scope rule while it lasts: simpler or easier only, nothing new.
- Next: cc closes ST0001 WP13, then the thread.

## Active Steel Threads

- **ST0001** (21 of 28 criteria):
  - WP13 (cc): the AT-00 rows for the seven open criteria, each with its measured red and its green (CI runs 41619489383 and 41621882730, vc's local macOS runs at b245b53 and 4beaa87). Then reword AC-01.3, withdraw AC-02.2 with AT-02.2 marked n/a, and run at lint, wp gate and wp done.
- **ST0002** (7 of 17 criteria):
  - WP04: AC-04.1 waits on a devbin fix to its generated manifest header and on Developer ID signing; AC-04.2 needs hv's tap edit and push.
  - WP05: prepared up to `brew install`, held for hv. WP07: Developer ID and notarisation are hv's.
  - Not started: WP06, the 2.0.0 release and the move to brew, on hv's approval.

## Upcoming Work

- hv's held list is hold 2 on vc's board: the keychain entries, Developer ID signing, the Intent fixes, the brew install, and the Devbin issue with the tap's description and CI.
- Candidates after the cutover, each under the scope rule: todo 6 on vc's board.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
