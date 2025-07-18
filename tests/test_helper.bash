#!/usr/bin/env bash

# Common test utilities for pplr tests

# Setup function - runs before each test
setup() {
    # Create a clean test data directory
    export PPLR_TEST_DATA="${PPLR_TEST_DATA:-$BATS_TEST_DIRNAME/fixtures}"
    mkdir -p "$PPLR_TEST_DATA"
    
    # Create A-Z directories
    for letter in {A..Z}; do
        mkdir -p "$PPLR_TEST_DATA/$letter"
    done
    
    # Create .index directory for search files
    mkdir -p "$PPLR_TEST_DATA/.index"
    
    # Set up environment
    export PPLR_DATA="$PPLR_TEST_DATA"
    export PPLR_DIR="$PPLR_DATA"
    export PPLR_ROOT="$(dirname "$BATS_TEST_DIRNAME")"
    export PPLR_BIN_DIR="$PPLR_ROOT/bin"
    export PPLR_TEMPLATE_DIR="$PPLR_ROOT/templates"
    
    # Create mock Claude command for testing
    setup_mock_claude
}

# Teardown function - runs after each test
teardown() {
    # Clean up test data
    if [ -d "$PPLR_TEST_DATA" ]; then
        rm -rf "$PPLR_TEST_DATA"/*
    fi
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

Role: Test Role
Company: Test Company
LinkedIn: https://linkedin.com/in/test
Email: test@example.com
Phone: +1-555-0123

## Bio
This is a test bio for $firstname $surname.
EOF
    
    echo "$person_dir"
}

# Helper function to set up mock Claude for testing
setup_mock_claude() {
    local mock_claude_dir="$PPLR_TEST_DATA/.mock_bin"
    mkdir -p "$mock_claude_dir"
    
    # Create mock claude command
    cat > "$mock_claude_dir/claude" << 'EOF'
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
    
    chmod +x "$mock_claude_dir/claude"
    export PATH="$mock_claude_dir:$PATH"
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

# Mock Claude functions for tag tests
mock_claude() {
    # Already set up in setup_mock_claude, but we need a more sophisticated version for tags
    local mock_claude_dir="$PPLR_TEST_DATA/.mock_bin"
    
    # Update mock claude to handle tag generation
    cat > "$mock_claude_dir/claude" << 'EOF'
#!/bin/bash
# Enhanced mock Claude for testing tags and search

input=$(cat)

# Check if this is a tag generation request
if echo "$input" | grep -qi "generate.*tags.*based.*content"; then
    # Tag generation response
    cat << 'RESPONSE'
{
  "profile_tags": ["test-role", "technology", "startup"],
  "meeting_tags": ["intro-meeting", "collaboration"],
  "generated_at": "2024-01-01T00:00:00Z", 
  "version": "1.0"
}
RESPONSE
elif echo "$input" | grep -q "SEARCH QUERY:"; then
    # Search request - use the existing search logic
    query=$(echo "$input" | grep 'SEARCH QUERY:' | sed 's/.*SEARCH QUERY: "\(.*\)".*/\1/')
    
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
else
    # Default response
    echo '{"error": "Unknown request type"}'
fi
EOF
    
    chmod +x "$mock_claude_dir/claude"
}

remove_mock_claude() {
    # Mock claude is cleaned up in teardown automatically
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

# Mock Claude for testing (creates a simple mock response)
mock_claude() {
    # Create a mock claude script
    mkdir -p "$HOME/.claude/local"
    cat > "$HOME/.claude/local/claude" << 'EOF'
#!/bin/bash
# Mock claude for testing
cat << 'RESPONSE'
{
  "profile_tags": ["test", "mock", "person"],
  "meeting_tags": ["test-meeting", "mock-meeting"],
  "generated_at": "2024-01-01T00:00:00Z",
  "version": "1.0"
}
RESPONSE
EOF
    chmod +x "$HOME/.claude/local/claude"
}

# Remove mock claude
remove_mock_claude() {
    rm -f "$HOME/.claude/local/claude"
}