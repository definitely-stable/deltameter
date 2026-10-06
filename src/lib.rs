#![forbid(unsafe_code)]

pub mod energy;

pub use energy::{
    Coverage, EnergyConfig, EnergyDeltaMeter, EnergyError, EnergyEstimate, EnergyProfile,
    EnergyRowHash, FailureTarget, RelativeError,
};
