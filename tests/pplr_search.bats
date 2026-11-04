#!/usr/bin/env bats

load test_helper
bats_require_minimum_version 1.5.0

@test "pplr search finds people by name" {
    create_test_person "Anderson" "James"
    create_test_person "Anderson" "Sarah"
    create_test_tags_index
    
    run "$PPLR" search "Anderson"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Anderson, James"
    assert_contains "$output" "Anderson, Sarah"
}

@test "pplr search finds people by role" {
    local person_dir=$(create_test_person "Tech" "Tom")
    # Update the About file to include CTO role (no `sed -i`: BSD and GNU differ)
    local about="$person_dir/About/Tom Tech (About).md"
    sed 's/Role: Test Role/Role: Chief Technology Officer/' "$about" > "$about.new" && mv "$about.new" "$about"
    create_test_tags_index
    
    run "$PPLR" search "technology"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tech, Tom"
}

@test "pplr search with no results" {
    create_test_person "Smith" "John"
    create_test_tags_index
    
    run "$PPLR" search "nonexistent"
    [ "$status" -eq 0 ]
    assert_contains "$output" "No matches found"
}

@test "pplr search falls back without Claude" {
    create_test_person "Tagged" "Terry"
    # Replace mock Claude with one that fails to trigger fallback
    cat > "$PPLR_TEST_BIN/claude" << 'EOF'
#!/bin/bash
# Failing mock Claude to test fallback
exit 1
EOF
    chmod +x "$PPLR_TEST_BIN/claude"
    "$PPLR" reindex >/dev/null

    run "$PPLR" search "test"
    [ "$status" -eq 0 ]  # Should succeed with fallback search
    assert_contains "$output" "Falling back to basic tag search"
}

@test "pplr search uses proper markdown URL format" {
    create_test_person "Anderson" "URL"
    create_test_tags_index
    
    run "$PPLR" search "Anderson"
    [ "$status" -eq 0 ]
    # Check for angle brackets and escaped parentheses in the output
    assert_contains "$output" "](<"
    assert_contains "$output" "\\(About\\).md>)"
}

@test "pplr grep performs simple text search" {
    create_test_person "Grep" "Gary"
    
    run "$PPLR" grep "Test Company"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Grep, Gary"
}

# AT-04.2 (ST0003 AC-04.1): a folder a walk cannot read is named on stderr,
# one line, and the walk carries on, its exit code unchanged (CHANGELOG
# 2.0.1). uid 0 reads a folder with no permissions, so the test skips there.
@test "pplr grep names a folder it cannot read on stderr, and still searches the rest" {
    [ "$(id -u)" -ne 0 ] || skip "uid 0 reads a folder with no permissions, so none is unreadable"
    create_test_person "Grep" "Gary" >/dev/null
    local shut="$PPLR_TEST_DATA/G/Shut"
    mkdir "$shut"
    chmod 000 "$shut"
    run --separate-stderr "$PPLR" grep "Test Company"
    chmod 755 "$shut"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Grep, Gary"
    [ "$stderr" = "pplr: cannot read $shut: Permission denied (os error 13)" ]
}

@test "pplr grep uses proper markdown URL format" {
    create_test_person "Format" "Test"

    run "$PPLR" grep "Test Company"
    [ "$status" -eq 0 ]
    # An angle-bracket link to the About file, named First Surname
    assert_contains "$output" "](<"
    assert_contains "$output" "Test Format (About).md>)"
}

@test "pplr grep -t searches each About/tags.yaml, and shows role and company from the About" {
    local person_dir=$(create_test_person "Tagged" "Terry")
    printf '%s\n' 'tags:' '  - engineer' '  - {tag: python, source: auto}' > "$person_dir/About/tags.yaml" # nopython: a tag's name

    run "$PPLR" grep -t "python" # nopython: the tag's name
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tagged, Terry"
    assert_contains "$output" "Test Role"
    assert_contains "$output" "at Test Company"
}

@test "pplr grep --tag searches in tag files" {
    local person_dir=$(create_test_person "Tagged" "Terry")
    printf '%s\n' 'tags:' '  - product' '  - agile' > "$person_dir/About/tags.yaml"

    run "$PPLR" grep --tag "agile"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tagged, Terry"
}
