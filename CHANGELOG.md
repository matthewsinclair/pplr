# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Smart tag regeneration with `--stale-only` flag for `pplr reindex`
- Configurable staleness threshold with `--max-age` option (e.g., 30d, 7days, 2weeks)
- Migration tool `pplr_migrate_tags` for moving tag files to new location
- Enhanced test coverage with proper mock Claude handling
- Comprehensive documentation updates

### Changed
- **BREAKING**: Individual tag files moved from `Index/tags.json` to `.index/tags.json`
- Natural language search now default behavior (removed `--tags` flag requirement)
- Search results now include clickable markdown links to About files
- Improved mock Claude detection to prevent false positives in production
- Test infrastructure uses absolute paths for better reliability

### Fixed
- Fixed all failing tests (31/31 now passing)
- Fixed mock Claude detection bug that prevented tag generation
- Fixed test helper path resolution issues
- Fixed search fallback behavior when Claude is unavailable
- Fixed JSON validation in tag generation tests

### Performance
- Optimized search context file (`tags_index.json`) to ~116KB for faster Claude queries
- Smart tag regeneration reduces unnecessary API calls
- Improved search response times with better fallback handling

## [1.0.0] - 2025-07-18

### Added
- Initial release of pplr (Personal Relationship Manager)
- Core commands: new, search, open, meetings, edit
- AI-powered tagging system using Claude
- Tag-based search functionality
- Comprehensive help system with --details flag
- BATS test suite
- Separated code and data directories
- Environment variable configuration

### Changed
- Renamed `pplr_search` to `pplr_grep` for simple text search
- New `pplr_search` now supports natural language queries
- Updated directory structure to separate code from data

### Fixed
- Fixed hardcoded paths in scripts
- Improved error handling in tag generation