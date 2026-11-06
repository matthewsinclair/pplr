---
verblock: "2026-11-06:v1.6: matts - globalfold: ISSUE:0001 and 0002 open, devbin 0323's adoption next"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**Pplr is taking notes and issues, and fixes none until hv schedules them**

- ISSUE:0001: Pplr's own `pplr use`, `pplr install` and `pplr version` go when Pplr adopts devbin#0326, replaced by `bin/devbin use` and `bin/devbin install` run in the checkout, with `pplr --version` printing the core's line. It waits for the devbin release that carries 0326 (its units have landed on devbin's main, its docs to come, unpushed); dc then builds it to the issue on devbin-vc's word (dc's hold 7). Until then the working switch stays and no verb is added.
- ISSUE:0002: Pplr's refusals print `Error: <message>`, matching neither devbin's convention nor Intent's. Recorded only: which convention Pplr takes is hv's call when the fix is scheduled.
- devbin 0323's adoption (vc todo 11): devbin v0.1.20 carries the core's ship of a Rust CLI, and Pplr vendors 0.1.20, but Pplr's `bin/.devbin/cmd/{lock-sync,macos,post-cut}` copies are still in the tree. Adopting it deletes them, declares Pplr's ship in config, and moves the signer to one notarytool keychain profile, which hv stores once; it is done when Pplr cuts a release with no project ship command left. hv's to schedule.

## Active Steel Threads

- None in Pplr.

## Upcoming Work

- The Linux release assets and their workflow, in a later release.

## Notes

- Nothing from `~/Dropbox/Career/People` ever enters this repo (D11); `bin/devbin people-leak` guards every commit.
- Build and test through `bin/devbin`. Each cargo or bats run takes a slot on the estate desk (`intent estate slots`), which devbin-vc runs.
- Before a cut, verify the tree as the cut stamps it: `bin/devbin version set X.Y.Z` and `bin/devbin lock-sync` in the verification clone, then `test all` again.
- Slot results are written only from measured exit status; commits are gated with `&&` and made with `git commit --only` (the index is shared).
- Pplr's CI runs macOS on `macos-15` (8a1760f), as Cldr's and Ccusage's do: GitHub stranded `macos-latest` jobs four times on 2026-11-04.
