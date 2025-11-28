#[derive(Debug, Default, Clone)]
pub struct Metrics {
    pub malformed_rows: u64,
    pub unknown_type: u64,
    pub missing_amount: u64,
    pub duplicate_tx: u64,
    pub tx_not_found: u64,
    pub wrong_client_ref: u64,
    pub ledger_errors: u64,
    pub locked_ignored: u64,
}
