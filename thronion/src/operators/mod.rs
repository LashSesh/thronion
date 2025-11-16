//! Operator-Module
//!
//! Implementiert die verschiedenen Operatoren des QRIK-Systems:
//! - Hamilton: Zeitevolution auf Metatron-Graph
//! - Omega5: 5D-Operatorfamilie
//! - Nullpoint: K_N Reset-Operator

pub mod hamiltonian;
pub mod nullpoint;
pub mod omega5;

pub use hamiltonian::{EigenSpectrum, HamiltonOperator};
pub use nullpoint::{GuardStatus, GuardValidator, GuardViolation, NullpointOperator};
pub use omega5::{
    DampingOperator, QuaternionRotation, ScalingOperator, State5D, TopologicalGuards,
    WormholeOperator,
};
