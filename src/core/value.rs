use std::fmt;
use super::error::CoreError;
use super::types::Type;

/// Runtime Dynamic Value representation with strict invariant verification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    /// 64-bit signed integer.
    Int(i64),
    /// Boolean value.
    Bool(bool),
    /// Homogeneous sequential list of values.
    List(Vec<Value>),
    /// Homogeneous 2D row-major matrix of values.
    Matrix {
        rows: usize,
        cols: usize,
        data: Vec<Value>,
    },
}

impl Value {
    /// Invariant check and static-like type inference for a runtime value.
    pub fn infer_type(&self) -> Result<Type, CoreError> {
        match self {
            Value::Int(_) => Ok(Type::Int),
            Value::Bool(_) => Ok(Type::Bool),
            Value::List(items) => {
                if items.is_empty() {
                    return Err(CoreError::IndeterminateEmptyCollection);
                }
                let first_type = items[0].infer_type()?;
                for (idx, item) in items.iter().enumerate().skip(1) {
                    let item_type = item.infer_type()?;
                    if item_type != first_type {
                        return Err(CoreError::HeterogeneousCollection {
                            index: idx,
                            expected: first_type,
                            actual: item_type,
                        });
                    }
                }
                Ok(Type::list(first_type))
            }
            Value::Matrix { rows, cols, data } => {
                if data.len() != rows * cols {
                    return Err(CoreError::InvalidMatrixDimensions {
                        rows: *rows,
                        cols: *cols,
                        data_len: data.len(),
                    });
                }
                if data.is_empty() {
                    return Err(CoreError::IndeterminateEmptyCollection);
                }
                let first_type = data[0].infer_type()?;
                for (idx, item) in data.iter().enumerate().skip(1) {
                    let item_type = item.infer_type()?;
                    if item_type != first_type {
                        return Err(CoreError::HeterogeneousCollection {
                            index: idx,
                            expected: first_type,
                            actual: item_type,
                        });
                    }
                }
                Ok(Type::matrix(first_type))
            }
        }
    }

    /// Verifies all internal structural and type invariants.
    pub fn validate_invariants(&self) -> Result<(), CoreError> {
        match self {
            Value::Int(_) | Value::Bool(_) => Ok(()),
            Value::List(items) => {
                if items.is_empty() {
                    return Ok(());
                }
                let first_type = items[0].infer_type()?;
                for (idx, item) in items.iter().enumerate().skip(1) {
                    let item_type = item.infer_type()?;
                    if item_type != first_type {
                        return Err(CoreError::HeterogeneousCollection {
                            index: idx,
                            expected: first_type,
                            actual: item_type,
                        });
                    }
                    item.validate_invariants()?;
                }
                Ok(())
            }
            Value::Matrix { rows, cols, data } => {
                if data.len() != rows * cols {
                    return Err(CoreError::InvalidMatrixDimensions {
                        rows: *rows,
                        cols: *cols,
                        data_len: data.len(),
                    });
                }
                if data.is_empty() {
                    return Ok(());
                }
                let first_type = data[0].infer_type()?;
                for (idx, item) in data.iter().enumerate().skip(1) {
                    let item_type = item.infer_type()?;
                    if item_type != first_type {
                        return Err(CoreError::HeterogeneousCollection {
                            index: idx,
                            expected: first_type,
                            actual: item_type,
                        });
                    }
                    item.validate_invariants()?;
                }
                Ok(())
            }
        }
    }

    /// Safely unwrap as integer.
    pub fn as_int(&self) -> Result<i64, CoreError> {
        match self {
            Value::Int(v) => Ok(*v),
            other => Err(CoreError::ValueConversion {
                expected: "Int",
                actual: other.to_string(),
            }),
        }
    }

    /// Safely unwrap as bool.
    pub fn as_bool(&self) -> Result<bool, CoreError> {
        match self {
            Value::Bool(v) => Ok(*v),
            other => Err(CoreError::ValueConversion {
                expected: "Bool",
                actual: other.to_string(),
            }),
        }
    }

    /// Safely unwrap as slice of values.
    pub fn as_list(&self) -> Result<&[Value], CoreError> {
        match self {
            Value::List(v) => Ok(v.as_slice()),
            other => Err(CoreError::ValueConversion {
                expected: "List",
                actual: other.to_string(),
            }),
        }
    }

    /// Safely unwrap as matrix reference.
    pub fn as_matrix(&self) -> Result<(usize, usize, &[Value]), CoreError> {
        match self {
            Value::Matrix { rows, cols, data } => Ok((*rows, *cols, data.as_slice())),
            other => Err(CoreError::ValueConversion {
                expected: "Matrix",
                actual: other.to_string(),
            }),
        }
    }

    /// Calculates multi-modal continuous diagnostic distance (Anti-Plateau metric).
    /// Returns 0.0 for identical values, and a smooth positive distance otherwise.
    pub fn diagnostic_distance(&self, other: &Value) -> f64 {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                let diff = (a.saturating_sub(*b)).abs() as f64;
                // Normalized continuous curve: 1.0 - 1.0 / (1.0 + diff)
                diff / (1.0 + diff)
            }
            (Value::Bool(a), Value::Bool(b)) => {
                if a == b {
                    0.0
                } else {
                    1.0
                }
            }
            (Value::List(a), Value::List(b)) => {
                // Levenshtein-like distance on list elements
                let m = a.len();
                let n = b.len();
                if m == 0 && n == 0 {
                    return 0.0;
                }
                if m == 0 {
                    return 1.0;
                }
                if n == 0 {
                    return 1.0;
                }

                let mut dp = vec![vec![0.0f64; n + 1]; m + 1];
                for i in 0..=m {
                    dp[i][0] = i as f64;
                }
                for j in 0..=n {
                    dp[0][j] = j as f64;
                }

                for i in 1..=m {
                    for j in 1..=n {
                        let cost = a[i - 1].diagnostic_distance(&b[j - 1]);
                        let delete = dp[i - 1][j] + 1.0;
                        let insert = dp[i][j - 1] + 1.0;
                        let substitute = dp[i - 1][j - 1] + cost;
                        dp[i][j] = delete.min(insert).min(substitute);
                    }
                }

                let max_len = m.max(n) as f64;
                dp[m][n] / max_len
            }
            (
                Value::Matrix {
                    rows: r1,
                    cols: c1,
                    data: d1,
                },
                Value::Matrix {
                    rows: r2,
                    cols: c2,
                    data: d2,
                },
            ) => {
                if r1 != r2 || c1 != c2 {
                    return 1.0;
                }
                if d1.is_empty() {
                    return 0.0;
                }
                let total_dist: f64 = d1
                    .iter()
                    .zip(d2.iter())
                    .map(|(x, y)| x.diagnostic_distance(y))
                    .sum();
                total_dist / (d1.len() as f64)
            }
            _ => 1.0, // Maximum penalty for cross-type incompatibility
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Matrix { rows, cols, data } => {
                write!(f, "Mat{}x{}[", rows, cols)?;
                for r in 0..*rows {
                    if r > 0 {
                        write!(f, "; ")?;
                    }
                    for c in 0..*cols {
                        if c > 0 {
                            write!(f, ", ")?;
                        }
                        let idx = r * cols + c;
                        if let Some(val) = data.get(idx) {
                            write!(f, "{}", val)?;
                        }
                    }
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_inference_and_invariants() {
        let v_int = Value::Int(42);
        assert_eq!(v_int.infer_type().unwrap(), Type::Int);

        let v_list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(v_list.infer_type().unwrap(), Type::list(Type::Int));
        assert!(v_list.validate_invariants().is_ok());

        let v_bad_list = Value::List(vec![Value::Int(1), Value::Bool(true)]);
        assert!(v_bad_list.infer_type().is_err());
    }

    #[test]
    fn test_matrix_validation() {
        let mat = Value::Matrix {
            rows: 2,
            cols: 2,
            data: vec![Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4)],
        };
        assert_eq!(mat.infer_type().unwrap(), Type::matrix(Type::Int));

        let invalid_dim_mat = Value::Matrix {
            rows: 2,
            cols: 2,
            data: vec![Value::Int(1), Value::Int(2)],
        };
        assert!(invalid_dim_mat.validate_invariants().is_err());
    }

    #[test]
    fn test_diagnostic_distance() {
        let a = Value::Int(10);
        let b = Value::Int(10);
        let c = Value::Int(20);
        assert_eq!(a.diagnostic_distance(&b), 0.0);
        assert!(a.diagnostic_distance(&c) > 0.0);

        let list_a = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let list_b = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        let list_c = Value::List(vec![Value::Int(1), Value::Int(4), Value::Int(3)]);
        assert_eq!(list_a.diagnostic_distance(&list_b), 0.0);
        assert!(list_a.diagnostic_distance(&list_c) > 0.0);
    }
}
