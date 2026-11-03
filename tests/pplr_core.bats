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

# ST0001 AC-00.7: on Linux, each command that needs a macOS service exits
# non-zero and names why. One row per gate the binary has: the command, then
# a phrase its message holds. On macOS the services are there, and each
# command's own tests cover it.
@test "AC-00.7 on Linux, every command that needs a macOS service exits non-zero naming why" {
    [ "$(uname)" != Darwin ] || skip "checks the Linux refusals; on macOS the services are there"
    create_test_person "Lovelace" "Ada" >/dev/null
    local gates=(
        "sync --check|the live Contacts store needs macOS"
        "handler --status|so it needs macOS"
        "applyicons Ada Lovelace|Finder icons need macOS"
        "setpicfordir Ada Lovelace|Finder icons need macOS"
        "setpicsfordirs|Finder icons need macOS"
        "cp Ada Lovelace|the macOS clipboard"
        "open Ada Lovelace|macOS's open"
        "open pplr://l/lovelace-ada|macOS's open"
        "linkedin Ada Lovelace|macOS's open"
        "email|macOS Mail's own index"
    )
    local gate why
    local -a argv
    for gate in "${gates[@]}"; do
        read -r -a argv <<< "${gate%%|*}"
        why=${gate#*|}
        run "$PPLR" "${argv[@]}"
        [ "$status" -ne 0 ] || { echo "pplr ${argv[*]} exited 0: $output" >&2; return 1; }
        assert_contains "$output" "$why" || { echo "pplr ${argv[*]} did not say why" >&2; return 1; }
    done
}
