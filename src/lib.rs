#![forbid(unsafe_code)]

pub mod analysis;
pub mod canonical;
pub mod core;
pub mod dsl;
pub mod generator;
pub mod novelty;
pub mod testgen;
pub mod vm;

pub use crate::analysis::{ComplexityAnalyzer, EntropyAnalyzer, RuntimeComplexity, StructuralComplexity};
pub use crate::canonical::{apply_rewrite_rule, TermRewriter};
pub use crate::core::{CoreError, Type, Value};
pub use crate::dsl::{BinaryOp, Expr, Grammar, SplitMix64, TypeChecker, TypeEnv, UnaryOp};
pub use crate::generator::{PipelineConfig, Problem, ProblemGenerator};
pub use crate::novelty::{BehavioralSignature, NoveltyArchive, StandardProbeSet};
pub use crate::testgen::{EdgeCaseGenerator, InputFuzzer, TestCase, TestEvaluationResult, TestSuite};
pub use crate::vm::{GasMeter, Interpreter, VmError};
