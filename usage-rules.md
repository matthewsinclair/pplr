# pplr Usage Rules

Concise guidelines for using the pplr personal relationship management system effectively.

## Core Principles

- **Consistent naming**: Always use "Surname, Firstname" format
- **Directory structure**: Respect the alphabetical organisation (A-Z folders)
- **File naming**: Follow established patterns (e.g., "Name (About).md", "YYYYMMDD Meeting Title")
- **No direct database manipulation**: Use pplr commands, don't manually edit index files

## Command Usage

### Creating People
```bash
# ALWAYS provide firstname and surname separately
pplr new "John" "Smith"  # ✓ Correct
pplr new "John Smith"    # ✗ Wrong

# LinkedIn URL is optional but recommended
pplr new "John" "Smith" "https://linkedin.com/in/johnsmith"
```

### Searching
```bash
# Use natural language for intelligent search (default)
pplr search "people in fintech"         # AI-powered search
pplr search "founders in healthcare"    # Natural language queries

# Use grep for literal text matching
pplr grep "exact phrase"                # Simple text search
```

### Tag Generation
```bash
# Generate tags before using search
pplr tag "John" "Smith"     # One person
pplr tag "Smith" -g         # All Smiths
pplr tag --all              # Everyone (slow)

# Tags are stored in .index/tags.json - don't edit manually
```

## File Formats

### About Files
- Must include Role and Company fields
- Use verblock for version tracking
- Keep bio concise and professional
- Store in About/ subdirectory

### Meeting Files
- Name: "YYYYMMDD Meeting Title"
- Include attendee links: [[Surname, Firstname]]
- Add summary, key takeaways, action items
- Store in Meetings/YYYYMMDD Meeting Title/ directory

### Tags (Auto-generated)
- Located in .index/tags.json per person
- Two categories: profile_tags and meeting_tags
- Use lowercase with hyphens (e.g., "machine-learning")
- Regenerate with `pplr tag` commands

## Best Practices

### Organisation
- One person per directory
- Use About/ for all profile-related files
- Use Meetings/ for all meeting records
- Use Client/ for client-specific information

### Maintenance
- Run `pplr reindex` after bulk changes
- Use `pplr reindex --tags` to rebuild everything
- Use `pplr reindex --tags --stale-only` for smart regeneration
- Use `pplr reindex --tags --stale-only --max-age=7d` for custom staleness
- Keep About files updated with role changes
- Archive old meetings but don't delete

### Search Optimisation
- Generate tags for better search: `pplr tag --all`
- Natural language search is now default (no --tags needed)
- Use specific search terms for best results
- Search automatically uses AI-powered analysis

### Utility Commands
```bash
# System information
pplr count                              # Total number of people
pplr index                              # Generate markdown index
pplr json                               # Generate JSON index

# File management
pplr cp "John" "Smith"                  # Copy person's path to clipboard
pplr edit "John" "Smith"                # Edit About file in $EDITOR
```

## Common Patterns

### Finding People
```bash
# By role or company
pplr search "cto at startup"
pplr search "people from Google"

# By meeting history
pplr search "met last month"
pplr meetings 2024-01-01 2024-12-31

# By industry
pplr search "fintech"
pplr search "healthcare investors"
```

### Managing Relationships
```bash
# Quick access
pplr open "John" "Smith"                # Open About file
pplr open -t linkedin "John" "Smith"    # Open LinkedIn directly
pplr open -t profile "John" "Smith"     # Open PDF profile
pplr linkedin "John" "Smith"            # Shortcut for LinkedIn

# View information
pplr about "John" "Smith"               # Display About file in terminal
pplr tag "John" "Smith" -s              # Show existing tags

# Track interactions
pplr meetings                           # Recent meetings
pplr meetings 2024-01-01 2024-12-31     # Meetings in date range
pplr search "action items for me"       # Search meeting content
```

## Anti-patterns to Avoid

- Don't create people without About files
- Don't edit .index/ files (index.json, index.md, tags_index.json) manually
- Don't mix firstname/surname order
- Don't store sensitive data (passwords, keys)
- Don't use special characters in names
- Don't delete .index/ directories

## Integration Notes

### For AI/LLM Tools
- Tags provide semantic categorisation
- Meeting files contain temporal context
- About files have structured professional data
- All content is in plain text (Markdown/JSON)

### For Scripts/Automation
- All commands return standard exit codes
- JSON output available via `pplr_json`
- Paths are consistent and predictable
- File formats are version-controlled