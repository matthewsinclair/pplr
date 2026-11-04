---
verblock: "2026-11-04:v1.4: matts - the molt run on every sleeve, devbin 0.1.20 and Molt 1.4.3"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**The molt run on every sleeve, then devbin 0.1.20 and Molt 1.4.3**

- hv runs `molt upgrade --self && molt upgrade && molt maintain` on rhadamanth, gyges and kovacs, to Molt 1.4.2. On rhadamanth, Molt-matts' uncommitted `config/arngz/desks/zeopit.yaml` is committed or restored first (vc todo 14).
- hv pushes Ccusage's validated fix-all (vc todo 15).
- devbin 0.1.20, the formula renderer's line-length fix (devbin#0322, built by devbin-dc under vc), then the fleet's vendored devbin and the three taps republished without their CI exceptions (vc todo 16).
- Molt 1.4.3: Ccusage ISSUE:0003's step 2 and tests.yml's tag trigger, by pplr-dc (vc todo 17), then ISSUE:0003's rollout (vc todo 15).
- vc holds the pen for pplr, cldr, ccusage, Molt's release and devbin-dc, and slot control. Scope rule: simpler or easier only (vc decision 2).

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
