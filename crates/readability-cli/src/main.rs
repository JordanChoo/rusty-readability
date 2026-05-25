use std::io::Read;

use clap::{Parser, Subcommand, ValueEnum};
use readability_core::analyze::analyze;
use readability_types::{AnalyzeOptions, InputFormat, ResponseFormat};

#[derive(Parser)]
#[command(name = "readability", about = "Readability analysis CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze text from a file or stdin
    Analyze {
        /// Input file (use - for stdin)
        #[arg(default_value = "-")]
        input: String,

        /// Input format
        #[arg(long, value_enum, default_value = "auto")]
        format: CliInputFormat,

        /// Output format
        #[arg(long, value_enum, default_value = "full")]
        output: CliOutputFormat,

        /// Decimal places for rounding
        #[arg(long, default_value = "2")]
        round: u8,

        /// Include paragraph breakdown
        #[arg(long)]
        paragraphs: bool,

        /// Number of hardest sentences to show
        #[arg(long, default_value = "0")]
        hardest: u8,
    },
    /// Show version and engine info
    Version,
}

#[derive(Clone, ValueEnum)]
enum CliInputFormat {
    Auto,
    Plain,
    Html,
    Markdown,
}

#[derive(Clone, ValueEnum)]
enum CliOutputFormat {
    Full,
    Compact,
    ScoresOnly,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            input,
            format,
            output,
            round,
            paragraphs,
            hardest,
        } => {
            let text = read_input(&input);

            let options = AnalyzeOptions {
                input_format: match format {
                    CliInputFormat::Auto => InputFormat::Auto,
                    CliInputFormat::Plain => InputFormat::Plain,
                    CliInputFormat::Html => InputFormat::Html,
                    CliInputFormat::Markdown => InputFormat::MarkdownLite,
                },
                response_format: match output {
                    CliOutputFormat::Full => ResponseFormat::Full,
                    CliOutputFormat::Compact => ResponseFormat::Compact,
                    CliOutputFormat::ScoresOnly => ResponseFormat::ScoresOnly,
                },
                round,
                include_paragraphs: paragraphs,
                include_hardest_sentences: hardest,
                ..Default::default()
            };

            match analyze(&text, &options) {
                Ok(result) => {
                    let json = serde_json::to_string_pretty(&result).unwrap();
                    println!("{json}");
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Commands::Version => {
            println!("readability-cli {}", env!("CARGO_PKG_VERSION"));
            println!("engine: rusty-readability");
            println!("formulas: flesch, flesch-kincaid, gunning-fog, coleman-liau, smog, ari, dale-chall, spache");
        }
    }
}

fn read_input(path: &str) -> String {
    if path == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).expect("Failed to read stdin");
        buf
    } else {
        std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("Error reading {path}: {e}");
            std::process::exit(1);
        })
    }
}
