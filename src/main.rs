use std::env;
use std::fs::File;

use transactions_ledger::io::{CsvEmitter, CsvIngester, Emitter, Ingester, IngestEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = env::args()
        .nth(1)
        .expect("usage: transaction-manager <input.csv>");
    let file = File::open(input_path)?;

    let ingester = CsvIngester;
    let emitter = CsvEmitter;

    // Ingest test: just iterate events and count them
    let mut tx_count = 0u64;
    let mut malformed = 0u64;
    let mut unknown = 0u64;

    for ev in ingester.ingest(Box::new(file)) {
        match ev {
            IngestEvent::Tx(t) => {
                tx_count += 1;
                eprintln!("TX: kind={:?} client={} tx={} amount={:?}",
                    t.kind, t.client, t.tx, t.amount);
            }
            IngestEvent::MalformedRow => malformed += 1,
            IngestEvent::UnknownType => unknown += 1,
        }
    }

    eprintln!("ingest summary: tx={} malformed={} unknown={}", tx_count, malformed, unknown);

    Ok(())
}