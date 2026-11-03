---
st_id: ST0002
title: Create a homebrew distro for the new Rust version of Pplr
---

# ST0002: Create a homebrew distro for the new Rust version of Pplr -- Acceptance

> **THIS FILE IS A GENERATED VIEW, AND A ROW AUTHORED HERE IS DISCARDED BY THE NEXT SYNC.** The acceptance contract is canon in the thread model; this file renders it. Acceptance Criteria (AC) are the ratified completeness boundary; Acceptance Tests (AT) are the small red-to-green tests that prove them.
>
> Done = every AC is covered by a GREEN AT, or (for a non-test AC) its named evidence is satisfied, AND the AC set is the ratified full boundary. Done is read from this map, never from a hand-ticked box.
>
> Test-backed satisfaction is COMPUTED from covering green ATs and never stored -- storing it would be double truth. An AC has four states, not two: beyond satisfied and unsatisfied, a requirement can be **descoped** to a named thread or **withdrawn** with its reason on the record. Both are non-blocking and both are reported separately, so a thread that descoped half its contract looks like one.

## Acceptance Criteria

### ST-level

- AC-00.1 (non-test) From the tap on rhadamanth, `HOMEBREW_NO_AUTO_UPDATE=1 brew install matthewsinclair/pplr/pplr` succeeds, `brew test pplr` passes (including the signature check), and `pplr --version` reads `pplr 2.0.0 (brew, <sha12>)`, naming the release commit. -- evidence: hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8): with HOMEBREW_NO_AUTO_UPDATE=1, brew install matthewsinclair/pplr/pplr installed 2.0.0 from the tap after codesign --verify --strict -R on pplr, pplr-contacts and Pplr Links.app; brew test passed, re-verifying the three and running the version line; pplr --version read pplr 2.0.0 (brew, dd6dc5b96a2f), dd6dc5b being the release commit (tag v2.0.0) -- satisfied: yes
- AC-00.2 (non-test) T10 holds on rhadamanth: `pplr use dev`, `use opt` and `use brew` each give the right version line, a clean doctor and a passing shadow check. -- evidence: hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8), step 7: bin/devbin t10-probe ~/Devel/prj/Pplr, with the opt tree installed at ~/Devel/scratch/pplr-opt: dev, opt and brew each ok on use, on a version line naming the tree (pplr 2.0.0 (dev|opt|brew, dd6dc5b96a2f)), on doctor's Install section and on the shadow check; it ended on brew -- satisfied: yes
- AC-00.3 (non-test) rhadamanth runs brew's pplr, with no link into the old checkout. The user docs name brew alone (T1), and docs/releasing.md covers dev and opt for the maintainer. -- evidence: hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8), step 7: pplr --version read pplr 2.0.0 (brew, dd6dc5b96a2f) with ~/.local/bin/pplr gone, so nothing links into the old checkout; doctor's Install section passed (brew answers, the manifest and all three signatures); the pplr:// handler is brew's Pplr Links.app, the one lsregister lists. The docs landed in 02c2771: the README names Homebrew alone, and docs/releasing.md covers dev, opt, signing and the cut -- satisfied: yes

### WP-01 -- Release and install design for pplr (status: Done)

- AC-01.1 (non-test) The release and install design (E1 to E8, the T1 to T10 table, the install layout) is settled on this thread, with hv's answers to its open questions recorded. -- evidence: install-release.md on ST0002 (settled; vc accepted amendments 1-5; hv's rulings 5574e38) -- satisfied: yes

### WP-02 -- devbin release core and CI candidates (status: Done)

- AC-02.1 `bin/devbin release check` and `bin/devbin doctor` pass with the release and formula config, and tests.yml fires on candidate/** and runs the cargo gates and the bats suite on macOS and Linux. -- satisfied: yes (computed)
- AC-02.2 (non-test) tools/lock-sync (zsh) rewrites Cargo.lock for a version bump and prints Cargo.lock, and the CHANGELOG opens 2.0.0 as `## [2.0.0] - unreleased`. -- evidence: scratch-clone runs for 111b72e: version-only lock rewrite printing Cargo.lock; new-dependency refusal restoring the lock byte for byte; CHANGELOG opens ## [2.0.0] - unreleased -- satisfied: yes

### WP-03 -- Install, manifest, tree identity, version line and use (status: Done)

- AC-03.1 `pplr install` publishes only from a clean dev tree and writes the T3 manifest, and it refuses an existing prefix without --force, or a prefix another tool wrote. -- satisfied: yes (computed)
- AC-03.2 The version line reads `pplr <version> (<tree>, <sha12>[+dirty])` in dev, opt and brew, built from stamps taken at build time, with the tree named by the one tree function. -- satisfied: yes (computed)
- AC-03.3 `pplr use dev|opt|brew` moves only the link set {pplr} in ~/.local/bin, and refuses a missing tree or a brew copy that would not answer. Doctor's shadow check fails on a link that is shadowed or split across trees. -- satisfied: yes (computed)

### WP-04 -- Assets formula, tap and tap CI (status: Done)

- AC-04.1 packaging/homebrew/pplr.rb is a devbin `assets` template, rendered through `bin/devbin formula render` with nothing typed. Its install verifies every macOS binary's signature against its designated requirement and writes the T3 manifest, and its test re-verifies the signatures and runs the version line. -- satisfied: yes (computed)
- AC-04.2 (non-test) The tap matthewsinclair/homebrew-pplr is described as "Homebrew tap for pplr". -- evidence: hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8), step 5: gh repo view matthewsinclair/homebrew-pplr --json description printed {"description":"Homebrew tap for pplr"}, and vc read the same through gh api; the tap's CI waits for devbin's manifest-header fix (vc decision 24) -- satisfied: yes

### WP-05 -- Walking skeleton through a scratch tap (status: Cancelled)

- AC-05.1 (non-test) A scaffold build installs from a locally rendered formula in a scratch tap on rhadamanth, T10's three trees are proved, and nothing is tagged or pushed on the real repository. -- WITHDRAWN: superseded by the real install from the tap (AC-00.1, AC-00.2): hv ruled no rehearsal, vc decision 24 (by vc)

### WP-06 -- Release 2.0.0 and move rhadamanth to brew (status: Done)

- AC-06.1 (non-test) 2.0.0 is cut by `bin/devbin release`. Its gates pass locally and in CI for candidate/v2.0.0, and the formula is published to the tap by `bin/devbin formula publish`. -- evidence: hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8): bin/devbin release v2.0.0 in a fresh release clone made dd6dc5b and passed its gates there (check all; test all with cargo 107 unit and 39 cli, bats 106 of 106, the Linux test the one skip; doctor) and CI 41646707500 on candidate/v2.0.0 on both runners; it tagged v2.0.0 (d32ed6e), pushed to local and upstream and made the release; bin/devbin formula publish pushed Formula/pplr.rb to the tap, whose main is 4fc7022 -- satisfied: yes
- AC-06.2 (non-test) The Contacts grant holds across upgrades: after a `brew upgrade` or `brew reinstall`, `pplr sync --check` from the keg reads the live store with no new permission prompt, and any first-use prompt for the signed sidecar is recorded. -- evidence: hv's 2.0.0 release sitting on 2026-11-03, step 8, from iTerm: pplr sync --check, then brew reinstall pplr (codesign --verify ran on pplr, pplr-contacts and Pplr Links.app; the 2.0.0 keg installed), then pplr sync --check: macOS showed no Contacts prompt at either read, both reads returned the same report line for line, and pplr resolved to brew's (pplr 2.0.0 (brew, dd6dc5b96a2f)). A pplr:// link clicked opened in brew's pplr. -- satisfied: yes

### WP-07 -- Developer ID signing and notarisation (status: Done)

- AC-07.1 Every macOS binary pplr ships (`pplr`, `pplr-contacts` and the `Pplr Links.app` from `pplr handler`) is signed with Developer ID Application: Geodica Pty Ltd (76BQL8L47U), with the hardened runtime and a secure timestamp, and passes `codesign --verify --strict -R <requirement>` and `spctl`. -- satisfied: yes (computed)
- AC-07.2 (non-test) The signer is project code in bash or zsh (`bin/.devbin/cmd/macos`, no Python). It runs on rhadamanth only, notarises every release build with `notarytool`, and the release cannot attach an unsigned macOS asset. -- evidence: bin/.devbin/cmd/macos is zsh and refuses on CI, off macOS and without the identity; in hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8) it notarised both builds with notarytool (the proof at main and the release at dd6dc5b, submission 7feafe51-18a8-4e39-bae8-c8500c07fc47, each Accepted). pack refuses a binary that fails verify, spctl included (6b1328f), and packs the signed asset alone into dist/release, the only folder bin/devbin release attach --from dist/release takes; an ad hoc asset goes to dist/adhoc (ef9bb46) -- satisfied: yes

### WP-08 -- tools/ becomes devbin project commands: bin/devbin the one entry point (status: Done)

- AC-08.1 Every script that was in tools/ is a devbin project command in bin/.devbin/cmd/, reached through bin/devbin, and tools/ is gone. -- satisfied: yes (computed)

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Release and install design for pplr (status: Done)

_(no tests in this group)_

### WP-02 -- devbin release core and CI candidates (status: Done)

- AT-02.1 `.github/workflows/tests.yml` -- covers AC-02.1 -- status: green -- tests.yml's push trigger with devbin's release gates: devbin release check validates the release block and asks whether a push of candidate/<tag> starts tests.yml, and devbin doctor checks the install and its config; red by a measured mutant of c4929d0, tests.yml's push branches without candidate/** (7e5e61d in dc's scratch clone): devbin release check exit 1, a push of candidate/v2.0.0 starts no run (ci-state --fires answers quiet); it runs no cargo, so no slot; green at c4929d0 unmutated in the same clone: devbin release check exit 0 (candidate/** matches candidate/v2.0.0) and devbin doctor exit 0 (16 checks, 0 failed), as at 251ea31 in the shared checkout, and release check 0 again with this header (f809e29 on f8b8db8). On CI 41601061427, hv's push of candidate/ci-probe at e589c45 started tests.yml through candidate/**, and the cargo gates and the bats suite passed on macOS and Linux

### WP-03 -- Install, manifest, tree identity, version line and use (status: Done)

- AT-03.1 `tests/cli.rs` -- covers AC-03.1 -- status: green -- red before fed44e6 (no install module, so these tests did not build; the old version test expected the bare version), green at fed44e6: cargo test 19 of 19 end to end
- AT-03.2 `tests/cli.rs` -- covers AC-03.2 -- status: green -- red before fed44e6 (no install module, so these tests did not build; the old version test expected the bare version), green at fed44e6: cargo test 19 of 19 end to end
- AT-03.3 `tests/cli.rs` -- covers AC-03.3 -- status: green -- red before fed44e6 (no install module, so these tests did not build; the old version test expected the bare version), green at fed44e6: cargo test 19 of 19 end to end

### WP-04 -- Assets formula, tap and tap CI (status: Done)

- AT-04.1 `packaging/homebrew/pplr.rb` -- covers AC-04.1 -- status: green -- packaging/homebrew/pplr.rb: rendered by bin/devbin formula render with nothing typed, its install lays the asset out through devbin's @@INSTALL@@ (the T3 manifest, 0755, the link) and runs verify_signatures, codesign --verify --strict -R against REQUIREMENTS, which src/doctor.rs's the_formula_checks_the_same_requirements_the_signer_signs_to holds equal to packaging/macos/signing; its test re-verifies and runs the version line; red at 15eb284 (on main as ef9bb46) in slot 01M6GJ1NAVRTACSS8QGR6Y7XX0: those requirement strings refuse an ad hoc build (bin/devbin macos verify exit 1, pplr does not meet its designated requirement); the render from bin/devbin skeleton --adhoc exit 0, no placeholder left, ruby -c clean, macOS alone; green at v2.0.0 in hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8): bin/devbin formula publish rendered the template with nothing typed and pushed Formula/pplr.rb to the tap (main 4fc7022); brew install matthewsinclair/pplr/pplr ran codesign --verify --strict -R on pplr, pplr-contacts and Pplr Links.app and installed 15 files with the manifest; brew test passed; brew's pplr --version read pplr 2.0.0 (brew, dd6dc5b96a2f), and doctor confirmed the manifest matches every runtime file

### WP-05 -- Walking skeleton through a scratch tap (status: Cancelled)

_(no tests in this group)_

### WP-06 -- Release 2.0.0 and move rhadamanth to brew (status: Done)

_(no tests in this group)_

### WP-07 -- Developer ID signing and notarisation (status: Done)

- AT-07.1 `bin/.devbin/cmd/macos` -- covers AC-07.1 -- status: green -- bin/.devbin/cmd/macos verify: each binary pplr ships (pplr, pplr-contacts, Pplr Links.app) is held to its designated requirement from packaging/macos/signing (Developer ID Application: Geodica Pty Ltd (76BQL8L47U)) with codesign --verify --strict -R, the hardened runtime and the entitlement read back, and spctl; red at 15eb284 (on main as ef9bb46) in slot 01M6GJ1NAVRTACSS8QGR6Y7XX0: bin/devbin macos verify on an ad hoc build from bin/devbin skeleton --adhoc exit 1, pplr does not meet its designated requirement; green at dd6dc5b, the release commit (tag v2.0.0), in hv's 2.0.0 release sitting on 2026-11-03 (vc slot 01M6GM1SJZYN8GS5NHE69DDNR8): bin/devbin macos prepare signed pplr, pplr-contacts and Pplr Links.app with the Developer ID, verify passed each designated requirement with the hardened runtime and the entitlement, notarytool Accepted (submission 7feafe51-18a8-4e39-bae8-c8500c07fc47), and pack's verify passed spctl before packing; brew install and brew test re-ran codesign --verify --strict -R on all three

### WP-08 -- tools/ becomes devbin project commands: bin/devbin the one entry point (status: Done)

- AT-08.1 `tests/project_commands.bats` -- covers AC-08.1 -- status: green -- tests/project_commands.bats: tools/ is gone, and every command in bin/.devbin/cmd/, the list read from the directory, resolves through bin/devbin from its project handler, by bin/devbin help --why; red at the series' base f15ccf8, the test copied into a worktree of it: bats exit 1, tools/ still there (slot 01M6GDHWBFZ8K4KW8FK93DP20B); green at a5dbb50 in the same slot: bats exit 0, and in devbin test all, bats 106 of 106 with 1 skip, the AC-00.7 Linux test; on main as 492df0e, which equals a5dbb50 outside intent/ at 3cd9879

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
