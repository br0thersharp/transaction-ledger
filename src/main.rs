use std::env;
use std::fs::File;
use transactions_ledger::io::csv_reader::read_csv;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = env::args()
        .nth(1)
        .expect("usage: transaction-manager <input.csv>");
    let file = File::open(input_path)?;
    Ok(())
}