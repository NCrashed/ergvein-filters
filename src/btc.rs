use bitcoin::{Block, BlockHash, OutPoint, Script, Transaction};
use bitcoin::consensus::serialize;
use bitcoin::util::bip158::{BlockFilterWriter, BlockFilterReader, Error};
use std::io::Cursor;

/// A BIP158 like filter that diverge only in which data is added to the filter.
///
/// Ergvein wallet adds only segwit scripts and data carrier to save bandwith for mobile clients.
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

    /// Match any transaction output scripts
    pub fn match_tx_outputs(&self, block_hash: &BlockHash, tx: &Transaction) -> Result<bool, Error> {
        let scripts: Vec<Vec<u8>> = tx.output.iter().map(|o| serialize(&o.script_pubkey) ).collect();
        let mut query = scripts.iter().map(|s| &s[..]);
        self.match_any(block_hash, &mut query)
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
    fn block_00000000000000000007fc62780dee62d79ba02e7d325d7503e80c4da8b16b72() {
        let filter_content = Vec::from_hex("fd5905dd063e02ca26a4a81f3aa51dde1374a8b0f4384f461ff3821c02f6496e836bc1b286a98af106222b9710d5061462d61cd64ce4c4e822cf200f7d3c2051e8ad16954826daeaae235de0361863df40f22a56ad33c0d1f367f56637f89fb33e322e60ef4ec80fc0481ee29e7f8a483a49d24a293fc3aab2351282e0f4059d794e3309d723d47f50a2bd9b6c025ab36144809e9fff2e956907482179d41c5911fc1996a0ae489b80352d9349e382f7c173c6a76be2ff37fd5fafd0360e2d5d81a8c7cb0b616926bbecd10e1f10d693b9fd9e6fe290d4a8c4ab3477ddbeacb2b6bd44578a4ac57a810a064268b39fc6017709347018e5ec951e2da8d747386ca3ad8bae12c8059f0b42de04cc5e16369402d3dfd25e83d0eccef458d3e2895e0670ffe543878dab80fdcd0b02b4341a30fc817a8b8d5e36b466d3e7f3b14951a8a14bc061a54b9a1b000608d50c7dbc1223e2ce3f5a990f81686ba730261289898c787ba0ec672f65715f87c1f826530de182c271f4d399f82a9360489ac2c9151456e32096ff79ec1db1539a4418602ac32df65aa9242b303d8574b407b6ab97b246e3720cb0d8a34a01cac6c1c39821f960a5ec6c163bce31a658b240bb8acc7f80fefd4efd8a546203b4df21aaa72bba3a1a319904801c46c4d5248252276d25e4794f5756d96bc537866fd21a4912da77cf3eaba85cbf24d84ed22271da44fe8df9350adaa3a475afd151bd0102fa113f750f47f758d0c61adde86a059a333cf15b691de3ea337394afa783d8a4b8f234d380718ce465895d5508b1fc70b82705437608402fc9b69c315d5c7774fac5996f221548bf529692ca3efd003fab6ded7735b0b73eb450aaf212e378c078918d33be7ec5dfb5799b3e7abda5d5c230d70ab2a1145df0aa4f5f24d402078737752b38aae6415620724eb85f06ba68edafe07c9b8ad1b1b84cd55d5271a50d301dd8f155e436b6a9dcd7ac51e311351cde36759e334fc6285a3301b419dccd60bfcc6f063a607197cf530a8a7828f324012a51e33a73c67fe824a4eaeb1817985835854399e346ca383a5ffafe557a097da0e44734e043a279e71beffcea71df4a1b1d02fcffc40f6269a670ac62d770f29ff49345020527fbeec37ecb9cca46580f9888854baa29f7d66b86d3ad17f1c2791fdd7fc6cd7c620ae4796faa59fecc494a5fe1e44763c87730f990474755aa3476dd82702630d0fb60ff2f5f292ede5cbf5b2b842aaf0b6d5ee311be4dfd62070514c8b4af8eeb4174df80f0b678e3ff7e75378592ca909dadf4b273446d5c3f771d6122573e1a701035a314989f748ba45488d2069b06495b2c88acc4ff86885a0ff862da72126a84ced35dc2ee1b466a9e12d492a2689da58e79717990e43e93dd3eb60dfe870e0186e00e9e510a3a2c4b63bc6007506b2951f903a00562d5aea8487e7b901951cff66feb4c18adf394983427709531f4fdcc053c55d19052b5407d800e8948948679b26a6ff6bf14479344df17b69126abcc819e128417c04711af74f14d9ac9e23308eb3f8b392fc31d13011136972a05d9e7f0254783b0b00370f3bc637ebd87fb7cb360137ec188fbffff65d3af6550f813d82738085f6f49570e85e6920479bfd43165ed9b5534dcd9f676a9ee4d57e21ea761753e298e299b1bb1d6a040eeddda13d607be931a8f214853ec7e42438116520c951765500f93e0c0946527f36d5d929f1ad9645b8d55d77992f286fd73969084efd218de4eaa4ecb022f8a2c3f0f8ef11ea5139ce6387e1121ee42e727bd17fd0ff49c752ba0975cbb9e38105dd3a00494eac75d4536112b22fb807f1a87460a177a8d8bc9b5cf044c49a6e1042914a496244478111a94f57c19421e880ede046e356c781604efad60587bce24aecbf33c9c985a370397d659d4b61e6120af108271df844bb9b3c2d2f9b747c6110fee9e24f5b1af90f76cd51abd0802f0964b913b21f16eb82e4f9b93598a2505de7eace41f3592f075686693db02c4777598e60dbac1adf9295e479f944d04e0c2854abc097561632ecb544974cc4d6fa990b788a49d096965a6fde804aa058ed324e287ad1625e72a994901c4c734290431f17fcb4750e1e031c861fc5546c6049a4d5c27f8a6fdbf8008da38d47c3b3f90b13ef609f617851047136d74120f099d0b57667937a7ea5ae893797f04ed15e9c40d0658faba6044c95140008ba4601614a1e31592adbd8e2dc1a6875783b7ba6c55ee1b3edf10abffba836d68e5aea5d2805bdd0c6f6c4a69a6e8311b063d3473b7eb77af51d1f432d904f214b9ceaad4e7981e0d45939ac7b9ffc659393bba4bc57e3ca03ab1fa49a7cccd2b07f6f76357fb38de1ea1acec2cbc0764cbf8bc629ef078641bce68ec7c5ce664cc01e431755630c7d9322cc2172c45eda4d5bf11c47feb1d7185deb54e2bb098799af5641faf222d8076449fa8f397850a45054b913f7c51c1d4046fbc8862eb9b48d54de2de6e29db327a33366f83eec604f4b1c5058d3f5231619705991c609521e82fd4e78df47d7753bd99bba4fcd9564851dc17fa471155c69ca1d40b28d9dcc09eec3f1cd2df008fd6f184a656c63d9d03cc3144730137616e22c377df067209caba40b0915da4f1b9a331ab448a56b36ff65abd4da72827f08ad7a5f48093c48cb0d8dd7acff2f0875d249cbb299b26439e5a583210f467fd16b7778b66ec35aac37793dd6df64327d5b99f00f57a0d75a34a7e80e71ce49e249c9f54f94402abb3532682d22c8ec3365f3991000b6d90254b5ec9f597ddbce04c4a940cb20065ca73eac99bd77d3482f05faae47749c5d694ff6fe2c22b9c6b5e64d373c4e08302604d315326b79026d722f0cbb0e47f82f6f08e58e711cf15687f5c117437a465f4be2418a9d520f1a74db534adc81f29872703f8ad40a4436fb8591340a9075334aaac0ac63450b6e4f42e98e250a2b0a188ba6eab0e0ea62e25b3b405bcf8d0cd07d39b2a9a663bc017ab3e097fd14655e77c69e9aa956525da31ad0e62d64011c8bd21712eeab12d90e176385edcdbaa4405b6e313e13d873e6c262e678f3a4a1d36b4eb07ee2e12940555dbd390e946c4ed4b10bbbf8157a44f051567706cb2c4da5a8f414735e7993b718cb8b73c302a25c7137e4ae4f1244c1ac8106215ed25eb5157f46dff6b6eef19f67c9c61d417920830667be6d4a39973163b6ff347f74c79089c68c75520ca8954d4e3edf73356383e00aed15d0be5a91564adbd47b878550512e25a78fce545d3abf8ddcb0ae5102ce65b6c5f044482d300a8f73ac4e2281c0fb0801abfe6df919afdc626b192de7db2e556ae5ab44e1313d1c918aaf36cc5e876d3ba47ddd0227fb3c4ae01bbc22384fce0b7fd13828564b232a6efc82b750780830e0c481535fd66fc04585e4c4a39d38721871f8dca2334c5984473aada86564abee895b145283da174534c7470713336e49c922719089c88c3326b5063dfb6490dcc816a4cff00797db66ada0e1584c9c490ca22a4b54f4d08f73821351040eb529605b09f8954299ad6adb44ba853d8aa045e037c1df1ade9a18a242c1bcfa92fa660056a80811202ef559befca1112b510d0509571b2ec223c82b066d52d4693207e753267e1a5e1558a95006f4edc7f6e287d02c299626c1b75f6d376afde5d857e5dc2c4dbaf64297a5e7cd63be9110671a4616455f40021d0661f4abab024a1043a4c3e86d2acc6423cfbd3b0db4036674500742bfd55b69726cc98a08597e23b5a918eef57c3975bd88fb10724fcce710fc424963d64554d797e14855c04033923b391d7bce4c1eebb2dd0f2e953d1a7cca1a1e02eedda944418bfc9cd416e0f27bad1c7aac181e200df5b71f9c4e5420e9df39e0b9102b7d4d170339629cbc847a74d587d7cd7e1357020a469ab60577c73a9d8f90c1b75d3e5a62ef0637002026fe4c11c9c10b516ca830c77114faf27c694f04c41c9bd7e00af2dba1b16e782562265d2fcf0b8d66f7ed097e37f515e51794543cad4de04d093a281cac1f5c23dc8707244bf1b18a2bb02d44b2afdad1b0a426470d8ee15b2a37957435b45f61535e7e93feaa1b3e14b75414882724ab6964656edab419c7063ceb09574d8ec6c46384f9d5502846a4c8d243d592ad76c6130228b1d91a459171219ffad458f5f19590cbc19cd83d1edc8e8a581ef28ce6819ea938e82061602bf46518c575ed597df53149e70fb84959155a448e140be82fdf8f051802017f3337873dce90e1fc6e1f7d2a3667fdaf39aa338df2cf12c9fccf4e2714867c312f8f417ea1de915bc7a2fa0d62fe4266822f7002ece3276c0275134e15a3b892466fe169575705cc98120596e58af4ffaba4c5773d752c31c28106713d22ae917be4431ea671c82e2ca9acbe718b5087cc5c746867c6efe93355ef33859c8080617de4ff7d1511bb934a8b0dfeeb33df80e7bb97ed48dcfe0d2241bf46851fed8b793480cee57d71e9f1aed91fa0512783e118956402094650ec9fe21a7db0c949c362a286d269ff2696f9a75b056538bc0fadf7db9ca838b481a3d8ca5ea30bb0259da6b88a9ef0efe2306a5ee04b084c32b916bde6bbe7375e95af910e62c695c6dc865ba2e810a45f9717b629b0487aaf470d05fae038bb32833e8814d98fde24d388966f8591f2ea0d7e5a3f16bd413f5ed51c07722ed1499a48ff9846f9fbddfa0f6eda9b8936ba0874877f0ece7fc498f361e0b06ceaa83861801b4673ec2f2f241a180bd7e1b86a1d1192ae73d3382581836e3ae2daa28eae44df16145ad5ae4c5ca6b830fd0e9e9f30de7e226331ddbaf6d5059d1986a28bc7bbdd9dc70a4d0ac284a4992e5d952964bd809165e57969a2c016219d0ce9e8f0864f96c1e68605c106f6e24df84bb3c9bfa16020d18edd73f42a08e0ec874bb72204d57f13a6f667a9c5c08151d7427788447404eb921ed099f1a3a7580f2f3345c721f9402541cf685a4f0ebbebc77800b6f05ab8fcc747fea059fd1df0c254369b8cda1ff0f8fa28cf18373f68ee33dcbca3348fb563c298bf292085ec960").unwrap();
        let test_filter = ErgveinFilter::new(filter_content.as_slice());

        let hash_content = Vec::from_hex("00000000000000000007fc62780dee62d79ba02e7d325d7503e80c4da8b16b72").unwrap();
        let block_hash = deserialize(&hash_content).unwrap();
        let tx_content = Vec::from_hex("01000000000101392550f02f8936c5675c2a23f79cc66f6738daa4164b8258a42c59d06c4fc9340000000000ffffffff021920980100000000160014e31594dfc81060a5626841f7a66fcfe0c4e35365678d000000000000160014b7ce167a800057b5bb715a48964739395e64341802483045022100fcf9097b918de57e5d3ad61ead9b68a061937aaeec653e0ac4a7a5e407dd506e02206e36e13bf6500aa0f1d38ae4727ef65ac3dc44c9660bbaa0f671c67e66c675ae0121028efb5bdfc12f462c8155793e17f732bf32fd7fff382288198e5e7e66cf97aabc00000000").unwrap();
        let tx: Transaction = deserialize(&tx_content).unwrap();

        assert_eq!(test_filter.match_tx_outputs(&block_hash, &tx).unwrap(), true);
    }

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
