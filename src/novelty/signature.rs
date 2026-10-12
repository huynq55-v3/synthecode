use crate::core::{Type, Value};
use crate::dsl::Expr;
use crate::vm::Interpreter;
use super::probe::StandardProbeSet;

/// High-dimensional behavioral signature vector derived from evaluating a problem against the Standard Probe Set.
#[derive(Debug, Clone, PartialEq)]
pub struct BehavioralSignature {
    pub vector: Vec<f64>,
}

impl BehavioralSignature {
    /// Extracts a continuous behavioral feature vector for an oracle expression.
    pub fn compute(
        expr: &Expr,
        param_types: &[Type],
        interp: &mut Interpreter,
    ) -> Self {
        let probes = StandardProbeSet::get_probes(param_types);
        let mut vector = Vec::with_capacity(probes.len() * 3);

        for probe in probes {
            match interp.eval(expr, &probe) {
                Ok(val) => {
                    Self::encode_value(&val, &mut vector);
                }
                Err(_) => {
                    // Sentinel feature for runtime error / timeout
                    vector.push(-1.0);
                    vector.push(-1.0);
                    vector.push(-1.0);
                }
            }
        }

        Self { vector }
    }

    fn encode_value(val: &Value, vec: &mut Vec<f64>) {
        match val {
            Value::Int(i) => {
                let sign = if *i > 0 { 1.0 } else if *i < 0 { -1.0 } else { 0.0 };
                let magnitude = (*i as f64).abs();
                let scaled = magnitude / (1.0 + magnitude);
                vec.push(sign);
                vec.push(scaled);
                vec.push(0.0);
            }
            Value::Bool(b) => {
                vec.push(if *b { 1.0 } else { -1.0 });
                vec.push(0.0);
                vec.push(0.0);
            }
            Value::List(items) => {
                let len = items.len() as f64;
                let norm_len = len / (1.0 + len);
                vec.push(norm_len);

                if items.is_empty() {
                    vec.push(0.0);
                    vec.push(0.0);
                } else if let Ok(first_int) = items[0].as_int() {
                    let sum: f64 = items.iter().filter_map(|it| it.as_int().ok()).map(|x| x as f64).sum();
                    let mean = sum / len;
                    let norm_mean = mean / (1.0 + mean.abs());
                    vec.push(first_int as f64 / (1.0 + (first_int.abs() as f64)));
                    vec.push(norm_mean);
                } else {
                    vec.push(1.0);
                    vec.push(0.0);
                }
            }
            Value::Matrix { rows, cols, data: _ } => {
                vec.push(*rows as f64);
                vec.push(*cols as f64);
                vec.push(0.0);
            }
        }
    }

    /// Computes normalized Euclidean distance between two behavioral vectors.
    pub fn distance(&self, other: &Self) -> f64 {
        if self.vector.is_empty() || other.vector.is_empty() {
            return 1.0;
        }

        let min_len = self.vector.len().min(other.vector.len());
        let mut sum_sq = 0.0;

        for i in 0..min_len {
            let diff = self.vector[i] - other.vector[i];
            sum_sq += diff * diff;
        }

        let penalty = (self.vector.len().abs_diff(other.vector.len())) as f64;
        let dist = (sum_sq + penalty).sqrt() / (min_len.max(1) as f64).sqrt();
        dist.clamp(0.0, 1.0)
    }

    /// Computes Cosine distance in [0.0, 2.0].
    pub fn cosine_distance(&self, other: &Self) -> f64 {
        let min_len = self.vector.len().min(other.vector.len());
        if min_len == 0 {
            return 1.0;
        }

        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..min_len {
            let a = self.vector[i];
            let b = other.vector[i];
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }

        if norm_a <= 1e-9 || norm_b <= 1e-9 {
            return 1.0;
        }

        let cos_sim = dot / (norm_a.sqrt() * norm_b.sqrt());
        (1.0 - cos_sim).clamp(0.0, 2.0)
    }
}
