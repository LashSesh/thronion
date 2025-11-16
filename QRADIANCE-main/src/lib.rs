//! # QRIK - Quantum-Resonant Invariant Kernel
//!
//! Production-fähige Implementierung eines klassisch simulierbaren
//! Quantenresonanz-Systems für cybernetic applications.
//!
//! ## Architektur
//!
//! QRIK fusioniert mehrere Komponenten:
//!
//! - **Core**: DTL (Dynamic Tripolar Logic), 13D-Hilbert-Raum, Metatron-Graph
//! - **Operators**: Hamilton, Ω₅-Familie, Nullpunkt-Operator
//! - **Resonance**: Kuramoto-Netzwerk, Resonant Absorber Layer
//! - **Mandorla**: Eigenstate-Fusion, Temporal Information Crystals
//! - **Delta**: Unified Kernel, Evolutionäre Optimierung
//!
//! ## Quick Start
//!
//! ```rust
//! use qrik::prelude::*;
//!
//! // Erstelle QRIK-System mit Standardparametern
//! let mut kernel = DeltaKernel::default();
//!
//! // Evolve System
//! for _ in 0..100 {
//!     kernel.evolve(0.01);
//! }
//!
//! // Prüfe Stabilität
//! assert!(kernel.coherence_gradient() < 1.0);
//! ```
//!
//! ## Features
//!
//! - **Tripolar Logic**: 58.5% höhere Informationskapazität als binär
//! - **Resonanzbasiertes Routing**: Addressfreie Kommunikation
//! - **DDoS-Resistenz**: Adaptive spektrale Filterung (>95% Absorption)
//! - **Topologische Guards**: Selbstheilende Invarianten
//! - **Keine Quanten-Hardware**: Vollständig klassisch implementiert
//!
//! ## Anwendungen
//!
//! - Cyber-Defense (DDoS-Mitigation)
//! - Kognitive Swarm-Systeme
//! - Semantische Ledgers
//! - Self-Healing Infrastructure

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod core;
pub mod delta;
pub mod mandorla;
pub mod operators;
pub mod resonance;
pub mod utils;

/// Prelude für häufig verwendete Typen
pub mod prelude {
    pub use crate::core::{DTLClass, DTLState, MetatronGraph, QuantumState, HILBERT_DIM};
    pub use crate::delta::{DeltaKernel, EvolutionaryOptimizer, QRIKParams};
    pub use crate::mandorla::{InformationBlock, LivingCrystal, MandorlaOperator, TemporalCrystal};
    pub use crate::operators::{HamiltonOperator, NullpointOperator};
    pub use crate::resonance::{KuramotoNetwork, ResonantAbsorber, SpectralFingerprint};
}

/// QRIK Library Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// QRIK Information Capacity (Bit/Symbol)
///
/// log₂(3) ≈ 1.585 Bit/Symbol
pub const TRIPOLAR_CAPACITY: f64 = 1.585;

/// Informationskapazitäts-Vorteil gegenüber Binär
///
/// (1.585 - 1.0) / 1.0 ≈ 58.5%
pub const BINARY_ADVANTAGE: f64 = 0.585;

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_information_capacity() {
        let capacity = 3.0_f64.log2();
        assert_abs_diff_eq!(TRIPOLAR_CAPACITY, capacity, epsilon = 0.001);
    }

    #[test]
    fn test_binary_advantage() {
        let advantage = (TRIPOLAR_CAPACITY - 1.0) / 1.0;
        assert_abs_diff_eq!(BINARY_ADVANTAGE, advantage, epsilon = 0.001);
    }
}
