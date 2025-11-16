//! Resonanz-Module
//!
//! Implementiert resonanzbasiertes Routing und adaptive Filterung:
//! - Kuramoto: Synchronisationsnetzwerk
//! - Absorber: Resonant Absorber Layer (RAL)
//! - Spectrum: FFT-basiertes Spektral-Fingerprinting

pub mod absorber;
pub mod kuramoto;
pub mod spectrum;

pub use absorber::{AbsorberStats, ResonantAbsorber};
pub use kuramoto::KuramotoNetwork;
pub use spectrum::{SpectralFingerprint, TrafficType};
