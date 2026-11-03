---
verblock: "2026-11-02:v0.1: Matthew Sinclair - Initial version (ST0001 WP03)"
---

# Goldens from the legacy pplr

What the bash, Python and Swift pplr prints and writes, captured over a synthetic People tree, so the Rust port can be held to it byte for byte (ST0001 design D4, AC-00.4, AC-03.1). Every person, address and note here is invented (D11).

## What is here

| Path            | What it is                                                                                                                             |
| --------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `capture`       | The zsh driver. `tests/golden/capture [SCENARIO...]` re-captures all scenarios, or the ones named, into `cases/`.                      |
| `scenarios.zsh` | The one list of scenarios: each a name, optional settings, and one or more `pplr` steps run in order.                                  |
| `fixture/`      | The sandbox seed: `Career/People/` (the data tree), `contacts.json` (the Contacts store), `refs/` and `refs-link`, `mail/`, `inputs/`. |
| `xlsx-cells`    | A workbook's content as text, the form its goldens are kept in; the capture and the replay both call it.                               |
| `fakes/`        | Stand-ins on `PATH`: a pinned `date`, `mail-recent`, `curl`, and a tripwire for every tool no scenario may reach.                      |
| `cases/`        | The goldens, one directory per scenario, and `LEGACY`, the commit whose code produced them.                                            |

## How a capture runs

1. The legacy code is exported with `git archive` from a pinned commit (`507b0f6`, the last that changed it; `PPLR_GOLDEN_REF` overrides it), so only tracked files take part, and its Swift engine is built once in the export.
2. Each scenario gets a fresh sandbox copied from `fixture/`. `@SANDBOX@` in a fixture file, argument or setting becomes the sandbox path.
3. Each step runs `bin/pplr` from the sandbox with a clean environment: `HOME` in the sandbox (its `.gitconfig` names the author `Matthew Sinclair`), `LANG=en_GB.UTF-8`, `TZ=Europe/London`, `PPLR_DATA` and `PPLR_DIR` at `Career/People`, `PPLR_TODAY=2026-06-15`, `PPLR_CONTACTS_JSON`, `PPLR_BACKUP_DIR`, `PPLR_CACHE_DIR`, `PPLR_REFS_DIR` and `PPLR_HANDLER_APP` inside the sandbox, and `uv` offline. A scenario's own settings are in its `env` file.
4. A tripwire call fails the capture, and so does any trace of the temporary directory left in a golden.

The sandbox mirrors a Dropbox root because `links` and `rename` recognise paths into the tree by their `Career/People/` segment.

## A case

```
cases/<name>/
  env                   the scenario's own settings, KEY=VALUE per line (absent when none)
  steps/NN.args         one argument per line, after `pplr`
  steps/NN.stdout       normalised as below
  steps/NN.stderr       normalised as below
  steps/NN.exit         the exit code
  manifest              every path left in the sandbox, bytewise sorted:
                          f <sha256> <path>   a file
                          x <sha256> <path>   a workbook, hashed by its .cells dump
                          l <target> <path>   a symlink
                          d - <path>          a directory
  files/<path>          each file the steps added or changed (a workbook as <path>.cells)
  removed               each fixture path the steps removed (absent when none)
```

`home/`, `mail/` and `inputs/` are inputs and are not recorded.

## Normalisation, and the time-dependent fields

Applied to every stdout, stderr and changed text file, in this order. This list is the "time-dependent fields" exception that AC-00.4 names: the replay applies the same rewrites to the Rust binary's output before comparing.

| Rewrite                                                                                        | Why                                                                                                                                                                                                                              |
| ---------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| the sandbox path, also as `/var...` and `\/`-escaped, to `@SANDBOX@`                           | Foundation drops `/private`, and Swift's `JSONEncoder` escapes `/`.                                                                                                                                                              |
| the export path to `@LEGACY@`                                                                  | The legacy code's own location.                                                                                                                                                                                                  |
| today's real date to `2026-06-15`                                                              | `tags.py` reads the real date (the `_tags/` pages' generated-note line). Bash reads the pinned `date`; `contact.py` reads `PPLR_TODAY`. The capture refuses to run on 2026-06-15 itself, or when the fixture holds today's date. |
| `"generated_at": "..."` to `@GENERATED_AT@`                                                    | `tags_index.json`, written in UTC to the second.                                                                                                                                                                                 |
| a plan's `"generated" : "YYYY-MM-DD HH:MM"` and `sync plan, YYYY-MM-DD HH:MM` to `@GENERATED@` | The Swift engine stamps the plan with the time of day.                                                                                                                                                                           |
| `YYYYMMDD-HHMMSS` to `@STAMP@`, in text and in backup names                                    | The Swift engine names each backup by the second.                                                                                                                                                                                |

The bash scripts read a fake `date` that answers `2026-06-15 09:30:00` local time, so the default workbook is `_out/contacts-plan-20260615-0930.xlsx`. The Rust port takes the same day and time from `PPLR_TODAY`, which the replay sets to `2026-06-15 09:30:00` (design 3.5).

## The replay (`every_golden_of_a_ported_command_replays` in tests/cli.rs)

- Every case runs against the Rust binary, with the same settings and a sandbox seeded from `fixture/`, and the test prints which cases ran. Stdout, stderr, exit code and the tree must match after the rewrites above, except the real-date rewrite: the port reads `PPLR_TODAY` (design 3.5).
- **The sandbox root must be as long as `/private/tmp/pplr-golden.XXXXXXXX`.** `rename` cuts each running-text line at 200 columns, absolute path included, so those lines change with the root's length.
- A workbook compares by its `.cells` dump, which `xlsx-cells` writes for the capture and the replay alike: one `sheet<TAB>cell<TAB>value` line per non-empty cell, with the XML entities decoded and newlines as `\n`. It reads inline strings, as the legacy workbook holds them, shared strings, as the port writes them, and numbers. After each sheet's cells come its styling facts, sorted: each cell's solid fill, the frozen pane, the filter, each data validation and each conditional format with its fill, so the shaded review columns, the Decision list and the Low and Medium highlighting are compared too (AC-11.1). A colour is read without its alpha byte, and a conditional format's fill from its `bgColor`, else its `fgColor`, because openpyxl and rust_xlsxwriter write the same colour those different ways. Widths, fonts and wrapping are not compared. AC-11.1 does not name them, they change neither what a reviewer can enter nor what `--apply-plan` reads, and the two writers encode a width differently (openpyxl writes the `26` it is given, rust_xlsxwriter `26.7109375`, Excel's own padding), so comparing them would mean modelling each writer's arithmetic rather than reading the workbook.
- JSON on stdout is to compare semantically (design D4): Swift writes `"key" : value` and escapes `/`, which `serde_json` does not. The replay compares bytes today; the first WP to port a command that prints JSON makes it semantic.
- `open-print-cms-up` tells `fakes/curl` the CMS is up through `PPLR_GOLDEN_CMS`; WP04 decides how the replay tells the port.
- `email` scenarios feed the legacy code a listing through `fakes/mail-recent`. The port reads Mail's own stores instead (WP14), so the replay builds them under the sandbox's HOME from `fixture/mail/messages.json`, the messages the fake serves, holding only the tables and columns the port reads. `home/` is an input, so the stores are never compared.

## Not captured, and why

| Command                                                     | Why                                                                                            |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `about`, `cp`, `edit`, `linkedin`, `open` without `--print` | They hand off to a macOS service or an editor (design D9); the bats suite covers them.         |
| `applyicons`, `setpicfordir`, `setpicsfordirs`              | Finder icons through `fileicon`; nothing they write is a file in the tree.                     |
| `handler`                                                   | The app ships prebuilt and signed, and `handler --install` only registers it (hv; design 3.4). |
| `search`                                                    | Its answer comes from `claude`.                                                                |
| `doctor`                                                    | It reports on the machine it runs on.                                                          |
| `help`, `version`                                           | `help` is generated from the Rust command table and `version` reads `Cargo.toml` (design D7).  |

## Legacy behaviour the goldens hold

Parity first (design D10), so these are captured as they are, not fixed:

- `resolve pplr://T/Thistle,%20Ottoline` (the first marker form, naming a person's alias) finds nobody and exits 4.
- A Markdown link without angle brackets ends at its first `)`, so `.../Imogen%20Quennell%20(About).md` is read as `.../(About` and left as it is.
- `json` names a `(Picture).jpg` for a person who has no picture.
- `count` with no `.index/index.md` still prints a number.
- `meetings 2026-13` exits 0 with `No meetings found`.

## Re-capturing

Run `tests/golden/capture` from anywhere in the checkout. It needs `uv` with PyYAML and openpyxl cached, `swiftc`, `jq` and macOS. A second capture gives identical goldens; if it does not, a normalisation is missing. Commit `fixture/`, `scenarios.zsh` and `cases/` together.
