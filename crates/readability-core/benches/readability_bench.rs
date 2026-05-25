use criterion::{black_box, criterion_group, criterion_main, Criterion};
use readability_core::analyze::analyze;
use readability_types::AnalyzeOptions;

fn load_fixture(name: &str) -> String {
    let path = format!(
        "{}/fixtures/texts/{name}",
        env!("CARGO_MANIFEST_DIR").replace("/crates/readability-core", "")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {path}: {e}"))
}

fn bench_short_text(c: &mut Criterion) {
    let text = load_fixture("simple.txt");
    let opts = AnalyzeOptions::default();
    c.bench_function("analyze_short_86w", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

fn bench_medium_text(c: &mut Criterion) {
    let text = load_fixture("medium_500w.txt");
    let opts = AnalyzeOptions::default();
    c.bench_function("analyze_medium_500w", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

fn bench_long_text(c: &mut Criterion) {
    let text = load_fixture("long_5000w.txt");
    let opts = AnalyzeOptions::default();
    c.bench_function("analyze_long_5000w", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

fn bench_batch(c: &mut Criterion) {
    let text = load_fixture("medium_500w.txt");
    let opts = AnalyzeOptions::default();
    let inputs: Vec<(&str, &str)> = (0..10)
        .map(|i| {
            let _ = i;
            ("doc", text.as_str())
        })
        .collect();

    c.bench_function("batch_10x500w", |b| {
        b.iter(|| {
            readability_core::analyze::analyze_batch(black_box(&inputs), black_box(&opts))
        })
    });
}

fn bench_children_text(c: &mut Criterion) {
    let text = load_fixture("children.txt");
    let opts = AnalyzeOptions::default();
    c.bench_function("analyze_children_72w", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

fn bench_academic_text(c: &mut Criterion) {
    let text = load_fixture("academic.txt");
    let opts = AnalyzeOptions::default();
    c.bench_function("analyze_academic_177w", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

fn bench_html_input(c: &mut Criterion) {
    let text = load_fixture("html_input.html");
    let opts = AnalyzeOptions::default();
    c.bench_function("analyze_html", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

fn bench_with_paragraphs(c: &mut Criterion) {
    let text = load_fixture("medium_500w.txt");
    let opts = AnalyzeOptions {
        include_paragraphs: true,
        include_hardest_sentences: 5,
        ..Default::default()
    };
    c.bench_function("analyze_500w_with_paragraphs", |b| {
        b.iter(|| analyze(black_box(&text), black_box(&opts)))
    });
}

criterion_group!(
    benches,
    bench_short_text,
    bench_medium_text,
    bench_long_text,
    bench_batch,
    bench_children_text,
    bench_academic_text,
    bench_html_input,
    bench_with_paragraphs,
);
criterion_main!(benches);
