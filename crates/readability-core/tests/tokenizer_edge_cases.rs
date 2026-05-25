use readability_core::analyze::analyze;
use readability_types::AnalyzeOptions;

fn opts() -> AnalyzeOptions {
    AnalyzeOptions::default()
}

#[test]
fn single_word() {
    let result = analyze("Hello", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.words, 1);
    assert_eq!(stats.sentences, 1);
}

#[test]
fn single_sentence_no_punctuation() {
    let result = analyze("The cat sat on the mat", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 6);
    assert_eq!(stats.sentences, 1);
}

#[test]
fn multiple_exclamation_marks() {
    let result = analyze("Wow!!! That is amazing!!!", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 2);
}

#[test]
fn ellipsis_handling() {
    let result = analyze("Well... I think so. Maybe not.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.sentences >= 2);
}

#[test]
fn abbreviation_mr_mrs() {
    let result = analyze("Mr. Smith and Mrs. Jones went to Dr. Brown.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 1);
}

#[test]
fn contraction_handling() {
    let result = analyze("Don't stop. Can't believe it's true. They're here.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 7, "Contractions should count as words, got {}", stats.words);
}

#[test]
fn hyphenated_words() {
    let result = analyze("The well-known state-of-the-art system works.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 3);
}

#[test]
fn numbers_excluded() {
    let result = analyze("There are 42 cats and 100 dogs.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    // Pure numbers should not count as words
    assert!(stats.words >= 4 && stats.words <= 6);
}

#[test]
fn unicode_smart_quotes() {
    let result = analyze("\u{201C}Hello,\u{201D} she said. \u{201C}Goodbye.\u{201D}", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 3);
    assert_eq!(stats.sentences, 2);
}

#[test]
fn curly_apostrophe() {
    let result = analyze("Don\u{2019}t worry. It\u{2019}s fine.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 4);
}

#[test]
fn paragraph_boundaries() {
    let result = analyze("First paragraph.\n\nSecond paragraph.\n\nThird paragraph.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.paragraphs, 3);
    assert_eq!(stats.sentences, 3);
}

#[test]
fn very_long_sentence() {
    let words: Vec<&str> = std::iter::repeat("word").take(100).collect();
    let text = format!("{}.", words.join(" "));
    let result = analyze(&text, &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 1);
    assert_eq!(stats.words, 100);
}

#[test]
fn mixed_punctuation() {
    let result = analyze("Is it true? Yes! Of course. Really?", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 4);
}

#[test]
fn us_acronym_not_sentence_break() {
    let result = analyze("The U.S. government issued a statement.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 1);
}

#[test]
fn decimal_numbers_not_sentence_break() {
    let result = analyze("Version 3.14 is now stable. Use it.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 2);
}

#[test]
fn bom_stripped() {
    let result = analyze("\u{FEFF}Hello world. Test sentence.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 4);
}

#[test]
fn tabs_and_extra_spaces() {
    let result = analyze("Hello    world.\t\tTest    sentence.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 4);
}

#[test]
fn em_dash_handling() {
    let result = analyze("The cat\u{2014}a big one\u{2014}sat down.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 5);
}

#[test]
fn empty_returns_error() {
    let result = analyze("", &opts());
    assert!(result.is_err());
}

#[test]
fn whitespace_only_returns_error() {
    let result = analyze("   \n\n\t  ", &opts());
    assert!(result.is_err());
}

#[test]
fn single_character() {
    let result = analyze("A", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.words, 1);
}

#[test]
fn type_token_ratio() {
    let result = analyze("The cat sat on the mat. The dog ran.", &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.type_token_ratio > 0.0 && stats.type_token_ratio <= 1.0);
}

#[test]
fn reading_time_reasonable() {
    let text = std::iter::repeat("word ").take(238).collect::<String>() + "done.";
    let result = analyze(&text, &opts()).unwrap();
    let stats = result.stats.unwrap();
    // ~239 words at 238 WPM should be ~60 seconds
    assert!(stats.reading_time_seconds >= 50 && stats.reading_time_seconds <= 70);
}
