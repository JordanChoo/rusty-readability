use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub fn flesch_reading_ease(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.words == 0 || stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let syllables = stats.syllables.ok_or(FormulaErrorCode::Internal)?;
    let asl = stats.words as f64 / stats.sentences as f64;
    let asw = syllables as f64 / stats.words as f64;
    Ok(206.835 - 1.015 * asl - 84.6 * asw)
}
