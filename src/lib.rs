#![forbid(unsafe_code)]

pub mod canonical;
pub mod core;
pub mod dsl;
pub mod vm;

pub use crate::canonical::{apply_rewrite_rule, TermRewriter};
pub use crate::core::{CoreError, Type, Value};
pub use crate::dsl::{BinaryOp, Expr, Grammar, SplitMix64, TypeChecker, TypeEnv, UnaryOp};
pub use crate::vm::{GasMeter, Interpreter, VmError};
