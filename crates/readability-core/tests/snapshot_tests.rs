use readability_core::analyze::analyze;
use readability_types::AnalyzeOptions;

fn default_opts() -> AnalyzeOptions {
    AnalyzeOptions::default()
}

fn full_opts() -> AnalyzeOptions {
    AnalyzeOptions {
        include_paragraphs: true,
        include_hardest_sentences: 3,
        ..Default::default()
    }
}

fn load_fixture(name: &str) -> String {
    let path = format!(
        "{}/fixtures/texts/{name}",
        env!("CARGO_MANIFEST_DIR").replace("/crates/readability-core", "")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {path}: {e}"))
}

#[test]
fn snapshot_simple_text() {
    let text = load_fixture("simple.txt");
    let result = analyze(&text, &default_opts()).unwrap();
    insta::assert_json_snapshot!("simple_default", result);
}

#[test]
fn snapshot_simple_text_with_paragraphs() {
    let text = load_fixture("simple.txt");
    let result = analyze(&text, &full_opts()).unwrap();
    insta::assert_json_snapshot!("simple_full", result);
}

#[test]
fn snapshot_academic_text() {
    let text = load_fixture("academic.txt");
    let result = analyze(&text, &default_opts()).unwrap();
    insta::assert_json_snapshot!("academic_default", result);
}

#[test]
fn snapshot_academic_text_with_paragraphs() {
    let text = load_fixture("academic.txt");
    let result = analyze(&text, &full_opts()).unwrap();
    insta::assert_json_snapshot!("academic_full", result);
}

#[test]
fn snapshot_children_text() {
    let text = load_fixture("children.txt");
    let result = analyze(&text, &default_opts()).unwrap();
    insta::assert_json_snapshot!("children_default", result);
}

#[test]
fn snapshot_html_input() {
    let text = load_fixture("html_input.html");
    let result = analyze(&text, &default_opts()).unwrap();
    insta::assert_json_snapshot!("html_default", result);
}

#[test]
fn snapshot_markdown_input() {
    let text = load_fixture("markdown_input.md");
    let result = analyze(&text, &default_opts()).unwrap();
    insta::assert_json_snapshot!("markdown_default", result);
}

#[test]
fn snapshot_short_text() {
    let text = load_fixture("short.txt");
    let result = analyze(&text, &default_opts()).unwrap();
    insta::assert_json_snapshot!("short_default", result);
}

#[test]
fn snapshot_no_hash() {
    let text = load_fixture("simple.txt");
    let mut opts = default_opts();
    opts.hash_text = false;
    let result = analyze(&text, &opts).unwrap();
    insta::assert_json_snapshot!("simple_no_hash", result);
}

#[test]
fn snapshot_no_stats() {
    let text = load_fixture("simple.txt");
    let mut opts = default_opts();
    opts.include_stats = false;
    let result = analyze(&text, &opts).unwrap();
    insta::assert_json_snapshot!("simple_no_stats", result);
}
