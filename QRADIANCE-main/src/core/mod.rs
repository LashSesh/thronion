//! Core-Module des QRIK-Systems
//!
//! Enthält fundamentale Strukturen:
//! - DTL: Dynamic Tripolar Logic
//! - Hilbert: 13-dimensionaler Quantenzustandsraum
//! - Metatron: Graph-Topologie

pub mod dtl;
pub mod hilbert;
pub mod metatron;

pub use dtl::{DTLClass, DTLState};
pub use hilbert::{QuantumState, HILBERT_DIM};
pub use metatron::{MetatronGraph, NodeType, NUM_EDGES, NUM_NODES};
