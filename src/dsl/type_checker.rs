use crate::core::{CoreError, Type};
use super::ast::{BinaryOp, Expr, UnaryOp};

/// Type environment representing variable types currently in lexical scope.
/// Index 0 is the innermost (most recently bound) variable when using de Bruijn indices,
/// or positional arguments for top-level problems.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    bindings: Vec<Type>,
}

impl TypeEnv {
    /// Create a new empty type environment.
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Create environment pre-populated with input parameter types.
    pub fn from_params(params: &[Type]) -> Self {
        Self {
            bindings: params.to_vec(),
        }
    }

    /// Push new variable types onto the lexical scope.
    pub fn extend(&self, new_bindings: &[Type]) -> Self {
        let mut extended = self.bindings.clone();
        extended.extend_from_slice(new_bindings);
        Self { bindings: extended }
    }

    /// Lookup type of variable by index.
    pub fn lookup(&self, index: usize) -> Result<Type, CoreError> {
        self.bindings.get(index).cloned().ok_or_else(|| {
            CoreError::IndexOutOfBounds {
                index,
                len: self.bindings.len(),
            }
        })
    }
}

/// TypeChecker performs static type inference and compositional type safety validation on AST expressions.
pub struct TypeChecker;

impl TypeChecker {
    /// Infer the static type of an expression under a given type environment.
    pub fn infer(expr: &Expr, env: &TypeEnv) -> Result<Type, CoreError> {
        match expr {
            Expr::Literal(val) => val.infer_type(),

            Expr::Var(idx) => env.lookup(*idx),

            Expr::Unary(op, inner) => {
                let inner_ty = Self::infer(inner, env)?;
                match op {
                    UnaryOp::Neg | UnaryOp::Abs => {
                        if inner_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(CoreError::TypeMismatch {
                                expected: "Int".to_string(),
                                actual: inner_ty.to_string(),
                            })
                        }
                    }
                    UnaryOp::Not => {
                        if inner_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(CoreError::TypeMismatch {
                                expected: "Bool".to_string(),
                                actual: inner_ty.to_string(),
                            })
                        }
                    }
                }
            }

            Expr::Binary(op, left, right) => {
                let left_ty = Self::infer(left, env)?;
                let right_ty = Self::infer(right, env)?;

                match op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(CoreError::TypeMismatch {
                                expected: "Int and Int".to_string(),
                                actual: format!("{} and {}", left_ty, right_ty),
                            })
                        }
                    }
                    BinaryOp::Lt | BinaryOp::Lte | BinaryOp::Gt | BinaryOp::Gte => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Bool)
                        } else {
                            Err(CoreError::TypeMismatch {
                                expected: "Int and Int".to_string(),
                                actual: format!("{} and {}", left_ty, right_ty),
                            })
                        }
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_ty == Type::Bool && right_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(CoreError::TypeMismatch {
                                expected: "Bool and Bool".to_string(),
                                actual: format!("{} and {}", left_ty, right_ty),
                            })
                        }
                    }
                    BinaryOp::Eq | BinaryOp::Neq => {
                        if left_ty == right_ty {
                            Ok(Type::Bool)
                        } else {
                            Err(CoreError::TypeMismatch {
                                expected: format!("identical types, got {}", left_ty),
                                actual: right_ty.to_string(),
                            })
                        }
                    }
                }
            }

            Expr::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_ty = Self::infer(cond, env)?;
                if cond_ty != Type::Bool {
                    return Err(CoreError::TypeMismatch {
                        expected: "Bool".to_string(),
                        actual: cond_ty.to_string(),
                    });
                }
                let then_ty = Self::infer(then_branch, env)?;
                let else_ty = Self::infer(else_branch, env)?;
                if then_ty != else_ty {
                    return Err(CoreError::TypeMismatch {
                        expected: then_ty.to_string(),
                        actual: else_ty.to_string(),
                    });
                }
                Ok(then_ty)
            }

            Expr::Length(list_expr) => {
                let ty = Self::infer(list_expr, env)?;
                if ty.is_collection() {
                    Ok(Type::Int)
                } else {
                    Err(CoreError::TypeMismatch {
                        expected: "Collection (List or Matrix)".to_string(),
                        actual: ty.to_string(),
                    })
                }
            }

            Expr::Head(list_expr) => {
                let ty = Self::infer(list_expr, env)?;
                match ty {
                    Type::List(elem) => Ok(*elem),
                    _ => Err(CoreError::TypeMismatch {
                        expected: "List<T>".to_string(),
                        actual: ty.to_string(),
                    }),
                }
            }

            Expr::Tail(list_expr)
            | Expr::Reverse(list_expr)
            | Expr::Sort(list_expr)
            | Expr::Dedup(list_expr) => {
                let ty = Self::infer(list_expr, env)?;
                match ty {
                    Type::List(_) => Ok(ty),
                    _ => Err(CoreError::TypeMismatch {
                        expected: "List<T>".to_string(),
                        actual: ty.to_string(),
                    }),
                }
            }

            Expr::Take(count_expr, list_expr) | Expr::Drop(count_expr, list_expr) => {
                let count_ty = Self::infer(count_expr, env)?;
                if count_ty != Type::Int {
                    return Err(CoreError::TypeMismatch {
                        expected: "Int".to_string(),
                        actual: count_ty.to_string(),
                    });
                }
                let list_ty = Self::infer(list_expr, env)?;
                match list_ty {
                    Type::List(_) => Ok(list_ty),
                    _ => Err(CoreError::TypeMismatch {
                        expected: "List<T>".to_string(),
                        actual: list_ty.to_string(),
                    }),
                }
            }

            Expr::Concat(left, right) => {
                let left_ty = Self::infer(left, env)?;
                let right_ty = Self::infer(right, env)?;
                if left_ty == right_ty && matches!(left_ty, Type::List(_)) {
                    Ok(left_ty)
                } else {
                    Err(CoreError::TypeMismatch {
                        expected: "Matching List<T> types".to_string(),
                        actual: format!("{} and {}", left_ty, right_ty),
                    })
                }
            }

            Expr::Min(list_expr) | Expr::Max(list_expr) | Expr::Sum(list_expr) => {
                let ty = Self::infer(list_expr, env)?;
                if ty == Type::list(Type::Int) {
                    Ok(Type::Int)
                } else {
                    Err(CoreError::TypeMismatch {
                        expected: "List<Int>".to_string(),
                        actual: ty.to_string(),
                    })
                }
            }

            Expr::Lambda { param_types, body } => {
                let extended_env = env.extend(param_types);
                let body_ty = Self::infer(body, &extended_env)?;
                Ok(Type::func(param_types.clone(), body_ty))
            }

            Expr::Map { list, func } => {
                let list_ty = Self::infer(list, env)?;
                let elem_ty = match &list_ty {
                    Type::List(inner) => inner.as_ref().clone(),
                    _ => {
                        return Err(CoreError::TypeMismatch {
                            expected: "List<T>".to_string(),
                            actual: list_ty.to_string(),
                        })
                    }
                };

                let func_ty = Self::infer(func, env)?;
                match func_ty {
                    Type::Fn(params, ret) if params.len() == 1 && params[0] == elem_ty => {
                        Ok(Type::list(*ret))
                    }
                    _ => Err(CoreError::TypeMismatch {
                        expected: format!("Fn([{}], Ret)", elem_ty),
                        actual: func_ty.to_string(),
                    }),
                }
            }

            Expr::Filter { list, pred } => {
                let list_ty = Self::infer(list, env)?;
                let elem_ty = match &list_ty {
                    Type::List(inner) => inner.as_ref().clone(),
                    _ => {
                        return Err(CoreError::TypeMismatch {
                            expected: "List<T>".to_string(),
                            actual: list_ty.to_string(),
                        })
                    }
                };

                let pred_ty = Self::infer(pred, env)?;
                match pred_ty {
                    Type::Fn(params, ret) if params.len() == 1 && params[0] == elem_ty && *ret == Type::Bool => {
                        Ok(list_ty)
                    }
                    _ => Err(CoreError::TypeMismatch {
                        expected: format!("Fn([{}], Bool)", elem_ty),
                        actual: pred_ty.to_string(),
                    }),
                }
            }

            Expr::Fold { list, init, func } => {
                let list_ty = Self::infer(list, env)?;
                let elem_ty = match &list_ty {
                    Type::List(inner) => inner.as_ref().clone(),
                    _ => {
                        return Err(CoreError::TypeMismatch {
                            expected: "List<T>".to_string(),
                            actual: list_ty.to_string(),
                        })
                    }
                };
                let init_ty = Self::infer(init, env)?;
                let func_ty = Self::infer(func, env)?;

                match func_ty {
                    Type::Fn(params, ret)
                        if params.len() == 2 && params[0] == init_ty && params[1] == elem_ty && *ret == init_ty =>
                    {
                        Ok(init_ty)
                    }
                    _ => Err(CoreError::TypeMismatch {
                        expected: format!("Fn([{}, {}], {})", init_ty, elem_ty, init_ty),
                        actual: func_ty.to_string(),
                    }),
                }
            }

            Expr::ZipWith { list1, list2, func } => {
                let list1_ty = Self::infer(list1, env)?;
                let list2_ty = Self::infer(list2, env)?;
                let elem1_ty = match &list1_ty {
                    Type::List(inner) => inner.as_ref().clone(),
                    _ => {
                        return Err(CoreError::TypeMismatch {
                            expected: "List<T1>".to_string(),
                            actual: list1_ty.to_string(),
                        })
                    }
                };
                let elem2_ty = match &list2_ty {
                    Type::List(inner) => inner.as_ref().clone(),
                    _ => {
                        return Err(CoreError::TypeMismatch {
                            expected: "List<T2>".to_string(),
                            actual: list2_ty.to_string(),
                        })
                    }
                };
                let func_ty = Self::infer(func, env)?;

                match func_ty {
                    Type::Fn(params, ret)
                        if params.len() == 2 && params[0] == elem1_ty && params[1] == elem2_ty =>
                    {
                        Ok(Type::list(*ret))
                    }
                    _ => Err(CoreError::TypeMismatch {
                        expected: format!("Fn([{}, {}], Ret)", elem1_ty, elem2_ty),
                        actual: func_ty.to_string(),
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_type_checking_primitives_and_collections() {
        let env = TypeEnv::from_params(&[Type::list(Type::Int)]);

        // len(x0) -> Int
        let len_expr = Expr::Length(Box::new(Expr::Var(0)));
        assert_eq!(TypeChecker::infer(&len_expr, &env).unwrap(), Type::Int);

        // reverse(x0) -> List<Int>
        let rev_expr = Expr::Reverse(Box::new(Expr::Var(0)));
        assert_eq!(
            TypeChecker::infer(&rev_expr, &env).unwrap(),
            Type::list(Type::Int)
        );

        // map(x0, \x -> x + 1) -> List<Int>
        let map_expr = Expr::Map {
            list: Box::new(Expr::Var(0)),
            func: Box::new(Expr::Lambda {
                param_types: vec![Type::Int],
                body: Box::new(Expr::Binary(
                    BinaryOp::Add,
                    Box::new(Expr::Var(1)),
                    Box::new(Expr::Literal(Value::Int(1))),
                )),
            }),
        };
        assert_eq!(
            TypeChecker::infer(&map_expr, &env).unwrap(),
            Type::list(Type::Int)
        );
    }
}
