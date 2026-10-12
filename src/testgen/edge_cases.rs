use crate::core::{Type, Value};

/// Generates deterministic extreme boundary test cases for algorithmic verification.
pub struct EdgeCaseGenerator;

impl EdgeCaseGenerator {
    /// Generates boundary inputs tailored to the parameter types of a problem.
    pub fn generate(param_types: &[Type]) -> Vec<Vec<Value>> {
        if param_types.is_empty() {
            return vec![vec![]];
        }

        let per_param_candidates: Vec<Vec<Value>> = param_types
            .iter()
            .map(|ty| Self::generate_single_param_edge_cases(ty))
            .collect();

        // Cartesian product or zip to form input argument tuples
        Self::combine_inputs(&per_param_candidates)
    }

    fn generate_single_param_edge_cases(ty: &Type) -> Vec<Value> {
        match ty {
            Type::Int => vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(-1),
                Value::Int(100),
                Value::Int(-100),
                Value::Int(7),
            ],

            Type::Bool => vec![Value::Bool(false), Value::Bool(true)],

            Type::List(elem_ty) => {
                match elem_ty.as_ref() {
                    Type::Int => vec![
                        // Empty list
                        Value::List(vec![]),
                        // Single element
                        Value::List(vec![Value::Int(0)]),
                        Value::List(vec![Value::Int(42)]),
                        Value::List(vec![Value::Int(-1)]),
                        // All identical (zero-entropy input)
                        Value::List(vec![
                            Value::Int(7),
                            Value::Int(7),
                            Value::Int(7),
                            Value::Int(7),
                        ]),
                        // Strictly ascending
                        Value::List(vec![
                            Value::Int(1),
                            Value::Int(2),
                            Value::Int(3),
                            Value::Int(4),
                            Value::Int(5),
                        ]),
                        // Strictly descending
                        Value::List(vec![
                            Value::Int(5),
                            Value::Int(4),
                            Value::Int(3),
                            Value::Int(2),
                            Value::Int(1),
                        ]),
                        // Mixed duplicate elements
                        Value::List(vec![
                            Value::Int(3),
                            Value::Int(1),
                            Value::Int(3),
                            Value::Int(2),
                            Value::Int(1),
                            Value::Int(2),
                        ]),
                        // Alternating signs
                        Value::List(vec![
                            Value::Int(10),
                            Value::Int(-10),
                            Value::Int(20),
                            Value::Int(-20),
                        ]),
                    ],
                    Type::Bool => vec![
                        Value::List(vec![]),
                        Value::List(vec![Value::Bool(true)]),
                        Value::List(vec![Value::Bool(false)]),
                        Value::List(vec![Value::Bool(true), Value::Bool(false), Value::Bool(true)]),
                        Value::List(vec![Value::Bool(false), Value::Bool(false), Value::Bool(false)]),
                    ],
                    _ => vec![Value::List(vec![])],
                }
            }

            Type::Matrix(elem_ty) => match elem_ty.as_ref() {
                Type::Int => vec![
                    Value::Matrix {
                        rows: 0,
                        cols: 0,
                        data: vec![],
                    },
                    Value::Matrix {
                        rows: 1,
                        cols: 1,
                        data: vec![Value::Int(0)],
                    },
                    Value::Matrix {
                        rows: 2,
                        cols: 2,
                        data: vec![
                            Value::Int(1),
                            Value::Int(0),
                            Value::Int(0),
                            Value::Int(1),
                        ],
                    },
                ],
                _ => vec![],
            },

            Type::Fn(_, _) => vec![],
        }
    }

    fn combine_inputs(candidates: &[Vec<Value>]) -> Vec<Vec<Value>> {
        if candidates.is_empty() {
            return vec![vec![]];
        }

        // If only 1 parameter, return directly
        if candidates.len() == 1 {
            return candidates[0].iter().map(|v| vec![v.clone()]).collect();
        }

        // For multiple parameters, form bounded pairwise combinations
        let mut results = Vec::new();
        let max_samples = 15;
        let p0 = &candidates[0];
        let p1 = &candidates[1];

        for (i, v0) in p0.iter().enumerate() {
            let v1 = &p1[i % p1.len()];
            let mut tuple = vec![v0.clone(), v1.clone()];
            for other_param in candidates.iter().skip(2) {
                tuple.push(other_param[i % other_param.len()].clone());
            }
            results.push(tuple);
            if results.len() >= max_samples {
                break;
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_case_generation_for_list() {
        let params = vec![Type::list(Type::Int)];
        let cases = EdgeCaseGenerator::generate(&params);
        assert!(!cases.is_empty());
        // Verify empty list is included
        assert!(cases.iter().any(|c| c[0] == Value::List(vec![])));
        // Verify sorted list is included
        assert!(cases.iter().any(|c| matches!(&c[0], Value::List(items) if items.len() > 1)));
    }
}
