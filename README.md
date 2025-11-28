# transactions-ledger

A small payments engine that reads a CSV of transactions, applies them in chronological order, and prints final client account states as CSV to stdout.

The design for this project is intentionally simple and the focus is on correctness, legibility, safety, and efficiency

## Build and run

```bash
cargo build
cargo run -- transactions.csv > accounts.csv
```

The program accepts exactly one argument: the input CSV path. Output is written to stdout.

## Input format

CSV columns:

- type: one of `deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`
- client: u16 client id
- tx: u32 transaction id (globally unique)
- amount: decimal with up to 4 places after the decimal point. Required for deposit and withdrawal only.

Rows are assumed to be in chronological order.

Whitespace around fields is accepted.

## Output format

CSV columns:

- client
- available: funds available for withdrawal/trading
- held: funds held due to disputes
- total = available + held
- locked: true if a chargeback occurred