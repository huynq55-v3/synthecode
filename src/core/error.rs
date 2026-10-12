use thiserror::Error;
use super::types::Type;

/// Core error types for the formal type system and dynamic value evaluation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum CoreError {
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        expected: String,
        actual: String,
    },

    #[error("Heterogeneous elements found in collection: expected element type {expected:?}, but element at index {index} has type {actual:?}")]
    HeterogeneousCollection {
        index: usize,
        expected: Type,
        actual: Type,
    },

    #[error("Invalid matrix dimensions: rows={rows}, cols={cols}, but data length is {data_len}")]
    InvalidMatrixDimensions {
        rows: usize,
        cols: usize,
        data_len: usize,
    },

    #[error("Empty collection cannot infer element type without type annotation")]
    IndeterminateEmptyCollection,

    #[error("Type conversion failed: expected {expected}, found {actual}")]
    ValueConversion {
        expected: &'static str,
        actual: String,
    },

    #[error("Index out of bounds: index {index} >= length {len}")]
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Arithmetic overflow in operation: {operation}")]
    IntegerOverflow {
        operation: &'static str,
    },
}
