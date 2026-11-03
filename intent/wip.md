---
verblock: "2026-11-03:v0.9: matts - pplr's release is one command (post-cut); Cldr next"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Release: pplr and Cldr, signed and notarised, one command each**

- vc holds the pen and slot control. Scope rule: simpler or easier only (vc decision 2).
- pplr: `bin/devbin release vX.Y.Z` is the whole release; its `after:` is `bin/devbin post-cut` (vc decision 27). Its first real run is the next pplr release.
- Cldr: the same shape, in Cldr's repo (todo 9 on vc's board).

## Active Steel Threads

- None in Pplr.

## Upcoming Work

- Credentials: Lamplight moves to `Geodica/Certificates`' `APPLE_APP_PASSWORD` (the one Conflab's CI already uses), then its own password is revoked.
- The Linux release assets and their workflow, in a later release; todo 6's behaviour-preserving cleanups, in a thread of their own.
- dc unclaims ST0002/04 and ST0002/07 at its next pickup.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
