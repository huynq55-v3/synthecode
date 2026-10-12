use synthecode::{BinaryOp, Expr, Grammar, SplitMix64, Type, TypeChecker, TypeEnv, Value};

#[test]
fn test_end_to_end_type_soundness() {
    let mut rng = SplitMix64::new(42);
    let env = TypeEnv::from_params(&[Type::list(Type::Int), Type::Int]);

    // Test Int generation
    for _ in 0..50 {
        let expr = Grammar::generate_typed(&Type::Int, &env, 4, &mut rng).unwrap();
        assert_eq!(TypeChecker::infer(&expr, &env).unwrap(), Type::Int);
    }

    // Test Bool generation
    for _ in 0..50 {
        let expr = Grammar::generate_typed(&Type::Bool, &env, 4, &mut rng).unwrap();
        assert_eq!(TypeChecker::infer(&expr, &env).unwrap(), Type::Bool);
    }

    // Test List<Int> generation
    for _ in 0..50 {
        let expr = Grammar::generate_typed(&Type::list(Type::Int), &env, 4, &mut rng).unwrap();
        assert_eq!(
            TypeChecker::infer(&expr, &env).unwrap(),
            Type::list(Type::Int)
        );
    }
}

#[test]
fn test_ill_typed_expressions_fail_gracefully() {
    let env = TypeEnv::from_params(&[Type::list(Type::Int)]);

    // Adding List and Int is illegal
    let ill_typed = Expr::Binary(
        BinaryOp::Add,
        Box::new(Expr::Var(0)), // List<Int>
        Box::new(Expr::Literal(Value::Int(10))),
    );
    assert!(TypeChecker::infer(&ill_typed, &env).is_err());
}
