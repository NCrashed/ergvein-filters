use bitcoin::util::bip158::GCSFilterWriter;
use ergo_lib::chain::{ergo_box::BoxId, transaction::Transaction};
use ergotree_ir::serialization::{SerializationError, SigmaSerializable, constant_store::ConstantStore, sigma_byte_reader::SigmaByteReader};
use ergotree_ir::ergo_tree::ErgoTree;
use sigma_ser::{peekable_reader::PeekableReader, vlq_encode::ReadSigmaVlqExt};
use std::io::{self, Cursor};

/// Golomb encoding parameter as in BIP-158, see also https://gist.github.com/sipa/576d5f09c3b86c3b1b75598d799fc845
const P: u8 = 19;
const M: u64 = 784931;

/// Compiles and writes a block filter
pub struct ErgoFilterWriter<'a> {
    reader: SigmaByteReader<PeekableReader<Cursor<&'a mut [u8]>>>,
    writer: GCSFilterWriter<'a>,
}

impl<'a> ErgoFilterWriter<'a> {
    /// Create a block filter writer
    pub fn new(writer: &'a mut dyn io::Write, block_id: &'a [u8], block: &'a mut [u8]) -> ErgoFilterWriter<'a> {
        let k0 = slice_to_u64_le(&block_id[0..8]);
        let k1 = slice_to_u64_le(&block_id[8..16]);
        let writer = GCSFilterWriter::new(writer, k0, k1, M, P);

        // FIXME: Maybe there're saner ways of creating SigmaByteReader.
        // NOTE: ConstantStore is not public
        let cursor = Cursor::new(block);
        let peekable = PeekableReader::new(cursor);
        let reader = SigmaByteReader::new(peekable, ConstantStore::empty());

        ErgoFilterWriter { reader, writer }
    }

    /// Add consumed output scripts of a block to filter
    pub fn add_scripts<M>(&mut self, script_for_coin: M) -> Result<(), SerializationError>
        where M: Fn(&BoxId) -> Result<ErgoTree, SerializationError>
    {
        let n_tx = {
            let n = self.reader.get_u32()?;
            if n == 10000002 { self.reader.get_u32()? } else { n }
        };

        for i in 1..n_tx {
            let tx = Transaction::sigma_parse(&mut self.reader)?;
            if i == 1 { continue; } // skip coinbase
            for out in tx.output_candidates {
                let script = out.ergo_tree.sigma_serialize_bytes();
                self.writer.add_element(&script);
            }
            for bid in tx.inputs {
                let script = script_for_coin(&bid.box_id)?.sigma_serialize_bytes();
                self.writer.add_element(&script);
            }
        }

        Ok(())
    }

    /// Write block filter
    pub fn finish(&mut self) -> Result<usize, io::Error> {
        self.writer.finish()
    }
}

macro_rules! define_slice_to_le {
    ($name: ident, $type: ty) => {
        #[inline]
        pub fn $name(slice: &[u8]) -> $type {
            assert_eq!(slice.len(), ::std::mem::size_of::<$type>());
            let mut res = 0;
            for i in 0..::std::mem::size_of::<$type>() {
                res |= (slice[i] as $type) << i*8;
            }
            res
        }
    }
}
define_slice_to_le!(slice_to_u64_le, u64);
