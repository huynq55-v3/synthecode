use crate::core::{Type, Value};
use crate::dsl::{Expr, SplitMix64};
use crate::vm::{Interpreter, VmError};
use super::edge_cases::EdgeCaseGenerator;
use super::fuzzer::InputFuzzer;

/// An individual deterministic test case with oracle expected output and gas profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestCase {
    pub inputs: Vec<Value>,
    pub expected_output: Value,
    pub gas_consumed: u64,
}

/// Comprehensive diagnostic test suite for fitness benchmarking and formal verification.
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub cases: Vec<TestCase>,
}

/// Graded multi-modal evaluation result providing continuous partial-credit gradients (Anti-Plateau).
#[derive(Debug, Clone, PartialEq)]
pub struct TestEvaluationResult {
    /// True if every single test case matched the reference oracle exactly.
    pub passed: bool,
    /// Number of exactly matching test cases.
    pub exact_matches: usize,
    /// Total test cases evaluated.
    pub total_cases: usize,
    /// Continuous partial-credit fitness in [0.0, 1.0]. 1.0 = perfect score.
    pub graded_fitness: f64,
    /// Per-case diagnostic distances.
    pub case_distances: Vec<f64>,
}

impl TestSuite {
    /// Builds a comprehensive test suite combining edge cases and fuzzed inputs,
    /// executing the reference oracle expression on the deterministic VM.
    pub fn build(
        oracle_expr: &Expr,
        param_types: &[Type],
        fuzz_count: usize,
        rng: &mut SplitMix64,
        interp: &mut Interpreter,
    ) -> Result<Self, VmError> {
        let edge_inputs = EdgeCaseGenerator::generate(param_types);
        let fuzzed_inputs = InputFuzzer::generate(param_types, fuzz_count, rng);

        let mut all_inputs = edge_inputs;
        all_inputs.extend(fuzzed_inputs);

        let mut cases = Vec::with_capacity(all_inputs.len());

        for input in all_inputs {
            let start_gas = interp.gas.consumed();
            match interp.eval(oracle_expr, &input) {
                Ok(expected) => {
                    let gas_cost = interp.gas.consumed().saturating_sub(start_gas);
                    cases.push(TestCase {
                        inputs: input,
                        expected_output: expected,
                        gas_consumed: gas_cost,
                    });
                }
                Err(_) => {
                    // Discard pathological edge cases that crash the oracle
                    continue;
                }
            }
        }

        if cases.is_empty() {
            return Err(VmError::EvaluationError(
                "Failed to generate any valid test cases for oracle".into(),
            ));
        }

        Ok(Self { cases })
    }

    /// Evaluates a candidate program AST against this test suite,
    /// computing continuous diagnostic fitness (Challenge D - Anti-Plateau).
    pub fn evaluate_candidate(&self, candidate: &Expr, interp: &mut Interpreter) -> TestEvaluationResult {
        let total = self.cases.len();
        if total == 0 {
            return TestEvaluationResult {
                passed: true,
                exact_matches: 0,
                total_cases: 0,
                graded_fitness: 1.0,
                case_distances: vec![],
            };
        }

        let mut exact_matches = 0;
        let mut total_distance = 0.0;
        let mut case_distances = Vec::with_capacity(total);

        for case in &self.cases {
            match interp.eval(candidate, &case.inputs) {
                Ok(actual) => {
                    let dist = actual.diagnostic_distance(&case.expected_output);
                    case_distances.push(dist);
                    total_distance += dist;
                    if dist == 0.0 && actual == case.expected_output {
                        exact_matches += 1;
                    }
                }
                Err(_) => {
                    // Maximum penalty for crash/timeout
                    case_distances.push(1.0);
                    total_distance += 1.0;
                }
            }
        }

        let avg_distance = total_distance / (total as f64);
        let graded_fitness = (1.0 - avg_distance).clamp(0.0, 1.0);
        let passed = exact_matches == total;

        TestEvaluationResult {
            passed,
            exact_matches,
            total_cases: total,
            graded_fitness,
            case_distances,
        }
    }

    /// Returns outputs across all test cases (used for Shannon Entropy analysis).
    pub fn oracle_outputs(&self) -> Vec<&Value> {
        self.cases.iter().map(|c| &c.expected_output).collect()
    }
}
