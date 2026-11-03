#!/usr/bin/env bats

# The people-leak guard (ST0001 AC-00.9, design D11), run over a scratch git
# repository against this sandbox's synthetic People tree: a staged file that
# holds a person's folder name, the name on their About file, their marker or
# an email from their About is refused, by path, and a clean one passes. The
# guard reads PPLR_DATA, which the sandbox points at its own tree.

load test_helper

@test "AT-00.9 people-leak refuses a staged file holding someone from PPLR_DATA, and passes a clean one" {
    create_test_person "Lovelace" "Ada" >/dev/null
    local repo="$BATS_TEST_TMPDIR/repo"
    git init -q "$repo"
    printf '%s\n' "seed" > "$repo/seed.txt"
    git -C "$repo" add seed.txt
    git -C "$repo" commit -q -m seed

    # Each leak: the staged file, then the line it holds
    local leaks=(
        "notes/folder.md|met Lovelace, Ada today"
        "notes/name.md|a note for Ada Lovelace"
        "notes/marker.md|see pplr://l/lovelace-ada"
        "notes/email.md|write to test@example.com"
    )
    local row path
    for row in "${leaks[@]}"; do
        path=${row%%|*}
        mkdir -p "$repo/$(dirname "$path")"
        printf '%s\n' "${row#*|}" > "$repo/$path"
        git -C "$repo" add -- "$path"
        run bash -c "cd '$repo' && '$PPLR_ROOT/bin/.devbin/cmd/people-leak'"
        [ "$status" -ne 0 ] || { echo "$path passed: $output" >&2; return 1; }
        assert_contains "$output" "people-leak: $path:" || { echo "$path refused without naming it" >&2; return 1; }
        git -C "$repo" rm -q --cached -- "$path"
        rm -- "$repo/$path"
    done

    mkdir -p "$repo/notes"
    printf '%s\n' "nothing about anyone" > "$repo/notes/clean.md"
    git -C "$repo" add -- notes/clean.md
    run bash -c "cd '$repo' && '$PPLR_ROOT/bin/.devbin/cmd/people-leak'"
    [ "$status" -eq 0 ] || { echo "a clean file was refused: $output" >&2; return 1; }
    assert_contains "$output" "people-leak: clean"
}
