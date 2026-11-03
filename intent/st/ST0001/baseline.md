# ST0001 baseline: the bats suite against the legacy code

Recorded by dc for WP01 (AC-01.4). The harness is the one landed in c454de2 and 5e0f7ac: a sandbox per test under `$BATS_TEST_TMPDIR`, inherited `PPLR_*` unset, nothing deleted outside the sandbox, `PPLR_UNDER_TEST` unset (so `bin/pplr`), and a runner that exits with bats' status.

## rhadamanth (macOS), 2026-11-02, bats lane slot 01M6EBWZSD9THBN1Q549VEK46T

| Run | Commit           | ok  | not ok | skipped | Runner exit | Time |
| --- | ---------------- | --- | ------ | ------- | ----------- | ---- |
| 1   | c454de2          | 98  | 1      | 0       | 1           | ---- |
| 2   | 5e0f7ac (staged) | 99  | 0      | 0       | 0           | 23s  |

Both runs were made with `PPLR_TEST_DATA` aimed at a path under the real HOME that did not exist, and it still did not exist afterwards.

Triage of run 1:

- `not ok 43 pplr search falls back without Claude`: a test defect introduced by the harness change. The test wrote its failing mock to the old `$PPLR_TEST_DATA/.mock_bin`, which the helper no longer creates. Fixed in 5e0f7ac by writing to `$PPLR_TEST_BIN`.

Run 1 is also the AC-01.1 evidence on rhadamanth: a failing test made the runner exit 1.

The failures the last CI run showed on macOS (the 7 contact tests, test 21 `doctor --fix`, test 33 `new` directory structure) do not reproduce on rhadamanth through this harness. Their CI triage waits for the CI legs below.

## The seam (AC-01.3)

- `PPLR_UNDER_TEST=/nonexistent/pplr bats tests/pplr_core.bats` fails every test in setup with `test_helper: PPLR_UNDER_TEST is not an executable file: /nonexistent/pplr`.
- `PPLR_UNDER_TEST=<a stub executable>` runs the stub: its output reached the test's assertion.

## CI: macos-latest and ubuntu-latest

Run 41467789463 on 1a545a6 (the harness and its ATs), legacy code, both runners to completion. The earlier run 41467418881 on e239302 cancelled the macOS leg when Ubuntu failed; ST0001 WP02 sets `fail-fast: false`.

| Runner        | Tests | ok  | not ok | skipped |
| ------------- | ----- | --- | ------ | ------- |
| macos-latest  | 102   | 93  | 9      | 13      |
| ubuntu-latest | 102   | 61  | 41     | 13      |

Bats counts a skipped test as ok, so ok includes the 13 skips. The harness ATs (1 to 3) pass on both runners.

Triage, one line per failing group:

| Tests                                         | Runners | Triage                                                                                                                                                                                                                                                                       |
| --------------------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 13 skipped (tag, tags, search fallback)       | both    | Environment gap: CI has no `uv`, so every test that needs the Python helpers skips. They go when the Python goes.                                                                                                                                                            |
| 4 to 10 (contact)                             | both    | Environment gap: `pplr contact` runs `contact.py` through `uv`, which CI lacks (test 7 prints "uv not found"). The tests carry no `uv` skip guard: a test defect too, moot once WP07 ports contact.                                                                          |
| 36 (`new` creates the directory structure)    | both    | Legacy defect: `templates/person/Client/` holds only a Finder `Icon` file, which git does not track, so a fresh checkout has no `Client/` and `new` never makes it. It passes on rhadamanth because the untracked file is there. The port's embedded template must carry it. |
| 24 (`doctor --fix`)                           | macOS   | Environment gap, inferred: doctor counts `bash` on PATH older than 4 as an error and exits with its error count. The macOS runner's `bash` is Apple's 3.2 and Ubuntu's is 5, which matches 24 failing on macOS alone. The log shows only the exit status.                    |
| 44 (`search` finds people by role)            | Ubuntu  | Test defect: `sed -i ''` is BSD sed's form, and GNU sed reads the empty argument as its script.                                                                                                                                                                              |
| 53 to 68, 70, 73, 85 to 93 (sync)             | Ubuntu  | Environment gap: the Swift engine needs the Contacts framework (`no such module 'Contacts'`), so every sync mode fails on Linux, even against the JSON fixture. Design D9: sync against the JSON store runs on Linux once WP09 and WP10 port it.                             |
| 76 to 80 (`resolve`, `links`, `open --print`) | Ubuntu  | Environment gap: the same engine build. These are pure tree logic, so they run on Linux once ported (WP04, WP08).                                                                                                                                                            |

No failure is a regression in the harness, and none is a defect in the data the legacy code writes. Every macOS failure is an environment gap or the template defect; every Linux-only failure is the Swift engine, the BSD `sed` in one test, or both.
