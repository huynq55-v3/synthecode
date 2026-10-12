use std::fmt;
use crate::core::{Type, Value};

/// Binary arithmetic, logic, and comparison operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Neq => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Lte => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Gte => write!(f, ">="),
            BinaryOp::And => write!(f, "&&"),
            BinaryOp::Or => write!(f, "||"),
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UnaryOp {
    Neg,
    Not,
    Abs,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Abs => write!(f, "abs"),
        }
    }
}

/// Abstract Syntax Tree (AST) node for the SyntheCode DSL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Expr {
    /// Literal constant value.
    Literal(Value),

    /// Variable access by de Bruijn / positional index.
    Var(usize),

    /// Unary operation (e.g. -x, !b, abs(x)).
    Unary(UnaryOp, Box<Expr>),

    /// Binary operation (e.g. a + b, a == b).
    Binary(BinaryOp, Box<Expr>, Box<Expr>),

    /// Branching ternary expression: `if cond then true_expr else false_expr`.
    IfThenElse {
        cond: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    // --- List Primitives ---
    /// List length: `len(list)`.
    Length(Box<Expr>),

    /// List head element: `head(list)`.
    Head(Box<Expr>),

    /// List tail (all elements except head): `tail(list)`.
    Tail(Box<Expr>),

    /// Reverse list order: `reverse(list)`.
    Reverse(Box<Expr>),

    /// Sort list in ascending order: `sort(list)`.
    Sort(Box<Expr>),

    /// Deduplicate consecutive/all identical elements: `dedup(list)`.
    Dedup(Box<Expr>),

    /// Take first N elements from list: `take(n, list)`.
    Take(Box<Expr>, Box<Expr>),

    /// Drop first N elements from list: `drop(n, list)`.
    Drop(Box<Expr>, Box<Expr>),

    /// Concatenate two lists: `concat(list1, list2)`.
    Concat(Box<Expr>, Box<Expr>),

    /// Minimum element of list: `min(list)`.
    Min(Box<Expr>),

    /// Maximum element of list: `max(list)`.
    Max(Box<Expr>),

    /// Sum of all numeric elements: `sum(list)`.
    Sum(Box<Expr>),

    // --- Higher-Order Functional Combinators ---
    /// Map a function over list elements: `map(list, \x -> f(x))`.
    Map {
        list: Box<Expr>,
        func: Box<Expr>,
    },

    /// Filter list by predicate: `filter(list, \x -> p(x))`.
    Filter {
        list: Box<Expr>,
        pred: Box<Expr>,
    },

    /// Fold accumulator over list: `fold(list, init, \(acc, elem) -> f(acc, elem))`.
    Fold {
        list: Box<Expr>,
        init: Box<Expr>,
        func: Box<Expr>,
    },

    /// Elementwise zip with binary function: `zip_with(list1, list2, \(a, b) -> f(a, b))`.
    ZipWith {
        list1: Box<Expr>,
        list2: Box<Expr>,
        func: Box<Expr>,
    },

    /// Anonymous lambda function abstraction: `\(x0, x1, ...) -> body`.
    Lambda {
        param_types: Vec<Type>,
        body: Box<Expr>,
    },
}

impl Expr {
    /// Calculate total AST node count (used as structural complexity metric).
    pub fn node_count(&self) -> usize {
        match self {
            Expr::Literal(_) | Expr::Var(_) => 1,
            Expr::Unary(_, inner)
            | Expr::Length(inner)
            | Expr::Head(inner)
            | Expr::Tail(inner)
            | Expr::Reverse(inner)
            | Expr::Sort(inner)
            | Expr::Dedup(inner)
            | Expr::Min(inner)
            | Expr::Max(inner)
            | Expr::Sum(inner)
            | Expr::Lambda { body: inner, .. } => 1 + inner.node_count(),
            Expr::Binary(_, left, right)
            | Expr::Take(left, right)
            | Expr::Drop(left, right)
            | Expr::Concat(left, right)
            | Expr::Map { list: left, func: right }
            | Expr::Filter { list: left, pred: right } => {
                1 + left.node_count() + right.node_count()
            }
            Expr::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => 1 + cond.node_count() + then_branch.node_count() + else_branch.node_count(),
            Expr::Fold { list, init, func } => {
                1 + list.node_count() + init.node_count() + func.node_count()
            }
            Expr::ZipWith { list1, list2, func } => {
                1 + list1.node_count() + list2.node_count() + func.node_count()
            }
        }
    }

    /// Calculate tree depth.
    pub fn depth(&self) -> usize {
        match self {
            Expr::Literal(_) | Expr::Var(_) => 1,
            Expr::Unary(_, inner)
            | Expr::Length(inner)
            | Expr::Head(inner)
            | Expr::Tail(inner)
            | Expr::Reverse(inner)
            | Expr::Sort(inner)
            | Expr::Dedup(inner)
            | Expr::Min(inner)
            | Expr::Max(inner)
            | Expr::Sum(inner)
            | Expr::Lambda { body: inner, .. } => 1 + inner.depth(),
            Expr::Binary(_, left, right)
            | Expr::Take(left, right)
            | Expr::Drop(left, right)
            | Expr::Concat(left, right)
            | Expr::Map { list: left, func: right }
            | Expr::Filter { list: left, pred: right } => {
                1 + left.depth().max(right.depth())
            }
            Expr::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => 1 + cond.depth().max(then_branch.depth()).max(else_branch.depth()),
            Expr::Fold { list, init, func } => {
                1 + list.depth().max(init.depth()).max(func.depth())
            }
            Expr::ZipWith { list1, list2, func } => {
                1 + list1.depth().max(list2.depth()).max(func.depth())
            }
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Literal(val) => write!(f, "{}", val),
            Expr::Var(idx) => write!(f, "x{}", idx),
            Expr::Unary(op, inner) => write!(f, "{}({})", op, inner),
            Expr::Binary(op, left, right) => write!(f, "({} {} {})", left, op, right),
            Expr::IfThenElse {
                cond,
                then_branch,
                else_branch,
            } => write!(f, "if {} then {} else {}", cond, then_branch, else_branch),
            Expr::Length(e) => write!(f, "len({})", e),
            Expr::Head(e) => write!(f, "head({})", e),
            Expr::Tail(e) => write!(f, "tail({})", e),
            Expr::Reverse(e) => write!(f, "reverse({})", e),
            Expr::Sort(e) => write!(f, "sort({})", e),
            Expr::Dedup(e) => write!(f, "dedup({})", e),
            Expr::Take(n, l) => write!(f, "take({}, {})", n, l),
            Expr::Drop(n, l) => write!(f, "drop({}, {})", n, l),
            Expr::Concat(a, b) => write!(f, "concat({}, {})", a, b),
            Expr::Min(e) => write!(f, "min({})", e),
            Expr::Max(e) => write!(f, "max({})", e),
            Expr::Sum(e) => write!(f, "sum({})", e),
            Expr::Map { list, func } => write!(f, "map({}, {})", list, func),
            Expr::Filter { list, pred } => write!(f, "filter({}, {})", list, pred),
            Expr::Fold { list, init, func } => write!(f, "fold({}, {}, {})", list, init, func),
            Expr::ZipWith { list1, list2, func } => write!(f, "zip_with({}, {}, {})", list1, list2, func),
            Expr::Lambda { param_types, body } => {
                write!(f, "λ(")?;
                for (i, t) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "x{}: {}", i, t)?;
                }
                write!(f, ") -> {}", body)
            }
        }
    }
}
