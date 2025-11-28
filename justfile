# justfile (repo root)

default:
    @just --list

build:
    cargo build

release:
    cargo build --release

run INPUT:
    cargo run -- {{INPUT}}

runr INPUT:
    cargo run --release -- {{INPUT}}

test:
    cargo test

fmt:
    cargo fmt

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

clean:
    cargo clean

# Run all fixture checks via bash script
check:
    ./scripts/check_fixtures.sh

# Check one pair explicitly
check_one INPUT EXPECTED:
    cargo build --release
    ./target/release/transactions-ledger {{INPUT}} > /tmp/tl_out.csv
    diff -u {{EXPECTED}} /tmp/tl_out.csv
    rm /tmp/tl_out.csv
    @echo "PASS"

