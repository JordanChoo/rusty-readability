pub static ABBREVIATIONS: &[&str] = &[
    "mr", "mrs", "ms", "dr", "prof", "sr", "jr", "st", "vs", "etc",
    "inc", "ltd", "dept", "est", "approx", "govt", "assn", "bros",
    "corp", "co", "ave", "blvd", "gen", "gov", "hon", "sgt", "capt",
    "cmdr", "lt", "col", "maj", "pvt", "rev", "vol",
];

pub static ABBREVIATIONS_WITH_PERIOD: &[&str] = &[
    "e.g", "i.e", "a.m", "p.m", "u.s", "u.k", "a.i",
];

pub fn is_abbreviation(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    let trimmed = lower.trim_end_matches('.');
    ABBREVIATIONS.contains(&trimmed) || ABBREVIATIONS_WITH_PERIOD.contains(&trimmed)
}
