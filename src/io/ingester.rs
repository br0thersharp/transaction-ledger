use serde::Deserialize;
use crate::core::errors::CoreError;
use crate::core::types::{Amount, ClientId, TxId, Transaction, TransactionType};
use crate::io::IngestEvent;

#[derive(Debug, Deserialize)]
struct CsvRow {
    #[serde(rename = "type")]
    kind: String,
    client: ClientId,
    tx: TxId,
    amount: Option<String>,
}

fn parse_kind(s: &str) -> Result<TransactionType, CoreError> {
    match s.trim() {
        "deposit" => Ok(TransactionType::Deposit),
        "withdrawal" => Ok(TransactionType::Withdrawal),
        "dispute" => Ok(TransactionType::Dispute),
        "resolve" => Ok(TransactionType::Resolve),
        "chargeback" => Ok(TransactionType::Chargeback),
        _ => Err(CoreError::UnknownTransactionType),
    }
}

pub fn read_csv<R: std::io::Read>(
    reader: R,
) -> impl Iterator<Item = IngestEvent> {
    let rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(reader);

    rdr.into_deserialize::<CsvRow>().map(|res| {
        let row = match res {
            Ok(r) => r,
            Err(_) => return IngestEvent::MalformedRow,
        };

        let kind = match parse_kind(&row.kind) {
            Ok(k) => k,
            Err(CoreError::UnknownTransactionType) => return IngestEvent::UnknownType,
            Err(_) => return IngestEvent::MalformedRow,
        };

        let amount = match (kind, row.amount) {
            (TransactionType::Deposit | TransactionType::Withdrawal, Some(a)) => {
                match Amount::from_str_4dp(&a) {
                    Ok(v) => Some(v),
                    Err(_) => None,
                }
            }
            _ => None,
        };

        IngestEvent::Tx(Transaction {
            kind,
            client: row.client,
            tx: row.tx,
            amount,
        })
    })
}

