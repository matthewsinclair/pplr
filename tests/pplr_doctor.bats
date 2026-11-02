#!/usr/bin/env bats

load test_helper

@test "pplr doctor runs and shows system health" {
    run "$PPLR" doctor
    # Check that it runs and produces expected output
    assert_contains "$output" "System Health Check"
    assert_contains "$output" "Environment Variables"
    assert_contains "$output" "Directory Structure"
    assert_contains "$output" "Dependencies"
    assert_contains "$output" "Summary"
}

@test "pplr doctor detects missing directories" {
    # Remove a letter directory
    rm -rf "$PPLR_TEST_DATA/Z"
    
    run "$PPLR" doctor
    # Doctor returns number of errors as exit code
    assert_contains "$output" "Missing directories"
    assert_contains "$output" "Z"
}

@test "pplr doctor --fix creates missing directories" {
    # Remove some directories
    rm -rf "$PPLR_TEST_DATA/Y"
    rm -rf "$PPLR_TEST_DATA/Z"
    rm -rf "$PPLR_TEST_DATA/.index"
    
    run "$PPLR" doctor --fix
    [ "$status" -eq 0 ]
    assert_contains "$output" "Created missing directories"
    assert_contains "$output" "Created .index directory"
    
    # Verify directories were created
    assert_dir_exists "$PPLR_TEST_DATA/Y"
    assert_dir_exists "$PPLR_TEST_DATA/Z"
    assert_dir_exists "$PPLR_TEST_DATA/.index"
}

@test "pplr doctor detects missing About files" {
    # Create a person without an About file
    local person_dir="$PPLR_TEST_DATA/T/Test, Person"
    mkdir -p "$person_dir/About"
    # Don't create the About file
    
    run "$PPLR" doctor
    # Should have warnings but not errors
    assert_contains "$output" "people missing About files"
}

@test "pplr doctor validates JSON files" {
    # Create invalid JSON
    mkdir -p "$PPLR_TEST_DATA/.index"
    echo "invalid json" > "$PPLR_TEST_DATA/.index/index.json"
    
    run "$PPLR" doctor
    [ "$status" -ne 0 ]
    assert_contains "$output" "index.json is invalid JSON"
}

