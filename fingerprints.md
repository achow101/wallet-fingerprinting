# List of known fingerprints

## Bitcoin Core

No preset inputs, no user custom things, just the default Bitcoin Core as a software wallet.

* Anti-fee-sniping with nLockTime
  * locktime is either current block height
  * or 10% randomly up to 100 blocks back, randomly
* Sequence either MAX - 1 or MAX - 2 (RBF)
* Changless txs within BnB changless window
* Change is close to 0.01 BTC
  * Especially if there are a lot of inputs
* Tx version 2
* Negative EV inputs

## Electrum

No preset inputs, no user custom things, just the default Electrum as a software wallet.

* Sequence either MAX - 1 or MAX - 2 (RBF)
* Anti-fee-sniping with nLockTime (same behavior as Core)
* BIP 69 sorted
* Tx Version 2
* Postive EV only unless SEND_MAX
* Same input types
* Change type same as input type
