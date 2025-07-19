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

# Count total number of people
pplr count

# Copy a person's file path to clipboard
pplr cp "John" "Smith"

# Regenerate index files (.index/index.md, .index/index.json, and .index/tags_index.json)
pplr reindex

# Regenerate indexes and all tags
pplr reindex --tags

# Only regenerate stale tags (missing or older than 30 days)
pplr reindex --tags --stale-only

# Generate JSON index
pplr json
```

## Codebase Architecture

This is a bash-based personal relationship management (PRM) system that organises professional contacts. The system uses:

- **Language**: Bash shell scripts
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
│       │   └── [Name] (Profile).pdf
│       ├── .index/
│       │   └── tags.json    # AI-generated tags for this person
│       ├── Meetings/
│       └── Client/
├── _Templates/
├── bin/                    # All pplr scripts
├── .index/                   # Search and indexing files
│   ├── index.json           # JSON index of all contacts
│   ├── index.md             # Markdown index of all contacts  
│   └── tags_index.json      # Optimized search context for Claude
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

## Recent Changes (July 2025)

- Individual tag files moved from `Index/tags.json` to `.index/tags.json`
- Natural language search is now the default (no `--tags` flag needed)
- Added intelligent tag regeneration with `--stale-only` flag
- Enhanced test infrastructure with proper mock Claude handling