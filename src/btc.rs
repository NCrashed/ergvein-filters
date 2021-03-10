
#[cfg(test)]
mod tests {
    use bitcoin::{Block, OutPoint,  Script, Transaction};
    use bitcoin::consensus::deserialize;
    use bitcoin::hashes::hex::FromHex;
    use bitcoin::util::bip158::{BlockFilter, Error};
    use std::fs;
    use std::io;
    use std::io::BufRead;
    use std::collections::HashMap;

    #[test]
    fn block_000000000000017c36b1c7c70f467244009c552e1732604a0f779fc6ff2d6112() {
        let filter_content = Vec::from_hex("13461a23a8ce05d6ce6a435b1d11d65707a3c6fce967152b8ae09f851d42505b3c41dd87b705d5f4cc2c3062ddcdfebe7a1e80").unwrap();
        let block = load_block("./test/block1");
        let txmap = make_inputs_map(load_txs("./test/block1-txs"));
        let filter = BlockFilter::new_script_filter(&block,
                                        |o| if let Some(s) = txmap.get(o) {
                                            Ok(s.clone())
                                        } else {
                                            Err(Error::UtxoMissing(o.clone()))
                                        }).unwrap();
        let test_filter = BlockFilter::new(filter_content.as_slice());

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
