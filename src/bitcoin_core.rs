use crate::behaviors;

use behaviors::{
    Heuristics,
    SequenceType,
};

use crate::util;

use util::WalletConfidence;

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

