pub fn flesch_ease_interpretation(score: f64) -> &'static str {
    match score {
        s if s >= 90.0 => "very easy",
        s if s >= 80.0 => "easy",
        s if s >= 70.0 => "fairly easy",
        s if s >= 60.0 => "standard / plain English",
        s if s >= 50.0 => "fairly difficult",
        s if s >= 30.0 => "difficult",
        _ => "very confusing",
    }
}

pub fn grade_band(grade: f64) -> String {
    let rounded = grade.round() as i32;
    match rounded {
        g if g <= 0 => "below 1st grade".to_string(),
        1..=12 => format!("{} grade", ordinal(rounded)),
        13..=16 => "college level".to_string(),
        _ => "college graduate level".to_string(),
    }
}

fn ordinal(n: i32) -> String {
    let suffix = match n % 10 {
        1 if n % 100 != 11 => "st",
        2 if n % 100 != 12 => "nd",
        3 if n % 100 != 13 => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}
