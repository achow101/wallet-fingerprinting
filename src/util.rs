use bitcoin::{
    OutPoint,
    Transaction,
    TxOut,
};
use bitcoincore_rpc::{Client, RpcApi};

use std::collections::HashMap;

pub fn get_previous_outputs(tx: &Transaction, rpc: &Client) -> HashMap<OutPoint, TxOut> {
    let mut out = HashMap::<OutPoint, TxOut>::new();
    for txin in tx.input.iter() {
        let prev_tx = rpc.get_raw_transaction(&txin.previous_output.txid, None).unwrap();
        out.insert(txin.previous_output, prev_tx.output[txin.previous_output.vout as usize].clone());
    }
    return out;
}

