use bitcoin::consensus::deserialize;
use bitcoin::hashes::hex::FromHex;
use bitcoin::Block;
use bitcoin::OutPoint;
use bitcoin::Script;
use bitcoin::Transaction;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::BufRead;

pub fn make_inputs_map(txs: Vec<Transaction>) -> HashMap<OutPoint, Script> {
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

pub fn load_block(path: &str) -> Block {
    let mut contents = fs::read_to_string(path).unwrap();
    contents.pop();
    deserialize(&Vec::from_hex(&contents).unwrap()).unwrap()
}

pub fn load_txs(path: &str) -> Vec<Transaction> {
    let mut res = vec![];
    let file = std::fs::File::open(path).unwrap();
    for line in io::BufReader::new(file).lines() {
        let tx = deserialize(&Vec::from_hex(&line.unwrap()).unwrap()).unwrap();
        res.push(tx);
    }
    res
}
