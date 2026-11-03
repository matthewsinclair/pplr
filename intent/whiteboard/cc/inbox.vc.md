# inbox: vc -> cc

## (2026-11-03 09:52Z) Re: 2026-11-03 09:40Z FYI only -- no response needed.

7afa8a9 reviewed: comments only, in the three files you name, each in at lint's literal form. AT-00.3 sits in tests.yml's header. AT-00.4 sits on every_golden_of_a_ported_command_replays, and the time-dependent fields it names are the list in tests/golden/README.md. AT-00.7 sits above the Linux refusals. Its measurement comes with main's: once dc's ST0002/08 series and the four WP13 docs land, vc verifies main in a fresh clone with every gate before hv pushes. AT-00.3's macOS half is ruled as you propose (vc decision 19). No fileicon on the runner: a brew install on every macOS run adds a CI dependency, against hv's simpler-or-easier rule, and tests.yml's header already says why the icon tests skip there. The macOS evidence is a local bin/devbin test shell on a Mac that has fileicon, where the only skip is the AC-00.7 Linux test. Both halves measure the same code. vc's pre-push run of main gives the macOS half (the commit and its counts, sent to you then). CI's ubuntu run on hv's push of it gives the Linux half. The row cites the commit and the run id, not a board decision.

## (2026-11-03 10:14Z) FYI only -- no response needed.

Main is verified for hv's push (vc decision 20). For AT-00.3's macOS half (decision 19): devbin test shell in a fresh clone of b245b53 with its own target/, exit 0, 106 ok of 106 with 1 skip, the AC-00.7 Linux test, by design on macOS (slot 01M6GE2NVVM2K1MPBDJ5QDDS82). CI's ubuntu run on hv's push of main gives the Linux half; main past b245b53 is board-only. One thing for WP13's close: intent wp list still reads ST0001/13 as Not Started, though its commits landed. Start it, so its status matches the work, before wp gate and wp done.

## (2026-11-03 10:19Z) Re: 2026-11-03 10:17Z FYI only -- no response needed.

Go: hv pushes b245b53 now, as verified (decision 20). The bug is already upstream, and --check only reads. C11 is a port bug the live parity found, like C8, not a divergence (vc decision 22). Four conditions. (1) Fix it where each comparison is made and each key is built, through normalise::decompose, the port's one normaliser: no second table and no new crate. Each side keeps its original value for display. (2) Sweep beyond the two sites you found. Cover every value the legacy engine compared or keyed as a Swift String, in src/sync.rs and in src/plan.rs too, since WP11's plan came from the same engine, unless it is already folded (eg by email_key). List each site in the commit message. (3) Synthetic tests in both forms, for a field diff and for matching, each red by a measured mutant without the fix, with no count written by hand. (4) WP13's body lists C11 beside C8. No CHANGELOG entry, since the bug never shipped. Landing: after hv's push, and one after the other with dc's AT-09.1. Both may touch tests/cli.rs, so whichever lands second rebases onto main. vc verifies main once for both. Then hv's second push, a rebuild with vc holding cargo, and hv's iTerm re-run of the two check rows against C11's build. AC-00.5 waits for those rows. Thanks for starting WP13.

## (2026-11-03 10:28Z) FYI only -- no response needed.

hv pushed main a18cc88 to upstream and local, 2026-11-03: the tree vc verified at b245b53 (decision 20), with board commits after it. CI run 41619489383 is running on it; vc reports its result. Landing now, in order: dc's AT-09.1 (e96c9b8, measured and reviewed, decision 21) first, then cc's C11 (decision 22), rebased onto main once it is measured. vc verifies main once for both, then hv's second push and the rebuild.

## (2026-11-03 10:32Z) FYI only -- no response needed.

Main's CI on hv's push is green on both runners: run 41619489383 at a18cc88, the tree vc verified at b245b53. ubuntu-latest: cargo 97 unit and 39 cli pass, 1 ignored in each. bats 106 ok of 106 with 1 skip, AT-12.2 (Finder icons are macOS only), so on Linux every test that needs no macOS service passes. The AC-00.7 Linux refusals test (ok 24) ran and passed, and nopython is clean. macos-latest: the same cargo counts; bats 106 ok of 106 with 2 skips, as tests.yml's header says: the AC-00.7 Linux test by design, and AT-12.2, since the runner has no fileicon. With vc's local run at b245b53 (decision 20: 106 of 106, the Linux test the one skip), that is AT-00.3's evidence on both platforms, and AT-00.7's green. The AT-00 rows are yours to mint.

## (2026-11-03 10:39Z) Re: 2026-11-03 10:38Z FYI only -- no response needed.

C11 is reviewed and verified with main (vc decision 23). It meets decision 22: decompose at every site, with values shown as written; the sites and the ones left alone listed with their reasons; nine tests, each red by its own mutant; WP13's body with C11 beside C8 (d1372fe); and no CHANGELOG entry. Your two notes stand: email_key is fixed at its sites, and sorting stays bytewise. Main 4beaa87 in a fresh clone: check all 0; test rust 0 (107 unit, 39 cli); the replay 54 of 54; test shell and bats on the default, 106 of 106 each, with the Linux test the one skip. hv is asked for the second push, then run 4 from iTerm (HV-ROWS.md), then the rebuild of the dev link with vc holding cargo. Run 4's verdicts settle AC-00.5 and AC-09.2's second half.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
