#!/usr/bin/env bash
set -euo pipefail

echo "Building release binary..."
cargo build --release >/dev/null

BIN="./target/release/transactions-ledger"
TMP="$(mktemp -t tl_out.XXXXXX.csv)"

# Collect inputs in a Bash 3.2 friendly way
inputs=()
while IFS= read -r f; do
  inputs+=("$f")
done < <(find fixtures -type f -name 'input_*.csv' | sort)

if [ "${#inputs[@]}" -eq 0 ]; then
  echo "No fixture inputs found under fixtures/"
  rm -f "$TMP"
  exit 1
fi

for in_file in "${inputs[@]}"; do
  exp_file="${in_file/input_/expected_}"

  if [ ! -f "$exp_file" ]; then
    echo "Missing expected file for $in_file"
    echo "Looked for: $exp_file"
    rm -f "$TMP"
    exit 1
  fi

  echo "Checking $in_file"
  "$BIN" "$in_file" > "$TMP"

  if diff -u "$exp_file" "$TMP" >/dev/null; then
    echo "  PASS"
  else
    echo "  FAIL"
    diff -u "$exp_file" "$TMP" || true
    rm -f "$TMP"
    exit 1
  fi
done

rm -f "$TMP"
echo "All fixtures passed."

