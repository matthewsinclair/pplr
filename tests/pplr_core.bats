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

# The version line, `pplr <Cargo.toml's version> (<tree>, <sha12>[+dirty])`
# (devbin T4, ST0002 WP03).
assert_version_output() {
    local version
    version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$PPLR_ROOT/Cargo.toml" | head -1)
    [[ "$output" =~ ^pplr\ ${version//./\\.}\ \((dev|opt|brew),\ ([0-9a-f]{12}(\+dirty)?|unknown)\)$ ]] || {
        echo "not a version line for $version: $output" >&2
        return 1
    }
}

@test "pplr version displays the implementation's version" {
    run "$PPLR" version
    [ "$status" -eq 0 ]
    assert_version_output
}

@test "pplr -v displays version" {
    run "$PPLR" -v
    [ "$status" -eq 0 ]
    assert_version_output
}

@test "pplr --version displays version" {
    run "$PPLR" --version
    [ "$status" -eq 0 ]
    assert_version_output
}
