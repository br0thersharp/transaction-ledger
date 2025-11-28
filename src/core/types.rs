use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use crate::core::errors::{CoreError, LedgerError};

pub type ClientId = u16;
pub type TxId = u32;

/// Fixed-precision amount newtype in 10^-4 units.
/// Stored as scaled i64 to avoid underflow hazards during subtraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Amount(i64);

impl Amount {
    pub const SCALE: i64 = 10_000;

    pub fn zero() -> Self { Amount(0) }

    pub fn as_i64(self) -> i64 { self.0 }

    pub fn from_str_4dp(s: &str) -> Result<Self, CoreError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(CoreError::ParseAmount);
        }

        if s.starts_with('-') {
            return Err(CoreError::NegativeAmount);
        }

        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() > 2 {
            return Err(CoreError::ParseAmount);
        }

        let whole_str = parts[0];
        if whole_str.is_empty() || !whole_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(CoreError::ParseAmount);
        }

        let whole: i64 = whole_str
            .parse()
            .map_err(|_| CoreError::ParseAmount)?;

        // allow "1." or no decimals as "1.0000"
        let decimals_str = if parts.len() == 2 { parts[1] } else { "" };
        let decimals_len = decimals_str.len();
        // we're allowed to assume that there are 4 decimal points precision, but if for some reason there are more, it's an error
        // I could truncate this but silent truncation is changing money and rounding rules weren't specified
        if decimals_len > 4 {
            return Err(CoreError::ParseAmount);
        }

        if decimals_len == 0 {
            return Ok(Amount(whole * Self::SCALE));
        }

        if !decimals_str.chars().all(|c| c.is_ascii_digit()) {
            return Err(CoreError::ParseAmount);
        }

        let mut decimals: i64 = decimals_str
            .parse()
            .map_err(|_| CoreError::ParseAmount)?;

        for _ in 0..(4 - decimals_len) {
            decimals *= 10;
        }

        Ok(Amount(whole * Self::SCALE + decimals))
    }

    pub fn checked_add(self, rhs: Amount) -> Result<Amount, LedgerError> {
        self.0
            .checked_add(rhs.0)
            .map(Amount)
            .ok_or(LedgerError::Overflow)
    }

    pub fn checked_sub(self, rhs: Amount) -> Result<Amount, LedgerError> {
        self.0
            .checked_sub(rhs.0)
            .map(Amount)
            .ok_or(LedgerError::Overflow)
    }
}

// Unchecked arithmetic for convenience only.
// Do NOT use these in ledger/state mutations; use checked_* instead.
impl Add for Amount {
    type Output = Amount;
    fn add(self, rhs: Amount) -> Amount { Amount(self.0 + rhs.0) }
}
impl Sub for Amount {
    type Output = Amount;
    fn sub(self, rhs: Amount) -> Amount { Amount(self.0 - rhs.0) }
}
impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Amount) { self.0 += rhs.0; }
}
impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Amount) { self.0 -= rhs.0; }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / Self::SCALE;
        let frac = (self.0 % Self::SCALE).abs();
        write!(f, "{}.{:04}", whole, frac)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Copy)]
pub struct Transaction {
    pub kind: TransactionType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Amount>,
}

#[derive(Debug, Clone)]
pub struct AccountRow {
    pub client: ClientId,
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::errors::{CoreError, LedgerError};

    #[test]
    fn parse_whole_number() {
        let a = Amount::from_str_4dp("3").unwrap();
        assert_eq!(a.as_i64(), 3 * Amount::SCALE);
        assert_eq!(a.to_string(), "3.0000");
    }

    #[test]
    fn parse_trailing_dot_is_ok() {
        let a = Amount::from_str_4dp("1.").unwrap();
        assert_eq!(a.to_string(), "1.0000");
    }

    #[test]
    fn parse_with_1_decimal_digit_pads_right() {
        let a = Amount::from_str_4dp("1.2").unwrap();
        assert_eq!(a.as_i64(), 12_000);
        assert_eq!(a.to_string(), "1.2000");
    }

    #[test]
    fn parse_with_4_decimal_digits_exact() {
        let a = Amount::from_str_4dp("1.2345").unwrap();
        assert_eq!(a.as_i64(), 12_345);
        assert_eq!(a.to_string(), "1.2345");
    }

    #[test]
    fn parse_rejects_more_than_4_decimal_digits() {
        let err = Amount::from_str_4dp("1.23456").unwrap_err();
        matches!(err, CoreError::ParseAmount);
    }

    #[test]
    fn parse_rejects_negative() {
        let err = Amount::from_str_4dp("-1.0000").unwrap_err();
        matches!(err, CoreError::NegativeAmount);
    }

    #[test]
    fn parse_rejects_embedded_garbage() {
        assert!(Amount::from_str_4dp("1a.0000").is_err());
        assert!(Amount::from_str_4dp("1.00a0").is_err());
        assert!(Amount::from_str_4dp("1.-2").is_err());
        assert!(Amount::from_str_4dp("1.2.3").is_err());
    }

    #[test]
    fn display_always_4dp() {
        let a = Amount::from_str_4dp("10.5").unwrap();
        assert_eq!(a.to_string(), "10.5000");
    }

    #[test]
    fn checked_add_overflow_detected() {
        let a = Amount(i64::MAX);
        let b = Amount(1);
        let err = a.checked_add(b).unwrap_err();
        matches!(err, LedgerError::Overflow);
    }

    #[test]
    fn checked_sub_overflow_detected() {
        let a = Amount(i64::MIN);
        let b = Amount(1);
        let err = a.checked_sub(b).unwrap_err();
        matches!(err, LedgerError::Overflow);
    }
}
