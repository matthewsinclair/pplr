#!/usr/bin/env bats

load test_helper

setup_vocab() {
    mkdir -p "$PPLR_TEST_DATA/_pplr"
    cat > "$PPLR_TEST_DATA/_pplr/vocabulary.yaml" << 'YAML'
version: 1
role:
  cto: {also: [chief-technology-officer]}
  founder: {}
sector:
  fintech: {also: [payments]}
relationship:
  bcg: {also: [bcg-x]}
  podcast: {}
place:
  london: {}
split:
  podcast-guest: [podcast]
YAML
    local dir="$PPLR_TEST_DATA/K/Kemp, Jon/About"
    mkdir -p "$dir"
    printf '%s\n' '# Jon Kemp (About)' '' '_Jon Kemp_' '- Role:     CTO' '- Company:  Acme' '- Phone:' '' '_About_' 'Jon builds things.' > "$dir/Jon Kemp (About).md"
}

@test "pplr tags apply writes tags.yaml, and keeps hand tags on a re-run" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_vocab
    echo '[{"person": "K/Kemp, Jon", "auto": ["cto", "fintech"], "inferred": [{"tag": "bcg", "evidence": "BCG DV 2018"}], "hand": ["london"]}]' > "$PPLR_TEST_DATA/r.json"
    run "$PPLR_BIN_DIR/pplr" tags apply "$PPLR_TEST_DATA/r.json"
    [ "$status" -eq 0 ]
    f="$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    grep -q '^  - london$' "$f"
    grep -q 'tag: cto, source: auto' "$f"
    grep -q 'tag: bcg, source: inferred, evidence: "BCG DV 2018"' "$f"
    echo '[{"person": "K/Kemp, Jon", "auto": ["founder"], "inferred": []}]' > "$PPLR_TEST_DATA/r2.json"
    run "$PPLR_BIN_DIR/pplr" tags apply "$PPLR_TEST_DATA/r2.json"
    grep -q '^  - london$' "$f"
    grep -q 'tag: founder, source: auto' "$f"
    ! grep -q 'tag: cto' "$f"
    ! grep -q 'tag: bcg' "$f"
}

@test "pplr tags check names tags outside the vocabulary and says where they fold" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_vocab
    printf '%s\n' 'tags:' '  - payments' '  - Fin-Tech' '  - {tag: london, source: inferred}' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    run "$PPLR_BIN_DIR/pplr" tags check
    [ "$status" -eq 1 ]
    assert_contains "$output" "payments is not in the vocabulary (the vocabulary folds it into fintech)"
    assert_contains "$output" "'Fin-Tech' is not a single lowercase word"
    assert_contains "$output" "london is inferred but not a relationship tag"
}

@test "pplr tags render writes one Tags line into the header, and replaces it on a re-run" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_vocab
    printf '%s\n' 'tags:' '  - london' '  - {tag: cto, source: auto}' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    run "$PPLR_BIN_DIR/pplr" tags render
    [ "$status" -eq 0 ]
    a="$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (About).md"
    [ "$(sed -n 7p "$a")" = '- Tags:     [#cto](../../../_tags/cto.md) [#london](../../../_tags/london.md) <!-- from tags.yaml: pplr tags render -->' ]
    printf '%s\n' 'tags:' '  - founder' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    "$PPLR_BIN_DIR/pplr" tags render
    [ "$(grep -c '^- Tags:' "$a")" -eq 1 ]
    grep -q '^- Tags:     \[#founder\](../../../_tags/founder.md) <!--' "$a"
}

@test "pplr tags render writes a page per tag, and an index, linking to each About" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_vocab
    printf '%s\n' 'tags:' '  - london' '  - {tag: cto, source: auto}' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    mkdir -p "$PPLR_TEST_DATA/S/Tolley, Jon/About"
    printf '%s\n' '# Jon Tolley (About)' '' '- Role:     Founder' '' '_About_' > "$PPLR_TEST_DATA/S/Tolley, Jon/About/Jon Tolley (About).md"
    printf '%s\n' 'tags:' '  - cto' > "$PPLR_TEST_DATA/S/Tolley, Jon/About/tags.yaml"
    run "$PPLR_BIN_DIR/pplr" tags render
    [ "$status" -eq 0 ]
    t="$PPLR_TEST_DATA/_tags"
    grep -qF '[Jon Kemp](<../K/Kemp, Jon/About/Jon Kemp (About).md>): CTO at Acme · #london' "$t/cto.md"
    grep -qF '[Jon Tolley](<../S/Tolley, Jon/About/Jon Tolley (About).md>): Founder' "$t/cto.md"
    grep -qF '[#london](london.md) 1' "$t/cto.md"
    grep -qF '[#cto](cto.md) 2' "$t/index.md"
    [ -f "$t/london.md" ]
    printf '%s\n' 'tags:' '  - cto' > "$PPLR_TEST_DATA/K/Kemp, Jon/About/tags.yaml"
    "$PPLR_BIN_DIR/pplr" tags render >/dev/null
    [ ! -f "$t/london.md" ]
}
