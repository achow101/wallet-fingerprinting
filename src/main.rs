#![feature(map_first_last)]

extern crate bitcoincore_rpc;
extern crate bitcoin;

use bitcoin::hashes::hex::FromHex;
use bitcoin::hash_types::Txid;
use bitcoin::{
    OutPoint,
    Transaction,
    TxOut,
};
use bitcoin::blockdata::constants::MAX_SEQUENCE;
use bitcoin::secp256k1::Signature;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::json::GetRawTransactionResult;

use factorial::Factorial;

use rawtx_rs::tx::TxInfo;
use rawtx_rs::script::SignatureType;

use std::collections::{
    BTreeSet,
    HashMap,
};
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

fn classify_sequences(tx: &Transaction) -> SequenceType {
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

fn probably_anti_fee_snipe(tx: &Transaction, confs: Option<u32>, rpc: &Client) -> bool {
    if tx.lock_time == 0 {
        return false;
    }

    let mut block_height = rpc.get_block_count().unwrap();
    if let Some(c) = confs {
        block_height -= u64::from(c) - 1;
    }

    return u64::from(tx.lock_time) >= block_height - 100;
}

fn probability_low_r_grinding(tx: &Transaction) -> f32 {
    let txinfo = TxInfo::new(tx).unwrap();

    let mut count_sigs: u32 = 0;
    for txininfo in txinfo.input_infos.iter() {
        for sigsinfo in txininfo.signature_info.iter() {
            if let SignatureType::Ecdsa = sigsinfo.sig_type {
                count_sigs += 1;
                let sig = Signature::from_der(sigsinfo.signature.split_last().unwrap().1).unwrap();
                let compact_sig = sig.serialize_compact();
                if compact_sig[0] >= 0x80 {
                    return 0.0;
                }
            }
        }
    }

    return 1.0 - 0.5_f32.powf(count_sigs as f32);
}

fn probability_bip69(tx: &Transaction) -> Option<f64> {
    // Not enough data for 1-in and 1-out to classify BIP69
    if tx.input.len() == 1 && tx.output.len() == 1 {
        return None;
    }

    let txinfo = TxInfo::new(tx).unwrap();

    if !txinfo.is_bip69_compliant() {
        return Some(0.0);
    }

    // Probability of not BIP69
    let mut prob: f64 = 1.0;
    let input_perms = tx.input.len().checked_factorial();
    match input_perms {
        Some(p) => {
            prob *= 1.0_f64 / p as f64;
        }
        None => {
            return Some(1.0);
        }
    }
    let output_perms = tx.output.len().checked_factorial();
    match output_perms {
        Some(p) => {
            prob *= 1.0_f64 / p as f64;
        }
        None => {
            return Some(1.0);
        }
    }

    // 1 - p for probability of being BIP69
    return Some(1.0 - prob);
}

fn maybe_bitcoin_core(txinfo: &GetRawTransactionResult, prevouts: &HashMap<OutPoint, TxOut>, rpc: &Client) -> bool {
    let tx = txinfo.transaction().unwrap();

    if tx.version != 2 {
        return false;
    }

    match classify_sequences(&tx) {
        SequenceType::OnlyRBF => {}
        SequenceType::OnlyNonFinal => {}
        _ => { return false; }
    }

    if !probably_anti_fee_snipe(&tx, txinfo.confirmations, rpc) {
        return false;
    }

    let prob_low_r = probability_low_r_grinding(&tx);
    if prob_low_r <= 0.5 {
        return false;
    }

    let prob_bip69 = probability_bip69(&tx);
    match prob_bip69 {
        Some(p) => {
            if p > 0.5 {
                return false;
            }
        }
        None => {}
    }

    return true;
}

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
}
