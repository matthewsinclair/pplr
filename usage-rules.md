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
# ALWAYS provide surname and firstname separately
pplr new "Smith" "John"  # ✓ Correct
pplr new "John Smith"    # ✗ Wrong

# LinkedIn URL is optional but recommended
pplr new "Smith" "John" "https://linkedin.com/in/johnsmith"
```

### Searching
```bash
# Use natural language for intelligent search
pplr search "people in fintech"         # Searches full text
pplr search "founder" --tags            # Fast tag search

# Use grep for literal text matching
pplr grep "exact phrase"
```

### Tag Generation
```bash
# Generate tags before using tag search
pplr tag "Smith" "John"     # One person
pplr tag "Smith" -g         # All Smiths
pplr tag --all              # Everyone (slow)

# Tags are stored in Index/tags.json - don't edit manually
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
- Located in Index/tags.json
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
- Keep About files updated with role changes
- Archive old meetings but don't delete

### Search Optimisation
- Generate tags for better search: `pplr tag --all`
- Use specific search terms
- Try tag search first (--tags) for speed
- Fall back to full-text search for complex queries

## Common Patterns

### Finding People
```bash
# By role or company
pplr search "cto at startup" --tags
pplr search "people from Google"

# By meeting history
pplr search "met last month"
pplr meetings 2024-01-01 2024-12-31

# By industry
pplr search "fintech" --tags
pplr search "healthcare investors"
```

### Managing Relationships
```bash
# Quick access
pplr open "Smith" "John"
pplr linkedin "Smith" "John"

# Track interactions
pplr meetings    # Recent meetings
pplr search "action items for me"
```

## Anti-patterns to Avoid

- Don't create people without About files
- Don't edit .index/ files (index.json, index.md, tags_index.json) manually
- Don't mix firstname/surname order
- Don't store sensitive data (passwords, keys)
- Don't use special characters in names
- Don't delete Index/ directories

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