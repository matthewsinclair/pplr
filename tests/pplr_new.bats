#!/usr/bin/env bats

load test_helper

@test "pplr new creates person with valid names" {
    run "$PPLR_BIN_DIR/pplr" new "John" "Smith"
    [ "$status" -eq 0 ]
    
    assert_dir_exists "$PPLR_TEST_DATA/S/Smith, John"
    assert_file_exists "$PPLR_TEST_DATA/S/Smith, John/About/John Smith (About).md"
}

@test "pplr new creates correct directory structure" {
    run "$PPLR_BIN_DIR/pplr" new "Mary" "Johnson"
    [ "$status" -eq 0 ]
    
    local person_dir="$PPLR_TEST_DATA/J/Johnson, Mary"
    assert_dir_exists "$person_dir"
    assert_dir_exists "$person_dir/About"
    assert_dir_exists "$person_dir/Meetings"
    assert_dir_exists "$person_dir/Client"
}

@test "pplr new with LinkedIn URL" {
    run "$PPLR_BIN_DIR/pplr" new "Alice" "Brown" "https://linkedin.com/in/alicebrown"
    [ "$status" -eq 0 ]
    
    assert_file_exists "$PPLR_TEST_DATA/B/Brown, Alice/About/Alice Brown (LinkedIn).webloc"
}

@test "pplr new fails without enough arguments" {
    run "$PPLR_BIN_DIR/pplr" new "Smith"
    [ "$status" -ne 0 ]
    assert_contains "$output" "Usage:"
}

@test "pplr new handles names with spaces" {
    run "$PPLR_BIN_DIR/pplr" new "Jan" "Van Der Berg"
    [ "$status" -eq 0 ]
    
    assert_dir_exists "$PPLR_TEST_DATA/V/Van Der Berg, Jan"
}

@test "pplr new preserves About file template structure" {
    run "$PPLR_BIN_DIR/pplr" new "Bob" "Wilson"
    [ "$status" -eq 0 ]
    
    local about_file="$PPLR_TEST_DATA/W/Wilson, Bob/About/Bob Wilson (About).md"
    assert_file_exists "$about_file"
    
    local content=$(cat "$about_file")
    assert_contains "$content" "Role:"
    assert_contains "$content" "Company:"
    assert_contains "$content" "LinkedIn:"
    assert_contains "$content" "Email:"
}

@test "pplr new doesn't overwrite existing person" {
    create_test_person "Davis" "Emma"
    
    # Try to create the same person again
    run "$PPLR_BIN_DIR/pplr" new "Emma" "Davis"
    [ "$status" -ne 0 ]
    assert_contains "$output" "already exists"
}