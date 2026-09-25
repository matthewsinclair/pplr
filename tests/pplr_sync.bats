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

@test "pplr sync reads several phones apart by a middle dot" {
    setup_contacts
    make_about Carron Elise "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:    +262 262 90 00 00 — switchboard · +262 692 90 00 01 — mobile')"
    run "$PPLR_BIN_DIR/pplr" sync --plan --json
    [ "$status" -eq 0 ]
    [ "$(echo "$output" | jq -r '.people[] | select(.key == "C/Carron, Elise") | .phones | join(",")')" = "+262262900000,+262692900001" ]
}

@test "pplr rename moves the person, renames their files, and repoints links" {
    make_about Grifiths Glen "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    touch "$PPLR_TEST_DATA/G/Grifiths, Glen/About/Glen Grifiths (Picture).jpg"
    refs="$PPLR_TEST_DATA/refs"; mkdir -p "$refs"
    printf '%s\n' '[Glen](<../G/Grifiths, Glen/About/Glen Grifiths (About).md>)' 'http://x/people/G/Grifiths%2C%20Glen/About/Glen%20Grifiths%20%28About%29.md' 'Met Glen Grifiths today' > "$refs/notes.md"
    run "$PPLR_BIN_DIR/pplr" rename "Grifiths, Glen" "Griffiths, Glen" --refs "$refs"
    [ "$status" -eq 0 ]
    [ -f "$PPLR_TEST_DATA/G/Griffiths, Glen/About/Glen Griffiths (About).md" ]
    [ -f "$PPLR_TEST_DATA/G/Griffiths, Glen/About/Glen Griffiths (Picture).jpg" ]
    [ ! -e "$PPLR_TEST_DATA/G/Grifiths, Glen" ]
    grep -q "^_Glen Griffiths_" "$PPLR_TEST_DATA/G/Griffiths, Glen/About/Glen Griffiths (About).md"
    [ "$(cat "$PPLR_TEST_DATA/G/Griffiths, Glen/.index/aliases")" = "G/Grifiths, Glen" ]
    grep -qF '../G/Griffiths, Glen/About/Glen Griffiths (About).md' "$refs/notes.md"
    grep -qF 'G/Griffiths%2C%20Glen/About/Glen%20Griffiths%20%28About%29.md' "$refs/notes.md"
    # running text is listed, not changed
    grep -qF 'Met Glen Grifiths today' "$refs/notes.md"
    assert_contains "$output" "Met Glen Grifiths today"
}

@test "pplr rename --dry-run changes nothing" {
    make_about Grifiths Glen "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    run "$PPLR_BIN_DIR/pplr" rename "Grifiths, Glen" "Griffiths, Glen" --dry-run --refs "$PPLR_TEST_DATA"
    [ "$status" -eq 0 ]
    [ -d "$PPLR_TEST_DATA/G/Grifiths, Glen" ]
    [ ! -e "$PPLR_TEST_DATA/G/Griffiths, Glen" ]
}

@test "pplr sync finds a renamed person by their old pplr URL, and replaces it" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    jq '(.[] | select(.id == "c3")).urls = [{"label": "pplr", "value": "pplr://t/turing-alan"}]' "$PPLR_CONTACTS_JSON" > "$PPLR_TEST_DATA/c.json"
    mv "$PPLR_TEST_DATA/c.json" "$PPLR_CONTACTS_JSON"
    mkdir -p "$PPLR_TEST_DATA/none"
    "$PPLR_BIN_DIR/pplr" rename "Turing, Alan" "Turing, Alan Mathison" --refs "$PPLR_TEST_DATA/none" >/dev/null
    run "$PPLR_BIN_DIR/pplr" sync --check --json
    [ "$(echo "$output" | jq -r '.linked[] | select(.contact == "c3") | "\(.person) \(.markerStale)"')" = "T/Turing, Alan Mathison true" ]
    "$PPLR_BIN_DIR/pplr" sync --link --apply >/dev/null
    [ "$(jq -r '.[] | select(.id == "c3") | [.urls[] | select(.label == "pplr") | .value] | join(",")' "$PPLR_CONTACTS_JSON")" = "pplr://t/turing-alan-mathison" ]
}

@test "pplr rename finds links under a refs directory reached through a symlink" {
    make_about Grifiths Glen "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    mkdir -p "$PPLR_TEST_DATA/real"; ln -s "$PPLR_TEST_DATA/real" "$PPLR_TEST_DATA/link"
    echo '[G](<G/Grifiths, Glen/About/Glen Grifiths (About).md>)' > "$PPLR_TEST_DATA/real/n.md"
    run "$PPLR_BIN_DIR/pplr" rename "Grifiths, Glen" "Griffiths, Glen" --refs "$PPLR_TEST_DATA/link"
    [ "$status" -eq 0 ]
    grep -qF 'G/Griffiths, Glen/About/Glen Griffiths (About).md' "$PPLR_TEST_DATA/real/n.md"
}

@test "pplr rename to a name that contains the old one rewrites each link once" {
    make_about Varley Hanna "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    mkdir -p "$PPLR_TEST_DATA/refs"
    echo '[H](<V/Varley, Hanna/About/Hanna Varley (About).md>) `V/Varley, Hanna`' > "$PPLR_TEST_DATA/refs/n.md"
    run "$PPLR_BIN_DIR/pplr" rename "Varley, Hanna" "Varley, Hannah" --refs "$PPLR_TEST_DATA/refs"
    [ "$status" -eq 0 ]
    [ "$(cat "$PPLR_TEST_DATA/refs/n.md")" = '[H](<V/Varley, Hannah/About/Hannah Varley (About).md>) `V/Varley, Hannah`' ]
    ! grep -q "Hannahh" "$PPLR_TEST_DATA/V/Varley, Hannah/About/Hannah Varley (About).md"
}

@test "pplr resolve finds a person, a path in their folder, and a former name" {
    make_about Kemp Jon "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    mkdir -p "$PPLR_TEST_DATA/K/Kemp, Jon/Meetings/20260924 Intro"; touch "$PPLR_TEST_DATA/K/Kemp, Jon/Meetings/20260924 Intro/Summary.md"
    run "$PPLR_BIN_DIR/pplr" resolve "pplr://k/kemp-jon"
    [ "$status" -eq 0 ]; [ "$output" = "$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (About).md" ]
    run "$PPLR_BIN_DIR/pplr" resolve "pplr://k/kemp-jon/Meetings/20260924%20Intro/Summary.md"
    [ "$output" = "$PPLR_TEST_DATA/K/Kemp, Jon/Meetings/20260924 Intro/Summary.md" ]
    echo "K/Kraf, Jon" > "$PPLR_TEST_DATA/K/Kemp, Jon/.index/aliases" 2>/dev/null || { mkdir -p "$PPLR_TEST_DATA/K/Kemp, Jon/.index"; echo "K/Kraf, Jon" > "$PPLR_TEST_DATA/K/Kemp, Jon/.index/aliases"; }
    run "$PPLR_BIN_DIR/pplr" resolve "pplr://k/kraf-jon"
    [ "$output" = "$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (About).md" ]
    run "$PPLR_BIN_DIR/pplr" resolve "pplr://x/nobody"
    [ "$status" -ne 0 ]
}

@test "pplr links rewrites path and CMS links as pplr:// URLs, and leaves the unknown alone" {
    make_about Kemp Jon "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    mkdir -p "$PPLR_TEST_DATA/K/Kemp, Jon/Meetings/20260924 Intro"; touch "$PPLR_TEST_DATA/K/Kemp, Jon/Meetings/20260924 Intro/Summary.md"
    notes="$PPLR_TEST_DATA/journal/2026/notes.md"; mkdir -p "$(dirname "$notes")"
    printf '%s\n' \
      '[Jon](<../../Career/People/K/Kemp, Jon/About/Jon Kemp (About).md>)' \
      '[Summary](<../../Career/People/K/Kemp, Jon/Meetings/20260924 Intro/Summary.md>)' \
      '[Jon](http://localhost:4360/people/K/Kemp%2C%20Jon/About/Jon%20Kemp%20%28About%29.md)' \
      '[Gone](<../../Career/People/G/Gone, Person/About/Person Gone (About).md>)' \
      '[Elsewhere](https://example.com/x)' > "$notes"
    run "$PPLR_BIN_DIR/pplr" links "$PPLR_TEST_DATA/journal"
    [ "$status" -eq 0 ]
    assert_contains "$output" "to convert                  3"
    assert_contains "$output" "no such person: G/Gone, Person"
    grep -qF 'Career/People/K/Kemp' "$notes"
    run "$PPLR_BIN_DIR/pplr" links "$PPLR_TEST_DATA/journal" --apply
    [ "$(sed -n 1p "$notes")" = '[Jon](pplr://k/kemp-jon)' ]
    [ "$(sed -n 2p "$notes")" = '[Summary](<pplr://k/kemp-jon/Meetings/20260924 Intro/Summary.md>)' ]
    [ "$(sed -n 3p "$notes")" = '[Jon](pplr://k/kemp-jon)' ]
    [ "$(sed -n 4p "$notes")" = '[Gone](<../../Career/People/G/Gone, Person/About/Person Gone (About).md>)' ]
    [ "$(sed -n 5p "$notes")" = '[Elsewhere](https://example.com/x)' ]
}

@test "pplr open --print names the file when no CMS answers" {
    make_about Kemp Jon "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    PPLR_CMS_URL="" run "$PPLR_BIN_DIR/pplr" open --print "pplr://k/kemp-jon"
    [ "$status" -eq 0 ]
    [ "$output" = "$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (About).md" ]
}

@test "pplr links follows a symlinked note to its file, once, and keeps the symlink" {
    make_about Kemp Jon "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    j="$PPLR_TEST_DATA/journal"; mkdir -p "$j"
    echo '[Jon](<../../Career/People/K/Kemp, Jon/About/Jon Kemp (About).md>)' > "$j/20260901 Day Notes.md"
    ln -s "20260901 Day Notes.md" "$j/aa_day_notes.md"
    run "$PPLR_BIN_DIR/pplr" links "$j" --apply
    [ "$status" -eq 0 ]
    assert_contains "$output" "converted                   1"
    [ -L "$j/aa_day_notes.md" ]
    [ "$(cat "$j/20260901 Day Notes.md")" = '[Jon](pplr://k/kemp-jon)' ]
}

@test "pplr links takes a path relative to where it was run" {
    make_about Kemp Jon "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    mkdir -p "$PPLR_TEST_DATA/elsewhere"
    echo '[Jon](<../../Career/People/K/Kemp, Jon/About/Jon Kemp (About).md>)' > "$PPLR_TEST_DATA/elsewhere/n.md"
    cd "$PPLR_TEST_DATA/elsewhere"
    run "$PPLR_BIN_DIR/pplr" links n.md
    assert_contains "$output" "to convert                  1"
}

@test "pplr pictures puts the photo under the About title once, and skips people with none" {
    make_about Kemp Jon "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    make_about Nobody Nemo "$(printf '%s\n' '- Role:' '- Company:' '- LinkedIn:' '- Email:' '- Phone:')"
    touch "$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (Picture).jpg"
    run "$PPLR_BIN_DIR/pplr" pictures
    [ "$status" -eq 0 ]
    assert_contains "$output" "1 added"
    a="$PPLR_TEST_DATA/K/Kemp, Jon/About/Jon Kemp (About).md"
    [ "$(sed -n 1p "$a")" = "# Jon Kemp (About)" ]
    [ "$(sed -n 3p "$a")" = '<img src="Jon Kemp (Picture).jpg" alt="Jon Kemp" width="160" align="right">' ]
    ! grep -q "<img" "$PPLR_TEST_DATA/N/Nobody, Nemo/About/Nemo Nobody (About).md"
    run "$PPLR_BIN_DIR/pplr" pictures
    assert_contains "$output" "0 added, 0 updated, 1 already there"
    [ "$(grep -c "<img" "$a")" -eq 1 ]
}

@test "pplr sync --apply-plan is a dry run by default, and changes nothing" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_contacts
    "$PPLR_BIN_DIR/pplr" sync --plan --out "$PPLR_TEST_DATA/plan.xlsx" >/dev/null
    before=$(cat "$PPLR_CONTACTS_JSON")
    run "$PPLR_BIN_DIR/pplr" sync --apply-plan "$PPLR_TEST_DATA/plan.xlsx"
    [ "$status" -eq 0 ]
    assert_contains "$output" "dry run"
    assert_contains "$output" "add (new iCloud cards)  1"
    assert_contains "$output" "N/Nobody, Nemo"
    [ "$(cat "$PPLR_CONTACTS_JSON")" = "$before" ]
}

@test "pplr sync --apply-plan --apply adds new cards and updates matched ones, removing nothing" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    "$PPLR_BIN_DIR/pplr" sync --plan --out "$PPLR_TEST_DATA/plan.xlsx" >/dev/null
    run "$PPLR_BIN_DIR/pplr" sync --apply-plan "$PPLR_TEST_DATA/plan.xlsx" --apply
    [ "$status" -eq 0 ]
    [ "$(ls "$PPLR_BACKUP_DIR" | wc -l | tr -d ' ')" -eq 1 ]
    # Nemo Nobody had no card: now one, in the group, with the pplr URL
    nemo=$(jq -r '.[] | select(.family == "Nobody") | "\(.groups | join(",")) \([.urls[] | select(.label == "pplr") | .value] | join(","))"' "$PPLR_CONTACTS_JSON")
    [ "$nemo" = "PPLR pplr://n/nobody-nemo" ]
    # Grace Hopper (same name): her role filled in, her own email kept, the pplr URL added
    grace=$(jq -r '.[] | select(.id == "c2") | "\(.jobTitle)|\(.emails | join(","))|\([.urls[] | select(.label == "pplr") | .value] | join(","))"' "$PPLR_CONTACTS_JSON")
    [ "$grace" = "Rear Admiral|grace@work.example|pplr://h/hopper-grace" ]
    [ -f "$PPLR_TEST_DATA/N/Nobody, Nemo/About/Nemo Nobody (Contacts).webloc" ]
    # Applying again finds nothing left to do, and never adds a card twice
    run "$PPLR_BIN_DIR/pplr" sync --apply-plan "$PPLR_TEST_DATA/plan.xlsx"
    assert_contains "$output" "add (new iCloud cards)  0"
    assert_contains "$output" "update                  0"
}

@test "pplr sync --apply-plan --name writes only the named people" {
    command -v uv >/dev/null 2>&1 || skip "uv not installed"
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    "$PPLR_BIN_DIR/pplr" sync --plan --out "$PPLR_TEST_DATA/plan.xlsx" >/dev/null
    run "$PPLR_BIN_DIR/pplr" sync --apply-plan "$PPLR_TEST_DATA/plan.xlsx" --apply --name "Hopper, Grace"
    [ "$status" -eq 0 ]
    assert_contains "$output" "written now             1"
    [ "$(jq '[.[] | select(.family == "Nobody")] | length' "$PPLR_CONTACTS_JSON")" -eq 0 ]
}

@test "pplr sync treats Gmail addresses with and without dots as one address" {
    setup_contacts
    jq '(.[] | select(.id == "c2")).emails = ["grace.hopper@gmail.com"]' "$PPLR_CONTACTS_JSON" > "$PPLR_TEST_DATA/c.json" && mv "$PPLR_TEST_DATA/c.json" "$PPLR_CONTACTS_JSON"
    make_about Hopper Grace "$(printf '%s\n' '- Role:     Rear Admiral' '- Company:  Navy' '- LinkedIn:' '- Email:    [gracehopper+navy@gmail.com](mailto:gracehopper+navy@gmail.com)' '- Phone:')"
    run "$PPLR_BIN_DIR/pplr" sync --check --json
    [ "$(echo "$output" | jq -r '.matched[] | select(.person == "H/Hopper, Grace") | .how')" = "email" ]
    [ "$(echo "$output" | jq '[.matched[] | select(.person == "H/Hopper, Grace") | .diffs[] | select(.field == "email")] | length')" -eq 0 ]
}

@test "pplr sync --photos only picks linked cards that have no photo" {
    setup_contacts
    export PPLR_BACKUP_DIR="$PPLR_TEST_DATA/backups"
    jq '(.[] | select(.id == "c1")).urls = [{"label": "pplr", "value": "pplr://l/lovelace-ada"}] | (.[] | select(.id == "c3")).hasImage = true' "$PPLR_CONTACTS_JSON" > "$PPLR_TEST_DATA/c.json" && mv "$PPLR_TEST_DATA/c.json" "$PPLR_CONTACTS_JSON"
    touch "$PPLR_TEST_DATA/L/Lovelace, Ada/About/Ada Lovelace (Picture).jpg" "$PPLR_TEST_DATA/T/Turing, Alan/About/Alan Turing (Picture).jpg"
    run "$PPLR_BIN_DIR/pplr" sync --photos
    [ "$status" -eq 0 ]
    assert_contains "$output" "L/Lovelace, Ada"
    [[ "$output" != *"Turing"* ]]
    run "$PPLR_BIN_DIR/pplr" sync --photos --apply
    [ "$(jq -r '.[] | select(.id == "c1") | .hasImage' "$PPLR_CONTACTS_JSON")" = "true" ]
}
