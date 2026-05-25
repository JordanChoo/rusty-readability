pub use readability_types::{Warning, WarningCode};

use crate::stats::TextStats;

pub fn generate_warnings(stats: &TextStats) -> Vec<Warning> {
    let mut warnings = Vec::new();

    if stats.words == 0 {
        warnings.push(Warning::new(WarningCode::EmptyText));
        return warnings;
    }

    if stats.words < 100 {
        warnings.push(Warning::new(WarningCode::ShortText));
    }

    if stats.sentences < 3 {
        warnings.push(Warning::new(WarningCode::FewSentences));
    }

    if stats.sentences <= 1 && stats.words > 0 {
        warnings.push(Warning::new(WarningCode::NoSentenceTerminator));
    }

    if stats.sentences < 30 {
        warnings.push(Warning::new(WarningCode::SmogShortSample));
    }

    warnings
}
