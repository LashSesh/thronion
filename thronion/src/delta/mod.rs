//! Delta-Module
//!
//! Implementiert den vereinheitlichten Delta-Kernel und Optimierung:
//! - Kernel: Fusioniert alle QRIK-Komponenten
//! - Optimizer: Evolutionäre Parameter-Optimierung

pub mod kernel;
pub mod optimizer;

pub use kernel::{DeltaKernel, QRIKParams};
pub use optimizer::{EvolutionaryOptimizer, OptimizationResult};
