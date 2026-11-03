#!/usr/bin/env bats

# The harness's own guarantees (ST0001 WP01): an honest runner, no deletion
# outside a test's sandbox, and the PPLR_UNDER_TEST seam. Each test runs the
# runner or the helper over a small bats file written into its own sandbox.

load test_helper

# Write a bats file that loads this suite's helper and holds the given tests
nested_suite() {
  local file="$BATS_TEST_TMPDIR/$1.bats"
  shift
  {
    echo "load '$PPLR_ROOT/tests/test_helper'"
    printf '%s\n' "$@"
  } > "$file"
  echo "$file"
}

@test "AT-01.1 the runner exits non-zero when a test fails, and zero when all pass" {
  local failing passing
  failing=$(nested_suite failing '@test "fails" { false; }' '@test "passes" { true; }')
  passing=$(nested_suite passing '@test "passes" { true; }')

  run "$PPLR_ROOT/tests/run_tests.sh" "$failing"
  [ "$status" -ne 0 ]
  assert_contains "$output" "not ok 1 fails"

  run "$PPLR_ROOT/tests/run_tests.sh" "$passing"
  [ "$status" -eq 0 ]
}

@test "AT-01.2 an inherited PPLR_TEST_DATA, or any PPLR_* path, is never touched" {
  local decoy="$BATS_TEST_TMPDIR/decoy" suite
  mkdir -p "$decoy/A"
  echo keep > "$decoy/sentinel"
  echo keep > "$decoy/A/sentinel"
  suite=$(nested_suite inherit \
    '@test "the sandbox replaces every inherited path" {' \
    '  case "$PPLR_TEST_DATA" in "$BATS_TEST_TMPDIR"/*) ;; *) false ;; esac' \
    '  [ "$PPLR_DATA" = "$PPLR_TEST_DATA" ]' \
    '  case "$PPLR_BACKUP_DIR" in "$BATS_TEST_TMPDIR"/*) ;; *) false ;; esac' \
    '  case "$PPLR_CACHE_DIR" in "$BATS_TEST_TMPDIR"/*) ;; *) false ;; esac' \
    '  case "$HOME" in "$BATS_TEST_TMPDIR"/*) ;; *) false ;; esac' \
    '  [ -z "${PPLR_BIN_DIR:-}" ]' \
    '}')

  PPLR_TEST_DATA="$decoy" PPLR_DATA="$decoy" PPLR_DIR="$decoy" \
    PPLR_BACKUP_DIR="$decoy" PPLR_CACHE_DIR="$decoy" PPLR_BIN_DIR="$decoy" \
    run "$PPLR_ROOT/tests/run_tests.sh" "$suite"
  [ "$status" -eq 0 ]
  [ "$(cat "$decoy/sentinel")" = keep ]
  [ "$(cat "$decoy/A/sentinel")" = keep ]
}

@test "AT-01.3 PPLR_UNDER_TEST picks the implementation, and the default is the dev build" {
  local stub="$BATS_TEST_TMPDIR/stub/pplr" suite
  mkdir -p "$(dirname "$stub")"
  printf '#!/bin/bash\necho "stub pplr $*"\n' > "$stub"
  chmod +x "$stub"
  suite=$(nested_suite seam \
    '@test "seam" {' \
    '  if [ -n "${EXPECT:-}" ]; then [ "$PPLR" = "$EXPECT" ]; run "$PPLR" ping; [ "$output" = "stub pplr ping" ];' \
    '  else [ "$PPLR" = "$PPLR_ROOT/target/debug/pplr" ]; fi' \
    '}')

  EXPECT="$stub" PPLR_UNDER_TEST="$stub" run "$PPLR_ROOT/tests/run_tests.sh" "$suite"
  [ "$status" -eq 0 ]

  run env -u PPLR_UNDER_TEST -u EXPECT "$PPLR_ROOT/tests/run_tests.sh" "$suite"
  [ "$status" -eq 0 ]

  PPLR_UNDER_TEST="$BATS_TEST_TMPDIR/missing" run "$PPLR_ROOT/tests/run_tests.sh" "$suite"
  [ "$status" -ne 0 ]
  assert_contains "$output" "PPLR_UNDER_TEST is not an executable file"

  # A checkout with no dev build fails the suite, naming cargo build; it never skips
  local bare="$BATS_TEST_TMPDIR/bare"
  mkdir -p "$bare/tests"
  cp "$PPLR_ROOT/tests/test_helper.bash" "$bare/tests/"
  printf '%s\n' "load '$bare/tests/test_helper'" '@test "needs a pplr" { true; }' > "$bare/tests/needs.bats"
  run env -u PPLR_UNDER_TEST "$PPLR_ROOT/tests/run_tests.sh" "$bare/tests/needs.bats"
  [ "$status" -ne 0 ]
  assert_contains "$output" "cargo build makes it"
}
