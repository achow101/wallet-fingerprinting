#![feature(map_first_last)]

extern crate bitcoincore_rpc;
extern crate bitcoin;

mod behaviors;

mod bitcoin_core;
use bitcoin_core::maybe_bitcoin_core;

mod electrum;
use electrum::maybe_electrum;

use bitcoin::{
    OutPoint,
    Transaction,
    TxOut,
};
use bitcoin::hashes::hex::FromHex;
use bitcoin::hash_types::Txid;
use bitcoincore_rpc::{Auth, Client, RpcApi};

use std::collections::HashMap;

use std::env;

fn get_previous_outputs(tx: &Transaction, rpc: &Client) -> HashMap<OutPoint, TxOut> {
    let mut out = HashMap::<OutPoint, TxOut>::new();
    for txin in tx.input.iter() {
        let prev_tx = rpc.get_raw_transaction(&txin.previous_output.txid, None).unwrap();
        out.insert(txin.previous_output, prev_tx.output[txin.previous_output.vout as usize].clone());
    }
    return out;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let txid = Txid::from_hex(&args[1]).unwrap();

    let rpc: Client = Client::new(&"http://localhost:8332".to_string(),
        Auth::UserPass("rpcuser".to_string(),
        "rpcpass".to_string())).unwrap();


    let txinfo = rpc.get_raw_transaction_info(&txid, None).unwrap();
    let prevouts = get_previous_outputs(&txinfo.transaction().unwrap(), &rpc);

    let is_core = maybe_bitcoin_core(&txinfo, &prevouts, &rpc);

    if is_core {
        println!("Maybe Bitcoin Core");
    } else {
        println!("Probably not Bitcoin COre");
    }

    let is_electrum = maybe_electrum(&txinfo, &prevouts, &rpc);
    if is_electrum {
        println!("Maybe Electrum");
    } else {
        println!("Probably not Electrum");
    }
}
