use crate::behaviors;

use behaviors::{
    classify_sequences,
    probably_anti_fee_snipe,
    probability_low_r_grinding,
    probability_bip69,
    SequenceType,
    spends_negative_ev,
};

use bitcoin::{
    OutPoint,
    TxOut,
};
use bitcoincore_rpc::Client;
use bitcoincore_rpc::json::GetRawTransactionResult;

use std::collections::HashMap;

pub fn maybe_electrum(txinfo: &GetRawTransactionResult, prevouts: &HashMap<OutPoint, TxOut>, rpc: &Client) -> bool {
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

    if spends_negative_ev(&tx, &prevouts) {
        return false;
    }

    return true;
}

