# pplr Usage Rules

Concise rules for using pplr, the personal relationship manager, with its People tree. `pplr help` lists every command, and `pplr help <command>` gives its options.

## Core Principles

- **One folder per person**: `<Letter>/<Surname>, <First>` under the data root, eg `K/Kemp, Jon`. The data root is `$PPLR_DATA`, and `$PPLR_DIR` is its fallback.
- **Name a person the way the command asks**: `<first> <surname>`, as two arguments, for `about`, `cp`, `edit`, `linkedin`, `new`, `open`, `applyicons` and `setpicfordir`. Every other command takes `"Surname, First"`, `"L/Surname, First"` or the person's `pplr://` marker.
- **The template sets the file names**: `About/<First> <Surname> (About).md` for the profile, and each meeting in `Meetings/YYYYMMDD <Title>/`, its notes in `YYYYMMDD <Title>.md`.
- **pplr writes what it generates**: change the source (`tags.yaml`, `contact.yaml`, the About file) and let pplr write the rest.
- **Dry runs first**: a command that writes Contacts or rewrites many files changes nothing until `--apply` (`sync --link`, `--apply-plan`, `--photos` and `--pull`, and `links`), or lists its changes under `--dry-run` (`rename`, `tags render`, `tags apply`, `pictures` and `contact scan`).
- **pplr's own files in the tree are under `_pplr/`**: `vocabulary.yaml`, `cadence.yaml`, and `templates/person` and `icons`, which override the ones inside the binary.

## People

```bash
pplr new John Smith https://www.linkedin.com/in/jsmith  # the template, its About filled in, the LinkedIn shortcut
pplr about John Smith                                    # print the About file
pplr edit John Smith                                     # open it in $VISUAL, else $EDITOR
pplr open -t linkedin John Smith                         # about, linkedin or profile
pplr rename "Smith, John" "Smith-Jones, John"            # the folder, its files, the links to it, an alias
pplr pictures "Smith, John"                              # their photo at the top of their About page
```

- DO give `new` the LinkedIn URL: without it, the About file keeps the template's placeholders and no shortcut is written.
- DO rename with `pplr rename`, never by moving the folder: it records the old name in the person's `.index/aliases`, repoints path-style links under `--refs`, and reindexes.
- DO keep the About header's fields (`- Role:`, `- Company:`, `- LinkedIn:`, `- Email:`, `- Phone:`) one to a line: every command reads them from there.

## Tags

- Each person's tags are in `About/tags.yaml`, a list under `tags:`.
  - A bare word is a hand tag: yours, and never rewritten.
  - `{tag: founder, source: auto}` and `{tag: mentor, source: inferred, evidence: ...}` come from a reviewed list through `pplr tags apply`, which keeps the hand tags already there. An inferred tag must be a relationship tag.
- Every tag is one word of lowercase letters and digits, and must be in `_pplr/vocabulary.yaml`, so add a new tag there first. The vocabulary gives each tag its facet (role, function, sector, org, relationship, place or cadence), and its `also` lists and `split` map name the old words it folds in.
- The cadence tags (eg `3month`) set how often to get in touch: see Contact cadence.

```bash
pplr tag "Smith, John"                 # their tags, by facet
pplr tag "Smith, John" +vip -contact   # add or remove hand tags, checked against the vocabulary
pplr tags check                        # every tags.yaml against the vocabulary
pplr tags render                       # each About's Tags line, the tag pages in _tags/, the tags index
```

- DO run `pplr tags check` after editing a `tags.yaml` by hand, and `pplr tags render` (or `pplr reindex`) after any change to tags.
- NEVER edit an About's `- Tags:` line, `_tags/` or `.index/tags_index.json`: `pplr tags render` rewrites them from `tags.yaml`.

## Contact cadence

- `About/contact.yaml` holds the last contact (`last:`, with its date, how and a link), the `next:` date, and any `cadence:` of the person's own, in days.
- Who is on the roster, and how often, comes from the person's tags and `_pplr/cadence.yaml`: `roster:` gives the days for a relationship tag, and `every:` the days for a cadence tag, which overrides it. A `cadence:` in `contact.yaml` overrides both. A meeting folder dated after today counts as booked, so that person is not due.

```bash
pplr contact scan                         # last contact from each newest meeting, next from the cadence
pplr contact due 7                        # who is due, up to a week from today
pplr contact log "Smith, John" --via call --note "Caught up on the raise"
pplr contact next "Smith, John" +2w       # a date, or +10d, +2w, +3m
pplr email                                # recent mail, numbered
pplr contact log --email 3                # log message 3 of that listing
```

- NEVER edit `About/<First> <Surname> (Contact).md`: it is a read-only view of `contact.yaml`, rewritten whenever `contact.yaml` is.

## Contacts (macOS)

- DO run `pplr sync --check` first: it only reads.
- Every other `sync` mode is a dry run until `--apply`. `--link --apply` and `--apply-plan --apply` save every card to a dated `.vcf` in `$PPLR_BACKUP_DIR` first, and `pplr sync --backup` does that on its own.
- `pplr sync --plan` writes a review workbook. Review its Merged sheet, then carry it out with `pplr sync --apply-plan <workbook> --apply`. pplr reads the sheets by their headers, so a column you add is safe.
- Nothing is removed in either direction: `--apply-plan` adds and updates cards, and `--pull` copies into pplr only what pplr lacks.
- The live Contacts store needs macOS and `pplr-contacts` beside `pplr`. Elsewhere, `sync` reads a dump named by `PPLR_CONTACTS_JSON`.

## Searching and indexes

```bash
pplr search "founders in fintech"   # natural language, by claude over .index/tags_index.json
pplr grep "exact phrase"            # text in the people files
pplr grep -t vip                    # text in the tags files
pplr meetings 202401 202412         # meetings in a range of YYYY, YYYYMM or YYYYMMDD dates; today's with none
pplr reindex                        # the indexes in .index/, the tag pages, each About's Tags line
pplr doctor                         # the tree, its indexes, the settings and the tools, one row each
```

- DO run `pplr reindex` after bulk changes, and `pplr doctor` when something looks wrong. `pplr doctor --fix` makes missing folders and reindexes.
- `pplr search` falls back to a keyword search when `claude` cannot answer.

## Generated files: never edit by hand

| File                                         | Written by                                    |
| -------------------------------------------- | --------------------------------------------- |
| `.index/index.json`, `.index/index.md`       | `pplr reindex` (or `pplr json`, `pplr index`) |
| `.index/tags_index.json`                     | `pplr tags render` or `pplr tags index`       |
| `<person>/.index/aliases`                    | `pplr rename`                                 |
| `<person>/.index/refreshed`                  | `pplr refresh stamp`                          |
| `_tags/`                                     | `pplr tags render`                            |
| an About's `- Tags:` line                    | `pplr tags render`                            |
| `About/<First> <Surname> (Contact).md`       | every command that writes `contact.yaml`      |
| `About/<First> <Surname> (Contacts).inetloc` | `pplr sync --link`                            |

## Anti-patterns to Avoid

- NEVER start a folder's name with `_` or `.` unless pplr owns it (`_pplr/`, `_tags/`, `.index/`): the people commands and the index skip such a folder at the top of the tree.
- NEVER store secrets (passwords, keys) in the tree.

## Integration Notes

- `pplr email --json` prints a versioned document for other tools, and `sync --check --json`, `sync --plan --json` and `links --json` print JSON too.
- Read `.index/index.json` and `.index/tags_index.json` for the whole tree; never write them.
- A command that fails exits non-zero and says why. On Linux, a command that needs a macOS service exits 1 and names it.
- Everything in the tree is plain text: Markdown, YAML and JSON.
