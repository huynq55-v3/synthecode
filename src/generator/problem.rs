use crate::analysis::{RuntimeComplexity, StructuralComplexity};
use crate::core::Type;
use crate::dsl::Expr;
use crate::novelty::BehavioralSignature;
use crate::testgen::{TestEvaluationResult, TestSuite};
use crate::vm::{GasMeter, Interpreter};

/// A fully synthesized, formally verified, deduplicated algorithmic problem.
#[derive(Debug, Clone)]
pub struct Problem {
    /// Unique identifier in the generation run.
    pub id: usize,
    /// Human/system identifier name.
    pub name: String,
    /// Formal parameter types expected as inputs.
    pub input_signature: Vec<Type>,
    /// Expected return type.
    pub output_type: Type,
    /// Reference oracle AST before canonicalization.
    pub raw_ast: Expr,
    /// Minimal Canonical Normal Form AST produced by TRS.
    pub canonical_ast: Expr,
    /// Comprehensive test suite (edge cases + fuzz vectors).
    pub test_suite: TestSuite,
    /// Behavioral signature vector evaluated against Standard Probe Set.
    pub behavioral_signature: BehavioralSignature,
    /// Structural Kolmogorov complexity proxy metrics.
    pub structural_complexity: StructuralComplexity,
    /// Runtime gas resource metrics.
    pub runtime_complexity: RuntimeComplexity,
    /// Shannon output entropy across the test suite.
    pub output_entropy: f64,
}

impl Problem {
    /// Evaluates a candidate program AST against this problem's test suite,
    /// providing continuous graded fitness without binary plateaus.
    pub fn evaluate_candidate(&self, candidate: &Expr, gas_budget: u64) -> TestEvaluationResult {
        let mut interp = Interpreter::new(GasMeter::new(gas_budget));
        self.test_suite.evaluate_candidate(candidate, &mut interp)
    }
}
