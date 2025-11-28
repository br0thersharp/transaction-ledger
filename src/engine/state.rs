use crate::core::types::*;
use crate::engine::store::TxStore;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AccountState {
    pub available: Amount,
    pub held: Amount,
    pub locked: bool,
}

impl AccountState {
    pub fn new() -> Self {
        Self {
            available: Amount::zero(),
            held: Amount::zero(),
            locked: false,
        }
    }

    pub fn total(&self) -> Amount {
        self.available + self.held
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxKind {
    Deposit,
    Withdrawal,
}

#[derive(Debug, Clone)]
pub struct TxRecord {
    pub client: ClientId,
    pub amount: Amount,
    pub kind: TxKind,
    pub disputed: bool,
}

#[derive(Debug)]
pub struct EngineState<S: TxStore> {
    pub accounts: HashMap<ClientId, AccountState>,
    pub store: S,
}

impl<S: TxStore> EngineState<S> {
    pub fn new(store: S) -> Self {
        Self {
            accounts: HashMap::new(),
            store,
        }
    }

    pub fn account_mut(&mut self, client: ClientId) -> &mut AccountState {
        self.accounts
            .entry(client)
            .or_insert_with(AccountState::new)
    }

    pub fn accounts_iter(&self) -> impl Iterator<Item = (&u16, &AccountState)> {
        self.accounts.iter()
    }
}
