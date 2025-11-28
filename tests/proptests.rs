use proptest::prelude::*;
use std::collections::HashMap;

use transactions_ledger::core::types::{Amount, Transaction, TransactionType};
use transactions_ledger::engine::{HashMapStore, Processor};
use transactions_ledger::io::IngestEvent;

// --------- helpers to generate amounts/events ---------

fn amount_strategy() -> impl Strategy<Value = Amount> {
    // generate scaled integer in 10^-4 units, non-negative
    // keep it reasonably small to avoid overflow in long runs
    (0i64..=1_000_000_000i64).prop_map(|scaled| {
        // scaled means: 123456 -> "12.3456"
        let whole = scaled / 10_000;
        let frac = (scaled % 10_000).abs();
        let s = format!("{}.{}", whole, format!("{:04}", frac));
        Amount::from_str_4dp(&s).unwrap()
    })
}

fn tx_kind_strategy() -> impl Strategy<Value = TransactionType> {
    prop_oneof![
        Just(TransactionType::Deposit),
        Just(TransactionType::Withdrawal),
        Just(TransactionType::Dispute),
        Just(TransactionType::Resolve),
        Just(TransactionType::Chargeback),
    ]
}

fn event_strategy(
    max_clients: u16,
    tx_id_pool: std::sync::Arc<std::sync::Mutex<Vec<u32>>>,
) -> impl Strategy<Value = IngestEvent> {
    // We want some disputes/resolves/chargebacks to reference real tx ids,
    // but also allow random/bogus ids.
    (
        tx_kind_strategy(),
        1u16..=max_clients,
        1u32..=50_000u32,
        prop::option::of(amount_strategy()),
    )
        .prop_map(move |(kind, client, tx, amount)| {
            match kind {
                TransactionType::Deposit | TransactionType::Withdrawal => {
                    // store this txid as something that can be referenced later
                    tx_id_pool.lock().unwrap().push(tx);
                    IngestEvent::Tx(Transaction {
                        kind,
                        client,
                        tx,
                        amount,
                    })
                }
                TransactionType::Dispute
                | TransactionType::Resolve
                | TransactionType::Chargeback => {
                    // pick a reference id sometimes from pool, sometimes random
                    let ref_tx = {
                        let pool = tx_id_pool.lock().unwrap();
                        if !pool.is_empty() && rand::random::<u8>() % 2 == 0 {
                            pool[rand::random::<usize>() % pool.len()]
                        } else {
                            tx
                        }
                    };
                    IngestEvent::Tx(Transaction {
                        kind,
                        client,
                        tx: ref_tx,
                        amount: None,
                    })
                }
            }
        })
}

fn stream_strategy(max_clients: u16) -> impl Strategy<Value = Vec<IngestEvent>> {
    let pool = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u32>::new()));
    prop::collection::vec(event_strategy(max_clients, pool), 1..500)
}

// --------- invariants ---------

fn assert_invariants(proc: &Processor<HashMapStore>) -> Result<(), TestCaseError> {
    for (_client, acct) in proc.state().accounts_iter() {
        prop_assert_eq!(acct.total(), acct.available + acct.held);
        prop_assert!(acct.held.as_i64() >= 0);
        prop_assert_eq!(
            acct.total().as_i64(),
            acct.available.as_i64() + acct.held.as_i64()
        );
    }
    Ok(())
}

// --------- property tests ---------

proptest! {
    #[test]
    fn invariants_hold_for_random_streams(events in stream_strategy(20)) {
        let mut proc = Processor::new(HashMapStore::new());

        for ev in events {
            proc.apply_event(ev);

            // invariants must hold after every event
            assert_invariants(&proc)?;
        }
    }
}

proptest! {
    #[test]
    fn locked_accounts_are_immutable(events in stream_strategy(10)) {
        let mut proc = Processor::new(HashMapStore::new());
        let mut locked_snapshots: HashMap<u16, (Amount, Amount, bool)> = HashMap::new();

        for ev in events {
            proc.apply_event(ev);

            for (&client, acct) in proc.state().accounts_iter() {
                if acct.locked {
                    locked_snapshots.entry(client).or_insert(
                        (acct.available, acct.held, acct.locked)
                    );
                }
            }

            // verify locked accounts haven't drifted
            for (&client, (avail, held, locked)) in locked_snapshots.iter() {
                let acct = proc.state().accounts.get(&client).unwrap();
                prop_assert_eq!(acct.available, *avail);
                prop_assert_eq!(acct.held, *held);
                prop_assert_eq!(acct.locked, *locked);
            }
        }
    }
}
