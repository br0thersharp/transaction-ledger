use crate::core::errors::LedgerError;
use crate::core::types::Amount;
use crate::engine::state::{AccountState, TxKind, TxRecord};

pub fn deposit(account: &mut AccountState, amount: Amount) -> Result<(), LedgerError> {
    account.available = account.available.checked_add(amount)?;
    Ok(())
}

pub fn withdrawal(account: &mut AccountState, amount: Amount) -> Result<(), LedgerError> {
    if account.available < amount {
        return Err(LedgerError::InsufficientFunds);
    }
    account.available = account.available.checked_sub(amount)?;
    Ok(())
}

pub fn dispute(account: &mut AccountState, rec: &mut TxRecord) -> Result<(), LedgerError> {
    if rec.disputed {
        return Err(LedgerError::TxAlreadyDisputed);
    }
    rec.disputed = true;

    match rec.kind {
        TxKind::Deposit => {
            account.available = account.available.checked_sub(rec.amount)?;
            account.held = account.held.checked_add(rec.amount)?;
        }
        TxKind::Withdrawal => {
            account.available = account.available.checked_add(rec.amount)?;
            account.held = account.held.checked_add(rec.amount)?;
        }
    }
    Ok(())
}

pub fn resolve(account: &mut AccountState, rec: &mut TxRecord) -> Result<(), LedgerError> {
    if !rec.disputed {
        return Err(LedgerError::TxNotDisputed);
    }
    rec.disputed = false;

    match rec.kind {
        TxKind::Deposit => {
            account.available = account.available.checked_add(rec.amount)?;
            account.held = account.held.checked_sub(rec.amount)?;
        }
        TxKind::Withdrawal => {
            account.available = account.available.checked_sub(rec.amount)?;
            account.held = account.held.checked_sub(rec.amount)?;
        }
    }
    Ok(())
}

pub fn chargeback(account: &mut AccountState, rec: &mut TxRecord) -> Result<(), LedgerError> {
    if !rec.disputed {
        return Err(LedgerError::TxNotDisputed);
    }
    rec.disputed = false;

    account.held = account.held.checked_sub(rec.amount)?;
    account.locked = true;

    Ok(())
}
