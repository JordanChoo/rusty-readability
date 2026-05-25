use readability_core::analyze::analyze;
use readability_types::AnalyzeOptions;

fn opts() -> AnalyzeOptions {
    AnalyzeOptions {
        round: 4,
        ..Default::default()
    }
}

const SIMPLE_TEXT: &str = "The cat sat on the mat. The dog ran fast. The bird flew high.";

const MODERATE_TEXT: &str = "\
Reading is a complex cognitive process that requires the brain to decode symbols into meaningful \
language. Effective reading comprehension involves multiple strategies including summarization, \
questioning, and making connections to prior knowledge. Students who develop strong reading habits \
early in life tend to perform better across all academic subjects.";

const COMPLEX_TEXT: &str = "\
The epistemological implications of quantum indeterminacy fundamentally challenge our classical \
understanding of causality and deterministic prediction. Contemporary philosophers of science have \
increasingly recognized that the measurement problem represents not merely a technical obstacle \
but a profound conceptual difficulty requiring a reconceptualization of the relationship between \
observer and observed phenomena. The Copenhagen interpretation, while operationally successful, \
leaves unresolved questions about the ontological status of superposition states.";

#[test]
fn simple_text_all_scores_valid() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    assert!(result.scores.flesch_reading_ease.as_ref().unwrap().valid);
    assert!(result.scores.flesch_kincaid_grade.as_ref().unwrap().valid);
    assert!(result.scores.gunning_fog.as_ref().unwrap().valid);
    assert!(result.scores.coleman_liau.as_ref().unwrap().valid);
    assert!(result.scores.smog.as_ref().unwrap().valid);
    assert!(result.scores.automated_readability_index.as_ref().unwrap().valid);
    assert!(result.scores.dale_chall.as_ref().unwrap().valid);
    assert!(result.scores.spache.as_ref().unwrap().valid);
}

#[test]
fn simple_text_has_high_flesch_ease() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let fre = result.scores.flesch_reading_ease.unwrap().score;
    assert!(fre > 80.0, "Simple text should be easy to read, got FRE={fre}");
}

#[test]
fn simple_text_has_low_grade_level() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let fk = result.scores.flesch_kincaid_grade.unwrap().score;
    assert!(fk < 5.0, "Simple text should be low grade, got FK={fk}");
}

#[test]
fn complex_text_has_low_flesch_ease() {
    let result = analyze(COMPLEX_TEXT, &opts()).unwrap();
    let fre = result.scores.flesch_reading_ease.unwrap().score;
    assert!(fre < 40.0, "Complex text should be hard to read, got FRE={fre}");
}

#[test]
fn complex_text_has_high_grade_level() {
    let result = analyze(COMPLEX_TEXT, &opts()).unwrap();
    let fk = result.scores.flesch_kincaid_grade.unwrap().score;
    assert!(fk > 12.0, "Complex text should be high grade, got FK={fk}");
}

#[test]
fn moderate_text_between_extremes() {
    let result = analyze(MODERATE_TEXT, &opts()).unwrap();
    let fre = result.scores.flesch_reading_ease.unwrap().score;
    assert!(fre > 15.0 && fre < 80.0, "Moderate text FRE should be between 15-80, got {fre}");
}

#[test]
fn consensus_grade_in_reasonable_range() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let grade = result.primary.score.unwrap();
    assert!(grade >= 0.0 && grade <= 20.0, "Grade should be in 0-20 range, got {grade}");
}

#[test]
fn formula_relative_ordering() {
    let simple = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let complex = analyze(COMPLEX_TEXT, &opts()).unwrap();

    let simple_fk = simple.scores.flesch_kincaid_grade.unwrap().score;
    let complex_fk = complex.scores.flesch_kincaid_grade.unwrap().score;
    assert!(complex_fk > simple_fk, "Complex text should have higher FK grade");

    let simple_fog = simple.scores.gunning_fog.unwrap().score;
    let complex_fog = complex.scores.gunning_fog.unwrap().score;
    assert!(complex_fog > simple_fog, "Complex text should have higher Fog index");

    let simple_fre = simple.scores.flesch_reading_ease.unwrap().score;
    let complex_fre = complex.scores.flesch_reading_ease.unwrap().score;
    assert!(simple_fre > complex_fre, "Simple text should have higher FRE");
}

#[test]
fn stats_counts_correct_for_simple() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert_eq!(stats.sentences, 3);
    assert!(stats.words >= 14 && stats.words <= 16);
}

#[test]
fn dale_chall_simple_text_mostly_familiar() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let dc = result.scores.dale_chall.unwrap();
    assert!(dc.score < 7.0, "Simple text should be low Dale-Chall, got {}", dc.score);
}

#[test]
fn smog_requires_polysyllables() {
    let result = analyze(COMPLEX_TEXT, &opts()).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.polysyllables.unwrap() > 0, "Complex text should have polysyllables");
    assert!(result.scores.smog.unwrap().score > 10.0, "Complex text should have high SMOG");
}

#[test]
fn deterministic_results() {
    let r1 = analyze(MODERATE_TEXT, &opts()).unwrap();
    let r2 = analyze(MODERATE_TEXT, &opts()).unwrap();

    assert_eq!(
        r1.scores.flesch_reading_ease.unwrap().score,
        r2.scores.flesch_reading_ease.unwrap().score,
    );
    assert_eq!(
        r1.scores.flesch_kincaid_grade.unwrap().score,
        r2.scores.flesch_kincaid_grade.unwrap().score,
    );
    assert_eq!(r1.primary.score, r2.primary.score);
}

// --- Mutation-killing tests ---

#[test]
fn flesch_ease_simple_text_positive() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let raw = result.scores.flesch_reading_ease.unwrap().raw;
    assert!(raw > 0.0, "FRE must be positive for simple text, got {raw}");
    assert!(raw > 50.0, "FRE for simple text should be well above 50, got {raw}");
}

#[test]
fn gunning_fog_simple_text_positive() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let raw = result.scores.gunning_fog.unwrap().raw;
    assert!(raw > 0.0, "Gunning Fog must be positive for real text, got {raw}");
}

#[test]
fn ari_simple_text_value_check() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let raw = result.scores.automated_readability_index.unwrap().raw;
    assert!(raw > -15.0, "ARI should be above -15 for simple text, got {raw}");
    assert!(raw < 20.0, "ARI should be below 20 for simple text, got {raw}");
}

#[test]
fn spache_simple_text_positive() {
    let result = analyze(SIMPLE_TEXT, &opts()).unwrap();
    let raw = result.scores.spache.unwrap().raw;
    assert!(raw > 0.0, "Spache must be positive for real text, got {raw}");
}

#[test]
fn dale_chall_grade_band_boundaries() {
    use readability_core::formulas::dale_chall::dale_chall_grade_band;

    assert_eq!(dale_chall_grade_band(4.0).1, 4.0);
    assert_eq!(dale_chall_grade_band(4.99).1, 4.0);
    assert_eq!(dale_chall_grade_band(5.0).1, 5.5);
    assert_eq!(dale_chall_grade_band(5.99).1, 5.5);
    assert_eq!(dale_chall_grade_band(6.0).1, 7.5);
    assert_eq!(dale_chall_grade_band(6.99).1, 7.5);
    assert_eq!(dale_chall_grade_band(7.0).1, 9.5);
    assert_eq!(dale_chall_grade_band(7.99).1, 9.5);
    assert_eq!(dale_chall_grade_band(8.0).1, 11.5);
    assert_eq!(dale_chall_grade_band(8.99).1, 11.5);
    assert_eq!(dale_chall_grade_band(9.0).1, 14.0);
    assert_eq!(dale_chall_grade_band(9.99).1, 14.0);
    assert_eq!(dale_chall_grade_band(10.0).1, 16.0);
    assert_eq!(dale_chall_grade_band(15.0).1, 16.0);
    assert_eq!(dale_chall_grade_band(3.0).1, 4.0);
}

#[test]
fn html_input_stripped_correctly() {
    let html = "<p>The <b>cat</b> sat on the mat.</p><p>The dog ran fast.</p>";
    let mut o = opts();
    o.input_format = readability_types::InputFormat::Html;
    let result = analyze(html, &o).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 10, "HTML stripped text should have words");
    assert!(stats.sentences >= 2, "Should detect sentences in stripped HTML");
}

#[test]
fn markdown_input_stripped_correctly() {
    let md = "# Title\n\nThe **cat** sat on the [mat](http://example.com). The dog ran fast.";
    let mut o = opts();
    o.input_format = readability_types::InputFormat::MarkdownLite;
    let result = analyze(md, &o).unwrap();
    let stats = result.stats.unwrap();
    assert!(stats.words >= 10, "Markdown stripped text should have words");
}
