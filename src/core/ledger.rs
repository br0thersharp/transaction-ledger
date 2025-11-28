use crate::core::errors::LedgerError;
use crate::core::types::Amount;
use crate::engine::state::{AccountState, TxKind, TxRecord};

pub fn deposit(account: &mut AccountState, amount: Amount) -> Result<(), LedgerError> {
    let new_available = account.available.checked_add(amount)?;
    account.available = new_available;
    Ok(())
}

pub fn withdrawal(account: &mut AccountState, amount: Amount) -> Result<(), LedgerError> {
    if account.available < amount {
        return Err(LedgerError::InsufficientFunds);
    }
    let new_available = account.available.checked_sub(amount)?;
    account.available = new_available;
    Ok(())
}

pub fn dispute(account: &mut AccountState, rec: &mut TxRecord) -> Result<(), LedgerError> {
    if rec.disputed {
        return Err(LedgerError::TxAlreadyDisputed);
    }
    if rec.kind != TxKind::Deposit {
        return Err(LedgerError::DisputeOnWithdrawal);
    }
    account.available = account.available.checked_sub(rec.amount)?;
    account.held = account.held.checked_add(rec.amount)?;
    rec.disputed = true;

    Ok(())
}

pub fn resolve(account: &mut AccountState, rec: &mut TxRecord) -> Result<(), LedgerError> {
    if !rec.disputed {
        return Err(LedgerError::TxNotDisputed);
    }
    if rec.kind != TxKind::Deposit {
        // should be unreachable but just in case
        return Err(LedgerError::DisputeOnWithdrawal);
    }
    account.available = account.available.checked_add(rec.amount)?;
    account.held = account.held.checked_sub(rec.amount)?;
    rec.disputed = false;

    Ok(())
}

pub fn chargeback(account: &mut AccountState, rec: &mut TxRecord) -> Result<(), LedgerError> {
    if !rec.disputed {
        return Err(LedgerError::TxNotDisputed);
    }
    if rec.kind != TxKind::Deposit {
        return Err(LedgerError::DisputeOnWithdrawal);
    }
    rec.disputed = false;

    account.held = account.held.checked_sub(rec.amount)?;
    account.locked = true;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Amount;
    use crate::engine::{AccountState, TxKind, TxRecord};

    fn amt(s: &str) -> Amount {
        Amount::from_str_4dp(s).unwrap()
    }

    fn acct(avail: &str, held: &str, locked: bool) -> AccountState {
        AccountState {
            available: amt(avail),
            held: amt(held),
            locked,
        }
    }

    fn dep_record(client: u16, amount: &str) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Deposit,
            amount: amt(amount),
            disputed: false,
        }
    }

    fn wd_record(client: u16, amount: &str) -> TxRecord {
        TxRecord {
            client,
            kind: TxKind::Withdrawal,
            amount: amt(amount),
            disputed: false,
        }
    }

    #[test]
    fn deposit_increases_available_and_total() {
        let mut a = acct("0.0000", "0.0000", false);
        deposit(&mut a, amt("1.5000")).unwrap();

        assert_eq!(a.available, amt("1.5000"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("1.5000"));
    }

    #[test]
    fn withdrawal_decreases_available_if_sufficient() {
        let mut a = acct("2.0000", "0.0000", false);
        withdrawal(&mut a, amt("1.2500")).unwrap();

        assert_eq!(a.available, amt("0.7500"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("0.7500"));
    }

    #[test]
    fn withdrawal_fails_if_insufficient() {
        let mut a = acct("1.0000", "0.0000", false);
        let res = withdrawal(&mut a, amt("1.0001"));

        assert!(res.is_err());
        assert_eq!(a.available, amt("1.0000"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("1.0000"));
    }

    #[test]
    fn dispute_on_deposit_moves_available_to_held_total_unchanged() {
        let mut a = acct("2.0000", "0.0000", false);
        let mut rec = dep_record(1, "1.5000");

        dispute(&mut a, &mut rec).unwrap();

        assert!(rec.disputed);
        assert_eq!(a.available, amt("0.5000"));
        assert_eq!(a.held, amt("1.5000"));
        assert_eq!(a.total(), amt("2.0000"));
    }

    #[test]
    fn dispute_on_withdrawal_is_rejected_and_no_change() {
        let mut a = acct("2.0000", "0.0000", false);
        let mut rec = wd_record(1, "1.0000");

        let res = dispute(&mut a, &mut rec);
        assert!(matches!(res, Err(LedgerError::DisputeOnWithdrawal)));

        assert!(!rec.disputed);
        assert_eq!(a.available, amt("2.0000"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("2.0000"));
    }

    #[test]
    fn resolve_releases_held_back_to_available() {
        let mut a = acct("0.5000", "1.5000", false);
        let mut rec = dep_record(1, "1.5000");
        rec.disputed = true;

        resolve(&mut a, &mut rec).unwrap();

        assert!(!rec.disputed);
        assert_eq!(a.available, amt("2.0000"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("2.0000"));
    }

    #[test]
    fn chargeback_reduces_held_and_total_and_locks() {
        let mut a = acct("0.5000", "1.5000", false);
        let mut rec = dep_record(1, "1.5000");
        rec.disputed = true;

        chargeback(&mut a, &mut rec).unwrap();

        assert!(a.locked);
        assert_eq!(a.available, amt("0.5000"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("0.5000"));
        assert!(!rec.disputed);
    }

    #[test]
    fn dispute_twice_errors() {
        let mut a = acct("2.0000", "0.0000", false);
        let mut rec = dep_record(1, "1.0000");

        dispute(&mut a, &mut rec).unwrap();
        let res = dispute(&mut a, &mut rec);

        assert!(res.is_err());
        assert_eq!(a.available, amt("1.0000"));
        assert_eq!(a.held, amt("1.0000"));
    }

    #[test]
    fn resolve_without_dispute_errors_and_no_change() {
        let mut a = acct("2.0000", "0.0000", false);
        let mut rec = dep_record(1, "1.0000");

        let res = resolve(&mut a, &mut rec);
        assert!(res.is_err());

        assert_eq!(a.available, amt("2.0000"));
        assert_eq!(a.held, amt("0.0000"));
    }

    #[test]
    fn chargeback_without_dispute_errors_and_no_change() {
        let mut a = acct("2.0000", "0.0000", false);
        let mut rec = dep_record(1, "1.0000");

        let res = chargeback(&mut a, &mut rec);
        assert!(res.is_err());

        assert!(!a.locked);
        assert_eq!(a.available, amt("2.0000"));
        assert_eq!(a.held, amt("0.0000"));
        assert_eq!(a.total(), amt("2.0000"));
    }
}
