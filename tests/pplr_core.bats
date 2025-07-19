#!/usr/bin/env bats

load test_helper

@test "pplr command exists and is executable" {
    [ -x "$PPLR_BIN_DIR/pplr" ]
}

@test "pplr shows help when run without arguments" {
    run "$PPLR_BIN_DIR/pplr"
    [ "$status" -eq 1 ]
    assert_contains "$output" "Usage:"
}

@test "pplr help shows all commands" {
    run "$PPLR_BIN_DIR/pplr" help
    [ "$status" -eq 0 ]
    assert_contains "$output" "search"
    assert_contains "$output" "new"
    assert_contains "$output" "open"
    assert_contains "$output" "meetings"
}

@test "pplr help --details shows README" {
    run "$PPLR_BIN_DIR/pplr" help --details
    [ "$status" -eq 0 ]
    assert_contains "$output" "pplr - Personal Relationship Manager"
}

@test "environment variables are set correctly" {
    [ -n "$PPLR_ROOT" ]
    [ -n "$PPLR_DATA" ]
    [ -n "$PPLR_BIN_DIR" ]
    [ -n "$PPLR_TEMPLATE_DIR" ]
    [ "$PPLR_DIR" = "$PPLR_DATA" ]
}

@test "test data directory structure is created" {
    assert_dir_exists "$PPLR_TEST_DATA"
    assert_dir_exists "$PPLR_TEST_DATA/A"
    assert_dir_exists "$PPLR_TEST_DATA/Z"
}

@test "pplr count works with empty directory" {
    run "$PPLR_BIN_DIR/pplr" count
    [ "$status" -eq 0 ]
    assert_contains "$output" "0"
}

@test "pplr count counts test people correctly" {
    create_test_person "Smith" "John"
    create_test_person "Doe" "Jane"
    
    run "$PPLR_BIN_DIR/pplr" count
    [ "$status" -eq 0 ]
    assert_contains "$output" "2"
}

@test "pplr version displays version from VERSION.md" {
    # Read expected version from VERSION.md
    expected_version=$(cat "$PPLR_ROOT/VERSION.md")
    run "$PPLR_BIN_DIR/pplr" version
    [ "$status" -eq 0 ]
    assert_contains "$output" "$expected_version"
}

@test "pplr -v displays version" {
    # Read expected version from VERSION.md
    expected_version=$(cat "$PPLR_ROOT/VERSION.md")
    run "$PPLR_BIN_DIR/pplr" -v
    [ "$status" -eq 0 ]
    assert_contains "$output" "$expected_version"
}

@test "pplr --version displays version" {
    # Read expected version from VERSION.md
    expected_version=$(cat "$PPLR_ROOT/VERSION.md")
    run "$PPLR_BIN_DIR/pplr" --version
    [ "$status" -eq 0 ]
    assert_contains "$output" "$expected_version"
}