#!/usr/bin/env bats

load test_helper

@test "pplr search finds people by name" {
    create_test_person "Anderson" "James"
    create_test_person "Anderson" "Sarah"
    create_test_tags_index
    
    run "$PPLR_BIN_DIR/pplr" search "Anderson"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Anderson, James"
    assert_contains "$output" "Anderson, Sarah"
}

@test "pplr search finds people by role" {
    local person_dir=$(create_test_person "Tech" "Tom")
    # Update the About file to include CTO role
    sed -i '' 's/Role: Test Role/Role: Chief Technology Officer/' "$person_dir/About/Tom Tech (About).md"
    create_test_tags_index
    
    run "$PPLR_BIN_DIR/pplr" search "technology"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tech, Tom"
}

@test "pplr search with no results" {
    create_test_person "Smith" "John"
    create_test_tags_index
    
    run "$PPLR_BIN_DIR/pplr" search "nonexistent"
    [ "$status" -eq 0 ]
    assert_contains "$output" "No matches found"
}

@test "pplr search falls back without Claude" {
    create_test_person "Tagged" "Terry"
    # Replace mock Claude with one that fails to trigger fallback
    cat > "$PPLR_TEST_DATA/.mock_bin/claude" << 'EOF'
#!/bin/bash
# Failing mock Claude to test fallback
exit 1
EOF
    chmod +x "$PPLR_TEST_DATA/.mock_bin/claude"
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    "$PPLR_BIN_DIR/pplr" reindex >/dev/null

    run "$PPLR_BIN_DIR/pplr" search "test"
    [ "$status" -eq 0 ]  # Should succeed with fallback search
    assert_contains "$output" "Falling back to basic tag search"
}

@test "pplr search uses proper markdown URL format" {
    create_test_person "Anderson" "URL"
    create_test_tags_index
    
    run "$PPLR_BIN_DIR/pplr" search "Anderson"
    [ "$status" -eq 0 ]
    # Check for angle brackets and escaped parentheses in the output
    assert_contains "$output" "](<"
    assert_contains "$output" "\\(About\\).md>)"
}

@test "pplr grep performs simple text search" {
    create_test_person "Grep" "Gary"
    
    run "$PPLR_BIN_DIR/pplr" grep "Test Company"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Grep, Gary"
}

@test "pplr grep uses proper markdown URL format" {
    create_test_person "Format" "Test"

    run "$PPLR_BIN_DIR/pplr" grep "Test Company"
    [ "$status" -eq 0 ]
    # An angle-bracket link to the About file, named First Surname
    assert_contains "$output" "](<"
    assert_contains "$output" "Test Format (About).md>)"
}

@test "pplr grep -t searches each About/tags.yaml, and shows role and company from the About" {
    local person_dir=$(create_test_person "Tagged" "Terry")
    printf '%s\n' 'tags:' '  - engineer' '  - {tag: python, source: auto}' > "$person_dir/About/tags.yaml"

    run "$PPLR_BIN_DIR/pplr" grep -t "python"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tagged, Terry"
    assert_contains "$output" "Test Role"
    assert_contains "$output" "at Test Company"
}

@test "pplr grep --tag searches in tag files" {
    local person_dir=$(create_test_person "Tagged" "Terry")
    printf '%s\n' 'tags:' '  - product' '  - agile' > "$person_dir/About/tags.yaml"

    run "$PPLR_BIN_DIR/pplr" grep --tag "agile"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tagged, Terry"
}
