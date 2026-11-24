# Releasing pplr

This page is for the maintainer of a pplr checkout: its trees, building and testing, signing, and cutting a release. A user installs pplr with Homebrew alone, as the README says.

pplr follows devbin's dev, opt and brew contract (T1 to T10 in devbin's `docs/dev-opt-brew.md`) through devbin's own implementation (devbin#0326, pplr#0001): `bin/devbin install` is the publish and `bin/devbin use` the switch, each run in this checkout, and pplr has no verb of its own for either. Its `install:` block in `bin/.devbin/config.yaml` declares the opt prefix, the link and the rest of the tree. ST0002's `design.md` records how the contract first applied to pplr, and its `install-release.md` the release and signing design; ST0002 is done, so both live in the store, and `intent st hydrate ST0002` puts them back on disk.

## The three trees

| Tree | Where                                                                | Written by                         |
| ---- | -------------------------------------------------------------------- | ---------------------------------- |
| dev  | the checkout, with its build in `target/release/`                    | `bin/devbin build`                 |
| opt  | `install.prefix` in `bin/.devbin/config.yaml`, eg `~/Devel/opt/pplr` | `bin/devbin install`, run from dev |
| brew | `$(brew --prefix)/opt/pplr/libexec`                                  | the formula                        |

A tree holds `bin/pplr` and, on macOS, `bin/pplr-contacts` and `Pplr Links.app`, beside its manifest, `.pplr-install`, which names each file with its sha256. The person template, the Finder icons and the README are inside the binary. `bin/devbin doctor`, run in the checkout, checks each tree against its manifest and names any file that has drifted. `pplr --version` names the tree it runs from and the commit it was built from, eg `pplr 2.0.0 (dev, 1a2b3c4d5e6f)`, with `+dirty` on a dev build made from uncommitted changes, and `pplr-contacts --version` says the same under its own name. `bin/devbin build` stamps the line; a plain `cargo build` prints `(unknown, unknown)`, which `bin/devbin doctor` fails.

- **dev**: `bin/devbin build` runs `cargo build --release`, then on macOS builds the Contacts sidecar (`swiftc`, from `swift/pplr-contacts/`) and `Pplr Links.app` (`osacompile`), all into `target/release/`, which `bin/devbin use dev` links.
- **opt**: `bin/devbin install [--prefix <dir>] [--force]` publishes the dev build as a working copy, which keeps what works apart from what is mid-development. It builds first, proves each binary by its `--version`, refuses a build made from uncommitted changes, and replaces an opt tree only with `--force`, and only one it wrote.
- **brew**: the formula, through devbin's `@@INSTALL@@`, lays the release's signed binaries out in the keg, writes the manifest, links `pplr` into Homebrew's `bin`, and checks each signature.

## Switching trees

`bin/devbin use dev|opt|brew`, run in the checkout, moves one link, `~/.local/bin/pplr`. `bin/devbin use brew` removes it, so Homebrew's `pplr` answers. `bin/devbin use` with no argument says which tree the link serves. The switch rests at brew: move to dev or opt only while working on pplr, and back with `bin/devbin use brew` once the work is released. `~/.local/bin` must come before Homebrew's `bin` on `PATH`, and `bin/devbin doctor` names a link that something shadows. `bin/devbin help use` says the rest.

The sidecar is never linked: each `pplr` runs the `pplr-contacts` in its own tree. `pplr handler --install` registers the `Pplr Links.app` of the tree it runs from, and `pplr://` links then run that tree's `pplr`; `pplr handler --status` names the app. Before removing a tree, unregister its app with that tree's own `pplr handler --uninstall`, run by its full path.

To upgrade each tree:

- **dev**: `git pull`, then `bin/devbin build`.
- **opt**: from dev, `bin/devbin install --force`.
- **brew**: `brew upgrade pplr`.

## Building and testing

```bash
bin/devbin build            # cargo build --release, then on macOS the sidecar and the app beside the binary
bin/devbin check all        # every read-only gate; bin/devbin help check names them
bin/devbin test all         # cargo test, then the bats suite against the Rust pplr
bin/devbin doctor           # devbin's own install, and pplr's switch, trees and version line
```

The bats suite runs against any pplr named by `PPLR_UNDER_TEST`, and the default is the checkout's dev build, `target/debug/pplr`, which the suite builds itself. Tests use synthetic data alone: nothing from a People tree is ever committed, and the pre-commit guard `bin/devbin people-leak` reads the tree at run time and refuses a staged file that matches it.

Building needs the Rust toolchain (`Cargo.toml` names the edition and `rust-version`), the Xcode command line tools for `swiftc`, and devbin; the bats suite needs `bats-core`, `zsh` and bash 5.

## Signing

Every macOS binary a release ships is signed with the Developer ID, with the hardened runtime and a secure timestamp, and notarised: `pplr`, `pplr-contacts` and `Pplr Links.app`. `bin/devbin macos` does it, and its header lists its steps. It runs on the maintainer's Mac only: it refuses on CI, off macOS, and without the signing identity in the keychain. `packaging/macos/signing` holds the identity, each binary's identifier and designated requirement, and the keychain prefix of the notary credentials. A real `prepare` reads the three credentials first, and stops, naming one that is missing or empty, before anything is signed. The formula template repeats the designated requirements, and a cargo test fails when the two differ.

```bash
bin/devbin build
bin/devbin macos prepare    # stage, sign, verify, notarise, then pack the asset alone into dist/release
```

`bin/devbin macos prepare --adhoc` runs the same steps with an ad-hoc signature and a dry run of the notarisation, packing into `dist/adhoc`, to prove them on a scratch build. A release is never ad-hoc.

## Cutting a release

A release is one command, on the maintainer's Mac, with the Developer ID identity and the notary credentials in the login keychain:

1. Open the release's section in `CHANGELOG.md` as `## [X.Y.Z] - unreleased`. The cut dates it, and its text is the release's notes.
2. `bin/devbin release vX.Y.Z` (`--dry-run` first shows each step). It asks once, then sets the version (`Cargo.lock` follows through `bin/devbin lock-sync`), commits, and runs the gates here and CI on `candidate/vX.Y.Z`. Once every one is green, it tags, pushes and makes the release on GitHub. Its `after:` step, `bin/devbin post-cut`, then builds the release at the tag, signs and notarises `pplr`, `pplr-contacts` and `Pplr Links.app` (`bin/devbin macos prepare`), attaches the asset with `SHA256SUMS`, and publishes the formula to the tap, `matthewsinclair/homebrew-pplr`, under the same yes, then pulls this machine's clone of the tap so `brew upgrade` sees the release.
3. `brew upgrade pplr` (`brew install matthewsinclair/pplr/pplr` the first time), `bin/devbin use brew` in the checkout, then `pplr --version`, which must read brew at the new version.

A step that fails stops everything after it and names the command that resumes: `bin/devbin release vX.Y.Z` again before the tag, `bin/devbin post-cut vX.Y.Z` after it. `bin/devbin help release` and `bin/devbin post-cut --help` describe both. The formula installs the macOS asset alone for now; the Linux binaries come in a later release, with the workflow that builds them.
