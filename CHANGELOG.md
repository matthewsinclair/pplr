# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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