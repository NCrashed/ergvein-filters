use bitcoin::util::bip158::{GCSFilterReader, Error};
use std::io::{self, Cursor};
use super::btc::BtcFilter;
use super::ergo::ErgoFilter;
use super::utils::{slice_to_u64_le, M, P};

pub enum FilterCurrency {
    Btc,
    Ergo,
}

/// A BIP158 like filter for any supported currency for ergvein wallet
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErgveinFilter {
    Btc(BtcFilter),
    Ergo(ErgoFilter),
}

impl ErgveinFilter {
    /// create a new filter from pre-computed data
    pub fn new(currency: FilterCurrency, content: &[u8]) -> ErgveinFilter {
        match currency {
            FilterCurrency::Btc => ErgveinFilter::Btc(BtcFilter::new(content)),
            FilterCurrency::Ergo => ErgveinFilter::Ergo(ErgoFilter::new(content)),
        }
    }

    /// Get currency tag of the filter
    pub fn currency(&self) -> FilterCurrency {
        match self {
            ErgveinFilter::Btc(_) => FilterCurrency::Btc,
            ErgveinFilter::Ergo(_) => FilterCurrency::Ergo,
        }
    }

    /// Get raw content of filter
    pub fn content(&self) -> &[u8] {
        match self {
            ErgveinFilter::Btc(filter) => filter.content.as_slice(),
            ErgveinFilter::Ergo(filter) => filter.content.as_slice(),
        }
    }

    /// match any query pattern
    pub fn match_any(&self, block_hash: &[u8], query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        let filter_reader = RawFilterReader::new(block_hash);
        filter_reader.match_any(&mut Cursor::new(self.content()), query)
    }

    /// match all query pattern
    pub fn match_all(&self, block_hash: &[u8], query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        let filter_reader = RawFilterReader::new(block_hash);
        filter_reader.match_all(&mut Cursor::new(self.content()), query)
    }
}

/// Reads and interpret a block filter
pub struct RawFilterReader {
    reader: GCSFilterReader
}

impl RawFilterReader {
    /// Create a block filter reader
    pub fn new(block_hash: &[u8]) -> RawFilterReader {
        let k0 = slice_to_u64_le(&block_hash[0..8]);
        let k1 = slice_to_u64_le(&block_hash[8..16]);
        RawFilterReader { reader: GCSFilterReader::new(k0, k1, M, P) }
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
