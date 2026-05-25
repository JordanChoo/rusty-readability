use readability_types::{Agreement, Stability};

pub fn compute_consensus(grade_estimates: &[f64]) -> Option<f64> {
    if grade_estimates.is_empty() {
        return None;
    }

    let mut sorted = grade_estimates.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();

    match n {
        1 => Some(sorted[0]),
        2 => Some((sorted[0] + sorted[1]) / 2.0),
        3 | 4 => {
            // Median
            if n % 2 == 0 {
                Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
            } else {
                Some(sorted[n / 2])
            }
        }
        _ => {
            // 5+: drop min and max, then take mean of the rest
            let trimmed = &sorted[1..n - 1];
            let sum: f64 = trimmed.iter().sum();
            Some(sum / trimmed.len() as f64)
        }
    }
}

pub fn compute_agreement(grade_estimates: &[f64]) -> Option<Agreement> {
    if grade_estimates.len() < 2 {
        return None;
    }

    let min = grade_estimates.iter().copied().fold(f64::INFINITY, f64::min);
    let max = grade_estimates.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let spread = max - min;

    let stability = if spread <= 2.0 {
        Stability::High
    } else if spread <= 5.0 {
        Stability::Medium
    } else {
        Stability::Low
    };

    Some(Agreement {
        min_grade: min,
        max_grade: max,
        spread,
        stability,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_estimate() {
        assert_eq!(compute_consensus(&[5.0]), Some(5.0));
    }

    #[test]
    fn two_estimates_average() {
        assert_eq!(compute_consensus(&[4.0, 6.0]), Some(5.0));
    }

    #[test]
    fn three_estimates_median() {
        assert_eq!(compute_consensus(&[3.0, 5.0, 7.0]), Some(5.0));
    }

    #[test]
    fn four_estimates_median() {
        assert_eq!(compute_consensus(&[2.0, 4.0, 6.0, 8.0]), Some(5.0));
    }

    #[test]
    fn five_estimates_trimmed_mean() {
        // Drop 1 and 9, mean of [3, 5, 7] = 5.0
        assert_eq!(compute_consensus(&[1.0, 3.0, 5.0, 7.0, 9.0]), Some(5.0));
    }

    #[test]
    fn empty() {
        assert_eq!(compute_consensus(&[]), None);
    }

    #[test]
    fn agreement_high() {
        let a = compute_agreement(&[5.0, 6.0, 5.5]).unwrap();
        assert_eq!(a.stability, Stability::High);
        assert!(a.spread <= 2.0);
    }

    #[test]
    fn agreement_low() {
        let a = compute_agreement(&[2.0, 12.0]).unwrap();
        assert_eq!(a.stability, Stability::Low);
    }
}
