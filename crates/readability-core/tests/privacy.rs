use readability_core::analyze::analyze;
use readability_types::AnalyzeOptions;

const SENSITIVE_TEXT: &str = "John Smith lives at 123 Main Street and his SSN is 555-12-3456.";

#[test]
fn error_does_not_contain_user_text() {
    let err = analyze("", &AnalyzeOptions::default()).unwrap_err();
    let err_str = err.to_string();
    assert!(!err_str.contains("John"));
    assert!(!err_str.contains("Smith"));
    assert!(!err_str.contains("555-12-3456"));
    assert!(!err_str.contains("Main Street"));
}

#[test]
fn result_does_not_echo_raw_text() {
    let result = analyze(SENSITIVE_TEXT, &AnalyzeOptions::default()).unwrap();
    let json = serde_json::to_string(&result).unwrap();

    assert!(!json.contains("555-12-3456"), "SSN should never appear in output");
    assert!(!json.contains("123 Main Street"), "Address should never appear in full output");
}

#[test]
fn text_hash_is_not_reversible() {
    let result = analyze(SENSITIVE_TEXT, &AnalyzeOptions::default()).unwrap();
    let hash = result.input.unwrap().text_hash.unwrap();
    assert!(hash.len() == 64, "SHA-256 hash should be 64 hex chars");
    assert!(!hash.contains("John"));
}

#[test]
fn warnings_do_not_contain_text() {
    let result = analyze("Short.", &AnalyzeOptions::default()).unwrap();
    if let Some(warnings) = &result.warnings {
        let warnings_json = serde_json::to_string(warnings).unwrap();
        assert!(!warnings_json.contains("Short."), "Warning messages should not echo user text");
    }
}

#[test]
fn hardest_sentence_preview_is_bounded() {
    let long_sentence = format!("{}.", "word ".repeat(200));
    let mut opts = AnalyzeOptions::default();
    opts.include_hardest_sentences = 3;
    let result = analyze(&long_sentence, &opts).unwrap();
    if let Some(hardest) = &result.hardest_sentences {
        for h in hardest {
            assert!(h.preview.len() <= 120, "Preview should be bounded at 120 chars, got {}", h.preview.len());
        }
    }
}
