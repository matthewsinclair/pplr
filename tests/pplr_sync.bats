#!/usr/bin/env bats

load test_helper

# A person with the real About header format
make_about() {
    local surname="$1" firstname="$2" body="$3"
    local letter=$(echo "$surname" | cut -c1)
    local dir="$PPLR_TEST_DATA/$letter/$surname, $firstname/About"
    mkdir -p "$dir"
    printf '%s\n' "# $firstname $surname (About)" "" "_${firstname} ${surname}_" "$body" > "$dir/$firstname $surname (About).md"
}

setup_contacts() {
    export PPLR_CONTACTS_JSON="$PPLR_TEST_DATA/contacts.json"
    cat > "$PPLR_CONTACTS_JSON" << 'EOF'
[
  {"id": "c1", "given": "Ada", "family": "Lovelace", "organization": "Analytical", "jobTitle": "Engineer",
   "emails": ["ada@example.com"], "phones": ["+447700900001"], "urls": [], "linkedin": ["ada"], "groups": ["PPLR"]},
  {"id": "c2", "given": "Grace", "family": "Hopper", "organization": "Navy", "jobTitle": "",
   "emails": ["grace@work.example"], "phones": [], "urls": [], "linkedin": [], "groups": []},
  {"id": "c3", "given": "Alan", "family": "Turing", "organization": "", "jobTitle": "",
   "emails": [], "phones": [], "urls": [{"label": "pplr", "value": "pplr://T/Turing, Alan"}], "linkedin": [], "groups": ["PPLR"]},
  {"id": "c4", "given": "Orphan", "family": "Card", "organization": "", "jobTitle": "",
   "emails": [], "phones": [], "urls": [], "linkedin": [], "groups": ["PPLR"]}
]
EOF
    make_about Lovelace Ada "$(printf '%s\n' '- Role:     [Engineer](https://x)' '- Company:  Analytical' '- LinkedIn: [in/ada](https://www.linkedin.com/in/Ada/)' '- Email:    [ada@example.com](mailto:ada@example.com)' '- Phone:    +44 (0) 7700 900001')"
    make_about Hopper Grace "$(printf '%s\n' '- Role:     Rear Admiral' '- Company:  Navy' '- LinkedIn:' '- Email:    [](mailto:)' '- Phone:')"
    make_about Turing Alan "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:    [alan@example.com](mailto:alan@example.com)' '- Phone:')"
    make_about Nobody Nemo "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
}

@test "pplr sync without a mode shows usage" {
    run "$PPLR_BIN_DIR/pplr" sync
    [ "$status" -eq 1 ]
    assert_contains "$output" "Usage: pplr sync --check"
}

@test "pplr sync --check reports each category" {
    setup_contacts
    run "$PPLR_BIN_DIR/pplr" sync --check
    [ "$status" -eq 0 ]
    assert_contains "$output" "linked (pplr marker)    1"
    assert_contains "$output" "matched by email        1"
    assert_contains "$output" "name only (confirm)     1"
    assert_contains "$output" "only in pplr            1"
    assert_contains "$output" "group orphans           1"
}

@test "pplr sync --check normalises phones and LinkedIn before comparing" {
    setup_contacts
    run "$PPLR_BIN_DIR/pplr" sync --check --json
    [ "$status" -eq 0 ]
    ada_diffs=$(echo "$output" | jq '[.matched[] | select(.person == "L/Lovelace, Ada") | .diffs[]] | length')
    [ "$ada_diffs" -eq 0 ]
}

@test "pplr sync --check shows differing fields for a linked person" {
    setup_contacts
    run "$PPLR_BIN_DIR/pplr" sync --check --json
    [ "$status" -eq 0 ]
    field=$(echo "$output" | jq -r '.linked[] | select(.person == "T/Turing, Alan") | .diffs[].field')
    [ "$field" = "email" ]
}

@test "pplr sync --check --verbose lists people only in pplr" {
    setup_contacts
    run "$PPLR_BIN_DIR/pplr" sync --check --verbose
    [ "$status" -eq 0 ]
    assert_contains "$output" "N/Nobody, Nemo"
}

@test "pplr sync --link is a dry run by default" {
    setup_contacts
    before=$(cat "$PPLR_CONTACTS_JSON")
    run "$PPLR_BIN_DIR/pplr" sync --link
    [ "$status" -eq 0 ]
    assert_contains "$output" "dry run"
    assert_contains "$output" "L/Lovelace, Ada"
    [ "$(cat "$PPLR_CONTACTS_JSON")" = "$before" ]
}

@test "pplr sync --link --apply adds the marker and group, and backs up first" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    run "$PPLR_BIN_DIR/pplr" sync --link --apply
    [ "$status" -eq 0 ]
    [ "$(ls "$PPLR_BACKUP_DIR" | wc -l | tr -d ' ')" -eq 1 ]
    marker=$(jq -r '.[] | select(.id == "c1") | .urls[] | select(.label == "pplr") | .value' "$PPLR_CONTACTS_JSON")
    [ "$marker" = "pplr://l/lovelace-ada" ]
    turing_groups=$(jq -r '.[] | select(.id == "c3") | .groups | join(",")' "$PPLR_CONTACTS_JSON")
    [ "$turing_groups" = "PPLR" ]
    hopper_groups=$(jq -r '.[] | select(.id == "c2") | .groups | length' "$PPLR_CONTACTS_JSON")
    [ "$hopper_groups" -eq 0 ]
}

@test "pplr sync --link --name links a confirmed name-only match" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    run "$PPLR_BIN_DIR/pplr" sync --link --apply --name "Hopper, Grace"
    [ "$status" -eq 0 ]
    hopper_groups=$(jq -r '.[] | select(.id == "c2") | .groups | join(",")' "$PPLR_CONTACTS_JSON")
    [ "$hopper_groups" = "PPLR" ]
}

@test "pplr sync --link --apply twice changes nothing the second time" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    "$PPLR_BIN_DIR/pplr" sync --link --apply
    run "$PPLR_BIN_DIR/pplr" sync --link --apply
    [ "$status" -eq 0 ]
    assert_contains "$output" "linked now              0"
    urls=$(jq -r '.[] | select(.id == "c1") | .urls | length' "$PPLR_CONTACTS_JSON")
    [ "$urls" -eq 1 ]
}

@test "pplr sync --link --apply writes a Contacts webloc in About" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    run "$PPLR_BIN_DIR/pplr" sync --link --apply
    [ "$status" -eq 0 ]
    w="$PPLR_TEST_DATA/L/Lovelace, Ada/About/Ada Lovelace (Contacts).webloc"
    [ -f "$w" ]
    [ "$(plutil -extract URL raw -o - "$w")" = "addressbook://c1" ]
}

@test "pplr sync --link --apply --limit 1 writes one and reports the rest as still to link" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    run "$PPLR_BIN_DIR/pplr" sync --link --apply --limit 1
    [ "$status" -eq 0 ]
    assert_contains "$output" "linked now              1"
    assert_contains "$output" "still to link           1"
}

@test "pplr sync --link --apply replaces a pplr URL in the old form, rather than adding one" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    run "$PPLR_BIN_DIR/pplr" sync --link --apply
    [ "$status" -eq 0 ]
    urls=$(jq -r '.[] | select(.id == "c3") | [.urls[] | select(.label == "pplr") | .value] | join(",")' "$PPLR_CONTACTS_JSON")
    [ "$urls" = "pplr://t/turing-alan" ]
}

@test "pplr sync --link groups iCloud cards only, and never writes a card with no account" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    jq '(.[] | select(.id == "c1")).account = "other" | (.[] | select(.id == "c3")).account = "none"' "$PPLR_CONTACTS_JSON" > "$PPLR_TEST_DATA/c.json"
    mv "$PPLR_TEST_DATA/c.json" "$PPLR_CONTACTS_JSON"
    jq '(.[] | select(.id == "c1")).groups = []' "$PPLR_CONTACTS_JSON" > "$PPLR_TEST_DATA/c.json"
    mv "$PPLR_TEST_DATA/c.json" "$PPLR_CONTACTS_JSON"
    run "$PPLR_BIN_DIR/pplr" sync --link --apply
    [ "$status" -eq 0 ]
    assert_contains "$output" "cannot be linked"
    ada=$(jq -r '.[] | select(.id == "c1") | "\(.groups | length) \([.urls[] | select(.label == "pplr")] | length)"' "$PPLR_CONTACTS_JSON")
    [ "$ada" = "0 1" ]
    turing=$(jq -r '.[] | select(.id == "c3") | [.urls[] | select(.label == "pplr") | .value] | join(",")' "$PPLR_CONTACTS_JSON")
    [ "$turing" = "pplr://T/Turing, Alan" ]
    run "$PPLR_BIN_DIR/pplr" sync --link
    assert_contains "$output" "to link                 0"
}

# Near misses for the plan: a nickname, a phone written as a link, a rename
setup_plan() {
    setup_contacts
    jq '. + [
      {"id": "c5", "given": "William", "family": "Mayhew", "organization": "Acme", "jobTitle": "",
       "emails": [], "phones": [], "urls": [], "linkedin": [], "groups": []},
      {"id": "c6", "given": "Glen", "family": "Griffiths", "organization": "", "jobTitle": "",
       "emails": [], "phones": ["+491700000001"], "urls": [], "linkedin": [], "groups": []}
    ]' "$PPLR_CONTACTS_JSON" > "$PPLR_TEST_DATA/c.json"
    mv "$PPLR_TEST_DATA/c.json" "$PPLR_CONTACTS_JSON"
    make_about Mayhew Bill "$(printf '%s\n' '- Role:' '- Company:  Acme Ltd' '- LinkedIn: [in/bill/](https://www.linkedin.com/in/bill-mayhew)' '- Email:' '- Phone:')"
    make_about Grifiths Glen "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:    [+49 170 0000001](tel:+491700000001)')"
}

@test "pplr sync --plan --json proposes a decision for every pplr person" {
    setup_plan
    run "$PPLR_BIN_DIR/pplr" sync --plan --json
    [ "$status" -eq 0 ]
    [ "$(echo "$output" | jq '.rows | length')" -eq 6 ]
    [ "$(echo "$output" | jq -r '.rows[] | select(.person == "N/Nobody, Nemo") | .decision')" = "Add" ]
    [ "$(echo "$output" | jq -r '.rows[] | select(.person == "L/Lovelace, Ada") | .confidence')" = "High" ]
}

@test "pplr sync --plan matches a nickname with the same surname and company" {
    setup_plan
    run "$PPLR_BIN_DIR/pplr" sync --plan --json
    [ "$status" -eq 0 ]
    row=$(echo "$output" | jq -r '.rows[] | select(.person == "M/Mayhew, Bill") | "\(.decision) \(.why)"')
    assert_contains "$row" "Update same surname, William for Bill, same company"
    card=$(echo "$output" | jq -r '.rows[] | select(.person == "M/Mayhew, Bill") | .card')
    [ "$(echo "$output" | jq -r --arg r "$card" '.cards[] | select(.ref == $r) | .id')" = "c5" ]
}

@test "pplr sync --plan reads a phone written as a link, and never marks a rename High" {
    setup_plan
    run "$PPLR_BIN_DIR/pplr" sync --plan --json
    [ "$status" -eq 0 ]
    row=$(echo "$output" | jq -r '.rows[] | select(.person == "G/Grifiths, Glen") | "\(.confidence) \(.why)"')
    [ "$row" = "Medium same phone, names differ" ]
}

@test "pplr sync --plan reads a LinkedIn link without its closing bracket" {
    setup_plan
    run "$PPLR_BIN_DIR/pplr" sync --plan --json
    [ "$(echo "$output" | jq -r '.people[] | select(.key == "M/Mayhew, Bill") | .linkedin')" = "bill-mayhew" ]
}

@test "pplr sync --plan writes a workbook with the four sheets" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_plan
    out="$PPLR_TEST_DATA/plan.xlsx"
    run "$PPLR_BIN_DIR/pplr" sync --plan --out "$out"
    [ "$status" -eq 0 ]
    assert_contains "$output" "Workbook: $out"
    sheets=$(uv run --quiet --with openpyxl python3 -c "import openpyxl,sys; print(','.join(openpyxl.load_workbook(sys.argv[1]).sheetnames))" "$out")
    [ "$sheets" = "Merged,pplr,Contacts,How to review" ]
}
