use std::env;
use std::fs::File;

use transactions_ledger::engine::{HashMapStore, Processor};
use transactions_ledger::io::{CsvEmitter, CsvIngester, Emitter, Ingester};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = env::args()
        .nth(1)
        .expect("usage: transactions_ledger <input.csv>");
    let file = File::open(input_path)?;

    let ingester = CsvIngester;
    let emitter = CsvEmitter;

    let mut processor = Processor::new(HashMapStore::new());

    for event in ingester.ingest(Box::new(file)) {
        processor.apply_event(event);
    }

    let rows = processor.results();

    let mut out = std::io::stdout();
    emitter.emit(&rows, &mut out)?;

    // for debugging or later dashboards/observability
    // eprintln!("metrics: {:?}", processor.metrics());

    Ok(())
}
