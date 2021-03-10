This is library for generation of client side filters for [ergvein](https://github.com/hexresearch/ergvein) mobile wallet.

Filters are [BIP-158](https://github.com/bitcoin/bips/blob/master/bip-0158.mediawiki) based but don't include non segwit scripts
to reduce size. Instead of 4 Gib per Bitcoin mainnet, light client should download only 400 Mib.
