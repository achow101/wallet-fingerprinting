#![feature(map_first_last)]

extern crate bitcoincore_rpc;
extern crate bitcoin;

mod behaviors;
use behaviors::check_heuristics;

mod bitcoin_core;
use bitcoin_core::analyze_bitcoin_core;

mod electrum;
use electrum::analyze_electrum;

mod util;
use util::{
    get_previous_outputs,
    WalletConfidence,
};

use bitcoin::hashes::hex::FromHex;
use bitcoin::hash_types::Txid;
use bitcoincore_rpc::{Auth, Client, RpcApi};

use std::collections::HashMap;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let txid = Txid::from_hex(&args[1]).unwrap();

    let rpc: Client = Client::new(&"http://localhost:8332".to_string(),
        Auth::UserPass("rpcuser".to_string(),
        "rpcpass".to_string())).unwrap();


    let txinfo = rpc.get_raw_transaction_info(&txid, None).unwrap();
    let tx = txinfo.transaction().unwrap();

    let prevouts = get_previous_outputs(&tx, &rpc);
    let tip = rpc.get_block_count().unwrap();

    let heur = check_heuristics(&tx, &prevouts, txinfo.confirmations, tip);

    println!("{}", txid);

    println!("{:?}", heur);

    let mut results = HashMap::<&str, WalletConfidence>::new();
    results.insert("Bitcoin Core", analyze_bitcoin_core(&heur));
    results.insert("Electrum",analyze_electrum(&heur));

    println!("Possible wallets:");
    for (wallet_name, result) in results.iter() {
        println!("\t{}\t\t{:?}", wallet_name, result);
    }
    println!();
}
