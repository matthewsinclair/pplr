#!/usr/bin/env bats

load test_helper

@test "pplr refresh next lists the never-checked first, then the oldest, and stamp records a check" {
    create_test_person "Adams" "Amy" >/dev/null
    create_test_person "Brown" "Bob" >/dev/null
    create_test_person "Clark" "Cy" >/dev/null
    mkdir -p "$PPLR_TEST_DATA/A/Adams, Amy/.index"; echo "2025-01-01 linkedin" > "$PPLR_TEST_DATA/A/Adams, Amy/.index/refreshed"
    run "$PPLR" refresh next 2
    [ "$status" -eq 0 ]
    [ "$output" = "$(printf 'B/Brown, Bob\nC/Clark, Cy')" ]
    "$PPLR" refresh stamp "Brown, Bob" >/dev/null
    "$PPLR" refresh stamp "Clark, Cy" >/dev/null
    run "$PPLR" refresh next 1
    [ "$output" = "A/Adams, Amy" ]
    run "$PPLR" refresh next 5 --older-than 30
    [ "$output" = "A/Adams, Amy" ]
    run "$PPLR" refresh status
    assert_contains "$output" "3 people, 2 checked in the last 90 days, 1 older, 0 never"
}
