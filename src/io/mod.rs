use std::io::{Read, Write};
use crate::core::types::{AccountRow, Transaction};

#[derive(Debug)]
pub enum IngestEvent {
    Tx(Transaction),
    MalformedRow,
    UnknownType,
}

pub trait Ingester {
    fn ingest<'a>(
        &self,
        input: Box<dyn Read + 'a>,
    ) -> Box<dyn Iterator<Item = IngestEvent> + 'a>;
}

pub trait Emitter {
    fn emit(
        &self,
        rows: &[AccountRow],
        out: &mut dyn Write,
    ) -> std::io::Result<()>;
}

pub mod formats;
pub use formats::csv::{CsvEmitter, CsvIngester};
