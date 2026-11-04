---
wp_id: WP-04
title: One walker for grep, links and rename, or the record of why not
scope: S
status: WIP
---

# WP-04: One walker for grep, links and rename, or the record of why not

## Objective

grep, links and rename walk through one walker that keeps each command's order, dot-name and symlink rules, and names each directory it cannot read on stderr (hv's ruling, vc decision 29); or the package records why one walker is not simpler, and is cancelled (AC-04.1).

## The four walks at 2.0.0

| Walk               | Where                                          | Roots                                                                                       | Order                                                                                              | Dot names                                  | Symlinks                                                                           | What it keeps                                                                                  |
| ------------------ | ---------------------------------------------- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| grep               | `grep.rs:134` `markdown_files`, `:145` `walk`  | each top folder of the data root, by `names()`, past dot-named ones; a symlinked top walked | by name: `names()` sorts each folder's names as lossy UTF-8, and each path is joined from its name | entered and kept below the top             | `symlink_metadata`: never entered, and a symlinked `.md` not kept                  | a regular file whose name ends `.md`                                                           |
| links              | `links.rs:219` `markdown_files`, `:243` `walk` | each argument: a file as itself, a folder canonicalised                                     | the folder's own; the caller then canonicalises each path, drops repeats and sorts by bytes        | passed over, file and folder, at any depth | `file_type`: never entered; a symlinked `.md` kept, then canonicalised to its file | any entry but a folder with the extension `md`                                                 |
| rename search      | `rename.rs:309`                                | each `--refs` root, canonicalised                                                           | the folder's own; two callers sort by bytes, and the third rewrites in any order                   | entered and kept                           | `file_type`: never entered, never kept                                             | a regular file with an extension in the caller's list; a folder named in `skip` is not entered |
| rename named_after | `rename.rs:365`                                | the person's folder                                                                         | the folder's own, as `find` meets them, which the dry run lists and the move follows               | entered and kept                           | `file_type`: never entered; kept when its name matches                             | any entry whose name starts with the prefix, kept before its contents                          |

All four read each folder once, take each entry by its own type, and enter only a real folder, never a symlink to one. All four pass over a folder they cannot read without a word (`names()` gives nothing, and each `read_dir` returns). An entry whose type cannot be read is passed over by grep, links and search; named_after still keeps it when its name matches, and does not enter it.

## One walker

`fs::walk(dir, order, visit)` reads a folder once, in name order (grep's, as `names()` sorts) or the folder's own, and shows each entry to the caller's `visit`, which keeps what it wants and says whether to enter it. The walk enters an entry only when `visit` says so and the entry is a real folder, so no caller can follow a symlinked folder. A folder it cannot read is named on stderr, one line each, `pplr: cannot read <folder>: <why>`, and the walk carries on with the exit code unchanged (hv, vc decision 29: a ruled divergence, with its own line in CHANGELOG 2.0.1). Each caller keeps its roots, and its rules go in its `visit`:

- grep: name order; keeps a regular file whose name ends `.md`; passes over a name that is not UTF-8, as joining its lossy name did; enters every folder.
- links: the folder's order; passes over a dot name and does not enter it; keeps any entry but a folder with the extension `md`.
- rename search: the folder's order; keeps a regular file with a listed extension; does not enter a `skip` folder.
- rename named_after: the folder's order; keeps any entry whose name starts with the prefix; enters every folder.

It reads simpler. Four recursions, each with its own read and its own silent failure, become one. The two rules every walk shares are written once: never entering a symlinked folder, and naming a folder it cannot read.

## What changes

The ruled divergence alone: a folder a walk cannot read is named on stderr. Each command's order, dot names, symlinks and what it keeps are 2.0.0's. No golden holds an unreadable folder, so none changes.

## Tests

AT-04.1 pins each command's rules through its own function, on one temporary tree: nested folders, dot-named files and folders, a symlinked folder, a symlinked note, a `skip` folder and a folder with no read permission. Each function's list is checked against the table above: grep's in name order; links' as its caller sorts it; search's and named_after's as sets, since their order is the folder's. The rules part is green at 2.0.0 too, so its red comes from mutants: a walker that enters a symlinked folder, and one that walks grep in the folder's order. A test through the binary holds the divergence: `pplr grep` over a tree with an unreadable folder names it on stderr in one line, still prints the other matches, and exits 0. It is red at 2.0.0's code, which says nothing.

## Acceptance

Acceptance Criteria for this work package are RENDERED into `ST0003/acceptance.md`, under the `WP-04` heading. THAT FILE IS A GENERATED VIEW -- a row authored there is discarded by the next sync. The contract is canon in the thread's model, and the verbs write it: `intent ac new` and `intent at new` mint a row, `intent ac edit` and `intent at edit` reword or re-cite one, and `intent ac satisfy|unsatisfy|descope|rescope|withdraw|reinstate` and `intent at green|red|na` move its state. This cover never restates them.

---

_Generated by Intent v3 from the thread canon. Do not edit this file -- it is rendered from the model, and `intent doctor` reports any hand-edit as skew._
