use crate::core::Value;
use crate::dsl::{BinaryOp, Expr, UnaryOp};

/// Single-step pattern rewrite matching rule.
/// Returns `Some(simplified_expr)` if a rewrite rule matched and triggered, otherwise `None`.
pub fn apply_rewrite_rule(expr: &Expr) -> Option<Expr> {
    // 1. Involuntary / Double Negation / Double Inversion laws
    match expr {
        // reverse(reverse(x)) => x
        Expr::Reverse(inner) => {
            if let Expr::Reverse(orig) = inner.as_ref() {
                return Some((**orig).clone());
            }
        }

        // neg(neg(x)) => x
        Expr::Unary(UnaryOp::Neg, inner) => {
            if let Expr::Unary(UnaryOp::Neg, orig) = inner.as_ref() {
                return Some((**orig).clone());
            }
        }

        // not(not(x)) => x
        Expr::Unary(UnaryOp::Not, inner) => {
            if let Expr::Unary(UnaryOp::Not, orig) = inner.as_ref() {
                return Some((**orig).clone());
            }
        }

        // sort(sort(x)) => sort(x) [Idempotence]
        Expr::Sort(inner) => {
            if let Expr::Sort(_) = inner.as_ref() {
                return Some((**inner).clone());
            }
            // sort(reverse(x)) => sort(x)
            if let Expr::Reverse(orig) = inner.as_ref() {
                return Some(Expr::Sort(orig.clone()));
            }
        }

        // dedup(dedup(x)) => dedup(x) [Idempotence]
        Expr::Dedup(inner) => {
            if let Expr::Dedup(_) = inner.as_ref() {
                return Some((**inner).clone());
            }
        }

        // min(sort(x)) => min(x)
        Expr::Min(inner) => {
            if let Expr::Sort(orig) = inner.as_ref() {
                return Some(Expr::Min(orig.clone()));
            }
            if let Expr::Reverse(orig) = inner.as_ref() {
                return Some(Expr::Min(orig.clone()));
            }
        }

        // max(sort(x)) => max(x)
        Expr::Max(inner) => {
            if let Expr::Sort(orig) = inner.as_ref() {
                return Some(Expr::Max(orig.clone()));
            }
            if let Expr::Reverse(orig) = inner.as_ref() {
                return Some(Expr::Max(orig.clone()));
            }
        }

        // sum(sort(x)) => sum(x) & sum(reverse(x)) => sum(x)
        Expr::Sum(inner) => {
            if let Expr::Sort(orig) = inner.as_ref() {
                return Some(Expr::Sum(orig.clone()));
            }
            if let Expr::Reverse(orig) = inner.as_ref() {
                return Some(Expr::Sum(orig.clone()));
            }
        }

        // len(sort(x)) => len(x) & len(reverse(x)) => len(x)
        Expr::Length(inner) => {
            if let Expr::Sort(orig) = inner.as_ref() {
                return Some(Expr::Length(orig.clone()));
            }
            if let Expr::Reverse(orig) = inner.as_ref() {
                return Some(Expr::Length(orig.clone()));
            }
        }

        // take(a, take(b, x)) where a, b are Int literals => take(min(a, b), x)
        Expr::Take(a_expr, inner_take) => {
            if let (Expr::Literal(Value::Int(a)), Expr::Take(b_expr, list_orig)) =
                (a_expr.as_ref(), inner_take.as_ref())
            {
                if let Expr::Literal(Value::Int(b)) = b_expr.as_ref() {
                    let min_take = (*a).min(*b);
                    return Some(Expr::Take(
                        Box::new(Expr::Literal(Value::Int(min_take))),
                        list_orig.clone(),
                    ));
                }
            }
        }

        // Branching Simplifications
        Expr::IfThenElse {
            cond,
            then_branch,
            else_branch,
        } => {
            // if true then A else B => A
            if let Expr::Literal(Value::Bool(true)) = cond.as_ref() {
                return Some((**then_branch).clone());
            }
            // if false then A else B => B
            if let Expr::Literal(Value::Bool(false)) = cond.as_ref() {
                return Some((**else_branch).clone());
            }
            // if C then A else A => A
            if then_branch == else_branch {
                return Some((**then_branch).clone());
            }
        }

        // Binary Arithmetic & Boolean Laws
        Expr::Binary(op, left, right) => {
            // Constant folding on literals
            if let (Expr::Literal(l_val), Expr::Literal(r_val)) = (left.as_ref(), right.as_ref()) {
                if let Some(folded) = fold_constant_binary(*op, l_val, r_val) {
                    return Some(Expr::Literal(folded));
                }
            }

            match op {
                // x + 0 => x, 0 + x => x
                BinaryOp::Add => {
                    if is_int_literal(right, 0) {
                        return Some((**left).clone());
                    }
                    if is_int_literal(left, 0) {
                        return Some((**right).clone());
                    }
                }

                // x - 0 => x, x - x => 0
                BinaryOp::Sub => {
                    if is_int_literal(right, 0) {
                        return Some((**left).clone());
                    }
                    if left == right {
                        return Some(Expr::Literal(Value::Int(0)));
                    }
                }

                // x * 1 => x, 1 * x => x, x * 0 => 0, 0 * x => 0
                BinaryOp::Mul => {
                    if is_int_literal(right, 1) {
                        return Some((**left).clone());
                    }
                    if is_int_literal(left, 1) {
                        return Some((**right).clone());
                    }
                    if is_int_literal(right, 0) || is_int_literal(left, 0) {
                        return Some(Expr::Literal(Value::Int(0)));
                    }
                }

                // x / 1 => x
                BinaryOp::Div => {
                    if is_int_literal(right, 1) {
                        return Some((**left).clone());
                    }
                }

                // x == x => true, x != x => false
                BinaryOp::Eq => {
                    if left == right {
                        return Some(Expr::Literal(Value::Bool(true)));
                    }
                }
                BinaryOp::Neq => {
                    if left == right {
                        return Some(Expr::Literal(Value::Bool(false)));
                    }
                }

                // Boolean identity & annihilation
                BinaryOp::And => {
                    if is_bool_literal(right, true) {
                        return Some((**left).clone());
                    }
                    if is_bool_literal(left, true) {
                        return Some((**right).clone());
                    }
                    if is_bool_literal(right, false) || is_bool_literal(left, false) {
                        return Some(Expr::Literal(Value::Bool(false)));
                    }
                    if left == right {
                        return Some((**left).clone());
                    }
                }

                BinaryOp::Or => {
                    if is_bool_literal(right, false) {
                        return Some((**left).clone());
                    }
                    if is_bool_literal(left, false) {
                        return Some((**right).clone());
                    }
                    if is_bool_literal(right, true) || is_bool_literal(left, true) {
                        return Some(Expr::Literal(Value::Bool(true)));
                    }
                    if left == right {
                        return Some((**left).clone());
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    None
}

#[inline]
fn is_int_literal(expr: &Expr, val: i64) -> bool {
    matches!(expr, Expr::Literal(Value::Int(v)) if *v == val)
}

#[inline]
fn is_bool_literal(expr: &Expr, val: bool) -> bool {
    matches!(expr, Expr::Literal(Value::Bool(v)) if *v == val)
}

fn fold_constant_binary(op: BinaryOp, l: &Value, r: &Value) -> Option<Value> {
    match (l, r) {
        (Value::Int(a), Value::Int(b)) => match op {
            BinaryOp::Add => a.checked_add(*b).map(Value::Int),
            BinaryOp::Sub => a.checked_sub(*b).map(Value::Int),
            BinaryOp::Mul => a.checked_mul(*b).map(Value::Int),
            BinaryOp::Div if *b != 0 => a.checked_div(*b).map(Value::Int),
            BinaryOp::Mod if *b != 0 => a.checked_rem(*b).map(Value::Int),
            BinaryOp::Eq => Some(Value::Bool(a == b)),
            BinaryOp::Neq => Some(Value::Bool(a != b)),
            BinaryOp::Lt => Some(Value::Bool(a < b)),
            BinaryOp::Lte => Some(Value::Bool(a <= b)),
            BinaryOp::Gt => Some(Value::Bool(a > b)),
            BinaryOp::Gte => Some(Value::Bool(a >= b)),
            _ => None,
        },
        (Value::Bool(a), Value::Bool(b)) => match op {
            BinaryOp::Eq => Some(Value::Bool(a == b)),
            BinaryOp::Neq => Some(Value::Bool(a != b)),
            BinaryOp::And => Some(Value::Bool(*a && *b)),
            BinaryOp::Or => Some(Value::Bool(*a || *b)),
            _ => None,
        },
        _ => None,
    }
}
