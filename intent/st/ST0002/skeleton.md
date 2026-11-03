# ST0002 WP05: the walking skeleton, so far

dc's record, 2026-11-02. By vc's ruling the real `brew install` and the T10 run on hv's machine are hv's: linking pplr into Homebrew's bin can shadow the live `~/.local/bin/pplr`, and `pplr use` moves that link.

## What is prepared

- `tools/skeleton [--adhoc] [<dir>]` (e23ffc7): a scratch clone of HEAD with dead push URLs, a release build, the macOS asset from `bin/devbin macos prepare`, a scratch `v<version>` tag in the clone only, the sums, and the formula rendered into a scratch tap pointing at the local asset. It prints hv's steps.
- `tools/t10-probe <dev-checkout>` (e23ffc7): `use dev`, `use opt`, `use brew`, each followed by the version line, doctor's Install section and the shadow check.

## The sandboxed run (HOME, PATH and HOMEBREW_PREFIX all scratch; no brew command)

| Step                                  | Result                                                                                                     |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `tools/skeleton --adhoc` at e23ffc7   | exit 0: built, signed ad hoc, verified, packed `pplr-2.0.0-aarch64-apple-darwin.tar.gz`, tagged, rendered  |
| `pplr install --prefix <scratch>/opt` | `Installed pplr 2.0.0 (opt, e23ffc73fc5c)`: the first real install, from a clean build                     |
| `pplr install ... --managed-by brew`  | `Installed pplr 2.0.0 (brew, e23ffc73fc5c)` into a fake keg, linked as brew links one                      |
| `use dev`, version, doctor, shadow    | all ok                                                                                                     |
| `use opt`, version, shadow            | ok; doctor failed only its signature rows, which d782e56 fixed (signatures are checked in brew trees only) |
| `use brew`, version, shadow           | ok, the link removed; doctor fails the signature rows, as it must for an ad-hoc asset in a brew tree       |

The run found three faults, fixed in e23ffc7: `version`, `doctor` and `help` needed a People tree; the probe's doctor check passed when doctor printed nothing; `install --managed-by brew` reported opt.

## hv's steps

As `tools/skeleton` prints them, from a Developer ID build (`tools/skeleton` without `--adhoc`, after the signing in WP07): tap the scratch formula, `brew trust`, `brew install`, `brew test`, `pplr install` for opt, `tools/t10-probe`, then uninstall, untrust and untap. Every brew probe runs with `HOMEBREW_NO_AUTO_UPDATE=1`.
