---
verblock: "2026-11-03:v0.2: Matthew Sinclair - Frozen: the capture went with the legacy code (ST0001 WP13)"
---

# Goldens from the legacy pplr

What the bash, Python and Swift pplr printed and wrote, captured over a synthetic People tree, so the Rust port is held to it byte for byte (ST0001 design D4, AC-00.4, AC-03.1). Every person, address and note here is invented (D11).

**The goldens are frozen as the legacy code's record.** That code is gone (ST0001 WP13), and its capture driver and list of scenarios went with it: `db45555` is the last commit to change them, and holds them as they were removed (`git show db45555:tests/golden/capture`). A golden changes only in the commit of a ruled divergence, and that commit names the ruling.

## What is here

| Path         | What it is                                                                                                                             |
| ------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| `fixture/`   | The sandbox seed: `Career/People/` (the data tree), `contacts.json` (the Contacts store), `refs/` and `refs-link`, `mail/`, `inputs/`. |
| `xlsx-cells` | A workbook's content as text, the form its goldens are kept in; the replay calls it, as the capture did.                               |
| `fakes/`     | Tripwires on `PATH` for the tools the port runs by name (`claude`, `fileicon`, `less`, `open`, `pbcopy`): no case may reach one.       |
| `cases/`     | The goldens, one directory per scenario, and `LEGACY`, the commit whose code produced them.                                            |

## How they were captured

1. The legacy code was exported with `git archive` from a pinned commit (`507b0f6`, the last that changed it), so only tracked files took part, and its Swift engine was built once in the export.
2. Each scenario got a fresh sandbox copied from `fixture/`. `@SANDBOX@` in a fixture file, argument or setting became the sandbox path.
3. Each step ran `bin/pplr` from the sandbox with a clean environment: `HOME` in the sandbox (its `.gitconfig` names the author `Matthew Sinclair`), `LANG=en_GB.UTF-8`, `TZ=Europe/London`, `PPLR_DATA` and `PPLR_DIR` at `Career/People`, `PPLR_TODAY=2026-06-15`, `PPLR_CONTACTS_JSON`, `PPLR_BACKUP_DIR`, `PPLR_CACHE_DIR`, `PPLR_REFS_DIR` and `PPLR_HANDLER_APP` inside the sandbox, and `uv` offline. A scenario's own settings are in its `env` file.
4. Fakes on `PATH` stood in for the clock (`date`), the CMS probe (`curl`) and `mail-recent`, and a tripwire for every other tool no scenario could reach. A tripwire call failed the capture, and so did any trace of the temporary directory left in a golden.

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

| Rewrite                                                                                        | Why                                                                                                                                                                                                                          |
| ---------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| the sandbox path, also as `/var...` and `\/`-escaped, to `@SANDBOX@`                           | Foundation drops `/private`, and Swift's `JSONEncoder` escapes `/`.                                                                                                                                                          |
| the export path to `@LEGACY@`                                                                  | The legacy code's own location.                                                                                                                                                                                              |
| today's real date to `2026-06-15`                                                              | `tags.py` read the real date (the `_tags/` pages' generated-note line). Bash read the pinned `date`; `contact.py` read `PPLR_TODAY`. The capture refused to run on 2026-06-15 itself, or when the fixture held today's date. |
| `"generated_at": "..."` to `@GENERATED_AT@`                                                    | `tags_index.json`, written in UTC to the second.                                                                                                                                                                             |
| a plan's `"generated" : "YYYY-MM-DD HH:MM"` and `sync plan, YYYY-MM-DD HH:MM` to `@GENERATED@` | The Swift engine stamped the plan with the time of day.                                                                                                                                                                      |
| `YYYYMMDD-HHMMSS` to `@STAMP@`, in text and in backup names                                    | The Swift engine named each backup by the second.                                                                                                                                                                            |

The bash scripts read a fake `date` that answered `2026-06-15 09:30:00` local time, so the default workbook is `_out/contacts-plan-20260615-0930.xlsx`. The Rust port takes the same day and time from `PPLR_TODAY`, which the replay sets to `2026-06-15 09:30:00` (design 3.5).

## The replay (`every_golden_of_a_ported_command_replays` in tests/cli.rs)

- Every case runs against the Rust binary, with the same settings and a sandbox seeded from `fixture/`, and the test prints which cases ran. Stdout, stderr, exit code and the tree must match after the rewrites above, except the real-date rewrite: the port reads `PPLR_TODAY` (design 3.5).
- **The sandbox root must be as long as `/private/tmp/pplr-golden.XXXXXXXX`.** `rename` cuts each running-text line at 200 columns, absolute path included, so those lines change with the root's length.
- A workbook compares by its `.cells` dump, which `xlsx-cells` writes, as it did for the capture: one `sheet<TAB>cell<TAB>value` line per non-empty cell, with the XML entities decoded and newlines as `\n`. It reads inline strings, as the legacy workbook holds them, shared strings, as the port writes them, and numbers. After each sheet's cells come its styling facts, sorted: each cell's solid fill, the frozen pane, the filter, each data validation and each conditional format with its fill, so the shaded review columns, the Decision list and the Low and Medium highlighting are compared too (AC-11.1). A colour is read without its alpha byte, and a conditional format's fill from its `bgColor`, else its `fgColor`, because openpyxl and rust_xlsxwriter write the same colour those different ways. Widths, fonts and wrapping are not compared. AC-11.1 does not name them, they change neither what a reviewer can enter nor what `--apply-plan` reads, and the two writers encode a width differently (openpyxl writes the `26` it is given, rust_xlsxwriter `26.7109375`, Excel's own padding), so comparing them would mean modelling each writer's arithmetic rather than reading the workbook.
- JSON on stdout compares as bytes, like everything else: the port writes it as the legacy code did, Swift's `"key" : value` included, so the semantic comparison design D4 allowed was never needed.
- `open-print-cms-up` sets `PPLR_GOLDEN_CMS=up`, which told the capture's fake `curl` the CMS was up; the replay reads it and runs a stub CMS on `127.0.0.1:47360` while the case replays.
- `email` scenarios fed the legacy code a listing through a fake `mail-recent`. The port reads Mail's own stores instead (WP14), so the replay builds them under the sandbox's HOME from `fixture/mail/messages.json`, the messages the fake served, holding only the tables and columns the port reads. `home/` is an input, so the stores are never compared.

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

Parity first (design D10), so these were captured as they are, not fixed:

- `resolve pplr://T/Thistle,%20Ottoline` (the first marker form, naming a person's alias) finds nobody and exits 4.
- A Markdown link without angle brackets ends at its first `)`, so `.../Imogen%20Quennell%20(About).md` is read as `.../(About` and left as it is.
- `json` names a `(Picture).jpg` for a person who has no picture.
- `count` with no `.index/index.md` still prints a number.
- `meetings 2026-13` exits 0 with `No meetings found`.

## Changing a golden

Only for a ruled divergence: a behaviour the port changes on purpose, by a ruling recorded on ST0001 (eg a vc decision or hv's word). The commit that changes the port changes the case's files to match, names the ruling, and the CHANGELOG lists the divergence.
