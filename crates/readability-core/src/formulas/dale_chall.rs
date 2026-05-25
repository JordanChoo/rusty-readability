use crate::stats::TextStats;
use readability_types::FormulaErrorCode;

pub struct DaleChallResult {
    pub raw_unadjusted: f64,
    pub adjusted: f64,
}

pub fn dale_chall(stats: &TextStats) -> Result<DaleChallResult, FormulaErrorCode> {
    if stats.words == 0 || stats.sentences == 0 {
        return Err(FormulaErrorCode::InsufficientText);
    }
    let dc_pdw = stats
        .dale_chall_difficult_percentage
        .ok_or(FormulaErrorCode::Internal)?;
    let asl = stats.words as f64 / stats.sentences as f64;

    let raw = 0.1579 * dc_pdw + 0.0496 * asl;
    let adjusted = if dc_pdw > 5.0 { raw + 3.6365 } else { raw };

    Ok(DaleChallResult {
        raw_unadjusted: raw,
        adjusted,
    })
}

pub fn dale_chall_grade_band(score: f64) -> (&'static str, f64) {
    match score {
        s if s < 5.0 => ("4th grade and below", 4.0),
        s if s < 6.0 => ("5th to 6th grade", 5.5),
        s if s < 7.0 => ("7th to 8th grade", 7.5),
        s if s < 8.0 => ("9th to 10th grade", 9.5),
        s if s < 9.0 => ("11th to 12th grade", 11.5),
        s if s < 10.0 => ("13th to 15th grade (college)", 14.0),
        _ => ("16th grade and above (college graduate)", 16.0),
    }
}
