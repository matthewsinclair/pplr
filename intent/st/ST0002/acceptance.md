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

- AC-00.1 (non-test) From the tap on rhadamanth, `HOMEBREW_NO_AUTO_UPDATE=1 brew install matthewsinclair/pplr/pplr` succeeds, `brew test pplr` passes (including the signature check), and `pplr --version` reads `pplr 2.0.0 (brew, <sha12>)`, naming the release commit. -- satisfied: no
- AC-00.2 (non-test) T10 holds: on rhadamanth, `pplr use dev`, `use opt` and `use brew` each give the right version line, a clean doctor and a passing shadow check, and on a Linux machine brew alone does. -- satisfied: no
- AC-00.3 (non-test) rhadamanth runs brew's pplr, with no link into the old checkout. The user docs name brew alone (T1), and docs/releasing.md covers dev and opt for the maintainer. -- satisfied: no

### WP-01 -- Release and install design for pplr (status: Done)

- AC-01.1 (non-test) The release and install design (E1 to E8, the T1 to T10 table, the install layout) is settled on this thread, with hv's answers to its open questions recorded. -- evidence: install-release.md on ST0002 (settled; vc accepted amendments 1-5; hv's rulings 5574e38) -- satisfied: yes

### WP-02 -- devbin release core and CI candidates (status: WIP)

- AC-02.1 `bin/devbin release check` and `bin/devbin doctor` pass with the release and formula config, and tests.yml fires on candidate/** and runs the cargo gates and the bats suite on macOS and Linux. -- satisfied: no (computed)
- AC-02.2 tools/lock-sync (zsh) rewrites Cargo.lock for a version bump and prints Cargo.lock, and the CHANGELOG opens 2.0.0 as `## [2.0.0] - unreleased`. -- satisfied: no (computed)

### WP-03 -- Install, manifest, tree identity, version line and use (status: Not Started)

- AC-03.1 `pplr install` publishes only from a clean dev tree and writes the T3 manifest, and it refuses an existing prefix without --force, or a prefix another tool wrote. -- satisfied: no (computed)
- AC-03.2 The version line reads `pplr <version> (<tree>, <sha12>[+dirty])` in dev, opt and brew, built from stamps taken at build time, with the tree named by the one tree function. -- satisfied: no (computed)
- AC-03.3 `pplr use dev|opt|brew` moves only the link set {pplr} in ~/.local/bin, and refuses a missing tree or a brew copy that would not answer. Doctor's shadow check fails on a link that is shadowed or split across trees. -- satisfied: no (computed)

### WP-04 -- Assets formula, tap and tap CI (status: Not Started)

- AC-04.1 packaging/homebrew/pplr.rb is a devbin `assets` template, rendered through `bin/devbin formula render` with nothing typed. The render passes `brew style` and `brew audit --strict`, its install verifies every macOS binary's signature against its designated requirement and writes the T3 manifest, and its test re-verifies the signatures and runs the version line. -- satisfied: no (computed)
- AC-04.2 (non-test) The tap matthewsinclair/homebrew-pplr is described as "Homebrew tap for pplr", and its CI runs style, audit, install and test on macOS and Ubuntu. -- satisfied: no

### WP-05 -- Walking skeleton through a scratch tap (status: Not Started)

- AC-05.1 (non-test) A scaffold build installs from a locally rendered formula in a scratch tap on rhadamanth, T10's three trees are proved, and nothing is tagged or pushed on the real repository. -- satisfied: no

### WP-06 -- Release 2.0.0 and move rhadamanth to brew (status: Not Started)

- AC-06.1 (non-test) 2.0.0 is cut by `bin/devbin release`. Its gates pass locally, on kovacs and in CI for candidate/v2.0.0, and the formula is published to the tap by `bin/devbin formula publish`. -- satisfied: no
- AC-06.2 (non-test) The Contacts grant holds across upgrades: after a `brew upgrade` or `brew reinstall`, `pplr sync --check` from the keg reads the live store with no new permission prompt, and any first-use prompt for the signed sidecar is recorded. -- satisfied: no

### WP-07 -- Developer ID signing and notarisation (status: Not Started)

- AC-07.1 Every macOS binary pplr ships (`pplr`, `pplr-contacts` and the `Pplr Links.app` from `pplr handler`) is signed with Developer ID Application: Geodica Pty Ltd (76BQL8L47U), with the hardened runtime and a secure timestamp, and passes `codesign --verify --strict -R <requirement>` and `spctl`. -- satisfied: no (computed)
- AC-07.2 (non-test) The signer is project code in bash or zsh (`bin/.devbin/cmd/macos`, no Python). It runs on rhadamanth only, notarises every release build with `notarytool`, and the release cannot attach an unsigned macOS asset. -- satisfied: no

## Acceptance Tests

### ST-level

_(no tests in this group)_

### WP-01 -- Release and install design for pplr (status: Done)

_(no tests in this group)_

### WP-02 -- devbin release core and CI candidates (status: WIP)

_(no tests in this group)_

### WP-03 -- Install, manifest, tree identity, version line and use (status: Not Started)

_(no tests in this group)_

### WP-04 -- Assets formula, tap and tap CI (status: Not Started)

_(no tests in this group)_

### WP-05 -- Walking skeleton through a scratch tap (status: Not Started)

_(no tests in this group)_

### WP-06 -- Release 2.0.0 and move rhadamanth to brew (status: Not Started)

_(no tests in this group)_

### WP-07 -- Developer ID signing and notarisation (status: Not Started)

_(no tests in this group)_

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
