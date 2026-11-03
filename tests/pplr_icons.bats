#!/usr/bin/env bats

load test_helper

@test "AT-12.2 pplr applyicons puts a custom icon on the About and Meetings folders and each meeting" {
    [ "$(uname)" = Darwin ] || skip "Finder icons are macOS only"
    command -v fileicon >/dev/null 2>&1 || skip "fileicon not installed (brew install fileicon)"
    local person_dir
    person_dir=$(create_test_person "Lovelace" "Ada")
    create_test_meeting "$person_dir" "20260615" "Intro"

    run "$PPLR" applyicons "Ada" "Lovelace"
    [ "$status" -eq 0 ]
    for target in "$person_dir/About" "$person_dir/Meetings" "$person_dir/Meetings/20260615 Intro"; do
        fileicon test -q "$target" || { echo "no custom icon on $target" >&2; return 1; }
    done
    # The person's own folder is setpicfordir's, not applyicons'
    ! fileicon test -q "$person_dir"
}

# ST0001 AC-00.7: elsewhere, a command that needs Finder exits 1 and says why
@test "AC-00.7 on Linux, applyicons, setpicfordir and setpicsfordirs exit 1 naming macOS" {
    [ "$(uname)" != Darwin ] || skip "checks the Linux refusal; on macOS AT-12.2 checks the icons"
    [ "$PPLR" != "$PPLR_ROOT/bin/pplr" ] || skip "the legacy pplr predates the Linux refusal (AC-00.7)"
    create_test_person "Lovelace" "Ada" >/dev/null

    run "$PPLR" applyicons "Ada" "Lovelace"
    [ "$status" -eq 1 ]
    assert_contains "$output" "Finder icons need macOS"

    run "$PPLR" setpicfordir "Ada" "Lovelace"
    [ "$status" -eq 1 ]
    assert_contains "$output" "Finder icons need macOS"

    run "$PPLR" setpicsfordirs
    [ "$status" -eq 1 ]
    assert_contains "$output" "Finder icons need macOS"
}
