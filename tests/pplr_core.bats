#!/usr/bin/env bats

load test_helper

@test "pplr command exists and is executable" {
    [ -x "$PPLR" ]
}

@test "pplr shows help when run without arguments" {
    run "$PPLR"
    [ "$status" -eq 1 ]
    assert_contains "$output" "Usage:"
}

@test "pplr help shows all commands" {
    run "$PPLR" help
    [ "$status" -eq 0 ]
    assert_contains "$output" "search"
    assert_contains "$output" "new"
    assert_contains "$output" "open"
    assert_contains "$output" "meetings"
}

@test "pplr help --details shows README" {
    run "$PPLR" help --details
    [ "$status" -eq 0 ]
    assert_contains "$output" "pplr - Personal Relationship Manager"
}

@test "the data environment is the test's own sandbox" {
    [ "$PPLR_DATA" = "$BATS_TEST_TMPDIR/data" ]
    [ "$PPLR_DIR" = "$PPLR_DATA" ]
    [ "$HOME" = "$BATS_TEST_TMPDIR/home" ]
}

@test "test data directory structure is created" {
    assert_dir_exists "$PPLR_TEST_DATA"
    assert_dir_exists "$PPLR_TEST_DATA/A"
    assert_dir_exists "$PPLR_TEST_DATA/Z"
}

@test "pplr count works with empty directory" {
    run "$PPLR" count
    [ "$status" -eq 0 ]
    assert_contains "$output" "0"
}

@test "pplr count counts test people correctly" {
    create_test_person "Smith" "John"
    create_test_person "Doe" "Jane"
    
    run "$PPLR" count
    [ "$status" -eq 0 ]
    assert_contains "$output" "2"
}

@test "pplr version displays version from VERSION.md" {
    # Read expected version from VERSION.md
    expected_version=$(cat "$PPLR_ROOT/VERSION.md")
    run "$PPLR" version
    [ "$status" -eq 0 ]
    assert_contains "$output" "$expected_version"
}

@test "pplr -v displays version" {
    # Read expected version from VERSION.md
    expected_version=$(cat "$PPLR_ROOT/VERSION.md")
    run "$PPLR" -v
    [ "$status" -eq 0 ]
    assert_contains "$output" "$expected_version"
}

@test "pplr --version displays version" {
    # Read expected version from VERSION.md
    expected_version=$(cat "$PPLR_ROOT/VERSION.md")
    run "$PPLR" --version
    [ "$status" -eq 0 ]
    assert_contains "$output" "$expected_version"
}