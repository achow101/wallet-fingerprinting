use bitcoin::{
    OutPoint,
    Transaction,
    TxIn,
    TxOut,
};
use bitcoin::blockdata::constants::MAX_SEQUENCE;
use bitcoin::consensus::Encodable;
use bitcoin::secp256k1::Signature;

use factorial::Factorial;

use rawtx_rs::tx::TxInfo;
use rawtx_rs::script::SignatureType;

use std::collections::{
    BTreeSet,
    HashMap,
};

const MAX_NON_FINAL_SEQUENCE: u32 = MAX_SEQUENCE - 1;
const MAX_BIP125_RBF_SEQUENCE: u32 = MAX_SEQUENCE - 2;

pub enum SequenceType {
    OnlyFinal,
    MixedFinal,
    OnlyNonFinal,
    OnlyRBF,
    MixedRBFNonFinal,
    Custom,
}

pub fn classify_sequences(tx: &Transaction) -> SequenceType {
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

pub fn get_input_vsize(txin: &TxIn) -> usize {
    let s = Vec::<u8>::new();
    let txin_size = txin.consensus_encode(s).unwrap();
    let s2 = Vec::<u8>::new();
    let wit_size = txin.witness.consensus_encode(s2).unwrap();
    return txin_size + (wit_size / 4);
}

pub fn probability_low_r_grinding(tx: &Transaction) -> f32 {
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

pub fn probably_anti_fee_snipe(tx: &Transaction, confs: Option<u32>, tip_height: u64) -> bool {
    if tx.lock_time == 0 {
        return false;
    }

    let mut block_height = tip_height;
    if let Some(c) = confs {
        block_height -= u64::from(c) - 1;
    }

    return u64::from(tx.lock_time) >= block_height - 100;
}

pub fn probability_bip69(tx: &Transaction) -> Option<f64> {
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

pub fn spends_negative_ev(tx: &Transaction, prevouts: &HashMap<OutPoint, TxOut>) -> bool {
    let mut fee: u64 = 0;
    for txin in tx.input.iter() {
        fee += prevouts.get(&txin.previous_output).unwrap().value;
    }
    for txout in tx.output.iter() {
        fee -= txout.value;
    }
    let feerate = fee as f64 / (tx.get_weight() as f64 / 4_f64);
    for txin in tx.input.iter() {
        let txin_vsize = get_input_vsize(&txin);
        let fee = feerate * txin_vsize as f64;
        let ev = prevouts.get(&txin.previous_output).unwrap().value as f64 - fee;
        if ev <= 0.0 {
            return true;
        }
    }
    return false;
}

pub struct Heuristics {
    pub tx_version: i32,
    pub sequence_type: SequenceType,
    pub anti_fee_snipe: bool,
    pub prob_low_r: f32,
    pub prob_bip69: Option<f64>,
    pub neg_ev: bool,
}

pub fn check_heuristics(tx: &Transaction, prevouts: &HashMap<OutPoint, TxOut>, confs: Option<u32>, tip_height: u64) -> Heuristics {
    let h = Heuristics {
        tx_version: tx.version,
        sequence_type: classify_sequences(&tx),
        anti_fee_snipe: probably_anti_fee_snipe(&tx, confs, tip_height),
        prob_low_r: probability_low_r_grinding(&tx),
        prob_bip69: probability_bip69(&tx),
        neg_ev: spends_negative_ev(&tx, &prevouts),
    };
    return h;
}
