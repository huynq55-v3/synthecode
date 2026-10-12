use crate::core::{Type, Value};
use crate::dsl::SplitMix64;

/// Property-based pseudorandom input fuzzer for diverse input space exploration.
pub struct InputFuzzer;

impl InputFuzzer {
    /// Generates `count` pseudorandom tuples of inputs matching the given parameter types.
    pub fn generate(
        param_types: &[Type],
        count: usize,
        rng: &mut SplitMix64,
    ) -> Vec<Vec<Value>> {
        let mut suite = Vec::with_capacity(count);

        for _ in 0..count {
            let mut tuple = Vec::with_capacity(param_types.len());
            for ty in param_types {
                tuple.push(Self::fuzz_value(ty, rng));
            }
            suite.push(tuple);
        }

        suite
    }

    /// Fuzzes a single value conforming strictly to type `ty`.
    pub fn fuzz_value(ty: &Type, rng: &mut SplitMix64) -> Value {
        match ty {
            Type::Int => {
                // Diverse distribution: small positive, negatives, and medium integers
                let mode = rng.next_usize(3);
                match mode {
                    0 => Value::Int(rng.next_i64_range(0, 30)),
                    1 => Value::Int(rng.next_i64_range(-30, 0)),
                    _ => Value::Int(rng.next_i64_range(-100, 100)),
                }
            }

            Type::Bool => Value::Bool(rng.next_bool()),

            Type::List(elem) => {
                let len = rng.next_usize(12) + 1; // 1 to 12 elements
                let mut items = Vec::with_capacity(len);
                for _ in 0..len {
                    items.push(Self::fuzz_value(elem, rng));
                }
                Value::List(items)
            }

            Type::Matrix(elem) => {
                let rows = rng.next_usize(3) + 1;
                let cols = rng.next_usize(3) + 1;
                let total = rows * cols;
                let mut data = Vec::with_capacity(total);
                for _ in 0..total {
                    data.push(Self::fuzz_value(elem, rng));
                }
                Value::Matrix { rows, cols, data }
            }

            Type::Fn(_, _) => Value::Int(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzer_generates_well_typed_values() {
        let mut rng = SplitMix64::new(12345);
        let params = vec![Type::list(Type::Int), Type::Int];
        let fuzzed = InputFuzzer::generate(&params, 20, &mut rng);

        assert_eq!(fuzzed.len(), 20);
        for tuple in fuzzed {
            assert_eq!(tuple.len(), 2);
            assert_eq!(tuple[0].infer_type().unwrap(), Type::list(Type::Int));
            assert_eq!(tuple[1].infer_type().unwrap(), Type::Int);
        }
    }
}
