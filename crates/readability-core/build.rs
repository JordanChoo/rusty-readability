use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    generate_word_set(
        "../../data/dale_chall_easy_words.txt",
        &out_dir,
        "dale_chall_words.rs",
        "DALE_CHALL_EASY_WORDS",
    );

    generate_word_set(
        "../../data/spache_easy_words.txt",
        &out_dir,
        "spache_words.rs",
        "SPACHE_EASY_WORDS",
    );

    println!("cargo::rerun-if-changed=../../data/dale_chall_easy_words.txt");
    println!("cargo::rerun-if-changed=../../data/spache_easy_words.txt");
    println!("cargo::rerun-if-changed=build.rs");
}

fn generate_word_set(input_path: &str, out_dir: &str, out_file: &str, set_name: &str) {
    let contents = fs::read_to_string(input_path)
        .unwrap_or_else(|e| panic!("Failed to read {input_path}: {e}"));

    let words: Vec<&str> = contents
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let dest = Path::new(out_dir).join(out_file);
    let file = fs::File::create(&dest).unwrap();
    let mut writer = BufWriter::new(file);

    let mut builder = phf_codegen::Set::new();
    for word in &words {
        builder.entry(*word);
    }

    writeln!(
        &mut writer,
        "static {set_name}: phf::Set<&'static str> = {};",
        builder.build()
    )
    .unwrap();
}
