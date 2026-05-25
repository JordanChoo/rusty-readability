use proptest::prelude::*;
use readability_core::analyze::{analyze, analyze_batch};
use readability_types::AnalyzeOptions;

fn default_opts() -> AnalyzeOptions {
    AnalyzeOptions::default()
}

fn assert_result_sane(result: &readability_types::AnalysisResult) {
    if let Some(ref stats) = result.stats {
        assert!(stats.words > 0);
        assert!(stats.sentences > 0);
        assert!(stats.average_words_per_sentence.is_finite());
        assert!(stats.average_characters_per_word.is_finite());
        assert!(stats.type_token_ratio.is_finite());
        assert!(stats.hapax_percentage.is_finite());
        assert!(stats.average_words_per_sentence >= 0.0);
        assert!(stats.average_characters_per_word >= 0.0);
        assert!(stats.type_token_ratio >= 0.0);
        assert!(stats.hapax_percentage >= 0.0);
        if let Some(spw) = stats.average_syllables_per_word {
            assert!(spw.is_finite());
            assert!(spw >= 0.0);
        }
    }

    fn check_score(s: &readability_types::Score) {
        assert!(s.raw.is_finite(), "raw score is not finite: {}", s.raw);
        assert!(s.score.is_finite(), "score is not finite: {}", s.score);
        if let Some(n) = s.normalized_0_100 {
            assert!(n.is_finite());
        }
    }

    if let Some(ref s) = result.scores.flesch_reading_ease { check_score(s); }
    if let Some(ref s) = result.scores.flesch_kincaid_grade { check_score(s); }
    if let Some(ref s) = result.scores.gunning_fog { check_score(s); }
    if let Some(ref s) = result.scores.coleman_liau { check_score(s); }
    if let Some(ref s) = result.scores.smog { check_score(s); }
    if let Some(ref s) = result.scores.automated_readability_index { check_score(s); }
    if let Some(ref s) = result.scores.spache { check_score(s); }

    if let Some(ref dc) = result.scores.dale_chall {
        assert!(dc.raw.is_finite());
        assert!(dc.score.is_finite());
        assert!(dc.raw_unadjusted.is_finite());
    }

    if let Some(score) = result.primary.score {
        assert!(score.is_finite());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn no_panic_on_arbitrary_utf8(s in "\\PC{1,500}") {
        let result = analyze(&s, &default_opts());
        match result {
            Ok(r) => assert_result_sane(&r),
            Err(_) => {} // EmptyText errors are fine
        }
    }

    #[test]
    fn no_panic_on_ascii(s in "[a-zA-Z0-9 .!?,;:'\"-]{1,500}") {
        let result = analyze(&s, &default_opts());
        match result {
            Ok(r) => assert_result_sane(&r),
            Err(_) => {}
        }
    }

    #[test]
    fn no_panic_on_punctuation_only(s in "[!?.,:;\"'\\-()\\[\\]{}]{1,100}") {
        let result = analyze(&s, &default_opts());
        // Should either error (no words) or produce sane results
        match result {
            Ok(r) => assert_result_sane(&r),
            Err(_) => {}
        }
    }

    #[test]
    fn no_panic_on_whitespace_heavy(s in "[ \\t\\n\\r]{0,50}[a-z]{1,10}[ \\t\\n\\r]{0,50}") {
        let result = analyze(&s, &default_opts());
        match result {
            Ok(r) => assert_result_sane(&r),
            Err(_) => {}
        }
    }

    #[test]
    fn no_panic_on_html_like(s in "<[a-z]{1,5}>[a-zA-Z ]{1,100}</[a-z]{1,5}>") {
        let result = analyze(&s, &default_opts());
        match result {
            Ok(r) => assert_result_sane(&r),
            Err(_) => {}
        }
    }
}

#[test]
fn no_panic_long_single_word() {
    let word = "a".repeat(100_000);
    let result = analyze(&word, &default_opts());
    match result {
        Ok(r) => assert_result_sane(&r),
        Err(_) => {}
    }
}

#[test]
fn no_panic_repetitive_text() {
    let text = "word ".repeat(5000);
    let result = analyze(&text, &default_opts()).unwrap();
    assert_result_sane(&result);
    assert!(result.stats.as_ref().unwrap().words >= 4000);
}

#[test]
fn no_panic_embedded_nulls() {
    let text = "Hello\0world. This\0is a\0test.";
    let result = analyze(text, &default_opts());
    match result {
        Ok(r) => assert_result_sane(&r),
        Err(_) => {}
    }
}

#[test]
fn no_panic_emoji_text() {
    let text = "I love coding! It makes me happy. The weather is great today.";
    let result = analyze(text, &default_opts()).unwrap();
    assert_result_sane(&result);
}

#[test]
fn no_panic_mixed_scripts() {
    let text = "Hello world. Bonjour le monde. Hola mundo.";
    let result = analyze(text, &default_opts()).unwrap();
    assert_result_sane(&result);
}

#[test]
fn batch_isolation() {
    let valid = "The cat sat on the mat. The dog ran fast.";
    let empty = "";
    let adversarial = "!@#$%^&*()";

    let inputs = vec![
        ("valid1", valid),
        ("empty", empty),
        ("adversarial", adversarial),
        ("valid2", valid),
    ];

    let results = analyze_batch(&inputs, &default_opts());

    assert_eq!(results.len(), 4);
    assert!(results[0].1.is_ok(), "valid1 should succeed");
    assert!(results[1].1.is_err(), "empty should fail");
    // adversarial may succeed or fail depending on content
    assert!(results[3].1.is_ok(), "valid2 should succeed regardless of earlier failures");

    if let Ok(r1) = &results[0].1 {
        if let Ok(r2) = &results[3].1 {
            assert_eq!(
                r1.stats.as_ref().unwrap().words,
                r2.stats.as_ref().unwrap().words,
                "Same text should produce identical word counts in batch"
            );
        }
    }
}

#[test]
fn empty_returns_error() {
    assert!(analyze("", &default_opts()).is_err());
    assert!(analyze("   ", &default_opts()).is_err());
    assert!(analyze("\n\n\n", &default_opts()).is_err());
    assert!(analyze("\t  \t", &default_opts()).is_err());
}

#[test]
fn json_serializable_no_nan() {
    let texts = vec![
        "Simple text.",
        "A single word",
        "!!!",
        "The quick brown fox jumped over the lazy dog.",
    ];

    for text in texts {
        if let Ok(result) = analyze(text, &default_opts()) {
            let json = serde_json::to_string(&result);
            assert!(json.is_ok(), "Failed to serialize result for: {text}");
            let json_str = json.unwrap();
            assert!(!json_str.contains("NaN"), "NaN found in JSON for: {text}");
            assert!(!json_str.contains("Infinity"), "Infinity found in JSON for: {text}");
        }
    }
}
