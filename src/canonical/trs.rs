use crate::dsl::Expr;
use super::rules::apply_rewrite_rule;

/// Term Rewriting System (TRS) for AST canonical reduction.
/// Reduces ASTs to their minimal Canonical Normal Form via innermost fixed-point rewriting.
#[derive(Debug, Clone)]
pub struct TermRewriter {
    pub max_iterations: usize,
}

impl Default for TermRewriter {
    fn default() -> Self {
        Self { max_iterations: 100 }
    }
}

impl TermRewriter {
    /// Create a new TermRewriter with custom maximum reduction iterations.
    pub const fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }

    /// Reduces an expression to its Canonical Normal Form.
    /// Returns the simplified AST and the total number of rewrite rules applied.
    pub fn normalize(&self, expr: &Expr) -> (Expr, usize) {
        let mut current = expr.clone();
        let mut total_rewrites = 0;

        for _ in 0..self.max_iterations {
            let (next, rewrites) = self.rewrite_step(&current);
            if rewrites == 0 {
                break;
            }
            total_rewrites += rewrites;
            current = next;
        }

        (current, total_rewrites)
    }

    /// Performs one bottom-up (innermost) traversal rewrite pass across the AST.
    fn rewrite_step(&self, expr: &Expr) -> (Expr, usize) {
        let mut rewrites = 0;

        // 1. First recursively reduce all sub-expressions
        let simplified_children = match expr {
            Expr::Literal(_) | Expr::Var(_) => expr.clone(),

            Expr::Unary(op, inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Unary(*op, Box::new(new_inner))
            }

            Expr::Binary(op, left, right) => {
                let (new_left, r1) = self.rewrite_step(left);
                let (new_right, r2) = self.rewrite_step(right);
                rewrites += r1 + r2;
                Expr::Binary(*op, Box::new(new_left), Box::new(new_right))
            }

            Expr::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => {
                let (new_c, r1) = self.rewrite_step(cond);
                let (new_t, r2) = self.rewrite_step(then_branch);
                let (new_e, r3) = self.rewrite_step(else_branch);
                rewrites += r1 + r2 + r3;
                Expr::IfThenElse {
                    cond: Box::new(new_c),
                    then_branch: Box::new(new_t),
                    else_branch: Box::new(new_e),
                }
            }

            Expr::Length(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Length(Box::new(new_inner))
            }

            Expr::Head(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Head(Box::new(new_inner))
            }

            Expr::Tail(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Tail(Box::new(new_inner))
            }

            Expr::Reverse(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Reverse(Box::new(new_inner))
            }

            Expr::Sort(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Sort(Box::new(new_inner))
            }

            Expr::Dedup(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Dedup(Box::new(new_inner))
            }

            Expr::Take(count_expr, list_expr) => {
                let (new_c, r1) = self.rewrite_step(count_expr);
                let (new_l, r2) = self.rewrite_step(list_expr);
                rewrites += r1 + r2;
                Expr::Take(Box::new(new_c), Box::new(new_l))
            }

            Expr::Drop(count_expr, list_expr) => {
                let (new_c, r1) = self.rewrite_step(count_expr);
                let (new_l, r2) = self.rewrite_step(list_expr);
                rewrites += r1 + r2;
                Expr::Drop(Box::new(new_c), Box::new(new_l))
            }

            Expr::Concat(left, right) => {
                let (new_left, r1) = self.rewrite_step(left);
                let (new_right, r2) = self.rewrite_step(right);
                rewrites += r1 + r2;
                Expr::Concat(Box::new(new_left), Box::new(new_right))
            }

            Expr::Min(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Min(Box::new(new_inner))
            }

            Expr::Max(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Max(Box::new(new_inner))
            }

            Expr::Sum(inner) => {
                let (new_inner, r) = self.rewrite_step(inner);
                rewrites += r;
                Expr::Sum(Box::new(new_inner))
            }

            Expr::Map { list, func } => {
                let (new_l, r1) = self.rewrite_step(list);
                let (new_f, r2) = self.rewrite_step(func);
                rewrites += r1 + r2;
                Expr::Map {
                    list: Box::new(new_l),
                    func: Box::new(new_f),
                }
            }

            Expr::Filter { list, pred } => {
                let (new_l, r1) = self.rewrite_step(list);
                let (new_p, r2) = self.rewrite_step(pred);
                rewrites += r1 + r2;
                Expr::Filter {
                    list: Box::new(new_l),
                    pred: Box::new(new_p),
                }
            }

            Expr::Fold { list, init, func } => {
                let (new_l, r1) = self.rewrite_step(list);
                let (new_i, r2) = self.rewrite_step(init);
                let (new_f, r3) = self.rewrite_step(func);
                rewrites += r1 + r2 + r3;
                Expr::Fold {
                    list: Box::new(new_l),
                    init: Box::new(new_i),
                    func: Box::new(new_f),
                }
            }

            Expr::ZipWith { list1, list2, func } => {
                let (new_l1, r1) = self.rewrite_step(list1);
                let (new_l2, r2) = self.rewrite_step(list2);
                let (new_f, r3) = self.rewrite_step(func);
                rewrites += r1 + r2 + r3;
                Expr::ZipWith {
                    list1: Box::new(new_l1),
                    list2: Box::new(new_l2),
                    func: Box::new(new_f),
                }
            }

            Expr::Lambda { param_types, body } => {
                let (new_b, r) = self.rewrite_step(body);
                rewrites += r;
                Expr::Lambda {
                    param_types: param_types.clone(),
                    body: Box::new(new_b),
                }
            }
        };

        // 2. Try applying top-level pattern matching rule on the resulting term
        if let Some(rewritten) = apply_rewrite_rule(&simplified_children) {
            (rewritten, rewrites + 1)
        } else {
            (simplified_children, rewrites)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::dsl::BinaryOp;

    #[test]
    fn test_reverse_reverse_inversion() {
        let trs = TermRewriter::default();
        // reverse(reverse(x0)) => x0
        let expr = Expr::Reverse(Box::new(Expr::Reverse(Box::new(Expr::Var(0)))));
        let (norm, count) = trs.normalize(&expr);
        assert_eq!(norm, Expr::Var(0));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sort_sort_idempotence() {
        let trs = TermRewriter::default();
        // sort(sort(x0)) => sort(x0)
        let expr = Expr::Sort(Box::new(Expr::Sort(Box::new(Expr::Var(0)))));
        let (norm, count) = trs.normalize(&expr);
        assert_eq!(norm, Expr::Sort(Box::new(Expr::Var(0))));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_min_sort_composition() {
        let trs = TermRewriter::default();
        // min(sort(x0)) => min(x0)
        let expr = Expr::Min(Box::new(Expr::Sort(Box::new(Expr::Var(0)))));
        let (norm, count) = trs.normalize(&expr);
        assert_eq!(norm, Expr::Min(Box::new(Expr::Var(0))));
        assert_eq!(count, 1);
    }

    #[test]
    fn test_algebraic_identities_and_constant_folding() {
        let trs = TermRewriter::default();
        // (x0 + 0) => x0
        let expr = Expr::Binary(
            BinaryOp::Add,
            Box::new(Expr::Var(0)),
            Box::new(Expr::Literal(Value::Int(0))),
        );
        let (norm, count) = trs.normalize(&expr);
        assert_eq!(norm, Expr::Var(0));
        assert_eq!(count, 1);

        // (5 + 10) => 15
        let fold_expr = Expr::Binary(
            BinaryOp::Add,
            Box::new(Expr::Literal(Value::Int(5))),
            Box::new(Expr::Literal(Value::Int(10))),
        );
        let (norm, count) = trs.normalize(&fold_expr);
        assert_eq!(norm, Expr::Literal(Value::Int(15)));
        assert_eq!(count, 1);
    }
}
