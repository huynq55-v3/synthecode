use std::fmt;

/// Formal Type System for SyntheCode DSL.
///
/// Designed to guarantee strong typing, static type soundness during
/// grammar expansion, and deterministic type inference.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Type {
    /// 64-bit signed integer.
    Int,
    /// Boolean logic type.
    Bool,
    /// Homogeneous sequentially ordered list.
    List(Box<Type>),
    /// Homogeneous 2D matrix with fixed row/col dimensions.
    Matrix(Box<Type>),
    /// First-class function type for higher-order combinators (e.g. Map, Filter, Fold).
    Fn(Vec<Type>, Box<Type>),
}

impl Type {
    /// Convenience constructor for Int.
    #[inline]
    pub const fn int() -> Self {
        Type::Int
    }

    /// Convenience constructor for Bool.
    #[inline]
    pub const fn bool() -> Self {
        Type::Bool
    }

    /// Convenience constructor for List of given element type.
    #[inline]
    pub fn list(elem: Type) -> Self {
        Type::List(Box::new(elem))
    }

    /// Convenience constructor for Matrix of given element type.
    #[inline]
    pub fn matrix(elem: Type) -> Self {
        Type::Matrix(Box::new(elem))
    }

    /// Convenience constructor for Function type.
    #[inline]
    pub fn func(param_types: Vec<Type>, return_type: Type) -> Self {
        Type::Fn(param_types, Box::new(return_type))
    }

    /// Checks if this type is a ground/base primitive type (Int or Bool).
    #[inline]
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Bool)
    }

    /// Checks if this is a collection type (List or Matrix).
    #[inline]
    pub fn is_collection(&self) -> bool {
        matches!(self, Type::List(_) | Type::Matrix(_))
    }

    /// Checks if this is a higher-order function type.
    #[inline]
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Fn(_, _))
    }

    /// Returns the element type if this is a collection type.
    pub fn element_type(&self) -> Option<&Type> {
        match self {
            Type::List(inner) | Type::Matrix(inner) => Some(inner.as_ref()),
            _ => None,
        }
    }

    /// Returns structural depth of the type (e.g. Int = 1, List<Int> = 2, List<List<Int>> = 3).
    pub fn depth(&self) -> usize {
        match self {
            Type::Int | Type::Bool => 1,
            Type::List(inner) | Type::Matrix(inner) => 1 + inner.depth(),
            Type::Fn(params, ret) => {
                let max_param = params.iter().map(|p| p.depth()).max().unwrap_or(0);
                1 + max_param.max(ret.depth())
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::List(elem) => write!(f, "List<{}>", elem),
            Type::Matrix(elem) => write!(f, "Matrix<{}>", elem),
            Type::Fn(params, ret) => {
                write!(f, "(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_constructors_and_display() {
        let t_int = Type::int();
        assert_eq!(t_int.to_string(), "Int");
        assert!(t_int.is_primitive());

        let t_list = Type::list(Type::int());
        assert_eq!(t_list.to_string(), "List<Int>");
        assert_eq!(t_list.element_type(), Some(&Type::Int));
        assert_eq!(t_list.depth(), 2);

        let t_matrix = Type::matrix(Type::bool());
        assert_eq!(t_matrix.to_string(), "Matrix<Bool>");
        assert_eq!(t_matrix.element_type(), Some(&Type::Bool));

        let t_fn = Type::func(vec![Type::int(), Type::int()], Type::bool());
        assert_eq!(t_fn.to_string(), "(Int, Int) -> Bool");
        assert!(t_fn.is_function());
    }
}
