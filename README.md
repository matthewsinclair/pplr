# pplr - Personal Relationship Manager

[![Tests](https://github.com/matthewsinclair/Pplr/actions/workflows/test.yml/badge.svg)](https://github.com/matthewsinclair/Pplr/actions/workflows/test.yml)

A command-line personal relationship management (PRM) system for organising professional contacts, meetings, and relationships.

## Overview

pplr (pronounced "peopler") is a CLI tool that helps you manage your professional network. It stores information about people you know, tracks meetings and interactions, and provides powerful search capabilities enhanced with AI-powered tagging.

## Features

- **Contact Management**: Store detailed information about professional contacts
- **Meeting Tracking**: Record and search meeting notes with attendees
- **Tags**: one `tags.yaml` per person against a shared vocabulary, with a generated page per tag
- **AI-Powered Search**: natural language queries over the index, with Claude AI
- **Smart Tagging**: Automatically generate searchable tags from profiles and meetings
- **LinkedIn Integration**: Store and quick-access LinkedIn profiles
- **Cross-Platform**: Works on macOS with Dropbox sync support

## Installation

1. Clone this repository to your preferred location:
   ```bash
   git clone <repo-url> ~/Devel/prj/Pplr
   ```
2. Set environment variables:
   ```bash
   # Code/config directory (this git repository)
   export PPLR_ROOT="$HOME/Devel/prj/Pplr"
   export PPLR_BIN_DIR="$PPLR_ROOT/bin"

   # Data directory (where your contacts are stored)
   export PPLR_DATA="$HOME/Dropbox/Career/People"
   export PPLR_DIR="$PPLR_DATA"  # Legacy compatibility
   ```
3. Add the bin directory to your PATH:
   ```bash
   export PATH="$PATH:$PPLR_BIN_DIR"
   ```

**Important**: pplr separates code and data into two distinct directories:

- **`PPLR_ROOT`**: Code, scripts, and configuration (this git repository)
- **`PPLR_DATA`**: Your personal contact database (typically in Dropbox for sync)

## Directory Structure

### Code Repository (`$PPLR_ROOT`)

This git repository contains:

```
Pplr/
├── bin/                         # pplr scripts and executables
│   ├── pplr                     # Main command
│   ├── pplr_search              # Natural language search
│   ├── pplr_new                 # Create new contacts
│   ├── pplr_tag                 # Generate AI tags
│   ├── pplr_reindex             # Rebuild indexes
│   └── [other commands]
├── templates/                   # Templates for new entries
├── tests/                       # Test suite
├── CHANGELOG.md                 # Change log
├── CLAUDE.md                    # Instructions for Claude Code
├── LICENSE.md                   # License information
├── README.md                    # This file
├── setup.sh                     # Setup script
└── usage-rules.md               # Usage guidelines
```

### Data Directory (`$PPLR_DATA`)

Your personal contact database (separate from git repo):

```
People/
├── A-Z/                          # Alphabetical directories
│   └── [Surname, Firstname]/     # Person's directory
│       ├── About/                # Profile information
│       │   ├── [Name] (About).md
│       │   ├── [Name] (LinkedIn).webloc
│       │   ├── [Name] (Picture).[jpg|png]
│       │   ├── [Name] (Profile).pdf
│       │   └── tags.yaml         # Their tags (the only place they live)
│       ├── .index/               # Per-person data (eg aliases: former names)
│       ├── Meetings/             # Meeting records
│       │   └── YYYYMMDD Meeting Name/
│       │       └── YYYYMMDD Meeting Name.md
│       └── Client/               # Client-specific data
├── .index/                       # Global search and indexing files
│   ├── index.json                # JSON index of all contacts
│   ├── index.md                  # Markdown index of all contacts
│   └── tags_index.json           # Everyone: marker, role, company, picture, tags by facet
├── _pplr/                        # pplr config: vocabulary.yaml, templates/
├── _tags/                        # Generated tag pages: index.md and <tag>.md
└── _out/                         # Generated workbooks
```

## Commands

### Core Commands

#### `pplr new <firstname> <surname> [LinkedIn-URL]`

Create a new person entry with optional LinkedIn URL.

```bash
pplr new "John" "Smith" "https://linkedin.com/in/johnsmith"
```

#### `pplr search <query>`

Search for people using natural language queries powered by Claude AI, over `.index/tags_index.json` (built by `pplr reindex`). Without the Claude CLI it falls back to matching tags, role, company and name.

```bash
pplr search "people in fintech"              # Natural language search
pplr search "film production founders"       # Industry and role search
pplr search "engineers I should reconnect with"  # Smart recommendations
```

#### `pplr open [-t type] <firstname> <surname>`

Open a person's file in your default application.

```bash
pplr open "John" "Smith"                    # Open About file (default)
pplr open -t linkedin "John" "Smith"         # Open LinkedIn profile
pplr open -t profile "John" "Smith"          # Open PDF profile
```

Options:

- `-t, --type`: File type to open (about, linkedin, profile)

#### `pplr meetings [start-date] [end-date]`

Find meetings within a date range.

```bash
pplr meetings 2024-01-01 2024-12-31    # All meetings in 2024
pplr meetings 2024-03-15                # Meetings on specific date
pplr meetings                           # Recent meetings
```

### Tag Management

#### `pplr tag "Surname, First" [+tag] [-tag]...`

Show one person's tags by facet, or add and remove them. An added tag is yours (`hand`): later automated passes never overwrite it. Tags must be in the vocabulary; an old spelling is refused with the tag it folds into.

```bash
pplr tag "Kemp, Jon"                 # Their tags, by facet
pplr tag "Kemp, Jon" +vc -london     # Add vc, remove london
pplr reindex                          # Refresh the tag pages afterwards
```

### Utility Commands

#### `pplr grep [options] <text>`

Text search through people files or tag files.

```bash
pplr grep "conference"              # Search in markdown files
pplr grep -t "python"               # Search in tag files
pplr grep --tag "backend engineer"  # Search in tag files (long form)
```

Options:

- `-t, --tag`: Search in tag files instead of markdown files

#### `pplr count`

Count total number of people in the database.

```bash
pplr count
```

#### `pplr edit <firstname> <surname>`

Edit a person's About file.

```bash
pplr edit "John" "Smith"
```

#### `pplr linkedin <firstname> <surname>`

Open a person's LinkedIn profile in your browser.

```bash
pplr linkedin "John" "Smith"
```

#### `pplr cp <firstname> <surname>`

Copy a person's directory path to clipboard.

```bash
pplr cp "John" "Smith"
```

#### `pplr version`

Display the current version of pplr.

```bash
pplr version     # Shows version number
pplr -v          # Short form
pplr --version   # Long form
```

#### `pplr reindex`

Rebuild `.index/index.json`, `.index/index.md` and `.index/tags_index.json`, the tag pages in `_tags/`, and the Tags line in each About, from each person's `About/tags.yaml`.

```bash
pplr reindex
```

The Claude tagger that `--tags` ran is retired; see `pplr tag` and `pplr tags`.

#### `pplr://` links: `pplr resolve`, `pplr open`, `pplr links`, `pplr handler`

A `pplr://` URL names a person, not a place: `pplr://k/kemp-jon` is Jon Kemp (lowercase, accents dropped, other characters hyphens), and a path after it names something in his folder, eg `pplr://k/kemp-jon/Meetings/20260924 Intro/Summary.md`. It survives a note being filed into another folder, and, through `.index/aliases`, the person being renamed. Contacts cards carry the same URL.

```bash
pplr resolve pplr://k/kemp-jon            # The file it names: his About
pplr open pplr://k/kemp-jon               # His CMS page if the CMS is up, else the file
pplr open --print pplr://k/kemp-jon       # Where it would go
pplr links ~/Dropbox/Writing/Journal       # Dry run: which links into People would become pplr://
pplr links ~/Dropbox/Writing/Journal --apply
pplr handler --install                     # ~/Applications/Pplr Links.app, registered for pplr://
```

`pplr links` rewrites Markdown links into the people tree, whether paths (`../../Career/People/K/Kemp, Jon/...`) or CMS URLs (`http://localhost:4360/people/...`). A link to a person's About becomes the bare person URL; anything else keeps its path. A link whose person or file cannot be found is left as it is and listed. A symlinked note is followed to its file, so it is converted once and stays a symlink. `pplr open` uses `$PPLR_CMS_URL` (default `http://localhost:4360/people`; set it empty to always open the file). The handler lets `pplr://` links open from Obsidian, Contacts, Mail and the browser; a browser asks once before handing a link to it.

#### `pplr tags`: `check`, `show`, `render`, `apply`

A person's tags live in one place, `About/tags.yaml`, and every tag must be in the vocabulary, `$PPLR_DATA/_pplr/vocabulary.yaml`. Tags are single lowercase words, grouped by facet (role, function, sector, org, relationship, place); the vocabulary folds old spellings into its tags (`also`) and splits old compound tags (`split`).

```yaml
tags:
  - fintech # a bare word is your own tag
  - { tag: cto, source: auto, at: 2026-09-25 } # from their profile
  - { tag: bcg, source: inferred, evidence: "BCG DV 2017-2020, Profile.pdf" }
```

`hand` tags are never overwritten; `auto` and `inferred` tags are replaced when tags are applied again. Relationship tags (how you know someone) are never generated from a profile: they are hand tags, or inferred from evidence and reviewed.

```bash
pplr tags check                   # Every tags.yaml against the vocabulary
pplr tags show "Kemp, Jon"       # One person's tags, by facet
pplr tags render                  # The "- Tags: #cto #london" line in each About, from tags.yaml
pplr tags apply reviewed.json     # Write tags.yaml from a reviewed list; hand tags kept
```

The rendered line links each tag to its page, and ends with an HTML comment marking it as generated: change `tags.yaml`, then render again.

`render` also writes the tag pages, `$PPLR_DATA/_tags/`: `index.md` lists every tag by facet with a count, and `<tag>.md` lists everyone with that tag (name linked to their About, role and company, place), then the tags most often seen with it, each linking on. The pages are rewritten in full each time and a page for a tag no longer used is removed, so keep `_tags/` out of git.

#### `pplr pictures [--dry-run] ["Surname, First"]...`

Show each person's photo at the top of their About page: an `<img>` line under the title, floated right, pointing at `About/<First Surname> (Picture).jpg` (or `.png`). It is idempotent: run it again after adding people or pictures. People with no picture get no line. `PPLR_PICTURE_WIDTH` sets the width (default 160).

#### `pplr rename "Old, First" "New, First" [options]`

Rename a person: moves the folder (to a new letter if the surname's initial changes), renames the files named after them (`First Surname (About).md`, `(Picture).jpg` and the rest), and updates their name in the About file and in their own meeting notes' links. The old name is recorded in `.index/aliases`, so a Contacts card still carrying the old `pplr://` URL is found and the URL replaced on the next `pplr sync --link`. Path-style links elsewhere are repointed: the folder name, its URL-encoded forms, and the `First Surname (` file names. The old name in running text is listed, never changed. It then reindexes.

```bash
pplr rename "Grifiths, Glen" "Griffiths, Glen" --dry-run        # What would change
pplr rename "Grifiths, Glen" "Griffiths, Glen"                  # Links under $PPLR_REFS_DIR (default ~/Dropbox)
pplr rename "Grifiths, Glen" "Griffiths, Glen" --refs ~/Dropbox/Writing/Journal
```

#### `pplr sync --check [options]`

Compare pplr with Apple Contacts. Read-only: it changes nothing on either side.

```bash
pplr sync --check              # Summary, plus name-only, ambiguous and differing matches
pplr sync --check --verbose    # Also list everyone who is only in pplr
pplr sync --check --json       # The whole report as JSON
pplr sync --check --group NAME # The Contacts group that marks pplr people (default: PPLR)
```

Only the About header fields take part: the name (from the folder), Role, Company, Email, Phone and LinkedIn. Bios, notes and meetings never leave pplr. People are matched in this order: a `pplr` URL on the card (`pplr://<letter>/<surname-first>`, eg `pplr://b/bray-martin`; the first form, `pplr://B/Bray,%20Martin`, is still recognised), then a shared email, then a shared LinkedIn profile, then a shared phone, then the name alone, which is reported for you to confirm. Phones are compared as digits (UK numbers without a country code count as +44, and a trunk 0 after a country code is dropped), and LinkedIn by profile slug.

The Contacts side is a small Swift program, `src/pplr-contacts/main.swift`, built into `.build/` on first use. The first run asks for access to Contacts for the app running pplr (eg iTerm or Terminal).

#### `pplr sync --plan [options]`

Write a workbook for reviewing every pplr person against Contacts before anything changes. It is read-only. Beyond the matches `--check` makes, it scores near misses: a short form of the given name (Bill for William), part of a double-barrelled surname, a one-letter slip, a reversed name, plus a shared company or email domain.

```bash
pplr sync --plan                  # Workbook in $PPLR_DATA/_out/contacts-plan-<stamp>.xlsx
pplr sync --plan --out FILE.xlsx  # Somewhere else
pplr sync --plan --json           # The plan as JSON, no workbook
```

The workbook has four sheets. **Merged** has one row per pplr person, least certain first. Each row holds the proposed Decision (Add, Update, Link only, No change or Skip), the card it would change (a ref such as `C0123`), any duplicate cards, a confidence, the reason, and each field side by side. **pplr** and **Contacts** hold the two lists; the Contacts sheet says which person each card is proposed for. **How to review** explains the columns. Review by changing only the shaded columns on Merged: Decision, Card, Dupes and Notes.

An Update never removes anything from a card: pplr wins on name, company and role, and its emails, phones and LinkedIn are added beside the card's own. A rename is never proposed with High confidence, since pplr's spelling can be the wrong one. The workbook step runs with `uv` (`uv run --with openpyxl`), so nothing is installed globally. The workbook holds everyone's contact details: keep it out of git.

#### `pplr sync --link [options]`

Mark the cards that match pplr people as pplr's: a URL labelled `pplr` (`pplr://<letter>/<surname-first>`: lowercase, accents dropped, other characters hyphens; an older form is replaced) and, for cards in the default account (iCloud), membership of the `PPLR` group. Cards in other accounts, eg Gmail, carry the URL alone, so there is one `PPLR` group. A matched card in no account (a directory or Other Known card) cannot be written and is listed instead. Nothing else on the card changes. On the pplr side it writes `About/<First Surname> (Contacts).webloc`, which opens the card in Contacts (`addressbook://<card id>`; the id is this Mac's). It is a dry run unless `--apply` is given.

```bash
pplr sync --link                               # Dry run: who would be linked
pplr sync --link --apply                       # Link everyone matched by email or LinkedIn
pplr sync --link --apply --name "Webb, Owen"    # Also link a confirmed name-only match (repeatable)
pplr sync --link --apply --all-names           # Also link every name-only match
```

Ambiguous people (more than one possible card) are never linked. Running it again changes nothing for people already linked.

#### `pplr sync --backup`

Save every Contacts card to a dated `.vcf` in `$PPLR_BACKUP_DIR` (default `~/Library/Application Support/pplr/contacts-backups`). `--link --apply` does this first, every time. Apple's vCard export leaves out notes and photos, so keep a full Contacts Archive (File > Export > Contacts Archive) as well.

#### `pplr help [command]`

Show help for all commands or a specific command.

```bash
pplr help           # All commands
pplr help search    # Specific command
pplr help --details # Show this README
```

#### `pplr about <firstname> <surname>`

Display a person's About file content in the terminal.

```bash
pplr about "John" "Smith"
```

#### `pplr index`

Generate the markdown index of all people.

```bash
pplr index > index.md
```

### Visual Commands

#### `pplr applyicons <firstname> <surname>`

Apply the person's picture as their folder icon (macOS).

```bash
pplr applyicons "John" "Smith"
```

#### `pplr setpicsfordirs`

Set pictures as folder icons for all people directories.

```bash
pplr setpicsfordirs
```

## Tags and Search

Every tag is a single lowercase word from `_pplr/vocabulary.yaml`, in one of six facets: role (`cto`, `founder`), function (`product`, `ai`), sector (`fintech`, `climate`), org (`startup`, `studio`), relationship (how you know them: `bcg`, `client`, `podcast`, `mba`) and place (`uk`, `london`). Each tag records its source: `hand` (yours), `auto` (from the profile) or `inferred` (a relationship, with its evidence). See `pplr tags` above.

### Smart Search

The search command intelligently processes queries:

- Industry matching: "film" finds "TV", "media", "entertainment"
- Role matching: "tech" finds "CTO", "engineer", "developer"
- Temporal queries: "recent meetings", "last month"

## File Formats

### About File (Markdown)

```markdown
verblock(<version>)

# John Smith (About)

- Role: Chief Technology Officer
- Company: Tech Innovations Ltd
- LinkedIn: https://linkedin.com/in/johnsmith
- Email: john.smith@example.com
- Phone: +1-555-0123

## Bio

John is a technology leader with 15 years of experience...
```

### Meeting File (Markdown)

```markdown
verblock(<version>)

# 20240315 Strategy Discussion

## Meeting Summary

Purpose: Discuss Q2 technology strategy
Date: 2024-03-15
Attendees: [[Smith, John]], [[Doe, Jane]]

## Key Takeaways

- Agreement on cloud migration timeline
- Budget approved for new hires

## Action Items

- [ ] John: Prepare technical roadmap
- [ ] Jane: Review vendor proposals
```

### Tags File (YAML)

`About/tags.yaml`: a bare word is your own tag.

```yaml
tags:
  - vc
  - { tag: cto, source: auto, at: 2026-09-25 }
  - { tag: bcg, source: inferred, evidence: "BCG DV 2017-2020, Profile.pdf" }
```

## Best Practices

1. **Consistent Naming**: Always use "Surname, Firstname" format
2. **Regular Updates**: Keep About files current with role changes
3. **Meeting Notes**: Include attendees, topics, and action items
4. **Tags**: Tag people as you meet them (`pplr tag "Surname, First" +tag`), then `pplr reindex`
5. **Backups**: Use Dropbox or similar for automatic backups

## Requirements

- **Operating System**: macOS (primary), Linux (partial support)
- **Dependencies**:
  - bash 4.0+
  - jq (for JSON processing)
  - Claude CLI (for AI features)
  - Swift (the Xcode command line tools, for `pplr sync`)
- **Optional**: Dropbox for sync

## Troubleshooting

### Search returns no results

- Run `pplr reindex` to rebuild indexes
- Check their tags: `pplr tag "Surname, First"`, and `pplr tags check`
- Check if `.index/tags_index.json` exists and is recent

### Claude/AI features not working

- Ensure Claude CLI is installed and in PATH
- Check Claude is accessible: `which claude` and `echo "test" | claude`
- If using a mock for testing, ensure `PPLR_TEST_DATA` is not set in production

### `pplr sync` says it has no access to Contacts

- Allow the terminal app in System Settings > Privacy & Security > Contacts, then run it again

### Permission errors

- Check file permissions in PPLR_DIR
- Ensure scripts are executable: `chmod +x $PPLR_BIN_DIR/pplr_*`

## Contributing

pplr is a personal project, but suggestions and improvements are welcome. The codebase is simple bash scripts designed for maintainability and extensibility.

## License

This is a personal tool shared as-is for educational purposes.
