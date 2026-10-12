use synthecode::{
    BinaryOp, Expr, GasMeter, Interpreter, TermRewriter, UnaryOp, Value,
};

#[test]
fn test_inversion_and_idempotence() {
    let trs = TermRewriter::default();

    // reverse(reverse(sort(sort(x0)))) => sort(x0)
    let inner_sort = Expr::Sort(Box::new(Expr::Var(0)));
    let double_sort = Expr::Sort(Box::new(inner_sort));
    let rev1 = Expr::Reverse(Box::new(double_sort));
    let complex = Expr::Reverse(Box::new(rev1));
    let (norm, count) = trs.normalize(&complex);
    assert_eq!(norm, Expr::Sort(Box::new(Expr::Var(0))));
    assert_eq!(count, 2); // 1 for reverse(reverse), 1 for sort(sort)

    // not(not(x0 == x0)) => true
    let bool_complex = Expr::Unary(
        UnaryOp::Not,
        Box::new(Expr::Unary(
            UnaryOp::Not,
            Box::new(Expr::Binary(
                BinaryOp::Eq,
                Box::new(Expr::Var(0)),
                Box::new(Expr::Var(0)),
            )),
        )),
    );
    let (norm_bool, _) = trs.normalize(&bool_complex);
    assert_eq!(norm_bool, Expr::Literal(Value::Bool(true)));
}

#[test]
fn test_semantic_preservation_between_original_and_canonical() {
    let trs = TermRewriter::default();
    let sample_input = Value::List(vec![
        Value::Int(42),
        Value::Int(7),
        Value::Int(19),
        Value::Int(3),
    ]);

    let rev = Expr::Reverse(Box::new(Expr::Var(0)));
    let sorted = Expr::Sort(Box::new(rev));
    let original = Expr::Min(Box::new(sorted));

    let (canonical, _) = trs.normalize(&original);
    // Should be simplified to min(x0)
    assert_eq!(canonical, Expr::Min(Box::new(Expr::Var(0))));

    // Run both on VM to verify identical outputs
    let mut vm1 = Interpreter::new(GasMeter::new(10_000));
    let mut vm2 = Interpreter::new(GasMeter::new(10_000));

    let out_orig = vm1.eval(&original, &[sample_input.clone()]).unwrap();
    let out_canon = vm2.eval(&canonical, &[sample_input]).unwrap();

    assert_eq!(out_orig, out_canon);
    assert_eq!(out_orig, Value::Int(3));

    // Canonical form should use less gas!
    assert!(vm2.gas.consumed() < vm1.gas.consumed());
}
