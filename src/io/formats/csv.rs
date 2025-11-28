use std::io::{Read, Write};

use serde::Deserialize;

use crate::core::errors::CoreError;
use crate::core::types::{Amount, AccountRow, ClientId, TxId, Transaction, TransactionType};
use crate::io::{Emitter, Ingester, IngestEvent};

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

pub struct CsvIngester;

impl Ingester for CsvIngester {
    fn ingest<'a>(
        &self,
        input: Box<dyn Read + 'a>,
    ) -> Box<dyn Iterator<Item = IngestEvent> + 'a> {
        let rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .flexible(true)
            .from_reader(input);

        let iter = rdr.into_deserialize::<CsvRow>().map(|res| {
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
        });

        Box::new(iter)
    }
}

pub struct CsvEmitter;

impl Emitter for CsvEmitter {
    fn emit(
        &self,
        rows: &[AccountRow],
        out: &mut dyn Write,
    ) -> std::io::Result<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(out);

        wtr.write_record(["client", "available", "held", "total", "locked"])?;

        for r in rows {
            wtr.write_record(&[
                r.client.to_string(),
                r.available.to_string(),
                r.held.to_string(),
                r.total.to_string(),
                r.locked.to_string(),
            ])?;
        }

        wtr.flush()?;
        Ok(())
    }
}
