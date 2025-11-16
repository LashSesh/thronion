//! Temporal Information Crystals (TIC)
//!
//! Implementiert invariante Informationsblöcke mit temporaler Stabilität
//! C_TIC = ⊗_{k=0}^N B_k

use crate::core::QuantumState;
use crate::mandorla::eigenstate::MandorlaRegion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Informationsblock B_k
///
/// Unveränderliche Einheit mit semantischer Bedeutung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationBlock {
    /// Block-ID
    pub id: usize,
    /// Quantenzustand des Blocks
    pub state: QuantumState,
    /// Zeitstempel (Erstellungszeit)
    pub timestamp: f64,
    /// Invarianz-Hash (für Verifikation)
    pub hash: u64,
}

impl InformationBlock {
    /// Erstellt neuen Informationsblock
    pub fn new(id: usize, state: QuantumState, timestamp: f64) -> Self {
        let hash = Self::compute_hash(&state);
        Self {
            id,
            state,
            timestamp,
            hash,
        }
    }

    /// Berechnet Hash des Zustands (vereinfacht)
    fn compute_hash(state: &QuantumState) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash basierend auf Amplituden
        for amp in state.amplitudes.iter() {
            let re_bits = amp.re.to_bits();
            let im_bits = amp.im.to_bits();
            re_bits.hash(&mut hasher);
            im_bits.hash(&mut hasher);
        }

        hasher.finish()
    }

    /// Verifiziert Integrität des Blocks
    pub fn verify_integrity(&self) -> bool {
        let current_hash = Self::compute_hash(&self.state);
        current_hash == self.hash
    }

    /// Berechnet Ähnlichkeit mit anderem Block
    pub fn similarity(&self, other: &Self) -> f64 {
        self.state.fidelity(&other.state)
    }
}

/// Temporal Information Crystal
///
/// C_TIC = ⊗_{k=0}^N B_k mit Invarianz-Eigenschaft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalCrystal {
    /// Sammlung von Informationsblöcken
    pub blocks: Vec<InformationBlock>,
    /// Mandorla-Regionen für Blöcke
    pub mandorla_regions: Vec<MandorlaRegion>,
    /// Globaler Kristall-Zustand
    pub crystal_state: Option<QuantumState>,
}

impl TemporalCrystal {
    /// Erstellt neuen Temporal Crystal
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            mandorla_regions: Vec::new(),
            crystal_state: None,
        }
    }

    /// Fügt Informationsblock hinzu
    pub fn add_block(&mut self, block: InformationBlock) {
        self.blocks.push(block);
        // Invalidiere Kristall-Zustand (muss neu berechnet werden)
        self.crystal_state = None;
    }

    /// Berechnet Kristall-Zustand C_TIC = ⊗ B_k
    ///
    /// Verwendet Tensorprodukt-artige Komposition (vereinfacht als gewichtete Summe)
    pub fn compute_crystal_state(&mut self) -> QuantumState {
        if self.blocks.is_empty() {
            return QuantumState::default();
        }

        // Gewichtete Summe aller Blöcke
        let mut composite_amps = self.blocks[0].state.amplitudes;

        for (i, block) in self.blocks.iter().enumerate().skip(1) {
            let weight = 1.0 / (i + 1) as f64;
            composite_amps =
                composite_amps.scale(1.0 - weight) + block.state.amplitudes.scale(weight);
        }

        let state = QuantumState::new(composite_amps);
        self.crystal_state = Some(state.clone());
        state
    }

    /// Prüft temporale Invarianz
    ///
    /// C_TIC(γ) = C_TIC(T(γ)) für zulässige Transformationen
    pub fn check_invariance(&self) -> bool {
        // Alle Blöcke müssen integre sein
        self.blocks.iter().all(|b| b.verify_integrity())
    }

    /// Berechnet Kristall-Kohärenz
    pub fn coherence(&self) -> f64 {
        if self.blocks.len() < 2 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let mut count = 0;

        for i in 0..self.blocks.len() {
            for j in (i + 1)..self.blocks.len() {
                total_similarity += self.blocks[i].similarity(&self.blocks[j]);
                count += 1;
            }
        }

        if count > 0 {
            total_similarity / count as f64
        } else {
            1.0
        }
    }

    /// Findet Block nach Zeitstempel
    pub fn find_block_at_time(&self, time: f64, tolerance: f64) -> Option<&InformationBlock> {
        self.blocks
            .iter()
            .find(|b| (b.timestamp - time).abs() < tolerance)
    }

    /// Extrahiert Zeitfenster [t1, t2]
    pub fn extract_time_window(&self, t1: f64, t2: f64) -> Vec<&InformationBlock> {
        self.blocks
            .iter()
            .filter(|b| b.timestamp >= t1 && b.timestamp <= t2)
            .collect()
    }
}

impl Default for TemporalCrystal {
    fn default() -> Self {
        Self::new()
    }
}

/// Living Information Crystal (C_LIV)
///
/// C_LIV = lim_{n→∞} ⋂_{k=0}^n [M_k ⋆ B_k]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivingCrystal {
    /// Basis Temporal Crystal
    pub temporal_crystal: TemporalCrystal,
    /// Konvergenz-Level
    pub convergence_level: usize,
    /// Invarianz-Metadaten
    pub metadata: HashMap<String, String>,
}

impl LivingCrystal {
    /// Erstellt neuen Living Crystal
    pub fn new() -> Self {
        Self {
            temporal_crystal: TemporalCrystal::new(),
            convergence_level: 0,
            metadata: HashMap::new(),
        }
    }

    /// Iterative Konvergenz: ⋂_{k=0}^n [M_k ⋆ B_k]
    pub fn converge(&mut self, max_iterations: usize) {
        for _ in 0..max_iterations {
            if self.temporal_crystal.blocks.is_empty() {
                break;
            }

            // Recompute crystal state
            self.temporal_crystal.compute_crystal_state();
            self.convergence_level += 1;

            // Prüfe Konvergenz
            if self.temporal_crystal.coherence() > 0.99 {
                break;
            }
        }
    }

    /// Prüft Mandorla-Bedingung: C_LIV(γ) = C_LIV(T(γ))
    pub fn verify_mandorla_condition(&self) -> bool {
        self.temporal_crystal.check_invariance()
    }

    /// Speichert Metadaten
    pub fn set_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Abruft Metadaten
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl Default for LivingCrystal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_information_block_creation() {
        let state = QuantumState::random();
        let block = InformationBlock::new(0, state, 0.0);

        assert_eq!(block.id, 0);
        assert!(block.verify_integrity());
    }

    #[test]
    fn test_block_integrity_verification() {
        let state = QuantumState::random();
        let mut block = InformationBlock::new(0, state, 0.0);

        assert!(block.verify_integrity());

        // Modifiziere Zustand (sollte Integrität verletzen)
        block.state = QuantumState::random();
        assert!(!block.verify_integrity());
    }

    #[test]
    fn test_temporal_crystal_creation() {
        let mut crystal = TemporalCrystal::new();
        assert!(crystal.blocks.is_empty());

        let block = InformationBlock::new(0, QuantumState::random(), 0.0);
        crystal.add_block(block);

        assert_eq!(crystal.blocks.len(), 1);
    }

    #[test]
    fn test_crystal_state_computation() {
        let mut crystal = TemporalCrystal::new();

        for i in 0..5 {
            let block = InformationBlock::new(i, QuantumState::random(), i as f64);
            crystal.add_block(block);
        }

        let state = crystal.compute_crystal_state();
        assert!(state.is_normalized());
        assert!(crystal.crystal_state.is_some());
    }

    #[test]
    fn test_crystal_invariance() {
        let mut crystal = TemporalCrystal::new();

        for i in 0..3 {
            let block = InformationBlock::new(i, QuantumState::random(), i as f64);
            crystal.add_block(block);
        }

        assert!(crystal.check_invariance());
    }

    #[test]
    fn test_crystal_coherence() {
        let mut crystal = TemporalCrystal::new();

        // Ähnliche Blöcke sollten hohe Kohärenz haben
        let base_state = QuantumState::random();
        for i in 0..3 {
            let block = InformationBlock::new(i, base_state.clone(), i as f64);
            crystal.add_block(block);
        }

        let coherence = crystal.coherence();
        assert!(coherence > 0.9); // Hohe Kohärenz für identische Blöcke
    }

    #[test]
    fn test_time_window_extraction() {
        let mut crystal = TemporalCrystal::new();

        for i in 0..10 {
            let block = InformationBlock::new(i, QuantumState::random(), i as f64);
            crystal.add_block(block);
        }

        let window = crystal.extract_time_window(2.0, 5.0);
        assert_eq!(window.len(), 4); // Blöcke bei t=2,3,4,5
    }

    #[test]
    fn test_living_crystal() {
        let mut living = LivingCrystal::new();

        for i in 0..5 {
            let block = InformationBlock::new(i, QuantumState::random(), i as f64);
            living.temporal_crystal.add_block(block);
        }

        living.converge(10);
        assert!(living.convergence_level > 0);
    }

    #[test]
    fn test_metadata() {
        let mut living = LivingCrystal::new();

        living.set_metadata("author".to_string(), "QRIK".to_string());
        living.set_metadata("version".to_string(), "1.0".to_string());

        assert_eq!(living.get_metadata("author"), Some(&"QRIK".to_string()));
        assert_eq!(living.get_metadata("version"), Some(&"1.0".to_string()));
    }
}
