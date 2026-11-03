---
verblock: "2026-11-03:v0.8: matts - ST0002 closed; next, the one-command devbin release"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Devbin: make a release one command**

- vc holds the pen and slot control. Scope rule: simpler or easier only (vc decision 2).
- `bin/devbin release vX` as the one release command (vc decision 26, todo 8; ST0002 design.md ruling 10). Devbin is un-parked for it.
- Main's commits after the release (dd6dc5b) are local until hv pushes.

## Active Steel Threads

- None in Pplr. The devbin work lands in Devbin, shared by every Geodica project.

## Upcoming Work

- Devbin's one-command release (todo 8 on vc's board): a cut step for an asset built and signed locally, bare-binary signing in devbin's built-in `macos`, one keychain prefix for the estate, deterministic asset tarballs, `require_identity` capturing its output before matching, and `lock-sync` as a built-in.
- Credentials: Lamplight moves to `Geodica/Certificates`' `APPLE_APP_PASSWORD` (the one Conflab's CI already uses), then its own password is revoked.
- The Linux release assets and their workflow, in a later release; todo 6's behaviour-preserving cleanups, in a thread of their own.
- dc unclaims ST0002/04 and ST0002/07 at its next pickup.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
