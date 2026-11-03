#!/usr/bin/env bash

# Common test utilities for pplr tests.
#
# Every test runs in its own sandbox under $BATS_TEST_TMPDIR, which bats
# creates for the test and removes after it. Nothing deletes outside that
# sandbox, and no path taken from the environment is ever a deletion target.
#
# PPLR_UNDER_TEST names the pplr executable the suite tests, and defaults to
# the checkout's dev build, target/debug/pplr, which bin/devbin test shell builds
# first. Point it at another build to test that, eg an opt tree's:
#   PPLR_UNDER_TEST="$PPLR_OPT_PREFIX/bin/pplr" bats -r tests/

# The checkout these tests belong to
PPLR_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

setup() {
  [ -n "$BATS_TEST_TMPDIR" ] && [ -d "$BATS_TEST_TMPDIR" ] || {
    echo "test_helper: BATS_TEST_TMPDIR is not a directory; bats 1.4 or later is required" >&2
    return 1
  }

  # The implementation under test, as an absolute path
  local under_test="${PPLR_UNDER_TEST:-$PPLR_ROOT/target/debug/pplr}"
  [ -x "$under_test" ] && [ -f "$under_test" ] || {
    echo "test_helper: PPLR_UNDER_TEST is not an executable file: $under_test" >&2
    [ -n "${PPLR_UNDER_TEST:-}" ] ||
      echo "test_helper: that is the default, the dev build: cargo build makes it" >&2
    return 1
  }
  PPLR="$(cd "$(dirname "$under_test")" && pwd)/$(basename "$under_test")"

  # Nothing pplr reads from the caller's shell leaks into a test: a stray
  # PPLR_DATA would reach the real tree
  local var
  for var in $(compgen -v PPLR_); do
    [ "$var" = PPLR_UNDER_TEST ] || [ "$var" = PPLR_ROOT ] || unset "$var"
  done
  export -n PPLR_ROOT

  local sandbox="$BATS_TEST_TMPDIR"
  export HOME="$sandbox/home"
  export XDG_DATA_HOME="$HOME/.local/share"
  export XDG_CACHE_HOME="$HOME/.cache"
  export XDG_CONFIG_HOME="$HOME/.config"
  export PPLR_TEST_DATA="$sandbox/data"
  export PPLR_DATA="$PPLR_TEST_DATA"
  export PPLR_DIR="$PPLR_DATA"
  export PPLR_CACHE_DIR="$sandbox/cache"
  export PPLR_BACKUP_DIR="$sandbox/backups"
  export PPLR_REFS_DIR="$sandbox/refs"
  PPLR_TEST_BIN="$sandbox/bin"

  mkdir -p "$HOME" "$PPLR_CACHE_DIR" "$PPLR_TEST_BIN" "$PPLR_TEST_DATA/.index"
  # new writes the git author into a verblock (ST0001 design D10): the sandbox has its own
  printf '[user]\n\tname = Test Author\n\temail = test@example.invalid\n' > "$HOME/.gitconfig"
  local letter
  for letter in {A..Z}; do
    mkdir -p "$PPLR_TEST_DATA/$letter"
  done

  setup_mock_claude
}

# Helper function to create a test person
create_test_person() {
  local surname="$1"
  local firstname="$2"
  local first_letter=$(echo "$surname" | cut -c1 | tr '[:lower:]' '[:upper:]')
  local person_dir="$PPLR_TEST_DATA/$first_letter/$surname, $firstname"

  mkdir -p "$person_dir/About"
  mkdir -p "$person_dir/Meetings"
  mkdir -p "$person_dir/Client"

  # Create a basic About file
  cat > "$person_dir/About/$firstname $surname (About).md" << EOF
verblock(1.0.0)

# $firstname $surname (About)

- Role:     Test Role
- Company:  Test Company
- LinkedIn: https://linkedin.com/in/test
- Email:    test@example.com
- Phone:    +1-555-0123

## Bio
This is a test bio for $firstname $surname.
EOF

  echo "$person_dir"
}


# Put the mock claude first on PATH
setup_mock_claude() {
  cat > "$PPLR_TEST_BIN/claude" << 'EOF'
#!/bin/bash
# Mock Claude for testing
# Read the input and generate a simple response based on query

input=$(cat)
query=$(echo "$input" | grep 'SEARCH QUERY:' | sed 's/.*SEARCH QUERY: "\(.*\)".*/\1/')

# Simple mock responses based on query
case "$query" in
    *"Anderson"*)
        cat << 'RESPONSE'
{
  "results": [
    {
      "name": "Anderson, James",
      "path": "A/Anderson, James",
      "role": "Test Role",
      "company": "Test Company",
      "relevance_score": 0.9,
      "explanation": "Name matches Anderson"
    },
    {
      "name": "Anderson, Sarah", 
      "path": "A/Anderson, Sarah",
      "role": "Test Role",
      "company": "Test Company", 
      "relevance_score": 0.85,
      "explanation": "Name matches Anderson"
    }
  ],
  "total_matches": 2
}
RESPONSE
        ;;
    *"technology"*)
        cat << 'RESPONSE'
{
  "results": [
    {
      "name": "Tech, Tom",
      "path": "T/Tech, Tom", 
      "role": "Chief Technology Officer",
      "company": "Test Company",
      "relevance_score": 0.95,
      "explanation": "Role matches technology query"
    }
  ],
  "total_matches": 1
}
RESPONSE
        ;;
    *"nonexistent"*)
        cat << 'RESPONSE'
{
  "results": [],
  "total_matches": 0
}
RESPONSE
        ;;
    *)
        cat << 'RESPONSE'
{
  "results": [],
  "total_matches": 0
}
RESPONSE
        ;;
esac
EOF
  chmod +x "$PPLR_TEST_BIN/claude"
  export PATH="$PPLR_TEST_BIN:$PATH"
}

# Helper function to create a basic tags index for testing
create_test_tags_index() {
  local tags_index="$PPLR_TEST_DATA/.index/tags_index.json"

  cat > "$tags_index" << 'EOF'
{
  "generated_at": "2024-01-01T00:00:00Z",
  "people_count": 0,
  "people": []
}
EOF
}


# Replace the mock claude with one that also answers tag generation
mock_claude() {
  cat > "$PPLR_TEST_BIN/claude" << 'EOF'
#!/bin/bash
# Enhanced mock Claude for testing tags and search

input=$(cat)

# Check if this is a tag generation request
if echo "$input" | grep -qi "generate.*tags.*based.*content"; then
    # Tag generation response
    cat << 'RESPONSE'
{
  "profile_tags": ["test", "mock", "person"],
  "meeting_tags": ["test-meeting", "mock-meeting"],
  "generated_at": "2024-01-01T00:00:00Z",
  "version": "1.0"
}
RESPONSE
else
    # Search query handling
    query=$(echo "$input" | grep 'SEARCH QUERY:' | sed 's/.*SEARCH QUERY: "\(.*\)".*/\1/')
    
    # Simple mock responses based on query
    case "$query" in
        *"Anderson"*)
            cat << 'RESPONSE'
{
  "results": [
    {
      "name": "Anderson, James",
      "path": "A/Anderson, James",
      "role": "Test Role",
      "company": "Test Company",
      "relevance_score": 0.9,
      "explanation": "Name matches Anderson"
    }
  ],
  "total_matches": 1
}
RESPONSE
            ;;
        *)
            cat << 'RESPONSE'
{
  "results": [],
  "total_matches": 0
}
RESPONSE
            ;;
    esac
fi
EOF
  chmod +x "$PPLR_TEST_BIN/claude"
}

remove_mock_claude() {
  # The mock lives in the test's own sandbox, which bats removes
  true
}

# Helper function to create a test meeting
create_test_meeting() {
  local person_dir="$1"
  local date="$2"
  local title="$3"

  local meeting_dir="$person_dir/Meetings/$date $title"
  mkdir -p "$meeting_dir"

  cat > "$meeting_dir/$date $title.md" << EOF
verblock(1.0.0)

# $date $title

## Meeting Summary
Purpose: Test meeting
Date: $date
Attendees: Test Person

## Key Takeaways
- Test takeaway

## Action Items
- [ ] Test action
EOF
}

# Helper function to assert file exists
assert_file_exists() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo "File does not exist: $file" >&2
    return 1
  fi
}

# Helper function to assert directory exists
assert_dir_exists() {
  local dir="$1"
  if [ ! -d "$dir" ]; then
    echo "Directory does not exist: $dir" >&2
    return 1
  fi
}

# Helper function to assert string contains
assert_contains() {
  local haystack="$1"
  local needle="$2"
  if [[ ! "$haystack" =~ "$needle" ]]; then
    echo "String does not contain expected value" >&2
    echo "Expected to find: $needle" >&2
    echo "In: $haystack" >&2
    return 1
  fi
}
