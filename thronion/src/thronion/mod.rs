//! Thronion Fusion Layer
//!
//! This module implements the hybrid Gabriel-Mandorla fusion system that
//! combines classical machine learning with quantum-inspired algorithms.
//!
//! ## Components
//!
//! - **Gabriel Regions**: Classical clustering with adaptive learning
//! - **Mandorla Integration**: Quantum state intersection and fusion
//! - **Hybrid Scoring**: Combined classical distance + quantum fidelity
//! - **Thronion Kernel**: Unified decision engine
//!
//! ## Status
//!
//! Phase 3 (Fusion Layer) - Implementation started
//!
//! This module implements the core innovation of Thronion: combining
//! classical Gabriel Cell clustering (interpretable, fast) with quantum
//! Mandorla fusion (high-capacity, accurate) for optimal DDoS detection.

use crate::core::{QuantumState, HILBERT_DIM};
use crate::mandorla::MandorlaRegion;
use crate::tor::{CellTypeDistribution, TimingFeatures, TorCircuitMetadata};
use nalgebra::{SVector, Complex};
use ndarray::Array1;
use serde::{Deserialize, Serialize};

type Complex64 = Complex<f64>;

/// Classical traffic signature for Gabriel Cell clustering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicalSignature {
    /// Mean inter-arrival time (microseconds)
    pub mean_interval: f64,
    /// Standard deviation of intervals
    pub std_dev_interval: f64,
    /// Data cell ratio
    pub data_ratio: f64,
    /// Introduction cell ratio
    pub intro_ratio: f64,
    /// Total bytes transferred
    pub total_bytes: f64,
}

impl ClassicalSignature {
    /// Create from Tor circuit metadata
    pub fn from_metadata(metadata: &TorCircuitMetadata, timing: &TimingFeatures, dist: &CellTypeDistribution) -> Self {
        Self {
            mean_interval: timing.mean_interval,
            std_dev_interval: timing.std_dev_interval,
            data_ratio: dist.data_ratio,
            intro_ratio: dist.intro_ratio,
            total_bytes: metadata.total_bytes as f64,
        }
    }

    /// Convert to feature vector for distance calculations
    pub fn to_vector(&self) -> Array1<f64> {
        Array1::from(vec![
            self.mean_interval / 1000.0,      // Normalize to milliseconds
            self.std_dev_interval / 1000.0,   // Normalize to milliseconds
            self.data_ratio,
            self.intro_ratio,
            (self.total_bytes / 1024.0).ln(), // Log-scale bytes (KB)
        ])
    }
}

/// Hybrid Gabriel-Mandorla region combining classical and quantum representations
#[derive(Debug, Clone)]
pub struct GabrielRegion {
    /// Classical centroid (signature)
    pub classical_center: ClassicalSignature,
    /// Quantum state representation
    pub quantum_center: QuantumState,
    /// Mandorla region for quantum fusion
    pub mandorla: MandorlaRegion,
    /// Learning rate for adaptation
    pub learning_rate: f64,
    /// Number of samples seen
    pub sample_count: usize,
    /// Attack probability (0.0 = benign, 1.0 = attack)
    pub attack_probability: f64,
}

impl GabrielRegion {
    /// Create new hybrid region
    pub fn new(classical: ClassicalSignature, quantum: QuantumState, learning_rate: f64) -> Self {
        // Create Mandorla with self-intersection initially
        let mandorla = MandorlaRegion::new(quantum.clone(), quantum.clone(), 0.5);
        
        Self {
            classical_center: classical,
            quantum_center: quantum,
            mandorla,
            learning_rate,
            sample_count: 0,
            attack_probability: 0.5, // Neutral initially
        }
    }

    /// Compute hybrid resonance score
    /// R_hybrid = w_c × (1/(1 + d_euclidean)) + w_q × F_fidelity
    pub fn hybrid_resonance(&self, classical: &ClassicalSignature, quantum: &QuantumState) -> f64 {
        // Classical distance-based score (inverse distance)
        let c_vec = classical.to_vector();
        let center_vec = self.classical_center.to_vector();
        let euclidean_dist = (&c_vec - &center_vec).mapv(|x| x * x).sum().sqrt();
        let classical_score = 1.0 / (1.0 + euclidean_dist);

        // Quantum fidelity-based score
        let quantum_score = quantum.fidelity(&self.quantum_center);

        // Weighted combination (0.3 classical, 0.7 quantum as per architecture)
        const W_CLASSICAL: f64 = 0.3;
        const W_QUANTUM: f64 = 0.7;

        W_CLASSICAL * classical_score + W_QUANTUM * quantum_score
    }

    /// Update region with new sample (adaptive learning)
    pub fn update(&mut self, classical: ClassicalSignature, quantum: QuantumState, is_attack: bool) {
        self.sample_count += 1;
        
        // Update classical center (exponential moving average)
        let alpha = self.learning_rate;
        self.classical_center.mean_interval = 
            (1.0 - alpha) * self.classical_center.mean_interval + alpha * classical.mean_interval;
        self.classical_center.std_dev_interval = 
            (1.0 - alpha) * self.classical_center.std_dev_interval + alpha * classical.std_dev_interval;
        self.classical_center.data_ratio = 
            (1.0 - alpha) * self.classical_center.data_ratio + alpha * classical.data_ratio;
        self.classical_center.intro_ratio = 
            (1.0 - alpha) * self.classical_center.intro_ratio + alpha * classical.intro_ratio;
        self.classical_center.total_bytes = 
            (1.0 - alpha) * self.classical_center.total_bytes + alpha * classical.total_bytes;

        // Update quantum center (requires state blending - simplified version)
        // In full implementation, would use Mandorla fusion
        self.quantum_center = quantum;

        // Update attack probability
        let attack_indicator = if is_attack { 1.0 } else { 0.0 };
        self.attack_probability = 
            (1.0 - alpha) * self.attack_probability + alpha * attack_indicator;
    }

    /// Check if this region represents an attack pattern
    pub fn is_attack_region(&self) -> bool {
        self.attack_probability > 0.7
    }
}

/// Conversion utilities between classical and quantum representations
pub struct ConversionUtils;

impl ConversionUtils {
    /// Convert classical signature to quantum state
    /// Maps 5D classical features to 13D Hilbert space
    pub fn classical_to_quantum(signature: &ClassicalSignature) -> QuantumState {
        let vec = signature.to_vector();
        
        // Expand 5D to 13D with harmonic expansion
        let mut amplitudes = SVector::<Complex64, HILBERT_DIM>::zeros();
        
        // Direct mapping for first 5 dimensions (real amplitudes)
        for i in 0..5.min(vec.len()) {
            amplitudes[i] = Complex::new(vec[i], 0.0);
        }
        
        // Harmonic expansion for remaining dimensions
        for i in 5..HILBERT_DIM {
            let phase = i as f64 * std::f64::consts::PI / HILBERT_DIM as f64;
            amplitudes[i] = Complex::new(vec[i % 5] * phase.sin(), 0.0);
        }
        
        QuantumState::new(amplitudes)
    }

    /// Extract classical features from quantum state
    /// Projects 13D quantum state to 5D classical space
    pub fn quantum_to_classical(state: &QuantumState) -> ClassicalSignature {
        // Project to first 5 dimensions (use magnitude of complex amplitudes)
        let amps = &state.amplitudes;
        
        ClassicalSignature {
            mean_interval: amps[0].norm() * 1000.0, // Scale back to microseconds
            std_dev_interval: amps[1].norm() * 1000.0,
            data_ratio: amps[2].norm().min(1.0),
            intro_ratio: amps[3].norm().min(1.0),
            total_bytes: (amps[4].norm() * 1024.0).exp(), // Inverse log-scale
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classical_signature_creation() {
        let sig = ClassicalSignature {
            mean_interval: 100.0,
            std_dev_interval: 20.0,
            data_ratio: 0.8,
            intro_ratio: 0.1,
            total_bytes: 1024.0,
        };
        
        let vec = sig.to_vector();
        assert_eq!(vec.len(), 5);
        assert!(vec[2] >= 0.0 && vec[2] <= 1.0); // Ratio check
    }

    #[test]
    fn test_classical_to_quantum_conversion() {
        let sig = ClassicalSignature {
            mean_interval: 100.0,
            std_dev_interval: 20.0,
            data_ratio: 0.8,
            intro_ratio: 0.1,
            total_bytes: 1024.0,
        };
        
        let quantum = ConversionUtils::classical_to_quantum(&sig);
        assert!(quantum.is_normalized());
    }

    #[test]
    fn test_gabriel_region_creation() {
        let sig = ClassicalSignature {
            mean_interval: 100.0,
            std_dev_interval: 20.0,
            data_ratio: 0.8,
            intro_ratio: 0.1,
            total_bytes: 1024.0,
        };
        
        let quantum = ConversionUtils::classical_to_quantum(&sig);
        let region = GabrielRegion::new(sig, quantum, 0.1);
        
        assert_eq!(region.sample_count, 0);
        assert_eq!(region.learning_rate, 0.1);
    }

    #[test]
    fn test_hybrid_resonance() {
        let sig = ClassicalSignature {
            mean_interval: 100.0,
            std_dev_interval: 20.0,
            data_ratio: 0.8,
            intro_ratio: 0.1,
            total_bytes: 1024.0,
        };
        
        let quantum = ConversionUtils::classical_to_quantum(&sig);
        let region = GabrielRegion::new(sig.clone(), quantum.clone(), 0.1);
        
        // Self-resonance should be high
        let score = region.hybrid_resonance(&sig, &quantum);
        assert!(score > 0.7, "Self-resonance should be high, got {}", score);
    }

    #[test]
    fn test_region_update() {
        let sig = ClassicalSignature {
            mean_interval: 100.0,
            std_dev_interval: 20.0,
            data_ratio: 0.8,
            intro_ratio: 0.1,
            total_bytes: 1024.0,
        };
        
        let quantum = ConversionUtils::classical_to_quantum(&sig);
        let mut region = GabrielRegion::new(sig.clone(), quantum.clone(), 0.1);
        
        // Update with attack sample
        region.update(sig, quantum, true);
        assert_eq!(region.sample_count, 1);
        assert!(region.attack_probability > 0.5);
    }

    #[test]
    fn test_attack_detection() {
        let sig = ClassicalSignature {
            mean_interval: 100.0,
            std_dev_interval: 20.0,
            data_ratio: 0.8,
            intro_ratio: 0.1,
            total_bytes: 1024.0,
        };
        
        let quantum = ConversionUtils::classical_to_quantum(&sig);
        let mut region = GabrielRegion::new(sig.clone(), quantum.clone(), 0.5);
        
        // Update with multiple attack samples
        for _ in 0..5 {
            region.update(sig.clone(), quantum.clone(), true);
        }
        
        assert!(region.is_attack_region());
    }
}
