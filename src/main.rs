#![feature(map_first_last)]

extern crate bitcoincore_rpc;
extern crate bitcoin;

use bitcoin::hashes::hex::FromHex;
use bitcoin::hash_types::Txid;
use bitcoin::Transaction;
use bitcoin::blockdata::constants::MAX_SEQUENCE;
use bitcoincore_rpc::{Auth, Client, RpcApi};

use std::collections::BTreeSet;
use std::env;

const MAX_NON_FINAL_SEQUENCE: u32 = MAX_SEQUENCE - 1;
const MAX_BIP125_RBF_SEQUENCE: u32 = MAX_SEQUENCE - 2;

enum SequenceType {
    OnlyFinal,
    MixedFinal,
    OnlyNonFinal,
    OnlyRBF,
    MixedRBFNonFinal,
    Custom,
}

fn classify_sequences(tx: Transaction) -> SequenceType {
    let mut seqs = BTreeSet::new();
    for txin in tx.input.iter() {
        seqs.insert(txin.sequence);
    }
    if seqs.len() == 1 {
        match *seqs.first().unwrap() {
            MAX_SEQUENCE => { return SequenceType::OnlyFinal; }
            MAX_NON_FINAL_SEQUENCE => { return SequenceType::OnlyNonFinal; }
            MAX_BIP125_RBF_SEQUENCE => { return SequenceType::OnlyRBF; }
            _ => { return SequenceType::Custom; }
        }
    } else {
        match *seqs.last().unwrap() {
            MAX_SEQUENCE => { return SequenceType::MixedFinal; }
            MAX_NON_FINAL_SEQUENCE => { 
                if *seqs.first().unwrap() == MAX_BIP125_RBF_SEQUENCE {
                    return SequenceType::MixedRBFNonFinal;
                }
                return SequenceType::Custom;
            }
            _ => { return SequenceType::Custom; }
        }
    }
}

// fn probably_anti_fee_snipe(tx: Transaction) -> bool {
//    
//}

fn maybe_bitcoin_core(tx: Transaction) -> bool {
    match classify_sequences(tx) {
        SequenceType::OnlyRBF => {}
        SequenceType::OnlyNonFinal => {}
        _ => { return false; }
    }

   // if !probably_anti_fee_snipe(tx) {
   //     return false
   // }
    
    return true;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let txid = Txid::from_hex(&args[1]).unwrap();

    let rpc = Client::new(&"http://localhost:8332".to_string(),
                          Auth::UserPass("rpcuser".to_string(),
                                         "rpcpass".to_string())).unwrap();

    let tx = rpc.get_raw_transaction(&txid, None).unwrap();

    let is_core = maybe_bitcoin_core(tx);

    if is_core {
        println!("Maybe Bitcoin Core");
    } else {
        println!("Probably not Bitcoin COre");
    }
}
