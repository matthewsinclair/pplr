#!/bin/bash

# A simple test script to diagnose meeting finding issues
# Usage: ./test_meetings.sh YYYY[MM[DD]]

# Default location of PPLR_DIR if not already set
: "${PPLR_DIR:=$HOME/Dropbox/Career/People}"

# Normalize the date
normalize_date() {
  local input_date=$1
  
  # Handle different formats
  if [[ ${#input_date} -eq 4 ]]; then
    # YYYY format - first day of year
    echo "${input_date}0101"
  elif [[ ${#input_date} -eq 6 ]]; then
    # YYYYMM format - first day of month
    echo "${input_date}01"
  elif [[ ${#input_date} -eq 8 ]]; then
    # Full date
    echo "$input_date"
  else
    echo "Error: Invalid date format"
    exit 1
  fi
}

# Check if date argument is provided
if [ $# -lt 1 ]; then
  echo "Usage: $0 YYYY[MM[DD]]"
  exit 1
fi

# Get year from input
input_year=${1:0:4}
echo "Looking for meetings in year: $input_year"

# Convert input to a standardized date for comparison
start_date=$(normalize_date "$1")
echo "Normalized start date: $start_date"

# Change to PPLR_DIR
cd "$PPLR_DIR" || {
  echo "Error: Could not change to directory $PPLR_DIR."
  exit 1
}

echo "Searching in: $PPLR_DIR"
echo "Finding all meeting directories first..."

# Find all meeting directories with 8-digit date format
meeting_dirs=$(find . -path "*/Meetings/*" -type d | grep -E '/[0-9]{8}' | sort)

echo "Found $(echo "$meeting_dirs" | wc -l | xargs) meeting directories."

# Process each meeting directory
found_count=0

echo "Checking each meeting date..."
while IFS= read -r dir; do
  # Extract date from directory name
  date_part=$(basename "$dir" | grep -o "^[0-9]\{8\}")
  
  # If no date found, skip
  [ -z "$date_part" ] && continue
  
  # Extract year part
  year_part=${date_part:0:4}
  
  # Check if it's the target year
  if [ "$year_part" = "$input_year" ]; then
    echo "Found meeting: $dir (date: $date_part)"
    ((found_count++))
  fi
done <<< "$meeting_dirs"

echo "Total meetings found for $input_year: $found_count"