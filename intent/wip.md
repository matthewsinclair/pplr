---
verblock: "2026-11-04:v1.3: matts - Molt 1.4.1 and the molt run on every sleeve"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Molt 1.4.1, then the molt run on every sleeve**

- Molt 1.4.1: its pplr liberator in the Homebrew shape, with pplr's own dev or opt tree named and never faulted (vc todo 12), cut by Molt's own hand steps. Then `molt upgrade --self && molt upgrade && molt maintain` on rhadamanth, gyges and kovacs.
- vc holds the pen and slot control. Scope rule: simpler or easier only (vc decision 2).

## Active Steel Threads

- None in Pplr.

## Upcoming Work

- The notary credentials: all three signers move to one notarytool keychain profile (`--keychain-profile`), which hv creates once, together with the signer diagnostics the reviews left (vc todo 11).
- hv's call: a `version` subcommand for cldr and ccusage, as pplr has (vc todo 13).
- The Linux release assets and their workflow, in a later release.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Before a cut, verify the tree as the cut stamps it: `bin/devbin version set X.Y.Z` and `bin/devbin lock-sync` in the verification clone, then `test all` again.
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
