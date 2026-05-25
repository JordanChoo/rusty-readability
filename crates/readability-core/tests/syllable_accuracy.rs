use readability_core::syllables::count_syllables;

#[test]
fn known_syllable_counts() {
    let cases: &[(&str, u8)] = &[
        // 1 syllable
        ("cat", 1),
        ("dog", 1),
        ("the", 1),
        ("make", 1),
        ("time", 1),
        ("house", 1),
        ("once", 1),
        ("write", 1),
        ("bridge", 1),
        // 2 syllables
        ("happy", 2),
        ("water", 2),
        ("reading", 2),
        ("table", 2),
        ("little", 2),
        ("simple", 2),
        ("people", 2),
        ("business", 2),
        ("machine", 2),
        ("castle", 2),
        // 3 syllables
        ("computer", 3),
        ("beautiful", 3),
        ("chocolate", 3),
        ("exercise", 3),
        ("possible", 3),
        ("together", 3),
        ("family", 3),
        ("library", 3),
        ("separate", 3),
        ("restaurant", 3),
        // 4+ syllables
        ("international", 5),
        ("experience", 4),
        ("temperature", 4),
        ("individual", 5),
        ("vegetable", 4),
        ("comfortable", 3),
    ];

    let mut correct = 0;
    let total = cases.len();
    let mut failures = Vec::new();

    for &(word, expected) in cases {
        let actual = count_syllables(word);
        if actual == expected {
            correct += 1;
        } else {
            failures.push((word, expected, actual));
        }
    }

    let accuracy = correct as f64 / total as f64 * 100.0;

    if !failures.is_empty() {
        eprintln!("\nSyllable count failures ({} of {}):", failures.len(), total);
        for (word, expected, actual) in &failures {
            eprintln!("  {word}: expected {expected}, got {actual}");
        }
    }

    assert!(
        accuracy >= 85.0,
        "Syllable accuracy should be >= 85%, got {accuracy:.1}% ({correct}/{total})"
    );
}

#[test]
fn silent_e_words() {
    assert_eq!(count_syllables("make"), 1);
    assert_eq!(count_syllables("cake"), 1);
    assert_eq!(count_syllables("time"), 1);
    assert_eq!(count_syllables("take"), 1);
    assert_eq!(count_syllables("late"), 1);
    assert_eq!(count_syllables("name"), 1);
}

#[test]
fn terminal_le_words() {
    assert_eq!(count_syllables("table"), 2);
    assert_eq!(count_syllables("little"), 2);
    assert_eq!(count_syllables("simple"), 2);
    assert_eq!(count_syllables("castle"), 2);
    assert_eq!(count_syllables("bottle"), 2);
}

#[test]
fn y_as_vowel() {
    assert_eq!(count_syllables("happy"), 2);
    assert_eq!(count_syllables("baby"), 2);
    assert_eq!(count_syllables("mystery"), 3);
}

#[test]
fn exception_words_correct() {
    assert_eq!(count_syllables("business"), 2);
    assert_eq!(count_syllables("beautiful"), 3);
    assert_eq!(count_syllables("colonel"), 2);
    assert_eq!(count_syllables("queue"), 1);
    assert_eq!(count_syllables("recipe"), 3);
    assert_eq!(count_syllables("women"), 2);
    assert_eq!(count_syllables("wednesday"), 2);
}

#[test]
fn polysyllabic_threshold() {
    use readability_core::syllables::is_polysyllabic;
    assert!(is_polysyllabic("international"));
    assert!(is_polysyllabic("experience"));
    assert!(!is_polysyllabic("cat"));
    assert!(!is_polysyllabic("happy"));
}

#[test]
fn case_insensitive() {
    assert_eq!(count_syllables("HELLO"), count_syllables("hello"));
    assert_eq!(count_syllables("Beautiful"), count_syllables("beautiful"));
}

#[test]
fn minimum_one_syllable() {
    assert_eq!(count_syllables("a"), 1);
    assert_eq!(count_syllables("I"), 1);
    assert_eq!(count_syllables("x"), 1);
    assert_eq!(count_syllables("zz"), 1);
}

#[test]
fn cache_hit_rate_on_repeated_text() {
    use readability_core::syllables::cache;
    use readability_core::analyze::analyze;
    use readability_types::AnalyzeOptions;

    cache::reset_stats();

    let text = "The cat sat on the mat. The dog ran fast across the yard. \
                Birds sang in the tall trees near the old red barn. \
                The sun was bright and warm on that fine day. \
                Children played games in the open field by the lake. \
                It was a good day to be outside. The air was fresh and clean. \
                Flowers bloomed in every garden on the street.";

    let opts = AnalyzeOptions::default();
    let _ = analyze(text, &opts);
    let _ = analyze(text, &opts);

    let stats = cache::cache_stats();
    let hit_rate = stats.hit_rate();
    assert!(
        hit_rate >= 0.40,
        "Cache hit rate should be >= 40% after two passes over same text, got {:.1}% ({} hits, {} misses)",
        hit_rate * 100.0,
        stats.hits,
        stats.misses
    );
}
