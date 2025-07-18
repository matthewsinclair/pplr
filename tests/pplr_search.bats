#!/usr/bin/env bats

load test_helper

@test "pplr search finds people by name" {
    create_test_person "Anderson" "James"
    create_test_person "Anderson" "Sarah"
    
    run "$PPLR_BIN_DIR/pplr" search "Anderson"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Anderson, James"
    assert_contains "$output" "Anderson, Sarah"
}

@test "pplr search finds people by role" {
    local person_dir=$(create_test_person "Tech" "Tom")
    # Update the About file to include CTO role
    sed -i '' 's/Role: Test Role/Role: Chief Technology Officer/' "$person_dir/About/Tom Tech (About).md"
    
    run "$PPLR_BIN_DIR/pplr" search "technology"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tech, Tom"
}

@test "pplr search with no results" {
    create_test_person "Smith" "John"
    
    run "$PPLR_BIN_DIR/pplr" search "nonexistent"
    [ "$status" -eq 0 ]
    assert_contains "$output" "No matches found"
}

@test "pplr search --tags requires tag files" {
    create_test_person "Tagged" "Terry"
    
    run "$PPLR_BIN_DIR/pplr" search "test" --tags
    [ "$status" -eq 0 ]
    assert_contains "$output" "No matches found in tags"
}

@test "pplr search --tags finds tagged people" {
    local person_dir=$(create_test_person "Tagged" "Tina")
    mkdir -p "$person_dir/Index"
    
    # Create a tag file
    cat > "$person_dir/Index/tags.json" << EOF
{
  "profile_tags": ["technology", "startup", "founder"],
  "meeting_tags": ["funding", "pitch"],
  "generated_at": "2024-01-01T00:00:00Z",
  "version": "1.0"
}
EOF
    
    run "$PPLR_BIN_DIR/pplr" search "startup" --tags
    [ "$status" -eq 0 ]
    assert_contains "$output" "Tagged, Tina"
    assert_contains "$output" "startup"
}

@test "pplr grep performs simple text search" {
    create_test_person "Grep" "Gary"
    
    run "$PPLR_BIN_DIR/pplr" grep "Test Company"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Grep, Gary"
}