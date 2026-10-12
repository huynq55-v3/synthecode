use synthecode::{PipelineConfig, ProblemGenerator, Type};

#[test]
fn test_end_to_end_batch_generation_and_diversity() {
    let mut config = PipelineConfig::default();
    config.max_ast_depth = 3;
    config.novelty_threshold = 0.05;

    let mut engine = ProblemGenerator::new(2026, config);

    let input_types = vec![Type::list(Type::Int)];
    let output_type = Type::list(Type::Int);

    // Generate batch of 5 distinct algorithmic problems
    let problems = engine.generate_batch(&input_types, &output_type, 5);

    assert!(!problems.is_empty(), "Engine must generate at least 1 verified problem");
    println!("Generated {} novel problems in batch", problems.len());

    for problem in &problems {
        // 1. Verify entropy constraint
        assert!(
            problem.output_entropy >= 0.8,
            "Problem entropy must exceed min threshold: got {}",
            problem.output_entropy
        );

        // 2. Verify test suite non-empty
        assert!(!problem.test_suite.cases.is_empty());

        // 3. Verify reference solution passes 100% with fitness = 1.0
        let eval_res = problem.evaluate_candidate(&problem.canonical_ast, 50_000);
        assert!(eval_res.passed);
        assert_eq!(eval_res.exact_matches, eval_res.total_cases);
        assert!((eval_res.graded_fitness - 1.0).abs() < 1e-6);
    }

    // 4. Verify novelty archive stores distinct problems
    assert_eq!(engine.archive.len(), problems.len());
}

#[test]
fn test_deterministic_reproducibility() {
    let config = PipelineConfig::default();

    let mut engine1 = ProblemGenerator::new(999, config.clone());
    let mut engine2 = ProblemGenerator::new(999, config);

    let input_types = vec![Type::list(Type::Int)];
    let output_type = Type::Int;

    let p1 = engine1.generate_problem(&input_types, &output_type).unwrap();
    let p2 = engine2.generate_problem(&input_types, &output_type).unwrap();

    // Must be bit-for-bit identical from seed
    assert_eq!(p1.raw_ast, p2.raw_ast);
    assert_eq!(p1.canonical_ast, p2.canonical_ast);
    assert_eq!(p1.test_suite.cases.len(), p2.test_suite.cases.len());
    assert_eq!(p1.output_entropy, p2.output_entropy);
}
