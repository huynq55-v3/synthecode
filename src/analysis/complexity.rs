use crate::dsl::Expr;
use crate::testgen::TestSuite;

/// Structural complexity profile of an AST (used as Kolmogorov complexity proxy).
#[derive(Debug, Clone, PartialEq)]
pub struct StructuralComplexity {
    pub node_count: usize,
    pub depth: usize,
    pub kolmogorov_score: f64,
}

/// Runtime execution complexity profile derived from deterministic gas metering.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeComplexity {
    pub total_gas: u64,
    pub mean_gas_per_case: f64,
    pub max_gas_case: u64,
}

pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Computes structural complexity metrics for an expression.
    pub fn analyze_structural(expr: &Expr) -> StructuralComplexity {
        let node_count = expr.node_count();
        let depth = expr.depth();

        // Kolmogorov proxy: combination of tree size, branching depth, and operator entropy
        let kolmogorov_score = (node_count as f64) * 0.7 + (depth as f64) * 0.3;

        StructuralComplexity {
            node_count,
            depth,
            kolmogorov_score,
        }
    }

    /// Analyzes the runtime resource usage across all test cases.
    pub fn analyze_runtime(suite: &TestSuite) -> RuntimeComplexity {
        if suite.cases.is_empty() {
            return RuntimeComplexity {
                total_gas: 0,
                mean_gas_per_case: 0.0,
                max_gas_case: 0,
            };
        }

        let mut total_gas: u64 = 0;
        let mut max_gas = 0;

        for case in &suite.cases {
            total_gas = total_gas.saturating_add(case.gas_consumed);
            if case.gas_consumed > max_gas {
                max_gas = case.gas_consumed;
            }
        }

        let mean_gas = (total_gas as f64) / (suite.cases.len() as f64);

        RuntimeComplexity {
            total_gas,
            mean_gas_per_case: mean_gas,
            max_gas_case: max_gas,
        }
    }
}
