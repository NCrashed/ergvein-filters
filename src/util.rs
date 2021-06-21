use bitcoin::{util::bip158::{BlockFilterWriter, Error}, Script, Transaction, OutPoint};

// The trait is required to implement common functions
pub trait FilterWriter {
    fn add_filter_element(&mut self, data: &[u8]);
}

impl<'a> FilterWriter for BlockFilterWriter<'a> {
    fn add_filter_element(&mut self, data: &[u8]) {
        self.add_element(data);
    }
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

// If you are passing a block, skip the first (coinbase) transaction.
pub fn add_input_scripts<F>(writer: &mut dyn FilterWriter, txs: &[Transaction], script_for_coin: F) -> Result<(), Error>
    where
    F: Fn(&OutPoint) -> Result<Script, Error>
{
    for script in txs.iter()
        .flat_map(|t| t.input.iter().map(|i| &i.previous_output))
        .map(script_for_coin) {
        match script {
            Ok(script) => {
                if is_script_indexable(&script) {
                    writer.add_filter_element(script.as_bytes())
                }
            }
            Err(e) => return Err(e)
        }
    }
    Ok(())
}
