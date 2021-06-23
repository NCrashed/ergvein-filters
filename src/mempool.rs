use bitcoin::{OutPoint, Script, Transaction};
use bitcoin::util::bip158::{GCSFilterReader, GCSFilterWriter, Error};
use std::io::Cursor;
use std::io;
use crate::util::*;

/// Golomb encoding parameter as in BIP-158, see also https://gist.github.com/sipa/576d5f09c3b86c3b1b75598d799fc845
const P: u8 = 19;
const M: u64 = 784931;

/// A BIP158 like filter that diverge only in which data is added to the filter.
///
/// Ergvein wallet adds only segwit scripts and data carrier to save bandwith for mobile clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErgveinMempoolFilter {
    /// Golomb encoded filter
    pub content: Vec<u8>
}

impl ErgveinMempoolFilter {
    pub fn new(content: &[u8]) -> ErgveinMempoolFilter{
        ErgveinMempoolFilter { content: content.to_vec() }
    }

    /// Compute a SCRIPT_FILTER that contains spent and output scripts
    pub fn new_script_filter<M>(k0: u64, k1: u64, txs: Vec<Transaction>, script_for_coin: M) -> Result<ErgveinMempoolFilter, Error>
        where M: Fn(&OutPoint) -> Result<Script, Error> {
        let mut out = Cursor::new(Vec::new());
        {
            let mut writer = MempoolFilterWriter::new(&mut out, k0, k1);
            add_output_scripts(&mut writer, &txs);
            add_input_scripts(&mut writer, &txs, script_for_coin)?;
            writer.finish()?;
        }
        Ok(ErgveinMempoolFilter { content: out.into_inner() })
    }

    /// Match any transaction output scripts
    pub fn match_tx_outputs(&self, k0: u64, k1: u64, tx: &Transaction) -> Result<bool, Error> {
        let mut scripts = tx.output.iter().map(|o| o.script_pubkey.as_bytes() );
        self.match_any(k0, k1, &mut scripts)
    }

    /// match any query pattern
    pub fn match_any(&self,  k0: u64, k1: u64, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        let filter_reader = MempoolFilterReader::new(k0, k1);
        filter_reader.match_any(&mut Cursor::new(self.content.as_slice()), query)
    }

    /// match all query pattern
    pub fn match_all(&self, k0: u64, k1: u64, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        let filter_reader = MempoolFilterReader::new(k0, k1);
        filter_reader.match_all(&mut Cursor::new(self.content.as_slice()), query)
    }

}

/// Compiles and writes a block filter
pub struct MempoolFilterWriter<'a> {
    writer: GCSFilterWriter<'a>,
}

impl<'a> FilterWriter for MempoolFilterWriter<'a> {
    fn add_filter_element(&mut self, data: &[u8]) {
        self.writer.add_element(data);
    }
    fn is_block_filter(&mut self) -> bool{ false }
}

impl<'a> MempoolFilterWriter<'a> {
    /// Create a block filter writer
    pub fn new(writer: &'a mut dyn io::Write, k0: u64, k1: u64) -> MempoolFilterWriter<'a> {
        let writer = GCSFilterWriter::new(writer, k0, k1, M, P);
        MempoolFilterWriter { writer }
    }

    /// Add arbitrary element to a filter
    pub fn add_element(&mut self, data: &[u8]) {
        self.writer.add_element(data);
    }

    /// Write block filter
    pub fn finish(&mut self) -> Result<usize, io::Error> {
        self.writer.finish()
    }
}


/// Reads and interpret a block filter
pub struct MempoolFilterReader {
    reader: GCSFilterReader
}

impl MempoolFilterReader {
    /// Create a block filter reader
    pub fn new(k0: u64, k1: u64) -> MempoolFilterReader {
        MempoolFilterReader { reader: GCSFilterReader::new(k0, k1, M, P) }
    }

    /// match any query pattern
    pub fn match_any(&self, reader: &mut dyn io::Read, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        self.reader.match_any(reader, query)
    }

    /// match all query pattern
    pub fn match_all(&self, reader: &mut dyn io::Read, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        self.reader.match_all(reader, query)
    }
}
