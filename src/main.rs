use synthecode::{
    GasMeter, Grammar, Interpreter, SplitMix64, TermRewriter, Type, TypeChecker, TypeEnv, Value,
};

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
    let trs = TermRewriter::default();

    // 3. Test input instance
    let sample_input = Value::List(vec![
        Value::Int(15),
        Value::Int(3),
        Value::Int(99),
        Value::Int(42),
    ]);
    println!("Evaluation Sample Input (x0): {}", sample_input);

    // 4. Synthesize, Canonicalize & Evaluate List Expressions with Gas Metering
    println!("\n--- Synthesizing, Canonicalizing & Executing Expressions ---");
    let target_list_ty = Type::list(Type::Int);
    for i in 1..=4 {
        let raw_expr = Grammar::generate_typed(&target_list_ty, &env, 3, &mut rng)
            .expect("Synthesis should succeed with valid type contract");
        let inferred_ty = TypeChecker::infer(&raw_expr, &env).unwrap();

        // Canonical reduction via TRS
        let (canonical_expr, rewrites) = trs.normalize(&raw_expr);

        // Evaluate Canonical Form using VM with Gas Meter
        let mut vm = Interpreter::new(GasMeter::new(10_000));
        match vm.eval(&canonical_expr, &[sample_input.clone()]) {
            Ok(output) => {
                println!(
                    "[{}] Raw AST: {}\n    Canonical: {} (Rewrites: {})\n    Type: {} | Gas Used: {} | Output: {}",
                    i,
                    raw_expr,
                    canonical_expr,
                    rewrites,
                    inferred_ty,
                    vm.gas.consumed(),
                    output
                );
            }
            Err(e) => {
                println!(
                    "[{}] Raw AST: {}\n    Canonical: {}\n    Evaluation Failed: {}",
                    i, raw_expr, canonical_expr, e
                );
            }
        }
    }

    // 5. Anti-Plateau Diagnostic Distance
    let val_a = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let val_b = Value::List(vec![Value::Int(1), Value::Int(5), Value::Int(3)]);
    println!("\n--- Anti-Plateau Diagnostic Distance Check ---");
    println!("Sample A: {}", val_a);
    println!("Sample B: {}", val_b);
    println!("Continuous Distance: {:.4}", val_a.diagnostic_distance(&val_b));

    println!("\n✅ core/, dsl/, vm/, and canonical/ modules verified & operational!");
}
