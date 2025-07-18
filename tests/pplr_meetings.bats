#!/usr/bin/env bats

load test_helper

@test "pplr meetings finds meetings by date range" {
    local person_dir=$(create_test_person "Meeting" "Mike")
    create_test_meeting "$person_dir" "20240101" "New Year Meeting"
    create_test_meeting "$person_dir" "20240215" "February Sync"
    create_test_meeting "$person_dir" "20240301" "March Review"
    
    run "$PPLR_BIN_DIR/pplr" meetings 2024-02-01 2024-02-28
    [ "$status" -eq 0 ]
    assert_contains "$output" "February Sync"
    # Should not contain meetings outside date range
    [[ ! "$output" =~ "New Year Meeting" ]]
    [[ ! "$output" =~ "March Review" ]]
}

@test "pplr meetings finds meetings for specific date" {
    local person_dir=$(create_test_person "Daily" "Dan")
    create_test_meeting "$person_dir" "20240315" "Morning Standup"
    create_test_meeting "$person_dir" "20240315" "Afternoon Review"
    create_test_meeting "$person_dir" "20240316" "Next Day Meeting"
    
    run "$PPLR_BIN_DIR/pplr" meetings 2024-03-15
    [ "$status" -eq 0 ]
    assert_contains "$output" "Morning Standup"
    assert_contains "$output" "Afternoon Review"
    [[ ! "$output" =~ "Next Day Meeting" ]]
}

@test "pplr meetings with no arguments shows recent meetings" {
    local person_dir=$(create_test_person "Recent" "Rachel")
    # Create a meeting with today's date
    local today=$(date +%Y%m%d)
    create_test_meeting "$person_dir" "$today" "Today Meeting"
    
    run "$PPLR_BIN_DIR/pplr" meetings
    [ "$status" -eq 0 ]
    # Should find today's meeting
    assert_contains "$output" "Today Meeting"
}

@test "pplr meetings handles invalid date format" {
    run "$PPLR_BIN_DIR/pplr" meetings "invalid-date"
    [ "$status" -ne 0 ]
    assert_contains "$output" "Invalid date"
}

@test "pplr meetings finds meetings across multiple people" {
    local person1=$(create_test_person "Alpha" "Alice")
    local person2=$(create_test_person "Beta" "Bob")
    
    create_test_meeting "$person1" "20240501" "Alice Meeting"
    create_test_meeting "$person2" "20240501" "Bob Meeting"
    
    run "$PPLR_BIN_DIR/pplr" meetings 2024-05-01
    [ "$status" -eq 0 ]
    assert_contains "$output" "Alice Meeting"
    assert_contains "$output" "Bob Meeting"
}

@test "pplr meetings with no results" {
    create_test_person "Empty" "Eve"
    
    run "$PPLR_BIN_DIR/pplr" meetings 2020-01-01 2020-12-31
    [ "$status" -eq 0 ]
    assert_contains "$output" "No meetings found"
}