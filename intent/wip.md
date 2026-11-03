---
verblock: "2026-11-03:v0.7: matts - 2.0.0 shipped via brew; ST0002 closes, then one-command devbin release"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0002: close the Homebrew thread, then make a release one command**

- vc holds the pen and slot control. Scope rule: simpler or easier only (vc decision 2).
- Next: hv's step 8 from iTerm, the Contacts check across a reinstall (AC-06.2, hold 2 on vc's board). Then dc closes WP04, WP06 and WP07 with design.md's record (dc's doing 7), and vc runs `intent st done ST0002`.
- Then the first job after: `bin/devbin release vX` as the one release command (vc decision 26, todo 8).

## Active Steel Threads

- **ST0002** (15 of 16 criteria; AC-05.1 withdrawn): AC-06.2 waits for step 8. WP04 and WP07 close at dc's next pickup; WP06 once AC-06.2 is satisfied.

## Upcoming Work

- Devbin's one-command release (todo 8 on vc's board): a cut step for an asset built and signed locally, bare-binary signing in devbin's built-in `macos`, one keychain prefix for the estate, deterministic asset tarballs, and `lock-sync` as a built-in.
- Credentials: Lamplight moves to `Geodica/Certificates`' `APPLE_APP_PASSWORD` (the one Conflab's CI already uses), then its own password is revoked.
- cc: ST0001's views leave the disk (`intent sync --apply`), with `CLAUDE.md`'s pointer to its `design.md` reworded.
- The Linux release assets and their workflow, in a later release; todo 6's behaviour-preserving cleanups, in a thread of their own.
- Main's commits after the release (dd6dc5b) are local until hv pushes.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
