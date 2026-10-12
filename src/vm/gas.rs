use thiserror::Error;

/// VM execution errors including gas exhaustion and deterministic resource limits.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum VmError {
    #[error("Out of gas: limit {limit} exceeded (attempted to consume {attempted})")]
    OutOfGas {
        limit: u64,
        attempted: u64,
    },

    #[error("Call stack overflow: exceeded recursion limit of {limit} frames")]
    StackOverflow {
        limit: usize,
    },

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Argument count mismatch: expected {expected}, got {actual}")]
    ArgumentCountMismatch {
        expected: usize,
        actual: usize,
    },

    #[error("Core error: {0}")]
    Core(#[from] crate::core::CoreError),
}

/// Deterministic Gas Meter tracking compute budget and protecting against infinite loops/explosions.
#[derive(Debug, Clone)]
pub struct GasMeter {
    limit: u64,
    consumed: u64,
}

impl GasMeter {
    /// Create a new GasMeter with the specified instruction/resource limit.
    pub const fn new(limit: u64) -> Self {
        Self { limit, consumed: 0 }
    }

    /// Default gas limit for problem evaluation (e.g. 50,000 steps).
    pub const fn default_limit() -> Self {
        Self::new(50_000)
    }

    /// Attempt to consume `amount` units of gas.
    /// Returns `Err(VmError::OutOfGas)` if consumption would exceed the budget.
    #[inline]
    pub fn consume(&mut self, amount: u64) -> Result<(), VmError> {
        let new_total = self.consumed.saturating_add(amount);
        if new_total > self.limit {
            return Err(VmError::OutOfGas {
                limit: self.limit,
                attempted: new_total,
            });
        }
        self.consumed = new_total;
        Ok(())
    }

    /// Remaining available gas.
    #[inline]
    pub const fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Total gas consumed so far.
    #[inline]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Reset consumed gas counter back to 0.
    #[inline]
    pub fn reset(&mut self) {
        self.consumed = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_metering_limits() {
        let mut meter = GasMeter::new(100);
        assert_eq!(meter.remaining(), 100);

        assert!(meter.consume(60).is_ok());
        assert_eq!(meter.consumed(), 60);
        assert_eq!(meter.remaining(), 40);

        assert!(meter.consume(30).is_ok());
        assert_eq!(meter.remaining(), 10);

        // Exceeding limit must return OutOfGas error
        let err = meter.consume(20);
        assert!(matches!(err, Err(VmError::OutOfGas { limit: 100, attempted: 110 })));
    }
}
