use bitcoin::{Block, BlockHash, OutPoint, Script};
use bitcoin::util::bip158::{BlockFilterWriter, BlockFilterReader, Error};
use std::io::Cursor;

/// a computed or read block filter
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErgveinFilter {
    /// Golomb encoded filter
    pub content: Vec<u8>
}

impl ErgveinFilter {
    /// create a new filter from pre-computed data
    pub fn new (content: &[u8]) -> ErgveinFilter {
        ErgveinFilter { content: content.to_vec() }
    }

    /// Compute a SCRIPT_FILTER that contains spent and output scripts
    pub fn new_script_filter<M>(block: &Block, script_for_coin: M) -> Result<ErgveinFilter, Error>
        where M: Fn(&OutPoint) -> Result<Script, Error> {
        let mut out = Cursor::new(Vec::new());
        {
            let mut writer = BlockFilterWriter::new(&mut out, block);
            add_output_scripts(&mut writer, block);
            add_input_scripts(&mut writer, block, script_for_coin)?;
            writer.finish()?;
        }
        Ok(ErgveinFilter { content: out.into_inner() })
    }

    /// match any query pattern
    pub fn match_any(&self, block_hash: &BlockHash, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        let filter_reader = BlockFilterReader::new(block_hash);
        filter_reader.match_any(&mut Cursor::new(self.content.as_slice()), query)
    }

    /// match all query pattern
    pub fn match_all(&self, block_hash: &BlockHash, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        let filter_reader = BlockFilterReader::new(block_hash);
        filter_reader.match_all(&mut Cursor::new(self.content.as_slice()), query)
    }
}

fn is_script_indexable(script: &Script) -> bool {
    !script.is_empty() && (script.is_v0_p2wsh() || script.is_v0_p2wpkh() || script.is_op_return())
}

fn add_output_scripts(writer: &mut BlockFilterWriter, block: &Block) {
    for transaction in &block.txdata {
        for output in &transaction.output {
            if is_script_indexable(&output.script_pubkey) {
                writer.add_element(output.script_pubkey.as_bytes());
            }
        }
    }
}

fn add_input_scripts<F>(writer: &mut BlockFilterWriter, block: &Block, script_for_coin: F) -> Result<(), Error>
    where
    F: Fn(&OutPoint) -> Result<Script, Error>
{
    for script in block.txdata.iter()
        .skip(1) // skip coinbase
        .flat_map(|t| t.input.iter().map(|i| &i.previous_output))
        .map(script_for_coin) {
        match script {
            Ok(script) => {
                if is_script_indexable(&script) {
                    writer.add_element(script.as_bytes())
                }
            }
            Err(e) => return Err(e)
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use bitcoin::consensus::deserialize;
    use bitcoin::hashes::hex::FromHex;
    use bitcoin::Transaction;
    use std::collections::HashMap;
    use std::fs;
    use std::io;
    use std::io::BufRead;
    use super::*;

    #[test]
    fn block_000000000000017c36b1c7c70f467244009c552e1732604a0f779fc6ff2d6112() {
        let filter_content = Vec::from_hex("13461a23a8ce05d6ce6a435b1d11d65707a3c6fce967152b8ae09f851d42505b3c41dd87b705d5f4cc2c3062ddcdfebe7a1e80").unwrap();
        let block = load_block("./test/block1");
        let txmap = make_inputs_map(load_txs("./test/block1-txs"));
        let filter = ErgveinFilter::new_script_filter(&block,
                                        |o| if let Some(s) = txmap.get(o) {
                                            Ok(s.clone())
                                        } else {
                                            Err(Error::UtxoMissing(o.clone()))
                                        }).unwrap();
        let test_filter = ErgveinFilter::new(filter_content.as_slice());

        assert_eq!(test_filter.content, filter.content);

    }

    fn make_inputs_map(txs: Vec<Transaction>) -> HashMap<OutPoint, Script> {
        let mut map = HashMap::new();
        for tx in txs {
            let mut out_point = OutPoint {
                txid: tx.txid(),
                vout: 0,
            };
            for (i, out) in tx.output.iter().enumerate() {
                out_point.vout = i as u32;
                map.insert(out_point.clone(), out.script_pubkey.clone());
            }
        }
        map
    }

    fn load_block(path: &str) -> Block {
        let mut contents = fs::read_to_string(path).unwrap();
        contents.pop();
        deserialize(&Vec::from_hex(&contents).unwrap()).unwrap()
    }

    fn load_txs(path: &str) -> Vec<Transaction> {
        let mut res = vec![];
        let file = std::fs::File::open(path).unwrap();
        for line in io::BufReader::new(file).lines() {
            let tx = deserialize(&Vec::from_hex(&line.unwrap()).unwrap()).unwrap();
            res.push(tx);
        }
        res
    }
}
