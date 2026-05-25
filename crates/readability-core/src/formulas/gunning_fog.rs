use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub fn gunning_fog(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.words == 0 || stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let complex_words = stats.complex_words.ok_or(FormulaErrorCode::Internal)?;
    let asl = stats.words as f64 / stats.sentences as f64;
    let pcw = 100.0 * complex_words as f64 / stats.words as f64;
    Ok(0.4 * (asl + pcw))
}
