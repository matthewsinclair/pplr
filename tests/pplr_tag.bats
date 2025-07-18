#!/usr/bin/env bats

load test_helper

setup() {
    # Call parent setup
    load test_helper
    setup
    
    # Set up mock claude for tag tests
    mock_claude
}

teardown() {
    # Remove mock claude
    remove_mock_claude
    
    # Call parent teardown
    load test_helper
    teardown
}

@test "pplr tag generates tags for one person" {
    create_test_person "Tagger" "Tim"
    
    run "$PPLR_BIN_DIR/pplr" tag "Tagger" "Tim"
    [ "$status" -eq 0 ]
    
    assert_file_exists "$PPLR_TEST_DATA/T/Tagger, Tim/Index/tags.json"
    assert_contains "$output" "Tags saved"
}

@test "pplr tag shows existing tags" {
    local person_dir=$(create_test_person "Tagged" "Teresa")
    mkdir -p "$person_dir/Index"
    
    # Create existing tags
    cat > "$person_dir/Index/tags.json" << EOF
{
  "profile_tags": ["existing", "tags"],
  "meeting_tags": ["meeting-tag"],
  "generated_at": "2024-01-01T00:00:00Z",
  "version": "1.0"
}
EOF
    
    run "$PPLR_BIN_DIR/pplr" tag "Tagged" "Teresa" -s
    [ "$status" -eq 0 ]
    assert_contains "$output" "existing"
    assert_contains "$output" "meeting-tag"
}

@test "pplr tag handles non-existent person" {
    run "$PPLR_BIN_DIR/pplr" tag "Nobody" "Exists"
    [ "$status" -ne 0 ]
    assert_contains "$output" "not found"
}

@test "pplr tag -g generates for matching people" {
    create_test_person "Smith" "John"
    create_test_person "Smith" "Jane"
    create_test_person "Jones" "Bob"
    
    run "$PPLR_BIN_DIR/pplr" tag "Smith" -g
    [ "$status" -eq 0 ]
    
    assert_file_exists "$PPLR_TEST_DATA/S/Smith, John/Index/tags.json"
    assert_file_exists "$PPLR_TEST_DATA/S/Smith, Jane/Index/tags.json"
    # Jones should not have tags
    [ ! -f "$PPLR_TEST_DATA/J/Jones, Bob/Index/tags.json" ]
}

@test "pplr tag creates valid JSON" {
    create_test_person "Json" "Jerry"
    
    run "$PPLR_BIN_DIR/pplr" tag "Json" "Jerry"
    [ "$status" -eq 0 ]
    
    local tags_file="$PPLR_TEST_DATA/J/Json, Jerry/Index/tags.json"
    
    # Verify it's valid JSON
    run jq . "$tags_file"
    [ "$status" -eq 0 ]
    
    # Verify structure
    run jq -r '.profile_tags | type' "$tags_file"
    [ "$output" = "array" ]
    
    run jq -r '.meeting_tags | type' "$tags_file"
    [ "$output" = "array" ]
}