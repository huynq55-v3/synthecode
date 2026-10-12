use crate::core::{CoreError, Type, Value};
use super::ast::{BinaryOp, Expr, UnaryOp};
use super::type_checker::{TypeChecker, TypeEnv};

/// High-speed, deterministic 64-bit pseudo-random number generator (SplitMix64).
/// Completely self-contained with zero external dependencies.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xda942042e4dd58b5 } else { seed },
        }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    #[inline]
    pub fn next_usize(&mut self, upper: usize) -> usize {
        if upper <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % upper
    }

    #[inline]
    pub fn next_i64_range(&mut self, low: i64, high: i64) -> i64 {
        if high <= low {
            return low;
        }
        let range = (high - low + 1) as u64;
        low + (self.next_u64() % range) as i64
    }

    #[inline]
    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }
}

/// Grammar Production Engine for Type-Directed AST Synthesis.
pub struct Grammar;

impl Grammar {
    /// Autonomously generates a statically type-safe expression adhering to the target type.
    pub fn generate_typed(
        target: &Type,
        env: &TypeEnv,
        max_depth: usize,
        rng: &mut SplitMix64,
    ) -> Result<Expr, CoreError> {
        let expr = Self::gen_rec(target, env, 0, max_depth, rng)?;
        // Verify formal static type inference contract
        let inferred = TypeChecker::infer(&expr, env)?;
        if inferred != *target {
            return Err(CoreError::TypeMismatch {
                expected: target.to_string(),
                actual: inferred.to_string(),
            });
        }
        Ok(expr)
    }

    fn gen_rec(
        target: &Type,
        env: &TypeEnv,
        current_depth: usize,
        max_depth: usize,
        rng: &mut SplitMix64,
    ) -> Result<Expr, CoreError> {
        // Collect candidate variables from environment matching target type
        let matching_vars: Vec<usize> = (0..100)
            .take_while(|&idx| env.lookup(idx).is_ok())
            .filter(|&idx| env.lookup(idx).ok().as_ref() == Some(target))
            .collect();

        let at_leaf = current_depth >= max_depth || (current_depth > 0 && rng.next_usize(5) == 0);

        if at_leaf {
            if !matching_vars.is_empty() && (rng.next_bool() || matches!(target, Type::List(_))) {
                let picked_var = matching_vars[rng.next_usize(matching_vars.len())];
                return Ok(Expr::Var(picked_var));
            }
            return Ok(Self::gen_literal(target, rng));
        }

        match target {
            Type::Int => {
                let choice = rng.next_usize(8);
                match choice {
                    0 if !matching_vars.is_empty() => {
                        Ok(Expr::Var(matching_vars[rng.next_usize(matching_vars.len())]))
                    }
                    1 => Ok(Expr::Unary(
                        if rng.next_bool() { UnaryOp::Neg } else { UnaryOp::Abs },
                        Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                    )),
                    2 => {
                        let op = match rng.next_usize(4) {
                            0 => BinaryOp::Add,
                            1 => BinaryOp::Sub,
                            2 => BinaryOp::Mul,
                            _ => BinaryOp::Mod,
                        };
                        Ok(Expr::Binary(
                            op,
                            Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                            Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                        ))
                    }
                    3 => {
                        // Length of list
                        let elem_ty = if rng.next_bool() { Type::Int } else { Type::Bool };
                        let list_ty = Type::list(elem_ty);
                        Ok(Expr::Length(Box::new(Self::gen_rec(
                            &list_ty,
                            env,
                            current_depth + 1,
                            max_depth,
                            rng,
                        )?)))
                    }
                    4 => {
                        // Sum of list
                        let list_ty = Type::list(Type::Int);
                        Ok(Expr::Sum(Box::new(Self::gen_rec(
                            &list_ty,
                            env,
                            current_depth + 1,
                            max_depth,
                            rng,
                        )?)))
                    }
                    5 => {
                        // Min of list
                        let list_ty = Type::list(Type::Int);
                        Ok(Expr::Min(Box::new(Self::gen_rec(
                            &list_ty,
                            env,
                            current_depth + 1,
                            max_depth,
                            rng,
                        )?)))
                    }
                    6 => {
                        // Max of list
                        let list_ty = Type::list(Type::Int);
                        Ok(Expr::Max(Box::new(Self::gen_rec(
                            &list_ty,
                            env,
                            current_depth + 1,
                            max_depth,
                            rng,
                        )?)))
                    }
                    _ => Ok(Self::gen_literal(&Type::Int, rng)),
                }
            }

            Type::Bool => {
                let choice = rng.next_usize(5);
                match choice {
                    0 if !matching_vars.is_empty() => {
                        Ok(Expr::Var(matching_vars[rng.next_usize(matching_vars.len())]))
                    }
                    1 => Ok(Expr::Unary(
                        UnaryOp::Not,
                        Box::new(Self::gen_rec(&Type::Bool, env, current_depth + 1, max_depth, rng)?),
                    )),
                    2 => {
                        let op = if rng.next_bool() { BinaryOp::And } else { BinaryOp::Or };
                        Ok(Expr::Binary(
                            op,
                            Box::new(Self::gen_rec(&Type::Bool, env, current_depth + 1, max_depth, rng)?),
                            Box::new(Self::gen_rec(&Type::Bool, env, current_depth + 1, max_depth, rng)?),
                        ))
                    }
                    3 => {
                        let op = match rng.next_usize(4) {
                            0 => BinaryOp::Lt,
                            1 => BinaryOp::Lte,
                            2 => BinaryOp::Gt,
                            _ => BinaryOp::Eq,
                        };
                        Ok(Expr::Binary(
                            op,
                            Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                            Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                        ))
                    }
                    _ => Ok(Self::gen_literal(&Type::Bool, rng)),
                }
            }

            Type::List(_elem_ty) => {
                let choice = rng.next_usize(7);
                match choice {
                    0 if !matching_vars.is_empty() => {
                        Ok(Expr::Var(matching_vars[rng.next_usize(matching_vars.len())]))
                    }
                    1 => Ok(Expr::Reverse(Box::new(Self::gen_rec(
                        target,
                        env,
                        current_depth + 1,
                        max_depth,
                        rng,
                    )?))),
                    2 => Ok(Expr::Sort(Box::new(Self::gen_rec(
                        target,
                        env,
                        current_depth + 1,
                        max_depth,
                        rng,
                    )?))),
                    3 => Ok(Expr::Dedup(Box::new(Self::gen_rec(
                        target,
                        env,
                        current_depth + 1,
                        max_depth,
                        rng,
                    )?))),
                    4 => Ok(Expr::Take(
                        Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                        Box::new(Self::gen_rec(target, env, current_depth + 1, max_depth, rng)?),
                    )),
                    5 => Ok(Expr::Drop(
                        Box::new(Self::gen_rec(&Type::Int, env, current_depth + 1, max_depth, rng)?),
                        Box::new(Self::gen_rec(target, env, current_depth + 1, max_depth, rng)?),
                    )),
                    _ => {
                        if !matching_vars.is_empty() {
                            Ok(Expr::Var(matching_vars[rng.next_usize(matching_vars.len())]))
                        } else {
                            // Non-empty base list literal
                            Ok(Expr::Literal(Value::List(vec![
                                Value::Int(rng.next_i64_range(0, 10)),
                                Value::Int(rng.next_i64_range(0, 10)),
                            ])))
                        }
                    }
                }
            }

            _ => {
                if !matching_vars.is_empty() {
                    Ok(Expr::Var(matching_vars[0]))
                } else {
                    Ok(Self::gen_literal(target, rng))
                }
            }
        }
    }

    fn gen_literal(target: &Type, rng: &mut SplitMix64) -> Expr {
        match target {
            Type::Int => Expr::Literal(Value::Int(rng.next_i64_range(-10, 50))),
            Type::Bool => Expr::Literal(Value::Bool(rng.next_bool())),
            Type::List(elem) => {
                let len = rng.next_usize(4) + 1;
                let items: Vec<Value> = (0..len)
                    .map(|_| match elem.as_ref() {
                        Type::Int => Value::Int(rng.next_i64_range(0, 20)),
                        Type::Bool => Value::Bool(rng.next_bool()),
                        _ => Value::Int(0),
                    })
                    .collect();
                Expr::Literal(Value::List(items))
            }
            _ => Expr::Literal(Value::Int(0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_prng() {
        let mut rng1 = SplitMix64::new(42);
        let mut rng2 = SplitMix64::new(42);
        for _ in 0..10 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_type_directed_grammar_generation() {
        let mut rng = SplitMix64::new(1337);
        let env = TypeEnv::from_params(&[Type::list(Type::Int)]);

        for _ in 0..20 {
            let expr = Grammar::generate_typed(&Type::Int, &env, 3, &mut rng).unwrap();
            let ty = TypeChecker::infer(&expr, &env).unwrap();
            assert_eq!(ty, Type::Int);
        }
    }
}
