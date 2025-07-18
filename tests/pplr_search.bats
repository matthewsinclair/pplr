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
    # Don't create tags index - remove mock Claude to trigger fallback
    rm -f "$PPLR_TEST_DATA/.mock_bin/claude"
    
    run "$PPLR_BIN_DIR/pplr" search "test"
    [ "$status" -eq 0 ]  # Should succeed with fallback search
    assert_contains "$output" "Claude CLI not found"
    assert_contains "$output" "Falling back to basic tag search"
}

@test "pplr grep performs simple text search" {
    create_test_person "Grep" "Gary"
    
    run "$PPLR_BIN_DIR/pplr" grep "Test Company"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Grep, Gary"
}