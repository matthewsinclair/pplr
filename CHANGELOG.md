# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1] - 2025-07-19

### Added
- `pplr version` command to display current version
- `-v` and `--version` flags to main pplr command for quick version checking
- `-t|--tag` option to `pplr grep` for searching within tag files

### Changed
- Updated `pplr grep` output format to match `pplr search` format with person links and role/company info
- Improved markdown URL formatting in `pplr search` and `pplr grep` to use angle brackets and escape parentheses

### Fixed
- Markdown links now properly render in all markdown viewers by escaping special characters
- Test suite updated to dynamically read version from VERSION.md instead of hardcoding

## [1.0.0] - 2025-07-19

### Added
- Initial release of pplr (Personal Relationship Manager)
- Core commands: new, search, open, meetings, edit, tag, reindex, count, cp, linkedin, about, index
- AI-powered tagging system using Claude
- Natural language search functionality powered by Claude AI
- Smart tag regeneration with `--stale-only` flag for `pplr reindex`
- Configurable staleness threshold with `--max-age` option (e.g., 30d, 7days, 2weeks)
- Comprehensive help system with --details flag
- BATS test suite with 31 tests
- GitHub Actions CI/CD for automated testing
- Separated code and data directories
- Environment variable configuration
- Search results with clickable markdown links to About files

### Architecture
- Individual tag files stored in `.index/tags.json` for better organization
- Optimized search context file (`tags_index.json`) ~116KB for efficient Claude queries
- Test infrastructure uses absolute paths for better reliability
- Improved mock Claude detection to prevent false positives in production

### Fixed
- All parameters now use consistent firstname-surname order
- Mock Claude detection bug that prevented tag generation
- Test helper path resolution issues
- Search fallback behavior when Claude is unavailable
- Spinner termination message ("Terminated: 15") in search output

### Performance
- Smart tag regeneration reduces unnecessary API calls
- Improved search response times with better fallback handling
- Optimized directory structure for faster file access