# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

The pplr (Peopler) system uses the following commonly used commands:

### Creating and Managing People Entries

```bash
# Create a new person entry (with optional LinkedIn URL)
pplr new "John" "Smith" "https://linkedin.com/in/johnsmith"

# Search for people by name or content
pplr search "keyword"
pplr search "john"

# Open a person's About file
pplr open "John" "Smith"

# Edit a person's About file
pplr edit "John" "Smith"
```

### Managing Meetings

```bash
# Find meetings within a date range
pplr meetings 2024-01-01 2024-12-31

# Find meetings for a specific date
pplr meetings 2024-03-15
```

### Other Useful Commands

```bash
# Show help for all commands
pplr help

# Show version
pplr version
pplr -v
pplr --version

# Count total number of people
pplr count

# Text search through files
pplr grep "keyword"
pplr grep -t "python"  # Search in tag files

# Copy a person's file path to clipboard
pplr cp "John" "Smith"

# Regenerate index files (.index/index.md, .index/index.json, and .index/tags_index.json)
pplr reindex

# Show or edit one person's tags (About/tags.yaml, against _pplr/vocabulary.yaml)
pplr tag "Kemp, Jon" +vc -london

# Check everyone's tags against the vocabulary
pplr tags check

# Generate JSON index
pplr json

# Compare with Apple Contacts (read-only)
pplr sync --check
pplr sync --check --json
pplr sync --link            # dry run; --apply writes, after a .vcf backup
pplr sync --backup
```

## Codebase Architecture

This is a bash-based personal relationship management (PRM) system that organises professional contacts. The system uses:

- **Language**: Bash shell scripts; `pplr sync` uses a Swift engine (`src/pplr-contacts/`, compiled to `.build/`) for the Contacts framework
- **Data Storage**: Markdown files for content, JSON for indexing
- **Platform**: macOS-specific features (uses .webloc files for URLs)

### Directory Structure

```
People/
├── A-Z/                    # Alphabetical directories
│   └── [Surname, Firstname]/
│       ├── About/
│       │   ├── [Name] (About).md
│       │   ├── [Name] (LinkedIn).webloc
│       │   ├── [Name] (Picture).[jpg|png|etc]
│       │   ├── [Name] (Profile).pdf
│       │   └── tags.yaml    # This person's tags: the only place they live
│       ├── .index/
│       │   └── aliases      # Former names (pplr rename), so old pplr:// URLs resolve
│       ├── Meetings/
│       └── Client/
├── _pplr/                  # pplr config: vocabulary.yaml, templates/
├── _tags/                  # Generated tag pages (pplr reindex)
├── .index/                   # Search and indexing files
│   ├── index.json           # JSON index of all contacts
│   ├── index.md             # Markdown index of all contacts
│   └── tags_index.json      # Everyone: marker, role, company, picture, tags by facet
└── [other files]
```

### Key Components

- **Main Script**: `bin/pplr` - Entry point that delegates to sub-commands
- **Sub-commands**: Located in `bin/pplr_*` - Each handles specific functionality
- **Data Format**: Markdown files with verblock headers for About files
- **Indexing**: Automatic generation of .index/ files (index.md, index.json, tags_index.json) for navigation and search

### Environment Variables

- `PPLR_DIR`: Main directory (defaults to `$HOME/Dropbox/Career/People`)
- `PPLR_BIN_DIR`: Binary directory location

When modifying the codebase, maintain consistency with the existing bash script style and ensure all person entries follow the established directory structure.

## Tags

Tags are single lowercase words from `$PPLR_DATA/_pplr/vocabulary.yaml`, grouped into six facets. Each person's tags live only in `About/tags.yaml`, each with a source: `hand` (the owner's, never overwritten), `auto` (from the profile) or `inferred` (a relationship, with evidence). Relationship tags are never generated from a profile. The Claude tagger (`pplr reindex --tags`, the old `pplr tag`) is retired.
