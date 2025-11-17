//! Mandorla-Module
//!
//! Implementiert Eigenstate-Fusion und Temporal Information Crystals:
//! - Eigenstate: Mandorla-Schnittmengen und Fusionsoperatoren
//! - TIC: Temporal Information Crystals (invariante Blöcke)

pub mod eigenstate;
pub mod tic;

pub use eigenstate::{MandorlaConvolution, MandorlaOperator, MandorlaRegion};
pub use tic::{InformationBlock, LivingCrystal, TemporalCrystal};
