use crate::util::*;
use bitcoin::util::bip158::{BlockFilterReader, BlockFilterWriter, Error};
use bitcoin::{Block, BlockHash, OutPoint, Script, Transaction};
use std::io::Cursor;

/// A BIP158 like filter that diverge only in which data is added to the filter.
///
/// Ergvein wallet adds only segwit scripts and data carrier to save bandwith for mobile clients.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErgveinFilter {
    /// Golomb encoded filter
    pub content: Vec<u8>,
}

impl ErgveinFilter {
    /// create a new filter from pre-computed data
    pub fn new(content: &[u8]) -> ErgveinFilter {
        ErgveinFilter {
            content: content.to_vec(),
        }
    }

    /// Compute a SCRIPT_FILTER that contains spent and output scripts
    pub fn new_script_filter<M>(block: &Block, script_for_coin: M) -> Result<ErgveinFilter, Error>
    where
        M: Fn(&OutPoint) -> Result<Script, Error>,
    {
        let mut out = Cursor::new(Vec::new());
        {
            let mut writer = BlockFilterWriter::new(&mut out, block);
            add_output_scripts(&mut writer, &block.txdata);
            add_input_scripts(&mut writer, &block.txdata, script_for_coin)?;
            writer.finish()?;
        }
        Ok(ErgveinFilter {
            content: out.into_inner(),
        })
    }

    /// Match any transaction output scripts
    pub fn match_tx_outputs(
        &self,
        block_hash: &BlockHash,
        tx: &Transaction,
    ) -> Result<bool, Error> {
        let mut scripts = tx.output.iter().map(|o| o.script_pubkey.as_bytes());
        self.match_any(block_hash, &mut scripts)
    }

    /// match any query pattern
    pub fn match_any(
        &self,
        block_hash: &BlockHash,
        query: &mut dyn Iterator<Item = &[u8]>,
    ) -> Result<bool, Error> {
        let filter_reader = BlockFilterReader::new(block_hash);
        filter_reader.match_any(&mut Cursor::new(self.content.as_slice()), query)
    }

    /// match all query pattern
    pub fn match_all(
        &self,
        block_hash: &BlockHash,
        query: &mut dyn Iterator<Item = &[u8]>,
    ) -> Result<bool, Error> {
        let filter_reader = BlockFilterReader::new(block_hash);
        filter_reader.match_all(&mut Cursor::new(self.content.as_slice()), query)
    }
}
