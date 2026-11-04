---
verblock: "2026-11-04:v1.5: matts - EOD: Molt 1.4.3's re-cut, devbin 0.1.20, ISSUE:0003's restart"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Molt 1.4.3's re-cut, then ISSUE:0003's rollout; devbin 0.1.20's open issues**

- Molt 1.4.3 (Molt's own nodes, vc decision 31): 0088, 0089 and 0091 are merged on Molt's main and the merged tree ran green under both bashes. hv's first cut stopped when GitHub stranded the candidate's `macos-latest` job; hv ruled to pin Molt's macOS job to `macos-15` and cut again with `bin/devbin release v1.4.3`. molt-vc owns the route (vc todo 15).
- After the cut, on rhadamanth: hv's `brew upgrade molt`, molt-dc's cherry-pick of the held Molt-matts commit, Molt-matts' uncommitted `config/arngz/desks/homere.yaml` committed or restored, hv's molt upgrade, then ccusage-vc's session restart (ISSUE:0003 step 3) on hv's go; steps 4 and 5 each on hv's OK (vc todo 15).
- devbin 0.1.20: hv rules no cut until all 19 open devbin issues are in it; their runs come to vc's desk unit by unit (vc todo 16). Then hv's cut, the fleet's vendored devbin, and the three taps republished without their CI exceptions.
- devbin 0323 after that cut: the core carries a Rust CLI's ship, retiring pplr's, Cldr's and ccusage's `bin/.devbin/cmd/{lock-sync,macos,post-cut}`, with the notarytool keychain profile and the signer review findings (vc todo 11).
- vc holds the pen for pplr, cldr and ccusage, slot control on the estate desk, and coordination with Molt's and devbin's nodes. Scope rule: simpler or easier only (vc decision 2).

## Active Steel Threads

- None in Pplr.

## Upcoming Work

- hv's call: a `version` subcommand for cldr and ccusage, as pplr has (vc todo 13).
- The Linux release assets and their workflow, in a later release.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot from vc (`intent estate slots`).
- Before a cut, verify the tree as the cut stamps it: `bin/devbin version set X.Y.Z` and `bin/devbin lock-sync` in the verification clone, then `test all` again.
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
- Pplr's CI runs macOS on `macos-15` (8a1760f), as Cldr's and Ccusage's do: GitHub stranded `macos-latest` jobs four times on 2026-11-04.
