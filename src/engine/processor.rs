use crate::core::ledger;
use crate::core::types::*;
use crate::engine::metrics::Metrics;
use crate::engine::state::{AccountState, EngineState, TxKind, TxRecord};
use crate::engine::store::TxStore;
use crate::io::IngestEvent;

pub struct Processor<S: TxStore> {
    state: EngineState<S>,
    metrics: Metrics,
}

impl<S: TxStore> Processor<S> {
    pub fn new(store: S) -> Self {
        Self {
            state: EngineState::new(store),
            metrics: Metrics::default(),
        }
    }

    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }

    pub fn apply_event(&mut self, event: IngestEvent) {
        match event {
            IngestEvent::Tx(tx) => self.apply(tx),
            IngestEvent::MalformedRow => self.metrics.malformed_rows += 1,
            IngestEvent::UnknownType => self.metrics.unknown_type += 1,
        }
    }

    fn apply(&mut self, tx: Transaction) {
        let (accounts, store) = (&mut self.state.accounts, &mut self.state.store);

        // get or create account
        let account = accounts.entry(tx.client).or_insert_with(AccountState::new);

        if account.locked {
            self.metrics.locked_ignored += 1;
            return;
        }

        match tx.kind {
            TransactionType::Deposit => {
                let amount = match tx.amount {
                    Some(a) => a,
                    None => {
                        self.metrics.missing_amount += 1;
                        return;
                    }
                };

                if store.contains(tx.tx) {
                    self.metrics.duplicate_tx += 1;
                    return;
                }

                if ledger::deposit(account, amount).is_ok() {
                    store.insert(
                        tx.tx,
                        TxRecord {
                            client: tx.client,
                            amount,
                            kind: TxKind::Deposit,
                            disputed: false,
                        },
                    );
                } else {
                    self.metrics.ledger_errors += 1;
                }
            }

            TransactionType::Withdrawal => {
                let amount = match tx.amount {
                    Some(a) => a,
                    None => {
                        self.metrics.missing_amount += 1;
                        return;
                    }
                };

                if store.contains(tx.tx) {
                    self.metrics.duplicate_tx += 1;
                    return;
                }

                if ledger::withdrawal(account, amount).is_ok() {
                    store.insert(
                        tx.tx,
                        TxRecord {
                            client: tx.client,
                            amount,
                            kind: TxKind::Withdrawal,
                            disputed: false,
                        },
                    );
                } else {
                    self.metrics.ledger_errors += 1;
                }
            }

            TransactionType::Dispute => {
                let rec = match store.get_mut(tx.tx) {
                    Some(r) => r,
                    None => {
                        self.metrics.tx_not_found += 1;
                        return;
                    }
                };

                if rec.client != tx.client {
                    self.metrics.wrong_client_ref += 1;
                    return;
                }

                if ledger::dispute(account, rec).is_err() {
                    self.metrics.ledger_errors += 1;
                }
            }

            TransactionType::Resolve => {
                let rec = match store.get_mut(tx.tx) {
                    Some(r) => r,
                    None => {
                        self.metrics.tx_not_found += 1;
                        return;
                    }
                };

                if rec.client != tx.client {
                    self.metrics.wrong_client_ref += 1;
                    return;
                }

                if ledger::resolve(account, rec).is_err() {
                    self.metrics.ledger_errors += 1;
                }
            }

            TransactionType::Chargeback => {
                let rec = match store.get_mut(tx.tx) {
                    Some(r) => r,
                    None => {
                        self.metrics.tx_not_found += 1;
                        return;
                    }
                };

                if rec.client != tx.client {
                    self.metrics.wrong_client_ref += 1;
                    return;
                }

                if ledger::chargeback(account, rec).is_err() {
                    self.metrics.ledger_errors += 1;
                }
            }
        }
    }

    pub fn results(&self) -> Vec<AccountRow> {
        let mut rows: Vec<_> = self
            .state
            .accounts
            .iter()
            .map(|(&client, acc)| AccountRow {
                client,
                available: acc.available,
                held: acc.held,
                total: acc.total(),
                locked: acc.locked,
            })
            .collect();

        rows.sort_by_key(|r| r.client);
        rows
    }

    pub fn state(&self) -> &EngineState<S> { &self.state }   
}
