pub fn difficulty_label(avg_sentence_length: f64) -> &'static str {
    if avg_sentence_length < 15.0 {
        "easy"
    } else if avg_sentence_length <= 25.0 {
        "moderate"
    } else {
        "hard"
    }
}
