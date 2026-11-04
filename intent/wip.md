---
verblock: "2026-11-04:v1.2: matts - a patch of each end to end, then Molt 1.4.1 and the molt run"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Release: a patch of each, end to end by one command, then Molt 1.4.1 and the molt run on every sleeve**

- hv's end-to-end test from pristine trees: `bin/devbin release --patch` in pplr (2.0.2), Cldr (0.2.2) and ccusage (0.1.8), one after another, each tip verified first in a fresh clone with the stamped test run.
- Molt 1.4.1: its pplr liberator in the Homebrew shape (vc todo 12), cut by Molt's own hand steps; then `molt upgrade --self && molt upgrade && molt maintain` on rhadamanth, gyges and kovacs.
- vc holds the pen and slot control. Scope rule: simpler or easier only (vc decision 2).

## Active Steel Threads

- None in Pplr.

## Upcoming Work

- The notary credentials: all three signers move to one notarytool keychain profile (`--keychain-profile`), which hv creates once, together with the signer diagnostics the reviews left (vc todo 11).
- The Linux release assets and their workflow, in a later release.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Before a cut, verify the tree as the cut stamps it: `bin/devbin version set X.Y.Z` and `bin/devbin lock-sync` in the verification clone, then `test all` again.
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
