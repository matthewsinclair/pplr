#!/usr/bin/env bats

load test_helper

meeting() { mkdir -p "$PPLR_TEST_DATA/$1/Meetings/$2"; echo "# $2" > "$PPLR_TEST_DATA/$1/Meetings/$2/Notes.md"; }

@test "pplr contact scan takes last from the newest meeting, and next from a tag's cadence" {
    create_test_person "Adams" "Amy" >/dev/null
    create_test_person "Brown" "Bob" >/dev/null
    mkdir -p "$PPLR_TEST_DATA/_pplr"; printf "roster:\n  contact: 180\n  vip: 90\nevery:\n  12month: 365\n" > "$PPLR_TEST_DATA/_pplr/cadence.yaml"
    printf 'tags:\n  - vip\n' > "$PPLR_TEST_DATA/A/Adams, Amy/About/tags.yaml"
    meeting "A/Adams, Amy" "20260101 Intro"
    meeting "A/Adams, Amy" "20260301 Catch-up with Amy"
    meeting "B/Brown, Bob" "20260201 Coffee"
    export PPLR_TODAY=2026-09-25
    run "$PPLR_BIN_DIR/pplr" contact scan
    [ "$status" -eq 0 ]
    assert_contains "$output" "2 contact.yaml written (2 new)"
    run cat "$PPLR_TEST_DATA/A/Adams, Amy/About/contact.yaml"
    assert_contains "$output" 'last: {date: 2026-03-01, via: meeting, link: "pplr://a/adams-amy/Meetings/20260301 Catch-up with Amy"}'
    assert_contains "$output" "next: 2026-05-30"
    # no cadence: last is kept, no next
    run cat "$PPLR_TEST_DATA/B/Brown, Bob/About/contact.yaml"
    assert_contains "$output" "last: {date: 2026-02-01"
    [[ "$output" != *"next:"* ]]
    run "$PPLR_BIN_DIR/pplr" contact due
    [ "$output" = "2026-05-30  A/Adams, Amy  last 2026-03-01 meeting (208 days ago)" ]
    # a re-scan changes nothing
    run "$PPLR_BIN_DIR/pplr" contact scan
    assert_contains "$output" "0 contact.yaml written"
}

@test "pplr contact log moves next on, next snoozes, and a booked meeting is not due" {
    create_test_person "Adams" "Amy" >/dev/null
    mkdir -p "$PPLR_TEST_DATA/_pplr"; printf "roster:\n  contact: 180\n  vip: 90\nevery:\n  12month: 365\n" > "$PPLR_TEST_DATA/_pplr/cadence.yaml"
    printf 'tags:\n  - vip\n' > "$PPLR_TEST_DATA/A/Adams, Amy/About/tags.yaml"
    export PPLR_TODAY=2026-09-25
    "$PPLR_BIN_DIR/pplr" contact scan >/dev/null
    run "$PPLR_BIN_DIR/pplr" contact due
    [ "$output" = "2026-09-25  A/Adams, Amy  no contact yet" ]
    run "$PPLR_BIN_DIR/pplr" contact log "Adams, Amy" 2026-09-20 --via email --note "Sent the deck"
    [ "$output" = "Amy Adams: email on 2026-09-20; next 2026-12-19" ]
    run "$PPLR_BIN_DIR/pplr" contact due
    [ "$output" = "Nobody is due." ]
    run "$PPLR_BIN_DIR/pplr" contact next "Adams, Amy" +1w
    [ "$output" = "Amy Adams: next 2026-10-02" ]
    run "$PPLR_BIN_DIR/pplr" contact due 7
    assert_contains "$output" "2026-10-02  A/Adams, Amy  last 2026-09-20 email"
    meeting "A/Adams, Amy" "20261001 Lunch"
    run "$PPLR_BIN_DIR/pplr" contact due 7
    [ "$output" = "Nobody is due." ]
    run "$PPLR_BIN_DIR/pplr" contact show "Adams, Amy"
    assert_contains "$output" "cadence: 90 days, from #vip"
    assert_contains "$output" "(booked 2026-10-01)"
}

@test "pplr contact context shows the role, the latest update and the last meeting's notes" {
    create_test_person "Adams" "Amy" >/dev/null
    meeting "A/Adams, Amy" "20260301 Catch-up"
    echo "Talked about the Series A." >> "$PPLR_TEST_DATA/A/Adams, Amy/Meetings/20260301 Catch-up/Notes.md"
    export PPLR_TODAY=2026-09-25
    "$PPLR_BIN_DIR/pplr" contact scan >/dev/null
    run "$PPLR_BIN_DIR/pplr" contact context "Adams, Amy"
    [ "$status" -eq 0 ]
    assert_contains "$output" "# Amy Adams"
    assert_contains "$output" "Last contact: 2026-03-01 meeting"
    assert_contains "$output" "## Last meeting: 20260301 Catch-up"
    assert_contains "$output" "Talked about the Series A."
}

@test "pplr contact: a cadence tag overrides the roster tag's default" {
    create_test_person "Adams" "Amy" >/dev/null
    create_test_person "Brown" "Bob" >/dev/null
    mkdir -p "$PPLR_TEST_DATA/_pplr"; printf "roster:\n  contact: 180\n  vip: 90\nevery:\n  12month: 365\n" > "$PPLR_TEST_DATA/_pplr/cadence.yaml"
    printf 'tags:\n  - contact\n  - 12month\n' > "$PPLR_TEST_DATA/A/Adams, Amy/About/tags.yaml"
    printf 'tags:\n  - contact\n' > "$PPLR_TEST_DATA/B/Brown, Bob/About/tags.yaml"
    run "$PPLR_BIN_DIR/pplr" contact show "Adams, Amy"
    assert_contains "$output" "cadence: 365 days, from #12month"
    run "$PPLR_BIN_DIR/pplr" contact show "Brown, Bob"
    assert_contains "$output" "cadence: 180 days, from #contact"
}
