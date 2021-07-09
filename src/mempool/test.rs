use crate::mempool::ErgveinMempoolFilter;
use crate::test::utils::*;
use crate::util::is_script_indexable;
use bitcoin::util::bip158::Error;

#[test]
fn mempool_test() {
    let k0 = u64::from_le_bytes(*b"qwertyui");
    let k1 = u64::from_le_bytes(*b"opasdfgh");
    let block = load_block("./test/block1");
    let txmap = make_inputs_map(load_txs("./test/block1-txs"));
    let mut txs = block.txdata;
    txs.remove(0); // remove coinbase
    let txs2 = txs.clone();
    let filter = ErgveinMempoolFilter::new_script_filter(k0, k1, txs, |o| {
        if let Some(s) = txmap.get(o) {
            Ok(s.clone())
        } else {
            Err(Error::UtxoMissing(o.clone()))
        }
    })
    .unwrap();

    for (i, tx) in txs2.iter().enumerate() {
        let is_indexable = tx
            .output
            .iter()
            .any(|o| is_script_indexable(&o.script_pubkey));
        if is_indexable {
            assert!(
                filter.match_tx_outputs(k0, k1, &tx).unwrap(),
                "Tx #{} failed",
                i
            );
        }
    }
}
