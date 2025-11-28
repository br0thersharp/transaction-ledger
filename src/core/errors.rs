use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    ParseAmount,
    NegativeAmount,
    UnknownTransactionType,
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::ParseAmount => write!(f, "failed to parse amount"),
            CoreError::NegativeAmount => write!(f, "amount must be non-negative"),
            CoreError::UnknownTransactionType => write!(f, "unknown transaction type"),
        }
    }
}

impl std::error::Error for CoreError {}

#[derive(Debug)]
pub enum LedgerError {
    InsufficientFunds,
    TxAlreadyDisputed,
    TxNotDisputed,
    TxWrongClient,
    Overflow,
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedgerError::InsufficientFunds => write!(f, "insufficient funds"),
            LedgerError::TxAlreadyDisputed => write!(f, "transaction already disputed"), // for disputing the same tx twice
            LedgerError::TxNotDisputed => write!(f, "transaction not disputed"), // for performing a chargeback or a resolve on a non-disputed tx
            LedgerError::TxWrongClient => write!(f, "transaction-client mismatch"),
            LedgerError::Overflow => write!(f, "arithmetic overflow"), // could happen if amount is over 900 trillion
        }
    }
}

impl std::error::Error for LedgerError {}
