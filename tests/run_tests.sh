#!/bin/bash

# Run the bats suite, and exit with bats' own status.
# Usage: ./run_tests.sh [specific-test.bats ...]
#
# Each test makes and removes its own sandbox (see test_helper.bash), so this
# runner sets no data paths and deletes nothing. PPLR_UNDER_TEST names the pplr
# executable to test; it defaults to the checkout's bin/pplr.

TEST_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$TEST_DIR" || exit 1

if ! command -v bats >/dev/null 2>&1; then
  echo "Error: bats is not installed" >&2
  echo "Install with: brew install bats-core (macOS) or see https://github.com/bats-core/bats-core" >&2
  exit 1
fi

if [ $# -eq 0 ]; then
  echo "Running all tests against ${PPLR_UNDER_TEST:-bin/pplr}..."
  exec bats ./*.bats
fi

echo "Running $* against ${PPLR_UNDER_TEST:-bin/pplr}..."
exec bats "$@"
