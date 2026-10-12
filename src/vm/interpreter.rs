use crate::core::{CoreError, Value};
use crate::dsl::{BinaryOp, Expr, UnaryOp};
use super::gas::{GasMeter, VmError};

/// Deterministic Reference AST Interpreter instrumented with Gas Metering and Call-Stack limits.
#[derive(Debug)]
pub struct Interpreter {
    pub gas: GasMeter,
    stack_depth: usize,
    max_stack_depth: usize,
}

impl Interpreter {
    /// Create a new Interpreter with the given gas budget and default stack limit (128).
    pub fn new(gas: GasMeter) -> Self {
        Self {
            gas,
            stack_depth: 0,
            max_stack_depth: 128,
        }
    }

    /// Set maximum call stack depth.
    pub fn with_max_stack_depth(mut self, depth: usize) -> Self {
        self.max_stack_depth = depth;
        self
    }

    /// Evaluates an expression against an input parameter tuple / environment.
    pub fn eval(&mut self, expr: &Expr, env: &[Value]) -> Result<Value, VmError> {
        self.gas.consume(1)?;

        if self.stack_depth >= self.max_stack_depth {
            return Err(VmError::StackOverflow {
                limit: self.max_stack_depth,
            });
        }

        self.stack_depth += 1;
        let res = self.eval_inner(expr, env);
        self.stack_depth -= 1;
        res
    }

    fn eval_inner(&mut self, expr: &Expr, env: &[Value]) -> Result<Value, VmError> {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),

            Expr::Var(idx) => env.get(*idx).cloned().ok_or_else(|| {
                VmError::EvaluationError(format!("Unbound variable index x{} (env size: {})", idx, env.len()))
            }),

            Expr::Unary(op, inner) => {
                let v = self.eval(inner, env)?;
                match op {
                    UnaryOp::Neg => {
                        let i = v.as_int()?;
                        let res = i.checked_neg().ok_or(CoreError::IntegerOverflow {
                            operation: "unary negation",
                        })?;
                        Ok(Value::Int(res))
                    }
                    UnaryOp::Not => {
                        let b = v.as_bool()?;
                        Ok(Value::Bool(!b))
                    }
                    UnaryOp::Abs => {
                        let i = v.as_int()?;
                        let res = i.checked_abs().ok_or(CoreError::IntegerOverflow {
                            operation: "abs",
                        })?;
                        Ok(Value::Int(res))
                    }
                }
            }

            Expr::Binary(op, left, right) => {
                // Short-circuiting evaluation for boolean conjunction and disjunction
                if *op == BinaryOp::And {
                    let left_val = self.eval(left, env)?;
                    if !left_val.as_bool()? {
                        return Ok(Value::Bool(false));
                    }
                    let right_val = self.eval(right, env)?;
                    return Ok(Value::Bool(right_val.as_bool()?));
                } else if *op == BinaryOp::Or {
                    let left_val = self.eval(left, env)?;
                    if left_val.as_bool()? {
                        return Ok(Value::Bool(true));
                    }
                    let right_val = self.eval(right, env)?;
                    return Ok(Value::Bool(right_val.as_bool()?));
                }

                let l = self.eval(left, env)?;
                let r = self.eval(right, env)?;

                match op {
                    BinaryOp::Add => {
                        let res = l.as_int()?.checked_add(r.as_int()?).ok_or(CoreError::IntegerOverflow {
                            operation: "addition",
                        })?;
                        Ok(Value::Int(res))
                    }
                    BinaryOp::Sub => {
                        let res = l.as_int()?.checked_sub(r.as_int()?).ok_or(CoreError::IntegerOverflow {
                            operation: "subtraction",
                        })?;
                        Ok(Value::Int(res))
                    }
                    BinaryOp::Mul => {
                        let res = l.as_int()?.checked_mul(r.as_int()?).ok_or(CoreError::IntegerOverflow {
                            operation: "multiplication",
                        })?;
                        Ok(Value::Int(res))
                    }
                    BinaryOp::Div => {
                        let denom = r.as_int()?;
                        if denom == 0 {
                            return Err(CoreError::DivisionByZero.into());
                        }
                        let res = l.as_int()?.checked_div(denom).ok_or(CoreError::IntegerOverflow {
                            operation: "division",
                        })?;
                        Ok(Value::Int(res))
                    }
                    BinaryOp::Mod => {
                        let denom = r.as_int()?;
                        if denom == 0 {
                            return Err(CoreError::DivisionByZero.into());
                        }
                        let res = l.as_int()?.checked_rem(denom).ok_or(CoreError::IntegerOverflow {
                            operation: "modulo",
                        })?;
                        Ok(Value::Int(res))
                    }
                    BinaryOp::Eq => Ok(Value::Bool(l == r)),
                    BinaryOp::Neq => Ok(Value::Bool(l != r)),
                    BinaryOp::Lt => Ok(Value::Bool(l.as_int()? < r.as_int()?)),
                    BinaryOp::Lte => Ok(Value::Bool(l.as_int()? <= r.as_int()?)),
                    BinaryOp::Gt => Ok(Value::Bool(l.as_int()? > r.as_int()?)),
                    BinaryOp::Gte => Ok(Value::Bool(l.as_int()? >= r.as_int()?)),
                    BinaryOp::And | BinaryOp::Or => unreachable!(),
                }
            }

            Expr::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval(cond, env)?;
                if cond_val.as_bool()? {
                    self.eval(then_branch, env)
                } else {
                    self.eval(else_branch, env)
                }
            }

            Expr::Length(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                Ok(Value::Int(items.len() as i64))
            }

            Expr::Head(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                items.first().cloned().ok_or_else(|| {
                    VmError::EvaluationError("Cannot compute head of empty list".into())
                })
            }

            Expr::Tail(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                if items.is_empty() {
                    Ok(Value::List(Vec::new()))
                } else {
                    self.gas.consume(items.len() as u64)?;
                    Ok(Value::List(items[1..].to_vec()))
                }
            }

            Expr::Reverse(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                self.gas.consume(items.len() as u64)?;
                let mut reversed = items.to_vec();
                reversed.reverse();
                Ok(Value::List(reversed))
            }

            Expr::Sort(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                let cost = (items.len() as u64).saturating_mul(4);
                self.gas.consume(cost)?;
                let mut sorted = items.to_vec();
                sorted.sort();
                Ok(Value::List(sorted))
            }

            Expr::Dedup(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                self.gas.consume(items.len() as u64)?;
                let mut deduped = Vec::with_capacity(items.len());
                for item in items {
                    if deduped.last() != Some(item) {
                        deduped.push(item.clone());
                    }
                }
                Ok(Value::List(deduped))
            }

            Expr::Take(count_expr, list_expr) => {
                let count_val = self.eval(count_expr, env)?;
                let list_val = self.eval(list_expr, env)?;
                let n = count_val.as_int()?.max(0) as usize;
                let items = list_val.as_list()?;
                let take_len = n.min(items.len());
                self.gas.consume(take_len as u64)?;
                Ok(Value::List(items[..take_len].to_vec()))
            }

            Expr::Drop(count_expr, list_expr) => {
                let count_val = self.eval(count_expr, env)?;
                let list_val = self.eval(list_expr, env)?;
                let n = count_val.as_int()?.max(0) as usize;
                let items = list_val.as_list()?;
                let drop_idx = n.min(items.len());
                self.gas.consume((items.len() - drop_idx) as u64)?;
                Ok(Value::List(items[drop_idx..].to_vec()))
            }

            Expr::Concat(left, right) => {
                let l_val = self.eval(left, env)?;
                let r_val = self.eval(right, env)?;
                let l_items = l_val.as_list()?;
                let r_items = r_val.as_list()?;
                self.gas.consume((l_items.len() + r_items.len()) as u64)?;
                let mut combined = l_items.to_vec();
                combined.extend_from_slice(r_items);
                Ok(Value::List(combined))
            }

            Expr::Min(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                if items.is_empty() {
                    return Err(VmError::EvaluationError("Cannot compute min of empty list".into()));
                }
                self.gas.consume(items.len() as u64)?;
                let min_val = items.iter().min().cloned().unwrap();
                Ok(min_val)
            }

            Expr::Max(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                if items.is_empty() {
                    return Err(VmError::EvaluationError("Cannot compute max of empty list".into()));
                }
                self.gas.consume(items.len() as u64)?;
                let max_val = items.iter().max().cloned().unwrap();
                Ok(max_val)
            }

            Expr::Sum(list_expr) => {
                let val = self.eval(list_expr, env)?;
                let items = val.as_list()?;
                self.gas.consume(items.len() as u64)?;
                let mut sum: i64 = 0;
                for item in items {
                    sum = sum.checked_add(item.as_int()?).ok_or(CoreError::IntegerOverflow {
                        operation: "sum",
                    })?;
                }
                Ok(Value::Int(sum))
            }

            Expr::Lambda { .. } => {
                // When evaluated standalone, lambda is not directly callable as value;
                // it is invoked by higher-order combinators.
                Err(VmError::EvaluationError(
                    "First-class lambda values cannot be returned as primitive data".into(),
                ))
            }

            Expr::Map { list, func } => {
                let list_val = self.eval(list, env)?;
                let items = list_val.as_list()?;
                let mut mapped = Vec::with_capacity(items.len());
                for item in items {
                    let out = self.invoke_combinator(func, env, &[item.clone()])?;
                    mapped.push(out);
                }
                Ok(Value::List(mapped))
            }

            Expr::Filter { list, pred } => {
                let list_val = self.eval(list, env)?;
                let items = list_val.as_list()?;
                let mut filtered = Vec::new();
                for item in items {
                    let keep = self.invoke_combinator(pred, env, &[item.clone()])?;
                    if keep.as_bool()? {
                        filtered.push(item.clone());
                    }
                }
                Ok(Value::List(filtered))
            }

            Expr::Fold { list, init, func } => {
                let list_val = self.eval(list, env)?;
                let mut acc = self.eval(init, env)?;
                let items = list_val.as_list()?;
                for item in items {
                    acc = self.invoke_combinator(func, env, &[acc, item.clone()])?;
                }
                Ok(acc)
            }

            Expr::ZipWith { list1, list2, func } => {
                let l1_val = self.eval(list1, env)?;
                let l2_val = self.eval(list2, env)?;
                let items1 = l1_val.as_list()?;
                let items2 = l2_val.as_list()?;
                let min_len = items1.len().min(items2.len());
                let mut result = Vec::with_capacity(min_len);
                for i in 0..min_len {
                    let out = self.invoke_combinator(
                        func,
                        env,
                        &[items1[i].clone(), items2[i].clone()],
                    )?;
                    result.push(out);
                }
                Ok(Value::List(result))
            }
        }
    }

    /// Invokes a combinator functional argument (Lambda or nested expression).
    fn invoke_combinator(
        &mut self,
        func: &Expr,
        env: &[Value],
        args: &[Value],
    ) -> Result<Value, VmError> {
        self.gas.consume(2)?;
        match func {
            Expr::Lambda { param_types, body } => {
                if param_types.len() != args.len() {
                    return Err(VmError::ArgumentCountMismatch {
                        expected: param_types.len(),
                        actual: args.len(),
                    });
                }
                // Lexical scoping: bind closure args after outer environment
                let mut extended_env = env.to_vec();
                extended_env.extend_from_slice(args);
                self.eval(body, &extended_env)
            }
            // If func is an arbitrary sub-expression or closure
            other => self.eval(other, env),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::BinaryOp;

    #[test]
    fn test_eval_arithmetic_and_gas_consumption() {
        let mut interp = Interpreter::new(GasMeter::new(100));
        let expr = Expr::Binary(
            BinaryOp::Add,
            Box::new(Expr::Literal(Value::Int(15))),
            Box::new(Expr::Literal(Value::Int(27))),
        );
        let res = interp.eval(&expr, &[]).unwrap();
        assert_eq!(res, Value::Int(42));
        assert!(interp.gas.consumed() > 0);
    }

    #[test]
    fn test_eval_list_operations() {
        let mut interp = Interpreter::new(GasMeter::new(500));
        let input_list = Value::List(vec![Value::Int(3), Value::Int(1), Value::Int(2)]);
        let env = vec![input_list];

        // sort(x0)
        let sort_expr = Expr::Sort(Box::new(Expr::Var(0)));
        let sorted = interp.eval(&sort_expr, &env).unwrap();
        assert_eq!(
            sorted,
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );

        // reverse(sort(x0))
        let rev_sort = Expr::Reverse(Box::new(sort_expr));
        let rev_sorted = interp.eval(&rev_sort, &env).unwrap();
        assert_eq!(
            rev_sorted,
            Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)])
        );
    }

    #[test]
    fn test_gas_exhaustion_terminates() {
        let mut interp = Interpreter::new(GasMeter::new(2));
        // Needs at least 3 gas steps
        let expr = Expr::Binary(
            BinaryOp::Add,
            Box::new(Expr::Literal(Value::Int(1))),
            Box::new(Expr::Literal(Value::Int(2))),
        );
        let err = interp.eval(&expr, &[]);
        assert!(matches!(err, Err(VmError::OutOfGas { .. })));
    }
}
