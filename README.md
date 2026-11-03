# pplr - Personal Relationship Manager

[![Tests](https://github.com/matthewsinclair/pplr/actions/workflows/tests.yml/badge.svg)](https://github.com/matthewsinclair/pplr/actions/workflows/tests.yml)

A command-line personal relationship manager (PRM) for professional contacts, meetings and relationships, kept as plain files in a folder you own.

## Overview

pplr (pronounced "peopler") is one Rust binary that works on a People tree: a folder of plain Markdown, YAML and JSON files, one folder per person, kept wherever you like (eg in Dropbox, so that it syncs). It finds people, records meetings and when you last spoke to each person, keeps everyone's tags against one vocabulary, gives each person a stable `pplr://` URL, reads your recent mail, and compares and syncs the tree with Apple Contacts. The binary holds the code and the tree holds the data: nothing of yours lives inside pplr.

## Features

- **People and meetings**: one folder per person, with their About file, picture, LinkedIn shortcut and dated meeting notes
- **Tags**: one `tags.yaml` per person against a shared vocabulary, with a generated page per tag
- **Search**: natural-language queries over the index through the Claude CLI, and a keyword search without it
- **Contact cadence**: when you last spoke to each person, and who to get in touch with next
- **Mail**: recent mail, numbered, each sender marked when they are in pplr, read from Mail's own store
- **Apple Contacts**: compare, link, review a merge plan and apply it, copy back what cards know, add photos, back up
- **`pplr://` links**: stable URLs for people and their files, which survive a rename and open from other apps
- **macOS and Linux**: everything that needs no macOS service runs on Linux too, and the rest says why it cannot

## Installation

Install pplr with Homebrew:

```bash
brew install matthewsinclair/pplr/pplr
```

The formula, in the tap `matthewsinclair/homebrew-pplr`, installs the release's prebuilt binaries for macOS on Apple silicon; Linux binaries come in a later release. `pplr`, its Contacts sidecar `pplr-contacts` and `Pplr Links.app` are signed with a Developer ID and notarised, and the formula checks each signature. `brew upgrade pplr` takes a new release.

Then:

1. Point pplr at your People tree, if it is not at the default, `~/Dropbox/Career/People`:
   ```bash
   export PPLR_DATA="$HOME/Documents/People"
   ```
2. Run `pplr doctor`: one row for each check of the tree, its indexes, the settings and the tools, each saying what to fix.
3. On macOS, run `pplr handler --install`, so that `pplr://` links open from other apps.

pplr runs from any directory and needs no checkout: the person template, the Finder icons and this README are inside the binary.

## Configuration

Every setting is an environment variable, and each has a default.

| Setting              | What it sets                                                                           | Default                                                                      |
| -------------------- | -------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| `PPLR_DATA`          | the People tree                                                                        | `$PPLR_DIR`, else `~/Dropbox/Career/People`                                  |
| `PPLR_DIR`           | the tree's older name, which other tools read; `pplr doctor` warns when the two differ | none                                                                         |
| `PPLR_BACKUP_DIR`    | the Contacts backups, and the About copies `sync --pull` makes                         | `$XDG_DATA_HOME/pplr/contacts-backups` (`XDG_DATA_HOME` is `~/.local/share`) |
| `PPLR_CACHE_DIR`     | the saved `pplr email` listing                                                         | `$XDG_CACHE_HOME/pplr` (`XDG_CACHE_HOME` is `~/.cache`)                      |
| `PPLR_REFS_DIR`      | where `pplr rename` repoints links                                                     | `~/Dropbox`                                                                  |
| `PPLR_CMS_URL`       | the CMS `pplr open` tries first; empty always opens the file                           | `http://localhost:4360/people`                                               |
| `PPLR_PICTURE_WIDTH` | the width of the photo `pplr pictures` adds                                            | `160`                                                                        |
| `PPLR_ICON_DIR`      | the Finder icons, over the tree's `_pplr/icons` and the set inside the binary          | none                                                                         |
| `PPLR_TODAY`         | today, as a date or a date and time, for every command that depends on it              | the real date                                                                |
| `PPLR_CONTACTS_JSON` | a Contacts dump to use instead of the live store, eg on Linux                          | none                                                                         |
| `VISUAL`, `EDITOR`   | the editor `pplr edit` runs, `VISUAL` first                                            | none, and `edit` refuses                                                     |

## The People tree

```text
People/                                   # $PPLR_DATA
  B/                                      # one folder per letter
    Blackwood, Esme/                      # one folder per person: "Surname, First"
      About/
        Esme Blackwood (About).md         # the profile: its header fields, then notes
        Esme Blackwood (LinkedIn).webloc
        Esme Blackwood (Picture).jpg      # or .jpeg, or .png
        Esme Blackwood (Profile).pdf
        Esme Blackwood (Contacts).inetloc # opens the linked card (pplr sync --link)
        Esme Blackwood (Contact).md       # a read-only view of contact.yaml
        tags.yaml                         # their tags, the only place they live
        contact.yaml                      # last contact, next date, cadence
      Meetings/
        20260420 Intro/
          20260420 Intro.md
      Client/
      .index/                             # theirs: aliases (former names), refreshed
  .index/                                 # index.json, index.md, tags_index.json
  _pplr/                                  # pplr's config: vocabulary.yaml, cadence.yaml, templates/person, icons
  _tags/                                  # the generated tag pages: index.md and <tag>.md
  _out/                                   # generated workbooks (pplr sync --plan)
```

Every walk of the tree skips a folder whose name starts with `_` or `.`, so pplr's own folders never read as people. The tree's `_pplr/templates/person` and `_pplr/icons`, when present, override the template and icons inside the binary.

## Commands

`pplr help` lists every command with its options, and `pplr help <command>` shows one. A command that takes a person takes them as `<first> <surname>`, two arguments, for `about`, `cp`, `edit`, `linkedin`, `new`, `open`, `applyicons` and `setpicfordir`. Every other command takes `"Surname, First"`, `"L/Surname, First"` or the person's `pplr://` marker.

| Command          | What it does                                                                 |
| ---------------- | ---------------------------------------------------------------------------- |
| `about`          | print a person's About file                                                  |
| `applyicons`     | Finder icons on a person's About and Meetings folders and each meeting       |
| `contact`        | when you last spoke to each person, and who to get in touch with next        |
| `count`          | the number of people in the tree                                             |
| `cp`             | a person's file to the clipboard, or its path to stdout                      |
| `doctor`         | check the tree, its indexes, the settings and the tools                      |
| `edit`           | a person's About file in `$VISUAL`, else `$EDITOR`                           |
| `email`          | recent mail, or the inbox, numbered                                          |
| `grep`           | text search through the people files, or the tags files                      |
| `handler`        | register the app that opens `pplr://` links                                  |
| `help`           | the commands, one command's options, or this README                          |
| `index`          | the Markdown index of everyone, to stdout                                    |
| `install`        | the maintainer's: publish a checkout's build (`docs/releasing.md`)           |
| `json`           | the JSON index of everyone, to stdout                                        |
| `linkedin`       | open a person's LinkedIn page                                                |
| `links`          | rewrite Markdown links into the tree as `pplr://` URLs                       |
| `meetings`       | the meetings in a date range                                                 |
| `new`            | a new person, from the template                                              |
| `open`           | open a person's file, or a `pplr://` URL                                     |
| `pictures`       | each person's photo at the top of their About page                           |
| `refresh`        | when each person was last checked against their public profile               |
| `reindex`        | rebuild the indexes, the tag pages and each About's Tags line                |
| `rename`         | rename a person: their folder, their files and every link to them            |
| `resolve`        | print the file or folder a `pplr://` URL names                               |
| `search`         | natural-language search, through the Claude CLI                              |
| `setpicfordir`   | a person's picture as their folder's icon                                    |
| `setpicsfordirs` | everyone's picture as their folder's icon                                    |
| `sync`           | compare pplr with Apple Contacts; link, plan, apply, pull, photos and backup |
| `tag`            | show or edit one person's tags                                               |
| `tags`           | everyone's tags: check, show, render, index and apply                        |
| `use`            | the maintainer's: choose which build `pplr` runs (`docs/releasing.md`)       |
| `version`        | the version, the tree it runs from and its commit                            |

### People

#### `pplr new <firstname> <surname> [LinkedIn-URL]`

Create a person from the template. With a LinkedIn URL it also fills in their About file, with the git author (`git config user.name`) in its verblock, and writes the LinkedIn shortcut; without one, the About file keeps the template's placeholders.

```bash
pplr new "Esme" "Blackwood" "https://www.linkedin.com/in/esme-blackwood"
```

#### `pplr about <firstname> <surname>`

Print a person's About file.

```bash
pplr about "Esme" "Blackwood"
```

#### `pplr edit <firstname> <surname>`

Open a person's About file in `$VISUAL`, else `$EDITOR`. With neither set, it refuses and says so.

```bash
pplr edit "Esme" "Blackwood"
```

#### `pplr open [-t type] <firstname> <surname>`

Open a person's file in its default application, or a `pplr://` URL (see [pplr:// links](#pplr-links)).

```bash
pplr open "Esme" "Blackwood"               # The About file (the default)
pplr open -t linkedin "Esme" "Blackwood"   # The LinkedIn page
pplr open -t profile "Esme" "Blackwood"    # The PDF profile
```

`-t, --type` takes `about` (`a`), `linkedin` (`l`) or `profile` (`p`).

#### `pplr linkedin <firstname> <surname>`

Open a person's LinkedIn page in the browser.

```bash
pplr linkedin "Esme" "Blackwood"
```

#### `pplr cp [-t type] [-o] <firstname> <surname>`

Copy a person's file to the clipboard, or with `-o` print its path. `-t` takes the same types as `open`.

```bash
pplr cp "Esme" "Blackwood"              # The About file, to the clipboard (macOS)
pplr cp -t profile -o "Esme" "Blackwood" # The profile's path, to stdout
```

#### `pplr rename "Old, First" "New, First" [options]`

Rename a person: moves the folder (to a new letter if the surname's initial changes), renames the files named after them (`First Surname (About).md`, `(Picture).jpg` and the rest), and updates their name in the About file and in their own meeting notes' links. The old name is recorded in their `.index/aliases`, so a Contacts card still carrying the old `pplr://` URL is found and the URL replaced on the next `pplr sync --link`. Path-style links elsewhere are repointed: the folder name, its URL-encoded forms, and the `First Surname (` file names. The old name in running text is listed, never changed. It then reindexes.

```bash
pplr rename "Grifiths, Glen" "Griffiths, Glen" --dry-run        # What would change
pplr rename "Grifiths, Glen" "Griffiths, Glen"                  # Links under $PPLR_REFS_DIR (default ~/Dropbox)
pplr rename "Grifiths, Glen" "Griffiths, Glen" --refs ~/Dropbox/Writing/Journal
```

#### `pplr pictures [--dry-run] ["Surname, First"]...`

Show each person's photo at the top of their About page: an `<img>` line under the title, floated right, pointing at `About/<First Surname> (Picture).jpg` (or `.jpeg`, or `.png`). It is idempotent: run it again after adding people or pictures. People with no picture get no line. `PPLR_PICTURE_WIDTH` sets the width (default 160).

#### `pplr count`

The number of people in the tree.

### Meetings and contact

#### `pplr meetings [start-date] [end-date]`

The meetings in a date range, by folder name. A date is `YYYY`, `YYYYMM` or `YYYYMMDD`, with or without hyphens.

```bash
pplr meetings 2024             # Every meeting in 2024
pplr meetings 202401 202406    # January to June 2024
pplr meetings 20240315         # The meetings on one day
pplr meetings                  # Today's
```

#### `pplr contact`: `scan`, `due`, `show`, `log`, `next`, `context`, `render`

When you last spoke to each person and when to get in touch next, in `About/contact.yaml`:

```yaml
last:
  {
    date: 2026-01-28,
    via: meeting,
    link: "pplr://b/blackwood-esme/Meetings/20260128 Catch-up with Esme Blackwood",
  }
next: 2026-04-28
cadence: 90
```

`scan` takes `last` from the newest dated `Meetings` folder. `next` is `last` plus the cadence. Who is on the contact roster, and how often, comes from the person's tags and `_pplr/cadence.yaml`: a roster tag (`contact`, `vip`, `career`) puts someone on it with a default cadence, and a cadence tag (`3month`, `6month`, `12month`) overrides it, so `contact` and `12month` together is once a year. A `cadence` in `contact.yaml` overrides both. Off the roster, `last` is kept and no `next` is set. A `Meetings` folder dated after today counts as booked, so that person is not due.

```bash
pplr contact scan                        # last from meetings, next from the cadence (--dry-run)
pplr contact due                         # who is due a note today; due 14 looks two weeks ahead
pplr contact show "Kemp, Jon"            # last contact, next date and cadence
pplr contact log "Kemp, Jon" --via email --note "Sent the deck"   # an email today; next moves on
pplr contact next "Kemp, Jon" +2w        # snooze: a date, or +10d, +2w, +3m
pplr contact log --email 3               # message 3 of the last pplr email listing: its date, subject and person
pplr contact log pplr://k/kemp-jon --email "<CAL...@mail.gmail.com>"  # or any email by its Message-ID
pplr contact render                      # About/<First Surname> (Contact).md, a read-only view of contact.yaml, for everyone
pplr contact context "Kemp, Jon"         # role, latest update, last meeting's notes: to write from
```

`contact log` links one message: a second `--email` is refused.

#### `pplr email [recent|inbox] [--account TEXT] [--hours N] [--sent] [--limit N] [--json]`

Recent mail (the last 24 hours) or what is in the inbox now, numbered, one message per block with its `Message-ID`. The person each message is with is marked with their `pplr://` marker when they are in pplr: the sender, or for mail you sent, the first recipient in pplr. A person is found by the Email line of their About file's header. The listing is saved, so `pplr contact log --email 3` logs message 3 with its date and subject, and needs no person when the message is with someone in pplr. `--account work` keeps the accounts whose address contains that text. Junk, Trash, Deleted, Drafts, Spam and Bin are always left out, and Sent unless `--sent`; a message in two mailboxes is listed once.

pplr reads Mail's own index, `~/Library/Mail/V10/MailData/Envelope Index`, and the accounts store beside it, read-only, so the app it runs in needs Full Disk Access (System Settings > Privacy & Security > Full Disk Access); `pplr doctor` warns when it lacks it. The index's schema is Apple's and unversioned, so pplr checks the tables and columns it reads first, and refuses a store that lacks one, naming it, rather than reading wrong rows.

`--json` prints the messages as a document for other tools, and leaves the saved listing alone. Its `schema` is `1`, and it changes when a field does. Each message has `date` (local time with its offset, RFC 3339), `account` (the address of the account it arrived in), `from` (`Name <address>`, or the address), `address` (the sender's), `to` (every recipient's address: To, then Cc, then Bcc), `subject` and `message_id` (with its angle brackets). `pplr email` exits 0, or 1 with the cause on stderr: no Mail store (naming the path it looked in), no Full Disk Access, or a store whose schema pplr does not read; a usage error prints the usage line and exits 1.

#### `pplr refresh`: `next`, `stamp`, `status`

When each person was last checked against their public profile, in `<person>/.index/refreshed` (`2026-09-25 linkedin`), which moves with them on a rename.

```bash
pplr refresh next 20                   # The next 20 to check: never checked first, then the oldest
pplr refresh next 20 --older-than 180  # Only those not checked for 180 days
pplr refresh stamp "Kemp, Jon"         # Record a check today (source: linkedin)
pplr refresh status                    # How many are checked, and how recently
```

### Tags and search

#### `pplr tag "Surname, First" [+tag] [-tag]...`

Show one person's tags by facet, or add and remove them. An added tag is yours (`hand`): later automated passes never overwrite it. Tags must be in the vocabulary; an old spelling is refused with the tag it folds into.

```bash
pplr tag "Kemp, Jon"                  # Their tags, by facet
pplr tag "Kemp, Jon" +investor -london # Add investor, remove london
pplr reindex                          # Refresh the tag pages afterwards
```

#### `pplr tags`: `check`, `show`, `render`, `index`, `apply`

A person's tags live in one place, `About/tags.yaml`, and every tag must be in the vocabulary, `$PPLR_DATA/_pplr/vocabulary.yaml`. A tag is one word of lowercase letters and digits, in one of seven facets: role (`cto`, `founder`, `investor`), function (`engineering`), sector (`fintech`, `healthtech`), org (`startup`), relationship (how you know them: `contact`, `vip`, `career`, `mentor`, `podcast`), place (`london`, `berlin`) and cadence (`3month`, `6month`, `12month`). The vocabulary folds old spellings into its tags (`also`, eg `payments` into `fintech`) and splits old compound tags (`split`).

```yaml
tags:
  - vip # a bare word is your own tag
  - { tag: founder, source: auto, at: 2026-09-25 } # from their profile
  - { tag: mentor, source: inferred, evidence: "Meetings/20250312 Intro" }
```

`hand` tags are never overwritten; `auto` and `inferred` tags are replaced when tags are applied again. Relationship tags (how you know someone) are never generated from a profile: they are hand tags, or inferred from evidence and reviewed, and an inferred tag must be a relationship tag.

```bash
pplr tags check                   # Every tags.yaml against the vocabulary
pplr tags show "Kemp, Jon"        # One person's tags, by facet
pplr tags render                  # The "- Tags: #cto #london" line in each About, from tags.yaml
pplr tags index                   # .index/tags_index.json, which search reads
pplr tags apply reviewed.json     # Write tags.yaml from a reviewed list; hand tags kept
```

The rendered line links each tag to its page, and ends with an HTML comment marking it as generated: change `tags.yaml`, then render again.

`render` also writes the tag pages, `$PPLR_DATA/_tags/`: `index.md` lists every tag by facet with a count, and `<tag>.md` lists everyone with that tag (name linked to their About, role and company, place), then the tags most often seen with it, each linking on. The pages are rewritten in full each time and a page for a tag no longer used is removed, so keep `_tags/` out of git. It rebuilds `.index/tags_index.json` too.

#### `pplr search <query>`

Search for people with a natural-language query, through the Claude CLI (`claude`), over `.index/tags_index.json`: everyone's name, marker, role, company, picture and tags by facet. The query is handed to `claude` as text. Without `claude`, or when it cannot answer, it falls back to a keyword search of tags, role, company and name.

```bash
pplr search "founders in fintech"
pplr search "investors I met in Berlin"
```

#### `pplr grep [options] <text>`

Text search through the people files, or with `-t` through the tags files.

```bash
pplr grep "conference"     # The Markdown files
pplr grep -t fintech       # The tags files
pplr grep --tag mentor     # The same, long form
```

#### `pplr reindex`

Rebuild `.index/index.json`, `.index/index.md` and `.index/tags_index.json`, the tag pages in `_tags/`, and the Tags line in each About, from each person's `About/tags.yaml`.

#### `pplr index`, `pplr json`

The Markdown and the JSON index of everyone, printed to stdout.

```bash
pplr index > index.md
```

### pplr:// links

#### `pplr resolve`, `pplr open`, `pplr links`, `pplr handler`

A `pplr://` URL names a person, not a place: `pplr://k/kemp-jon` is Jon Kemp (lowercase, accents dropped, other characters hyphens), and a path after it names something in his folder, eg `pplr://k/kemp-jon/Meetings/20260924 Intro/Summary.md`. It survives a note being filed into another folder, and, through the person's `.index/aliases`, the person being renamed. Contacts cards carry the same URL, and `pplr://tag/<tag>` names a tag's page.

```bash
pplr resolve pplr://k/kemp-jon            # The file it names: his About
pplr open pplr://k/kemp-jon               # His CMS page if the CMS is up, else the file
pplr open --print pplr://k/kemp-jon       # Where it would go
pplr links ~/Dropbox/Writing/Journal      # Dry run: which links into People would become pplr://
pplr links ~/Dropbox/Writing/Journal --apply
pplr handler --install                    # Register the Pplr Links.app beside pplr for pplr://
pplr handler --status                     # The app, and which app macOS uses for pplr://
```

`pplr links` rewrites Markdown links into the people tree, whether paths (`../../Career/People/K/Kemp, Jon/...`) or CMS URLs (`http://localhost:4360/people/...`). A link to a person's About becomes the bare person URL; anything else keeps its path. A link whose person or file cannot be found is left as it is and listed. A symlinked note is followed to its file, so it is converted once and stays a symlink. `pplr open` uses `$PPLR_CMS_URL` (default `http://localhost:4360/people`; set it empty to always open the file).

`Pplr Links.app` sits beside `pplr` in every tree, signed with the Developer ID in a release, and runs the `pplr` beside it. `pplr handler --install` registers it with Launch Services by its tree's stable path, and `--uninstall` unregisters it. It lets `pplr://` links open from Obsidian, Contacts, Mail and the browser; a browser asks once before handing a link to it. macOS only.

### Apple Contacts

#### `pplr sync --check [options]`

Compare pplr with Apple Contacts. Read-only: it changes nothing on either side.

```bash
pplr sync --check              # Summary, plus name-only, ambiguous and differing matches
pplr sync --check --verbose    # Also list everyone who is only in pplr
pplr sync --check --json       # The whole report as JSON
pplr sync --check --group NAME # The Contacts group that marks pplr people (default: PPLR)
```

Only the About header fields take part: the name (from the folder), Role, Company, Email, Phone and LinkedIn. Bios, notes and meetings never leave pplr. People are matched in this order: a `pplr` URL on the card (`pplr://<letter>/<surname-first>`, eg `pplr://b/blackwood-esme`; the first form, `pplr://B/Blackwood,%20Esme`, is still recognised), then a shared email, then a shared LinkedIn profile, then a shared phone, then the name alone, which is reported for you to confirm. Phones are compared as digits (UK numbers without a country code count as +44, and a trunk 0 after a country code is dropped), LinkedIn by profile slug, and Gmail addresses with or without dots and `+suffix`.

A linked card in an account other than iCloud (eg Gmail) is counted on its own line, "in another account": it carries the `pplr` URL but cannot join the `PPLR` group, which lives in iCloud.

The Contacts side is `pplr-contacts`, a small Swift program beside `pplr` that reads the cards, writes the backup and finds the `PPLR` group; card and photo writes go to Contacts.app through `osascript` and `sips`. Nothing is compiled when pplr runs. The first run asks for access to Contacts for the app running pplr (eg iTerm or Terminal). The live store needs macOS; elsewhere, `sync` reads a dump named by `PPLR_CONTACTS_JSON`.

#### `pplr sync --plan [options]`

Write a workbook for reviewing every pplr person against Contacts before anything changes. It is read-only. Beyond the matches `--check` makes, it scores near misses: a short form of the given name (Bill for William), part of a double-barrelled surname, a one-letter slip, a reversed name, plus a shared company or email domain.

```bash
pplr sync --plan                  # Workbook in $PPLR_DATA/_out/contacts-plan-<stamp>.xlsx
pplr sync --plan --out FILE.xlsx  # Somewhere else
pplr sync --plan --json           # The plan as JSON, no workbook
```

The workbook has four sheets. **Merged** has one row per pplr person, least certain first. Each row holds the proposed Decision (Add, Update, Link only, No change or Skip), the card it would change (a ref such as `C0123`), any duplicate cards, a confidence, the reason, and each field side by side. **pplr** and **Contacts** hold the two lists; the Contacts sheet says which person each card is proposed for. **How to review** explains the columns. Review by changing only the shaded columns on Merged: Decision, Card, Dupes and Notes.

An Update never removes anything from a card: pplr wins on name, company and role, and its emails, phones and LinkedIn are added beside the card's own. A rename is never proposed with High confidence, since pplr's spelling can be the wrong one. pplr writes the workbook itself. The workbook holds everyone's contact details: keep it out of git.

#### `pplr sync --apply-plan WORKBOOK [--apply] [--limit N]`

Carry out a reviewed `--plan` workbook: each row's Decision on the Merged sheet, with the card its Card ref names. Add makes a new card in the default account (iCloud) with the name, company, role, emails, phones, LinkedIn, the `pplr` URL, the photo, and the `PPLR` group. Update brings the card up to date from pplr (name, company, role) and adds what it lacks (emails, phones, LinkedIn); it never removes anything. Link only adds the `pplr` URL and the group. No change and Skip do nothing, and duplicate cards (Dupes) are listed, never touched.

```bash
pplr sync --apply-plan _out/contacts-plan-20260925-1025.xlsx                   # Dry run: what would be written
pplr sync --apply-plan _out/contacts-plan-20260925-1025.xlsx --apply --limit 1 # Try one card
pplr sync --apply-plan _out/contacts-plan-20260925-1025.xlsx --apply           # The rest
```

Every card is saved to a dated `.vcf` first. It works from the current pplr data, so a change since the workbook was made is written as it is now. Applying the same workbook again writes nothing new, and never adds a card twice: a person whose `pplr` URL is already on a card is done. Each written card gets `About/<First Surname> (Contacts).inetloc`. The Merged and Contacts sheets are read by their header names, so a column you insert is safe, and a sheet that lacks one is refused, naming it. A missing option value or a `--limit` that is not a number is refused, with the usage.

#### `pplr sync --pull [--apply] [--name NAME]... [--skip NAME]... [--limit N]`

The other direction: copy into pplr what a linked or matched card knows and pplr lacks. Emails, phones and a LinkedIn missing from the About header are added beside what is there; a company or role is filled in only where pplr has none. pplr wins wherever it has a value, and nothing in pplr is removed. Name-only matches are left out until they are linked. A card linked across accounts is read as one, every copy together.

```bash
pplr sync --pull                                     # Dry run: what each person would gain
pplr sync --pull --apply --skip "Okafor, Chidi"      # Everyone but one (repeatable)
pplr sync --pull --apply --name "Quennell, Imogen"   # Only this person (repeatable)
```

A number saved in Contacts without its country code is read as UK, so an Australian `04...` number comes out as `+444...`. A `+44` number that is not a real UK number (ten digits after `+44`, never starting with 4) is left out and listed, to be put right in Contacts. Each About file is copied to `$PPLR_BACKUP_DIR/<stamp>-pull/` before it is written, and an About file that is not UTF-8 is refused, naming it, so pplr never rewrites a byte it could not read.

#### `pplr sync --photos [--apply] [--limit N]`

Give each linked card that has no photo at all the person's pplr picture (`About/<First Surname> (Picture).jpg`). A card that already has a photo is never touched. A dry run unless `--apply`, with a `.vcf` backup first. Photos set by script are stored as the card's thumbnail, which is what Contacts shows.

#### `pplr sync --link [options]`

Mark the cards that match pplr people as pplr's: a URL labelled `pplr` (`pplr://<letter>/<surname-first>`: lowercase, accents dropped, other characters hyphens; an older form is replaced) and, for cards in the default account (iCloud), membership of the `PPLR` group. Cards in other accounts, eg Gmail, carry the URL alone, so there is one `PPLR` group. A matched card in no account (a directory or Other Known card) cannot be written and is listed instead. Nothing else on the card changes. On the pplr side it writes `About/<First Surname> (Contacts).inetloc`, which opens the card in Contacts (`addressbook://<card id>`; the id is this Mac's). It is an `.inetloc` because macOS opens a `.webloc` only for web addresses. It is a dry run unless `--apply` is given.

```bash
pplr sync --link                                  # Dry run: who would be linked
pplr sync --link --apply                          # Link everyone matched by email or LinkedIn
pplr sync --link --apply --name "Okafor, Chidi"   # Also link a confirmed name-only match (repeatable)
pplr sync --link --apply --all-names              # Also link every name-only match
```

Ambiguous people (more than one possible card) are never linked. Running it again changes nothing for people already linked.

#### `pplr sync --backup`

Save every Contacts card to a dated `.vcf` in `$PPLR_BACKUP_DIR` (default `~/.local/share/pplr/contacts-backups`). `--link --apply` does this first, every time. Backups written before 2.0.0 stay in `~/Library/Application Support/pplr/contacts-backups`, which `pplr doctor` names while it is there. Apple's vCard export leaves out notes and photos, so keep a full Contacts Archive (File > Export > Contacts Archive) as well.

### Finder icons (macOS)

These need [`fileicon`](https://github.com/mklement0/fileicon) (`brew install fileicon`). On Linux they exit 1, saying they need macOS.

#### `pplr applyicons <firstname> <surname>`

Custom Finder icons on the person's About and Meetings folders and on each meeting's folder, from the tree's `_pplr/icons`, else the set inside the binary (`PPLR_ICON_DIR` overrides both).

#### `pplr setpicfordir [-V] <firstname> <surname>`

The person's picture as the icon of their own folder. `-V` says what it does.

#### `pplr setpicsfordirs`

Everyone's picture as their folder's icon.

### pplr itself

#### `pplr doctor [--fix]`

Check the tree, its index files, the settings and the tools pplr uses, one row per check, each saying what to fix: the install, the environment variables, the directory structure, the dependencies (`claude`, and on macOS `fileicon`, `osascript`, the Contacts sidecar and Mail's store), the index files, the configuration files and the common issues. `--fix` makes the letter folders and `.index`, reindexes, and removes old `Index/` folders. It exits with the number of checks that failed.

#### `pplr help [command]`

```bash
pplr help           # Every command
pplr help search    # One command
pplr help --details # This README
```

#### `pplr version`

```bash
pplr version     # pplr 2.0.0 (brew, 1a2b3c4d5e6f): the version, the tree it runs from and its commit
pplr -v          # The same
pplr --version   # The same
```

`pplr install` and `pplr use` are the maintainer's: see [`docs/releasing.md`](https://github.com/matthewsinclair/pplr/blob/main/docs/releasing.md).

## File Formats

### About file (Markdown)

What `pplr new "Esme" "Blackwood" "https://www.linkedin.com/in/esme-blackwood"` writes, with the Role, Company, Email and Phone then filled in. Every command reads these fields from the header, the first of each winning; the verblock's author is `git config user.name`.

```text
---
verblock: "2026-04-20:v0.1: Your Name - Initial version"
---
# Esme Blackwood (About)

_Esme Blackwood_
- Role:     [Founder](https://example.com)
- Company:  [Example Ltd](https://example.com)
- LinkedIn: [in/esme-blackwood](https://www.linkedin.com/in/esme-blackwood)
- Email:    [esme@example.com](mailto:esme@example.com)
- Phone:    +44 20 7946 0000

_About_
```

### Meeting file (Markdown)

`Meetings/YYYYMMDD <Title>/YYYYMMDD <Title>.md`, as the template lays it out:

```text
---
verblock: "2026-04-20:v0.1: Your Name - Initial version"
---
# 20260420 Intro

[Esme Blackwood](pplr://b/blackwood-esme)

##### Notes

##### Summary

##### Transcript
```

### Tags file (YAML)

`About/tags.yaml`: a bare word is your own tag. See [`pplr tags`](#pplr-tags-check-show-render-index-apply).

```yaml
tags:
  - vip
  - { tag: founder, source: auto, at: 2026-09-25 }
  - { tag: mentor, source: inferred, evidence: "Meetings/20250312 Intro" }
```

## Best Practices

1. **Consistent naming**: one folder per person, named "Surname, First"
2. **Regular updates**: keep About files current with role changes
3. **Meeting notes**: include who was there, what was discussed and what happens next
4. **Tags**: tag people as you meet them (`pplr tag "Surname, First" +tag`), then `pplr reindex`
5. **Renames**: always through `pplr rename`, never by moving the folder
6. **Backups**: keep the tree in Dropbox or similar, and a Contacts Archive beside the `.vcf` backups

## Requirements

- **Operating system**: macOS on Apple silicon; Linux on `x86_64` or `aarch64` comes in a later release
- **Optional**:
  - the Claude CLI (`claude`), for natural-language search
  - `fileicon` (`brew install fileicon`), for the Finder icon commands
  - on macOS, access to Contacts for `pplr sync`, and Full Disk Access for `pplr email`

Nothing else is needed at run time: no Python, no `jq` and no Swift toolchain.

## Troubleshooting

Start with `pplr doctor`: it checks the tree, its indexes, the settings and the tools, and says what to fix.

### Search returns no results

- Run `pplr reindex` to rebuild the indexes
- Check their tags: `pplr tag "Surname, First"`, and `pplr tags check`
- Check that `.index/tags_index.json` exists and is recent

### Search does not use Claude

- Make sure the Claude CLI is installed and on your PATH: `pplr doctor` says whether it finds `claude`
- Without it, `pplr search` falls back to a keyword search

### `pplr sync` says it has no access to Contacts

- Allow the terminal app in System Settings > Privacy & Security > Contacts, then run it again

### `pplr email` says it has no Full Disk Access

- Allow the terminal app in System Settings > Privacy & Security > Full Disk Access, then run it again

### A command says it needs macOS

- On Linux, the live Contacts store, `handler`, the Finder icon commands, `open`, `linkedin` and `cp` without `-o` need a macOS service, and say so. `sync` works there against a dump named by `PPLR_CONTACTS_JSON`

## Contributing

pplr is a personal project, but suggestions and improvements are welcome. It is one Rust crate, with a small Swift sidecar for Contacts and an AppleScript app for `pplr://` links. Building it from a checkout, testing it and releasing it are in [`docs/releasing.md`](https://github.com/matthewsinclair/pplr/blob/main/docs/releasing.md).

## License

MIT: see [LICENSE.md](LICENSE.md).
