# Whiteboard -- Pplr

This directory is the live channel for coordinating the sessions working on Pplr. Every file in it has exactly one writer. The protocol itself lives in the `in-whiteboard` skill and is not restated here.

This file is the project's **node roster**.

## The roster

| Node | Name              | Role       | Driven by | Owns                                                                                                                          |
| ---- | ----------------- | ---------- | --------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `cc` | Control Claude    | control    | a session | the port of Pplr to Rust (ST0001's porting work packages)                                                                     |
| `vc` | Validation Claude | validation | a session | the briefs for ST0001 and ST0002, reviewing landed work against them, the coordination record, and surfacing `hv/inbox.*`     |
| `dc` | DevX Claude       | devx       | a session | the test harness, the Rust scaffold, the Contacts sidecar, and the Homebrew distribution (ST0002); works to vc's instructions |
| `hv` | Hypervisor        | hypervisor | the human | scope, sequencing, release policy, tags and pushes, and every ratified decision                                               |

The store registered cc, dc and vc with their moniker as their role. The names and roles above are the estate's convention, and correcting the registry to match them is hv's decision.

`hv` is human-driven: it has no session loop, its `session_id` is `none`, and its heartbeat is advisory. **hv reads nothing on this board unprompted. The named reader of `hv/inbox.*` is `vc`, and whiteboard matters reach the human only when vc raises them.** An escalation is still written durably to `hv/inbox.<you>.md`. The write is the record and vc's surfacing is the delivery, so a write there is not delivered until vc has surfaced it.

**`hv/wip.md` carries hv's standing directives.** Read it at pickup.

## Reading this board

- `<node>/wip.md` is that node's live board, rendered from the store. Change it with `intent wb`, never by hand.
- `<node>/board.json` is the committed record of that board.
- `intent/wip.md` at the project root is the post-session snapshot; this directory is the live channel.

## Standing rule: code and data never mix

`~/Devel/prj/Pplr` holds code and synthetic fixtures only. `~/Dropbox/Career/People` holds the data. Nothing from People is ever committed here: no names, markers, emails, notes, generated files or command output. See ST0001's design, D11.
