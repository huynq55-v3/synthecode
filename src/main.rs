use synthecode::{Grammar, SplitMix64, Type, TypeChecker, TypeEnv, Value};

fn main() {
    println!("=======================================================");
    println!("🚀 SyntheCode Engine: Autonomous Algorithmic Synthesis");
    println!("=======================================================");

    // 1. Setup Typing Environment: Problem taking List<Int> as input (x0)
    let input_params = vec![Type::list(Type::Int)];
    let env = TypeEnv::from_params(&input_params);
    println!("Input Parameters: [x0: {}]", input_params[0]);

    // 2. Deterministic Seedable PRNG
    let mut rng = SplitMix64::new(2026);

    // 3. Synthesize Integer-returning Algorithmic Expression
    println!("\n--- Synthesizing Int Reduction Expressions ---");
    for i in 1..=3 {
        let expr = Grammar::generate_typed(&Type::Int, &env, 3, &mut rng)
            .expect("Synthesis should succeed with valid type contract");
        let inferred_ty = TypeChecker::infer(&expr, &env).unwrap();
        println!(
            "[{}] AST: {}\n    Type: {} | Nodes: {} | Depth: {}",
            i,
            expr,
            inferred_ty,
            expr.node_count(),
            expr.depth()
        );
    }

    // 4. Synthesize List-transforming Algorithmic Expression
    println!("\n--- Synthesizing List Transformation Expressions ---");
    let target_list_ty = Type::list(Type::Int);
    for i in 1..=3 {
        let expr = Grammar::generate_typed(&target_list_ty, &env, 3, &mut rng)
            .expect("Synthesis should succeed with valid type contract");
        let inferred_ty = TypeChecker::infer(&expr, &env).unwrap();
        println!(
            "[{}] AST: {}\n    Type: {} | Nodes: {} | Depth: {}",
            i,
            expr,
            inferred_ty,
            expr.node_count(),
            expr.depth()
        );
    }

    // 5. Anti-Plateau Diagnostic Distance
    let val_a = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let val_b = Value::List(vec![Value::Int(1), Value::Int(5), Value::Int(3)]);
    println!("\n--- Anti-Plateau Diagnostic Distance Check ---");
    println!("Sample A: {}", val_a);
    println!("Sample B: {}", val_b);
    println!("Continuous Distance: {:.4}", val_a.diagnostic_distance(&val_b));

    println!("\n✅ core/ and dsl/ modules verified & operational!");
}
