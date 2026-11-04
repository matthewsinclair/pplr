#!/usr/bin/env bats

load test_helper
bats_require_minimum_version 1.5.0

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

# AT-00.7 (ST0001 AC-00.7): on Linux, each command that needs a macOS service
# exits non-zero and names why. Since ST0003 AC-03.1 each row holds the
# refusal as 2.0.0 gave it: exit 1, and its whole stderr line. One row per
# gate the binary has: the command, then that line. On macOS the services are
# there, and each command's own tests cover it.
@test "AC-00.7 on Linux, every command that needs a macOS service exits 1 with its refusal" {
    [ "$(uname)" != Darwin ] || skip "checks the Linux refusals; on macOS the services are there"
    create_test_person "Lovelace" "Ada" >/dev/null
    local icons="pplr: Finder icons need macOS"
    local opens="pplr: pplr open hands files and URLs to macOS's open, which this system does not have"
    local gates=(
        "sync --check|pplr: the live Contacts store needs macOS; elsewhere pplr reaches Contacts only through a dump file named by PPLR_CONTACTS_JSON"
        "handler --status|pplr: the pplr:// handler is a macOS app, so it needs macOS"
        "applyicons Ada Lovelace|$icons"
        "setpicfordir Ada Lovelace|$icons"
        "setpicsfordirs|$icons"
        "cp Ada Lovelace|pplr: pplr cp (without -o) copies to the macOS clipboard with pbcopy, which this system does not have"
        "open Ada Lovelace|$opens"
        "open pplr://l/lovelace-ada|$opens"
        "linkedin Ada Lovelace|$opens"
        "email|pplr: no Mail store at $HOME/Library/Mail/V10/MailData/Envelope Index (pplr email reads macOS Mail's own index)"
    )
    local gate line
    local -a argv
    for gate in "${gates[@]}"; do
        read -r -a argv <<< "${gate%%|*}"
        line=${gate#*|}
        run --separate-stderr "$PPLR" "${argv[@]}"
        [ "$status" -eq 1 ] || { echo "pplr ${argv[*]} exited $status, not 1: $stderr" >&2; return 1; }
        [ "$stderr" = "$line" ] || { echo "pplr ${argv[*]} said: $stderr" >&2; return 1; }
    done
}
