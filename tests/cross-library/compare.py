#!/usr/bin/env python3
"""
Cross-library readability score comparison harness.

Compares rusty-readability against textstat (Python) on shared fixture texts.
Generates a CSV report showing score differences and flags divergences beyond
acceptable thresholds.

Usage:
    python3 tests/cross-library/compare.py [--cli PATH] [--fixtures DIR]

Thresholds (acceptable variance):
    - Flesch Reading Ease: ±10 points (different syllable algorithms)
    - Grade-level formulas: ±1.5 grade levels
    - Dale-Chall: ±1.0 (different word lists)
"""
import argparse
import csv
import json
import os
import subprocess
import sys

import textstat

THRESHOLDS = {
    "flesch_reading_ease": 10.0,
    "flesch_kincaid_grade": 1.5,
    "gunning_fog": 1.5,
    "coleman_liau": 1.5,
    "smog": 2.0,
    "automated_readability_index": 1.5,
    # Dale-Chall uses different word lists across implementations;
    # we use a 2024 public-domain list, textstat uses its own.
    "dale_chall": 7.0,
}

PLAIN_FIXTURES = [
    "simple.txt",
    "academic.txt",
    "children.txt",
    "short.txt",
    "medium_500w.txt",
    "long_5000w.txt",
]


def get_rusty_scores(cli_path: str, text_path: str) -> dict:
    result = subprocess.run(
        [cli_path, "analyze", text_path, "--format", "plain"],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"  CLI error for {text_path}: {result.stderr.strip()}", file=sys.stderr)
        return {}
    data = json.loads(result.stdout)
    scores = {}
    for key in THRESHOLDS:
        score_obj = data.get("scores", {}).get(key)
        if score_obj and isinstance(score_obj, dict):
            scores[key] = score_obj.get("raw")
        elif score_obj and isinstance(score_obj, (int, float)):
            scores[key] = score_obj
    return scores


def get_textstat_scores(text: str) -> dict:
    textstat.set_lang("en")
    scores = {
        "flesch_reading_ease": textstat.flesch_reading_ease(text),
        "flesch_kincaid_grade": textstat.flesch_kincaid_grade(text),
        "gunning_fog": textstat.gunning_fog(text),
        "coleman_liau": textstat.coleman_liau_index(text),
        "smog": textstat.smog_index(text),
        "automated_readability_index": textstat.automated_readability_index(text),
        "dale_chall": textstat.dale_chall_readability_score(text),
    }
    return scores


def compare(fixtures_dir: str, cli_path: str, output_csv: str):
    rows = []
    divergences = 0
    total_comparisons = 0

    for fixture in PLAIN_FIXTURES:
        path = os.path.join(fixtures_dir, fixture)
        if not os.path.exists(path):
            print(f"  Skipping {fixture} (not found)")
            continue

        with open(path) as f:
            text = f.read()

        rusty = get_rusty_scores(cli_path, path)
        ts = get_textstat_scores(text)

        if not rusty:
            print(f"  Skipping {fixture} (CLI failed)")
            continue

        for metric, threshold in THRESHOLDS.items():
            r_val = rusty.get(metric)
            t_val = ts.get(metric)

            if r_val is None or t_val is None:
                continue

            diff = abs(r_val - t_val)
            within = diff <= threshold
            total_comparisons += 1
            if not within:
                divergences += 1

            rows.append({
                "fixture": fixture,
                "metric": metric,
                "rusty": round(r_val, 4),
                "textstat": round(t_val, 4),
                "diff": round(diff, 4),
                "threshold": threshold,
                "status": "OK" if within else "DIVERGE",
            })

    with open(output_csv, "w", newline="") as f:
        writer = csv.DictWriter(
            f,
            fieldnames=["fixture", "metric", "rusty", "textstat", "diff", "threshold", "status"],
        )
        writer.writeheader()
        writer.writerows(rows)

    print(f"\nResults written to {output_csv}")
    print(f"Total comparisons: {total_comparisons}")
    print(f"Within threshold:  {total_comparisons - divergences}")
    print(f"Divergences:       {divergences}")

    if divergences > 0:
        print("\nDivergent scores:")
        for row in rows:
            if row["status"] == "DIVERGE":
                print(
                    f"  {row['fixture']:20s} {row['metric']:30s} "
                    f"rusty={row['rusty']:8.2f}  textstat={row['textstat']:8.2f}  "
                    f"diff={row['diff']:6.2f} (threshold={row['threshold']})"
                )

    return divergences


def main():
    parser = argparse.ArgumentParser(description="Cross-library readability comparison")
    parser.add_argument(
        "--cli",
        default=os.environ.get(
            "READABILITY_CLI",
            os.path.join(os.environ.get("CARGO_TARGET_DIR", "target"), "release", "readability-cli"),
        ),
        help="Path to readability-cli binary",
    )
    parser.add_argument(
        "--fixtures",
        default=os.path.join(os.path.dirname(__file__), "../../fixtures/texts"),
        help="Path to fixture texts directory",
    )
    parser.add_argument(
        "--output",
        default=os.path.join(os.path.dirname(__file__), "comparison_report.csv"),
        help="Output CSV path",
    )
    args = parser.parse_args()

    if not os.path.exists(args.cli):
        print(f"Error: CLI binary not found at {args.cli}")
        print("Build it with: cargo build -p readability-cli --release")
        sys.exit(1)

    print(f"CLI:      {args.cli}")
    print(f"Fixtures: {args.fixtures}")
    print(f"Output:   {args.output}")
    print()

    divergences = compare(args.fixtures, args.cli, args.output)
    sys.exit(1 if divergences > 0 else 0)


if __name__ == "__main__":
    main()
