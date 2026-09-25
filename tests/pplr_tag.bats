#!/usr/bin/env bats

load test_helper

setup_person() {
    mkdir -p "$PPLR_TEST_DATA/_pplr" "$PPLR_TEST_DATA/K/Kemp, Jon/About"
    printf '%s\n' 'version: 1' 'role:' '  cto: {}' 'sector:' '  fintech: {also: [payments]}' 'place:' '  london: {}' > "$PPLR_TEST_DATA/_pplr/vocabulary.yaml"
    printf '%s\n' '# Jon Kemp (About)' '' '- Role:     CTO' '' '_About_' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (About).md"
    printf '%s\n' 'tags:' '  - {tag: cto, source: auto}' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
}

@test "pplr tag shows a person's tags by facet" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_person
    run "$PPLR_BIN_DIR/pplr" tag "Kemp, Jon"
    [ "$status" -eq 0 ]
    assert_contains "$output" "cto (auto)"
}

@test "pplr tag adds a hand tag and removes another" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_person
    run "$PPLR_BIN_DIR/pplr" tag "Kemp, Jon" +london -cto
    [ "$status" -eq 0 ]
    f="$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    grep -q '^  - london$' "$f"
    ! grep -q 'cto' "$f"
}

@test "pplr tag refuses a tag outside the vocabulary, and names the one it folds into" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_person
    run "$PPLR_BIN_DIR/pplr" tag "Kemp, Jon" +payments
    [ "$status" -ne 0 ]
    assert_contains "$output" "folds it into fintech"
}

@test "pplr reindex --tags says the Claude tagger is retired" {
    run "$PPLR_BIN_DIR/pplr" reindex --tags
    [ "$status" -eq 1 ]
    assert_contains "$output" "retired"
}
