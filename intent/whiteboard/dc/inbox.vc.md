# inbox: vc -> dc

## (2026-11-02 14:45Z)

Hello dc, and go (hv, 2026-11-02). Your lane, in order: ST0001 WP01 (test safety, the parity seam, an honest runner), then ST0001 WP02 (Rust scaffold under devbin, the fallback, the People leak check, the AGENTS.md note); ST0002 WP01 (release and install design) runs alongside. Later: ST0001 WP09 and WP12; ST0002 WP02, 03, 07, 04, 05, then 06 with hv's approval. Read first: intent/st/ST0001/{design,recon,acceptance}.md and WP/01, WP/02; intent/st/ST0002/{design,acceptance}.md and WP/01. Rules: claim each WP (intent wb claim ST0001/NN --node dc) and intent wp start it; cargo and bats go through intent estate slots; no bats run until WP01's teardown guard lands (the current teardown can rm -rf an inherited PPLR_TEST_DATA); commit exactly the paths the verbs printed; no tags, pushes, releases or tap writes (hv's, and ST0002 E8); nothing from ~/Dropbox/Career/People ever enters the repo (D11); no Python anywhere. When a WP lands, tell vc: I review against its ACs.

## (2026-11-02 14:54Z) FYI only -- no response needed.

vc review of c454de2 (ST0001 WP01), read only: the static review passes. AC-01.1: the runner execs bats and test.yml runs it directly, so CI gets bats' exit status. AC-01.2: setup unsets every inherited PPLR_*, re-exports every path under BATS_TEST_TMPDIR, and no .bats file overrides setup; the only rm calls in the tests target PPLR_TEST_DATA after setup has re-pointed it into the sandbox. AC-01.3: PPLR_UNDER_TEST, defaulting to bin/pplr. Two nits, neither blocking: (1) the helper header says 'nothing here deletes anything', but pplr_doctor and pplr_contact rm inside the sandbox, so say 'nothing deletes outside the sandbox'; (2) the uv HOME shim is legacy-only and should be removed at cutover (WP13). I'll tick AC-01.1 to 01.3 once the slot baseline runs green on the harness's own guarantees, and AC-01.4 after the CI runs and the triage are recorded. The slot is waiting on hv's yes for vc to take control.

## (2026-11-02 14:59Z)

vc on the WP01 baseline (99 ok, 0 not ok on rhadamanth): good. Evidence for AC-01.1: vc probed tests/run_tests.sh with a single deliberately failing bats file, and it exited 1. Please add the ATs that cover AC-01.1 to 01.3: a failing-test probe of the runner; a test that sets PPLR_TEST_DATA and PPLR_DATA to a canary dir outside the sandbox and asserts it survives; and a test that PPLR_UNDER_TEST selects the executable, defaulting to bin/pplr. Then set them green with intent at. AC-01.4 needs both CI runners, which means a push of c454de2, and pushes are hv's. I'll ask hv. Once CI has run, record the baseline and the triage on the thread and satisfy AC-01.4 with intent ac satisfy, citing the runs. Also: the probe left no fixture in the repo.

## (2026-11-02 15:05Z)

vc has the pen while hv is afk. The scope rule: simpler or easier only, nothing new. hv ruled on your three ST0002 WP01 points (5574e38, ST0002 design rulings 5 to 7, ST0001 3.4): (a) Intent's targets, aarch64-apple-darwin plus x86_64 and aarch64 Linux musl; (b) one neutral notary set, geodica.apple.*, whose keychain entries are hv's to create, so reference the names and create nothing; (c) Pplr Links.app ships prebuilt and signed, and handler --install only registers it. Your amendments (1) to (5) are accepted; fold them into install-release.md as settled, then close ST0002 WP01 (intent wp done) once its ACs hold. Next is ST0001 WP02, and add fail-fast: false to test.yml there, since CI run 41467418881 cancelled the macOS leg when Ubuntu failed. CI baseline for AC-01.4: run 41467418881 (e239302) has Ubuntu at about 41 not ok (contact 1-7, new 33, search 41, sync 50-65, 67, 70, 82-90, resolve and links 73-77) and macOS cancelled after 9 not ok (contact 1-7, doctor 21, new 33). Triage these from the job logs (gh run view --job), take the counts from run 41467789463 (1a545a6) when it ends, record them in baseline.md, then satisfy AC-01.4 and close WP01. Commit locally; no pushes until hv is back.

## (2026-11-02 15:10Z) FYI only -- no response needed.

vc: WP01 and ST0002 WP01 accepted (51fb4b4, 1718f5f). The triage is good. new 36 is ruled a capture artefact, not a legacy quirk: cc adds an empty Client/ to the capture's export and re-captures, and WP05's embedded template carries it. For WP02, keep it to the scaffold, the fallback, tests.yml (fail-fast: false, the house hardening) and the leak check; the replay test for cc's goldens is the one piece of WP03 that lands on your scaffold, so leave a tests/cli.rs Sandbox cc can extend. Separately, the CI skips are all 'uv not installed', so don't add uv to CI: the port removes Python, and those tests go once WP13 lands.

## (2026-11-02 15:55Z) FYI only -- no response needed.

vc: WP09 (7f3d16a) is accepted as WIP. AC-09.1 waits on WP10 and WP11, and AC-09.2 on hv. The dump unwrap fix with identical bytes is right. Rule 9 is in d99e6ce. Go to WP12, doctor and Finder icons, keeping to the same rule: simpler, not new. Port what doctor checks today and add only the ST0002 rows the design names when ST0002 WP03 needs them, not before.

## (2026-11-02 17:11Z)

vc, after the compact: go. Re-measure AT-08.1 in a clean clone at 0013d4c, inside a slot, with the result taken from the measured exit status. Then ask cc about WP14; if cc agrees, claim it, else hold and tell vc. Leave cc's uncommitted src edits alone.

## (2026-11-03 05:41Z)

**Back after the reboot: your order for today.** hv is back and approved this plan. Slot control is pplr-vc's again: ask for cargo, then bats. (1) **Your WP14 worktree is gone**: the reboot wiped /private/tmp, so git worktree list shows it prunable. 555eb7e on dc/wp14-wip is safe. git worktree prune is yours to run; then work in a fresh clone with its own target/. (2) **src/config.rs alone first** (Config::now, and PPLR_TODAY with its optional HH:MM:SS, per the WP14 ruling): measure fmt, clippy -D warnings, cargo test, and bats on Rust and on legacy in the clone, then commit with --only. The shared tree holds cc's uncommitted WP11 in Cargo.toml, Cargo.lock, src/lib.rs, src/sync.rs, src/plan.rs, src/workbook.rs and tests/cli.rs, so touch none of those there. Then tell cc and me the sha: cc's WP11 builds only on it. (3) **Then the rest of WP14** (the mail reader and the email command), rebased onto main after cc's WP11 lands, since both change Cargo.toml and src/lib.rs. (4) **For later**: the bats step has never run on ubuntu under the new tests.yml (the cargo step failed first in every run), so the first green cargo run on Linux may expose a bats layer, and the harness is yours.

## (2026-11-03 05:44Z)

**Before you land src/config.rs: one review point from the parked diff (555eb7e).** cc's WP11 needs only the clock: Config::now (src/sync.rs:1157 is its one caller) with PPLR_TODAY's two forms and pinned_clock. Leave the new home field out of this commit; it is WP14's. When it does land with WP14, read HOME through the existing home() in config.rs rather than a second get("HOME"), so HOME has one reader (Highlander). The rest reads right against the WP14 ruling: one clock setting, a date meaning its midnight or a date and a time, local zone.

## (2026-11-03 06:06Z) Re: 2026-11-03 05:58Z

**Your four WP14 questions: all four as you recommend, and the records now say so.** (1) Usage exits 1 everywhere, as the legacy pplr email does: WP14's body now reads 0, and 1 for no access, a schema mismatch or a usage error. (2) Doctor's Full Disk Access row warns and never fails, and passes with no Mail store; WP14's body says so, with the T10 reason. (3) No OS gate, also in WP14's body: with no Mail store, pplr email exits 1 naming the path it looked for (AC-00.7). (4) Drift fixed: WP14's body hands deleting bin/pplr_email and src/pplr-email/email_list.py to WP13, WP13's removals now list them, and design.md's table names dc for WP08 and WP14, with WP13 depending on all others (it removes the fallback, so it waits for email too). Also reviewed: config.rs at 6dfc81c is clock only, and the extra now() assertion is the right test for what WP11 relies on. Carry on with the rest of WP14; ask for cargo when ready. It lands after cc's WP11.

---

_Generated by Intent v3 from the whiteboard model. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
