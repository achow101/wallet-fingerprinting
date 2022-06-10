use crate::behaviors;

use behaviors::{
    classify_sequences,
    probably_anti_fee_snipe,
    probability_low_r_grinding,
    probability_bip69,
    SequenceType,
};

use crate::util;

use util::WalletConfidence;

use bitcoin::{
    OutPoint,
    TxOut,
};
use bitcoincore_rpc::Client;
use bitcoincore_rpc::json::GetRawTransactionResult;

use std::collections::HashMap;

pub fn maybe_bitcoin_core(txinfo: &GetRawTransactionResult, _prevouts: &HashMap<OutPoint, TxOut>, rpc: &Client) -> WalletConfidence {
    let tx = txinfo.transaction().unwrap();

    if tx.version != 2 {
        return WalletConfidence::DefinitelyNot;
    }

    match classify_sequences(&tx) {
        SequenceType::OnlyRBF => {}
        SequenceType::OnlyNonFinal => {}
        _ => { return WalletConfidence::DefinitelyNot; }
    }

    let prob_low_r = probability_low_r_grinding(&tx);
    if prob_low_r <= 0.5 {
        return WalletConfidence::DefinitelyNot;
    }

    if !probably_anti_fee_snipe(&tx, txinfo.confirmations, rpc) {
        return WalletConfidence::ProbablyNot;
    }

    let prob_bip69 = probability_bip69(&tx);
    match prob_bip69 {
        Some(p) => {
            if p > 0.5 {
                return WalletConfidence::ProbablyNot;
            }
        }
        None => {}
    }

    return WalletConfidence::MaybeYes;
}

