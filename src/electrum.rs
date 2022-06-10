use crate::behaviors;

use behaviors::{
    Heuristics,
    SequenceType,
};

use crate::util;

use util::WalletConfidence;

use rawtx_rs::input::InputType;

use std::collections::HashSet;

pub fn analyze_electrum(h: &Heuristics) -> WalletConfidence {
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

    if h.mixed_input_types {
        return WalletConfidence::DefinitelyNot;
    }

    match h.maybe_same_change_type {
        Some(b) => {
            if !b {
                return WalletConfidence::DefinitelyNot;
            }
        }
        None => {}
    }

    let allowed_input_types = HashSet::from([InputType::P2pkh, InputType::P2shP2wpkh, InputType::P2wpkh]);
    let diff: HashSet<_> = h.input_types.difference(&allowed_input_types).collect();
    if !diff.is_empty() {
        return WalletConfidence::DefinitelyNot;
    }

    if !h.anti_fee_snipe {
        return WalletConfidence::ProbablyNot;
    }

    if h.neg_ev {
        return WalletConfidence::ProbablyNot;
    }

    match h.prob_bip69 {
        Some(p) => {
            if p < 0.5 {
                return WalletConfidence::ProbablyNot;
            }
        }
        None => {}
    }

    return WalletConfidence::MaybeYes;
}

