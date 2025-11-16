//! Utility-Module
//!
//! Hilfsfunktionen für:
//! - Linear Algebra
//! - ODE-Integration

pub mod integration;
pub mod linalg;

pub use integration::{EulerForward, RungeKutta4};
pub use linalg::{
    frobenius_norm, is_hermitian, is_unitary, matrix_exp_hermitian_3x3, normalize_vector, trace,
};
