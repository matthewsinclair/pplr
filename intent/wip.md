---
verblock: "2026-11-04:v1.1: matts - pplr 2.0.1, Cldr 0.2.1 and ccusage 0.1.7 released, one command each"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Release: pplr, Cldr and ccusage each release signed and notarised by one command**

- Nothing is in progress. pplr 2.0.1 (733e127), Cldr 0.2.1 and ccusage 0.1.7 were each released by `bin/devbin release vX.Y.Z` alone on 2026-11-04 (vc decisions 28 and 29); ST0003 is done.
- vc holds the pen and slot control. Scope rule: simpler or easier only (vc decision 2).

## Active Steel Threads

- None in Pplr.

## Upcoming Work

- The notary credentials: all three signers move to one notarytool keychain profile (`--keychain-profile`), which hv creates once, together with the signer diagnostics the reviews left (vc todo 11).
- Credentials: Lamplight moves to `Geodica/Certificates`' `APPLE_APP_PASSWORD` (the one Conflab's CI already uses), then its own password is revoked.
- The Linux release assets and their workflow, in a later release.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Before a cut, verify the tree as the cut stamps it: `bin/devbin version set X.Y.Z` and `bin/devbin lock-sync` in the verification clone, then `test all` again.
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
