# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Commands

The pplr (Peopler) system uses the following commonly used commands:

### Creating and Managing People Entries
```bash
# Create a new person entry (with optional LinkedIn URL)
pplr new "Smith" "John" "https://linkedin.com/in/johnsmith"

# Search for people by name or content
pplr search "keyword"
pplr search "john"

# Open a person's About file
pplr open "Smith" "John"

# Edit a person's About file
pplr edit "Smith" "John"
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
pplr cp "Smith" "John"

# Regenerate index files (index.md and index.json)
pplr reindex

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
│       ├── Meetings/
│       └── Client/
├── _Templates/
├── bin/                    # All pplr scripts
├── index.json
└── index.md
```

### Key Components
- **Main Script**: `bin/pplr` - Entry point that delegates to sub-commands
- **Sub-commands**: Located in `bin/pplr_*` - Each handles specific functionality
- **Data Format**: Markdown files with verblock headers for About files
- **Indexing**: Automatic generation of index.md and index.json for navigation

### Environment Variables
- `PPLR_DIR`: Main directory (defaults to `$HOME/Dropbox/Career/People`)
- `PPLR_BIN_DIR`: Binary directory location

When modifying the codebase, maintain consistency with the existing bash script style and ensure all person entries follow the established directory structure.