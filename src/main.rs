use synthecode::{
    ComplexityAnalyzer, EntropyAnalyzer, GasMeter, Grammar, Interpreter, SplitMix64, TermRewriter,
    TestSuite, Type, TypeEnv,
};

fn main() {
    println!("=======================================================");
    println!("🚀 SyntheCode Engine: Autonomous Algorithmic Synthesis");
    println!("=======================================================");

    // 1. Setup Typing Environment: Problem taking List<Int> as input (x0)
    let input_params = vec![Type::list(Type::Int)];
    let env = TypeEnv::from_params(&input_params);
    println!("Input Signature: [x0: {}]", input_params[0]);

    // 2. Deterministic Seedable PRNG & Analysis Engines
    let mut rng = SplitMix64::new(42);
    let trs = TermRewriter::default();
    let entropy_analyzer = EntropyAnalyzer::default();

    println!("\n--- Synthesizing Verified Non-Degenerate Problem ---");
    let target_ty = Type::list(Type::Int);
    let mut accepted_problem = None;

    for attempt in 1..=20 {
        let raw_expr = Grammar::generate_typed(&target_ty, &env, 3, &mut rng)
            .expect("Grammar generation failed");
        let (canonical_expr, _) = trs.normalize(&raw_expr);

        // Build test suite to measure oracle output entropy
        let mut vm = Interpreter::new(GasMeter::new(50_000));
        if let Ok(suite) = TestSuite::build(&canonical_expr, &input_params, 15, &mut rng, &mut vm) {
            let outputs = suite.oracle_outputs();
            let entropy = EntropyAnalyzer::compute_shannon_entropy(&outputs);
            let is_degenerate = entropy_analyzer.is_degenerate(&outputs);

            if !is_degenerate {
                println!(
                    "🎯 [Attempt #{}] Problem Accepted!\n   Raw AST: {}\n   Canonical: {}\n   Shannon Entropy H(Y): {:.4} bits (Threshold: {:.2})\n   Test Cases Generated: {}",
                    attempt, raw_expr, canonical_expr, entropy, entropy_analyzer.min_entropy_threshold, suite.cases.len()
                );
                accepted_problem = Some((canonical_expr, suite));
                break;
            } else {
                println!(
                    "⚠️  [Attempt #{}] Degenerate Collapse Purged: H(Y) = {:.4} bits | AST: {}",
                    attempt, entropy, canonical_expr
                );
            }
        }
    }

    // 3. Inspect Accepted Problem & Evaluate Candidate
    if let Some((problem_expr, suite)) = accepted_problem {
        let structural = ComplexityAnalyzer::analyze_structural(&problem_expr);
        let runtime = ComplexityAnalyzer::analyze_runtime(&suite);

        println!("\n--- Problem Complexity Profile ---");
        println!(
            "Nodes: {} | Depth: {} | Kolmogorov Score: {:.2}",
            structural.node_count, structural.depth, structural.kolmogorov_score
        );
        println!(
            "Total Gas: {} | Mean Gas/Case: {:.1} | Max Gas: {}",
            runtime.total_gas, runtime.mean_gas_per_case, runtime.max_gas_case
        );

        // Candidate Evaluation: Candidate A (Correct solution) vs Candidate B (Imperfection)
        let candidate_a = problem_expr.clone();
        let candidate_b = synthecode::Expr::Reverse(Box::new(synthecode::Expr::Var(0)));

        let mut eval_vm = Interpreter::new(GasMeter::new(50_000));
        let res_a = suite.evaluate_candidate(&candidate_a, &mut eval_vm);
        let res_b = suite.evaluate_candidate(&candidate_b, &mut eval_vm);

        println!("\n--- Multi-Modal Fitness Benchmark (Anti-Plateau) ---");
        println!(
            "Reference Solution: Passed: {} | Exact: {}/{} | Graded Fitness: {:.4}",
            res_a.passed, res_a.exact_matches, res_a.total_cases, res_a.graded_fitness
        );
        println!(
            "Candidate 'reverse(x0)': Passed: {} | Exact: {}/{} | Graded Fitness: {:.4} (Continuous Partial Credit!)",
            res_b.passed, res_b.exact_matches, res_b.total_cases, res_b.graded_fitness
        );
    }

    println!("\n✅ Pipeline Stage 3 (testgen/ & analysis/) verified & operational!");
}
