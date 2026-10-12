use std::collections::HashMap;
use crate::core::Value;

/// Shannon Information Entropy analyzer for detecting degenerate / vacuous problem collapse.
#[derive(Debug, Clone)]
pub struct EntropyAnalyzer {
    /// Minimum Shannon entropy required to admit a synthesized problem.
    pub min_entropy_threshold: f64,
}

impl Default for EntropyAnalyzer {
    fn default() -> Self {
        Self {
            // Require at least 0.8 bits of Shannon output entropy
            min_entropy_threshold: 0.8,
        }
    }
}

impl EntropyAnalyzer {
    /// Create a new analyzer with a custom entropy threshold.
    pub const fn new(min_entropy_threshold: f64) -> Self {
        Self {
            min_entropy_threshold,
        }
    }

    /// Computes the Shannon Information Entropy H(Y) across a collection of output values:
    /// H(Y) = - \sum_{y} p(y) * log2(p(y))
    pub fn compute_shannon_entropy(outputs: &[&Value]) -> f64 {
        let n = outputs.len();
        if n <= 1 {
            return 0.0;
        }

        let mut counts: HashMap<&Value, usize> = HashMap::new();
        for &val in outputs {
            *counts.entry(val).or_insert(0) += 1;
        }

        // If all outputs are identical (constant function), entropy is strictly 0.0
        if counts.len() == 1 {
            return 0.0;
        }

        let total = n as f64;
        let mut entropy = 0.0;

        for &count in counts.values() {
            let p = (count as f64) / total;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Determines if an output distribution suffers from degenerate problem collapse
    /// (e.g., constant functions returning 0, false, or empty vector).
    pub fn is_degenerate(&self, outputs: &[&Value]) -> bool {
        if outputs.is_empty() {
            return true;
        }

        let entropy = Self::compute_shannon_entropy(outputs);
        if entropy < self.min_entropy_threshold {
            return true;
        }

        // Additional safeguard: check if a single value dominates > 85% of cases
        let mut counts: HashMap<&Value, usize> = HashMap::new();
        for &val in outputs {
            *counts.entry(val).or_insert(0) += 1;
        }

        let max_count = counts.values().copied().max().unwrap_or(0);
        let dominance_ratio = (max_count as f64) / (outputs.len() as f64);
        if dominance_ratio > 0.85 {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_collapse_entropy_is_zero() {
        let zero = Value::Int(0);
        let outputs = vec![&zero, &zero, &zero, &zero, &zero];
        let entropy = EntropyAnalyzer::compute_shannon_entropy(&outputs);
        assert_eq!(entropy, 0.0);

        let analyzer = EntropyAnalyzer::default();
        assert!(analyzer.is_degenerate(&outputs));
    }

    #[test]
    fn test_diverse_outputs_have_high_entropy() {
        let v1 = Value::Int(1);
        let v2 = Value::Int(2);
        let v3 = Value::Int(3);
        let v4 = Value::Int(4);
        let outputs = vec![&v1, &v2, &v3, &v4];
        let entropy = EntropyAnalyzer::compute_shannon_entropy(&outputs);
        // For 4 equiprobable outcomes, H(Y) = log2(4) = 2.0 bits
        assert!((entropy - 2.0).abs() < 1e-6);

        let analyzer = EntropyAnalyzer::default();
        assert!(!analyzer.is_degenerate(&outputs));
    }
}
