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

- client: the client ID
- available: funds available for withdrawal/trading
- held: funds held due to disputes
- total = available + held
- locked: true if a chargeback occurred

All numeric values are printed with 4 decimal places.

## Design overview

The code is split into three layers:

- `io`: CSV ingester and emitter. Produces an event stream for the engine to consume and process.
- `engine`: orchestrates processing, owns state, tx store, and metrics.
- `core`: domain types (Amount, Transaction) and pure ledger rules.

Processing is streaming. The CSV is read row by row and applied immediately. No full file buffering.

### Safety and Robustness

My choice of datastore is a simple `HashMap` wrapped in an impl for `TxStore`. It would have been prefereable to use something like `rusqlite` since this would have allowed for atomic updates to `available` and `held` data

### Correctness

Most of the business logic is in the `core/ledger.rs` file that accounts for all the rules of state change, and in `core/types.rs` that contains the logic for supporting a decimal type with 4 decimal points of precision. For both of these I have added ample unit tests covering the categories I could think of
Additionally I have added property tests in `tests/proptests.rs` that create fuzzy input sets and run the service while making sure that internal invariants don't drift regardless of input

### State

For each client:

- available: Amount
- held: Amount
- locked: bool

For each referenced tx:

- client id
- amount
- kind (deposit or withdrawal)
- disputed flag

Txs are stored in an in-memory HashMap store. This is abstracted behind a `TxStore` trait so a different backend can be swapped in later.

### Metrics

Non-fatal anomalies are counted in `engine::metrics::Metrics`, including:

- malformed rows
- unknown transaction types
- missing amounts on deposit/withdrawal
- duplicate tx ids
- disputes on missing tx ids
- wrong-client references
- ledger rule failures
- operations ignored after lock

Metrics can be printed to stderr (currently commented out) at the end so stdout remains clean CSV output.

## Assumptions and edge cases

- Dispute, resolve, and chargeback reference a previous tx by id. If the id does not exist, the event is ignored and an error metric tabulated.
- Resolve and chargeback are ignored if the referenced tx is not currently disputed.
- Disputes only apply to deposits
  - The spec only specifies "transactions" but conceptually, it doesn't make sense to dispute a withdrawal, it doesn't fit the spirit of the spec
  - It's also dangerous allowing someone to potentially withdraw the account's full balance twice
- After a chargeback, the account is locked and all subsequent transactions for that client are ignored.

## Testing

Unit tests cover:

- Amount parsing and formatting, including up-to-4 decimal precision and whitespace tolerance.
- Checked arithmetic overflow detection.
- Basic ledger correctness.

To run unit tests:

```bash
just test
```

Integration tests cover:

- End to end scenarios meant to simulate the entire system working on real-world data

To run integration tests:

```bash
just check
```