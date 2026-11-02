---
verblock: "2026-11-02:v0.2: matts - DOING and TODO for ST0001 and ST0002, written at the post-move fold"
---

# Work In Progress

DOING and TODO only; finished work lives in its commit, its closed issue or its satisfied criterion. Lead each focus item with its thread id in bold, the thread's digits in place of `####`: the SessionStart hook shows the first such line as the session's `WIP:` line.

## Current Focus

**ST0001: port Pplr from bash, Python and run-time Swift to one Rust binary; ST0002 ships it through Homebrew, signed**

- Both threads are elaborated, and nothing has started. The design, work packages and criteria are in `intent/st/ST0001/` and `intent/st/ST0002/`. Each `design.md` carries hv's rulings in section 1, and its answers to hv's questions in section 7.
- Next: vc hands out the work. hv gave the go-ahead on 2026-11-02, after the move.

## Active Steel Threads

- **ST0001**: Port Pplr from shell scripts to Rust (14 WPs, 28 criteria).
  - dc: WP01 (test safety and the parity seam), then WP02 (the Rust scaffold), then WP09 (the Contacts store and sidecar) and WP12 (doctor and icons).
  - cc: WP03 (goldens) first, then WP04 to WP08, WP10, WP11 and WP14, then WP13 (the cutover).
- **ST0002**: Homebrew distribution (7 WPs, 16 criteria), all dc's. Order: WP01, WP02, WP03, WP07, WP04, WP05, WP06. WP06, the 2.0.0 release, is cut with hv's approval.

## Upcoming Work

- The first moves run in parallel: dc on ST0001 WP01, then WP02, with ST0002 WP01 alongside; cc on ST0001 WP03.

## Notes

- Heavy runs (`cargo`, `bats`) go through `intent estate slots`.
- No bats run before ST0001 WP01 lands: the current teardown can delete whatever an inherited `PPLR_TEST_DATA` names.
- Nothing from `~/Dropbox/Career/People` ever enters this repo (ST0001 D11).
