# pplr - Personal Relationship Manager

A command-line personal relationship management (PRM) system for organising professional contacts, meetings, and relationships.

## Overview

pplr (pronounced "peopler") is a CLI tool that helps you manage your professional network. It stores information about people you know, tracks meetings and interactions, and provides powerful search capabilities enhanced with AI-powered tagging.

## Features

- **Contact Management**: Store detailed information about professional contacts
- **Meeting Tracking**: Record and search meeting notes with attendees
- **AI-Powered Search**: Use natural language queries with Claude AI integration
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
│       │   └── [Name] (Profile).pdf
│       ├── .index/               # Generated data
│       │   └── tags.json         # AI-generated tags
│       ├── Meetings/             # Meeting records
│       │   └── YYYYMMDD Meeting Name/
│       │       └── YYYYMMDD Meeting Name.md
│       └── Client/               # Client-specific data
├── .index/                       # Global search and indexing files
│   ├── index.json                # JSON index of all contacts
│   ├── index.md                  # Markdown index of all contacts
│   └── tags_index.json           # Optimized search context for Claude
└── [other data files]
```

## Commands

### Core Commands

#### `pplr new "Surname" "Firstname" [LinkedIn-URL]`
Create a new person entry with optional LinkedIn URL.
```bash
pplr new "Smith" "John" "https://linkedin.com/in/johnsmith"
```

#### `pplr search <query>`
Search for people using natural language queries powered by Claude AI.
```bash
pplr search "people in fintech"              # Natural language search
pplr search "film production founders"       # Industry and role search  
pplr search "engineers I should reconnect with"  # Smart recommendations
```

#### `pplr open "Surname" "Firstname"`
Open a person's About file in your default editor.
```bash
pplr open "Smith" "John"
```

#### `pplr meetings [start-date] [end-date]`
Find meetings within a date range.
```bash
pplr meetings 2024-01-01 2024-12-31    # All meetings in 2024
pplr meetings 2024-03-15                # Meetings on specific date
pplr meetings                           # Recent meetings
```

### Tag Management

#### `pplr tag "Surname" "Firstname"`
Generate AI-powered tags for a specific person.
```bash
pplr tag "Smith" "John"
```

#### `pplr tag <partial-name> -g|--generate`
Generate tags for all people matching the partial name.
```bash
pplr tag "Smith" -g    # Tag all Smiths
```

#### `pplr tag --all`
Generate tags for everyone in the database (takes time).
```bash
pplr tag --all
```

#### `pplr tag "Surname" "Firstname" -s`
Show existing tags for a person.
```bash
pplr tag "Smith" "John" -s
```

### Utility Commands

#### `pplr grep <text>`
Simple text search through all files (formerly pplr search).
```bash
pplr grep "conference"
```

#### `pplr count`
Count total number of people in the database.
```bash
pplr count
```

#### `pplr edit "Surname" "Firstname"`
Edit a person's About file.
```bash
pplr edit "Smith" "John"
```

#### `pplr linkedin "Surname" "Firstname"`
Open a person's LinkedIn profile in your browser.
```bash
pplr linkedin "Smith" "John"
```

#### `pplr cp "Surname" "Firstname"`
Copy a person's directory path to clipboard.
```bash
pplr cp "Smith" "John"
```

#### `pplr reindex [options]`
Regenerate index files and optionally tags with intelligent regeneration.
```bash
pplr reindex                              # Just indexes
pplr reindex --tags                       # Indexes and regenerate all tags
pplr reindex --tags --stale-only          # Only regenerate missing/old tags
pplr reindex --tags --stale-only --max-age=7d   # Custom staleness threshold
```

Options:
- `--tags`: Generate tags using Claude AI
- `--stale-only`: Only regenerate tags that are missing or older than max-age
- `--max-age=N`: Set maximum age for stale detection (e.g., 30d, 7days, 2weeks)

#### `pplr help [command]`
Show help for all commands or a specific command.
```bash
pplr help           # All commands
pplr help search    # Specific command
pplr help --details # Show this README
```

### Visual Commands

#### `pplr applyicons "Surname" "Firstname"`
Apply the person's picture as their folder icon (macOS).
```bash
pplr applyicons "Smith" "John"
```

#### `pplr setpicsfordirs`
Set pictures as folder icons for all people directories.
```bash
pplr setpicsfordirs
```

## Migration Tools

#### `pplr migrate_tags [--force]`
Migrate individual tag files from old `Index/tags.json` to new `.index/tags.json` location.
```bash
pplr migrate_tags        # Preview migration
pplr migrate_tags --force # Execute migration
```

This tool:
- Creates `.index/` directories for each person
- Copies `Index/tags.json` to `.index/tags.json`
- Validates JSON integrity before removing old files
- Removes empty `Index/` directories

## AI-Powered Features

### Tag System

pplr uses Claude AI to analyse person profiles and meeting content to generate searchable tags:

**Profile Tags** (from About files):
- Professional roles: `cto`, `founder`, `engineer`
- Industries: `fintech`, `healthcare`, `ai`
- Skills: `machine-learning`, `product-management`
- Company types: `startup`, `enterprise`

**Meeting Tags** (from Meeting files):
- Topics: `partnerships`, `funding`, `product-development`
- Meeting types: `intro-meeting`, `follow-up`
- Technologies: `kubernetes`, `blockchain`
- Outcomes: `investment`, `collaboration`

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

Role: Chief Technology Officer
Company: Tech Innovations Ltd
LinkedIn: https://linkedin.com/in/johnsmith
Email: john.smith@example.com
Phone: +1-555-0123

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

### Tags File (JSON)
```json
{
  "profile_tags": ["cto", "technology", "startup", "cloud-expert"],
  "meeting_tags": ["strategy", "cloud-migration", "hiring"],
  "generated_at": "2024-03-15T10:30:00Z",
  "version": "1.0"
}
```

## Best Practices

1. **Consistent Naming**: Always use "Surname, Firstname" format
2. **Regular Updates**: Keep About files current with role changes
3. **Meeting Notes**: Include attendees, topics, and action items
4. **Tag Generation**: Run `pplr tag --all` periodically for best search results
5. **Backups**: Use Dropbox or similar for automatic backups

## Requirements

- **Operating System**: macOS (primary), Linux (partial support)
- **Dependencies**: 
  - bash 4.0+
  - jq (for JSON processing)
  - Claude CLI (for AI features)
- **Optional**: Dropbox for sync

## Troubleshooting

### Search returns no results
- Run `pplr reindex` to rebuild indexes
- Ensure tags exist: `pplr tag "Name" -s`
- Generate tags if needed: `pplr tag --all`
- Check if `.index/tags_index.json` exists and is recent

### Claude/AI features not working
- Ensure Claude CLI is installed and in PATH
- Check Claude is accessible: `which claude` and `echo "test" | claude`
- If using a mock for testing, ensure `PPLR_TEST_DATA` is not set in production

### Permission errors
- Check file permissions in PPLR_DIR
- Ensure scripts are executable: `chmod +x $PPLR_BIN_DIR/pplr_*`

## Recent Changes (July 2025)

### Directory Structure Update
Individual tag files have moved from `Index/tags.json` to `.index/tags.json` for better organization:
- Old: `People/S/Smith, John/Index/tags.json`
- New: `People/S/Smith, John/.index/tags.json`

**Migration**: Run `pplr migrate_tags` to automatically migrate existing tag files.

### Enhanced Features
- **Natural Language Search**: Now powered by Claude AI by default (no --tags flag needed)
- **Smart Tag Regeneration**: `pplr reindex --tags --stale-only` only updates old/missing tags
- **Improved Performance**: Optimized search context reduces API calls
- **Better Error Handling**: Graceful fallback when Claude is unavailable

## Contributing

pplr is a personal project, but suggestions and improvements are welcome. The codebase is simple bash scripts designed for maintainability and extensibility.

## License

This is a personal tool shared as-is for educational purposes.
