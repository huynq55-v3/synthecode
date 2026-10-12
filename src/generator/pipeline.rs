use crate::analysis::{ComplexityAnalyzer, EntropyAnalyzer};
use crate::canonical::TermRewriter;
use crate::core::Type;
use crate::dsl::{Grammar, SplitMix64, TypeEnv};
use crate::novelty::{BehavioralSignature, NoveltyArchive};
use crate::testgen::TestSuite;
use crate::vm::{GasMeter, Interpreter};
use super::problem::Problem;

/// Configuration parameters governing autonomous problem synthesis.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_ast_depth: usize,
    pub fuzz_count: usize,
    pub gas_limit: u64,
    pub min_entropy_threshold: f64,
    pub novelty_threshold: f64,
    pub max_attempts_per_problem: usize,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_ast_depth: 3,
            fuzz_count: 15,
            gas_limit: 50_000,
            min_entropy_threshold: 0.8,
            novelty_threshold: 0.08,
            max_attempts_per_problem: 200,
        }
    }
}

/// End-to-end Problem Generation Orchestrator combining all formal verification,
/// algebraic reduction, entropy filtering, and novelty diversity stages.
pub struct ProblemGenerator {
    config: PipelineConfig,
    rng: SplitMix64,
    trs: TermRewriter,
    entropy_analyzer: EntropyAnalyzer,
    pub archive: NoveltyArchive,
    next_id: usize,
}

impl ProblemGenerator {
    /// Initializes a new problem generator from a 64-bit seed.
    pub fn new(seed: u64, config: PipelineConfig) -> Self {
        let entropy_analyzer = EntropyAnalyzer::new(config.min_entropy_threshold);
        let archive = NoveltyArchive::new(config.novelty_threshold);

        Self {
            config,
            rng: SplitMix64::new(seed),
            trs: TermRewriter::default(),
            entropy_analyzer,
            archive,
            next_id: 1,
        }
    }

    /// Autonomously generates a single verified, non-degenerate, and novel problem.
    pub fn generate_problem(
        &mut self,
        input_types: &[Type],
        output_type: &Type,
    ) -> Option<Problem> {
        let env = TypeEnv::from_params(input_types);

        for _ in 0..self.config.max_attempts_per_problem {
            // Stage 1: Type-directed grammar expansion
            let raw_ast = match Grammar::generate_typed(
                output_type,
                &env,
                self.config.max_ast_depth,
                &mut self.rng,
            ) {
                Ok(ast) => ast,
                Err(_) => continue,
            };

            // Stage 2: Canonical reduction via Term Rewriting System (TRS)
            let (canonical_ast, _) = self.trs.normalize(&raw_ast);

            // Stage 3: Deterministic VM Oracle Execution & Test Suite Construction
            let mut oracle_vm = Interpreter::new(GasMeter::new(self.config.gas_limit));
            let suite = match TestSuite::build(
                &canonical_ast,
                input_types,
                self.config.fuzz_count,
                &mut self.rng,
                &mut oracle_vm,
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Stage 4: Information Entropy Analyzer (Degenerate Problem Collapse Filter)
            let outputs = suite.oracle_outputs();
            let entropy = EntropyAnalyzer::compute_shannon_entropy(&outputs);
            if self.entropy_analyzer.is_degenerate(&outputs) {
                continue;
            }

            // Stage 5: Behavioral Signature Extraction across Standard Probe Set
            let mut probe_vm = Interpreter::new(GasMeter::new(self.config.gas_limit));
            let sig = BehavioralSignature::compute(&canonical_ast, input_types, &mut probe_vm);

            // Stage 6: Novelty Archive Admission (Diversity Protection)
            let id = self.next_id;
            if !self.archive.insert(id, sig.clone()) {
                // Rejected due to behavioral similarity with existing problem
                continue;
            }

            self.next_id += 1;

            // Stage 7: Complexity Profiling
            let structural_complexity = ComplexityAnalyzer::analyze_structural(&canonical_ast);
            let runtime_complexity = ComplexityAnalyzer::analyze_runtime(&suite);

            let problem = Problem {
                id,
                name: format!("Problem_{}_{}", output_type, id),
                input_signature: input_types.to_vec(),
                output_type: output_type.clone(),
                raw_ast,
                canonical_ast,
                test_suite: suite,
                behavioral_signature: sig,
                structural_complexity,
                runtime_complexity,
                output_entropy: entropy,
            };

            return Some(problem);
        }

        None
    }

    /// Generates a batch of distinct, verified algorithmic problems.
    pub fn generate_batch(
        &mut self,
        input_types: &[Type],
        output_type: &Type,
        count: usize,
    ) -> Vec<Problem> {
        let mut batch = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(p) = self.generate_problem(input_types, output_type) {
                batch.push(p);
            }
        }
        batch
    }
}
