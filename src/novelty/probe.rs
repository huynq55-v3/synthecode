use crate::core::{Type, Value};

/// Fixed, standardized probe vectors used to compute objective behavioral signatures across problems.
pub struct StandardProbeSet;

impl StandardProbeSet {
    /// Generates a deterministic probe vector suite tailored to the input types.
    pub fn get_probes(param_types: &[Type]) -> Vec<Vec<Value>> {
        if param_types.is_empty() {
            return vec![vec![]];
        }

        let per_param: Vec<Vec<Value>> = param_types
            .iter()
            .map(|ty| Self::probes_for_type(ty))
            .collect();

        // Pairwise Cartesian combinations
        let mut combinations = Vec::new();
        let max_probes = 16;
        let p0 = &per_param[0];

        for (i, v0) in p0.iter().enumerate() {
            let mut tuple = vec![v0.clone()];
            for other_param in per_param.iter().skip(1) {
                tuple.push(other_param[i % other_param.len()].clone());
            }
            combinations.push(tuple);
            if combinations.len() >= max_probes {
                break;
            }
        }

        combinations
    }

    fn probes_for_type(ty: &Type) -> Vec<Value> {
        match ty {
            Type::Int => vec![
                Value::Int(0),
                Value::Int(1),
                Value::Int(-1),
                Value::Int(5),
                Value::Int(10),
                Value::Int(-20),
                Value::Int(42),
                Value::Int(100),
            ],

            Type::Bool => vec![Value::Bool(false), Value::Bool(true)],

            Type::List(elem) => match elem.as_ref() {
                Type::Int => vec![
                    Value::List(vec![]),
                    Value::List(vec![Value::Int(0)]),
                    Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
                    Value::List(vec![Value::Int(3), Value::Int(2), Value::Int(1)]),
                    Value::List(vec![Value::Int(5), Value::Int(5), Value::Int(5)]),
                    Value::List(vec![Value::Int(-5), Value::Int(10), Value::Int(-15), Value::Int(20)]),
                    Value::List(vec![Value::Int(1), Value::Int(3), Value::Int(2), Value::Int(4)]),
                    Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30), Value::Int(40), Value::Int(50)]),
                ],
                Type::Bool => vec![
                    Value::List(vec![]),
                    Value::List(vec![Value::Bool(true)]),
                    Value::List(vec![Value::Bool(false)]),
                    Value::List(vec![Value::Bool(true), Value::Bool(false)]),
                ],
                _ => vec![Value::List(vec![])],
            },

            Type::Matrix(_) => vec![
                Value::Matrix { rows: 0, cols: 0, data: vec![] },
                Value::Matrix { rows: 2, cols: 2, data: vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)] },
            ],

            Type::Fn(_, _) => vec![],
        }
    }
}
