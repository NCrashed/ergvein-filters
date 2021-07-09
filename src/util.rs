use bitcoin::{util::bip158::{BlockFilterWriter, Error}, Script, Transaction, OutPoint};

// The trait is required to implement common functions
pub trait FilterWriter {
    fn add_filter_element(&mut self, data: &[u8]);
    fn is_block_filter(&mut self) -> bool;
}

impl<'a> FilterWriter for BlockFilterWriter<'a> {
    fn add_filter_element(&mut self, data: &[u8]) {
        self.add_element(data);
    }
    fn is_block_filter(&mut self) -> bool{ true }
}

pub fn is_script_indexable(script: &Script) -> bool {
    !script.is_empty() && (script.is_v0_p2wsh() || script.is_v0_p2wpkh() || script.is_op_return())
}

pub fn add_output_scripts(writer: &mut dyn FilterWriter, txs: &[Transaction]) {
    for transaction in txs {
        for output in &transaction.output {
            if is_script_indexable(&output.script_pubkey) {
                writer.add_filter_element(output.script_pubkey.as_bytes());
            }
        }
    }
}

pub fn add_input_scripts<F>(writer: &mut dyn FilterWriter, txs: &[Transaction], script_for_coin: F) -> Result<(), Error>
    where
    F: Fn(&OutPoint) -> Result<Script, Error>
{
    // If this is the block filter, skip the coinbase
    let n = if writer.is_block_filter() {1} else {0};
    for script in txs.iter()
    .skip(n)
    .flat_map(|t|
        t.input.iter()
        .filter_map(|i| if i.script_sig.is_empty(){ Some(&i.previous_output)} else {None})
    ).map(script_for_coin) {
        match script {
        Ok(script) => {
            if is_script_indexable(&script) {
                writer.add_filter_element(script.as_bytes())
            }
        }
        Err(e) => if !writer.is_block_filter() {
                // If it's a mempool filter, just skip invalid inputs
                return Err(e)
            }
        }
    }
    Ok(())
}
