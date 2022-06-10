use crate::behaviors;

use behaviors::{
    Heuristics,
    SequenceType,
};

use crate::util;

use util::WalletConfidence;

use rawtx_rs::input::InputType;

use std::collections::HashSet;

pub fn analyze_bitcoin_core(h: &Heuristics) -> WalletConfidence {
    if h.tx_version != 2 {
        return WalletConfidence::DefinitelyNot;
    }

    match h.sequence_type {
        SequenceType::OnlyRBF => {}
        SequenceType::OnlyNonFinal => {}
        _ => { return WalletConfidence::DefinitelyNot; }
    }

    if h.prob_low_r <= 0.5 {
        return WalletConfidence::DefinitelyNot;
    }

    let allowed_input_types = HashSet::from([InputType::P2pk, InputType::P2pkh, InputType::P2shP2wpkh, InputType::P2wpkh, InputType::P2trkp]);
    let diff: HashSet<_> = h.input_types.difference(&allowed_input_types).collect();
    if !diff.is_empty() {
        return WalletConfidence::DefinitelyNot;
    }

    if !h.anti_fee_snipe {
        return WalletConfidence::ProbablyNot;
    }

    match h.prob_bip69 {
        Some(p) => {
            if p > 0.5 {
                return WalletConfidence::ProbablyNot;
            }
        }
        None => {}
    }

    return WalletConfidence::MaybeYes;
}

