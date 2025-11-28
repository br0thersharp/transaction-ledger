use crate::core::LedgerError;
use crate::core::errors::CoreError;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub type ClientId = u16;
pub type TxId = u32;

/// Fixed-precision amount newtype in 10^-4 units.
/// Stored as scaled i64 to avoid underflow hazards during subtraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Amount(i64);

impl Amount {
    pub const SCALE: i64 = 10_000;

    pub fn zero() -> Self { Amount(0) }

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

        let whole: i64 = parts[0]
            .parse()
            .map_err(|_| CoreError::ParseAmount)?;

        let decimals_str = if parts.len() == 2 { parts[1] } else { "0" };
        let decimals_len = decimals_str.len();
        if decimals_len > 4 {
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