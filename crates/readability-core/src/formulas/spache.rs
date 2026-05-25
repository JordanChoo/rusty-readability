use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub fn spache(stats: &TextStats) -> Result<f64, FormulaErrorCode> {
    if stats.words == 0 || stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let sp_unfam_pct = stats
        .spache_unique_unfamiliar_percentage
        .ok_or(FormulaErrorCode::Internal)?;
    let asl = stats.words as f64 / stats.sentences as f64;
    Ok(0.121 * asl + 0.082 * sp_unfam_pct + 0.659)
}
