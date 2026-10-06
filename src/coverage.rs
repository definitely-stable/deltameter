/// Evidence attached to an estimate.
///
/// `Proven` is reserved for a finite-sample theorem matching the concrete
/// implementation and configuration.
///
/// `Asymptotic` is weaker. Its relative standard error is model metadata from
/// an asymptotic analysis and is not a finite-sample failure-probability bound.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Coverage {
    Proven {
        failure_probability_upper_bound: f64,
    },
    Asymptotic {
        relative_standard_error: f64,
    },
}
