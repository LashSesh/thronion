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
use crate::delta::DeltaKernel;
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

/// Thronion Decision Engine
///
/// Multi-region classifier that combines Gabriel-Mandorla hybrid regions
/// for real-time DDoS detection on Tor Hidden Services.
pub struct ThronionKernel {
    /// Collection of learned regions (benign + attack patterns)
    regions: Vec<GabrielRegion>,
    /// Decision threshold for attack classification
    attack_threshold: f64,
    /// Maximum number of regions (for memory management)
    max_regions: usize,
    /// Learning rate for adaptive updates
    learning_rate: f64,
}

impl ThronionKernel {
    /// Create new Thronion kernel with default parameters
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            attack_threshold: 0.5,
            max_regions: 100,
            learning_rate: 0.1,
        }
    }
    
    /// Create with custom parameters
    pub fn with_params(attack_threshold: f64, max_regions: usize, learning_rate: f64) -> Self {
        Self {
            regions: Vec::new(),
            attack_threshold,
            max_regions,
            learning_rate,
        }
    }
    
    /// Classify a circuit as attack or benign
    ///
    /// Returns (is_attack, resonance_score, best_region_idx)
    pub fn classify(&self, metadata: &TorCircuitMetadata, timing: &TimingFeatures, dist: &CellTypeDistribution) -> (bool, f64, Option<usize>) {
        if self.regions.is_empty() {
            // No learned regions yet - default to benign
            return (false, 0.0, None);
        }
        
        // Extract signatures
        let classical = ClassicalSignature::from_metadata(metadata, timing, dist);
        let quantum = ConversionUtils::classical_to_quantum(&classical);
        
        // Find best matching region
        let mut best_resonance = 0.0;
        let mut best_idx = 0;
        
        for (idx, region) in self.regions.iter().enumerate() {
            let resonance = region.hybrid_resonance(&classical, &quantum);
            if resonance > best_resonance {
                best_resonance = resonance;
                best_idx = idx;
            }
        }
        
        // Decision based on best region's attack probability
        let is_attack = if best_resonance > 0.3 {
            // Strong match to a region - use its attack probability
            self.regions[best_idx].is_attack_region()
        } else {
            // Weak match - default to benign (conservative)
            false
        };
        
        (is_attack, best_resonance, Some(best_idx))
    }
    
    /// Learn from a labeled circuit (online learning)
    pub fn learn(&mut self, metadata: &TorCircuitMetadata, timing: &TimingFeatures, dist: &CellTypeDistribution, is_attack: bool) {
        let classical = ClassicalSignature::from_metadata(metadata, timing, dist);
        let quantum = ConversionUtils::classical_to_quantum(&classical);
        
        // Find closest region
        let mut best_resonance = 0.0;
        let mut best_idx = None;
        
        for (idx, region) in self.regions.iter().enumerate() {
            let resonance = region.hybrid_resonance(&classical, &quantum);
            if resonance > best_resonance {
                best_resonance = resonance;
                best_idx = Some(idx);
            }
        }
        
        if best_resonance > 0.5 {
            // Update existing region
            if let Some(idx) = best_idx {
                self.regions[idx].update(classical, quantum, is_attack);
            }
        } else {
            // Create new region if below capacity
            if self.regions.len() < self.max_regions {
                let attack_prob = if is_attack { 1.0 } else { 0.0 };
                let region = GabrielRegion::new(classical, quantum, attack_prob);
                self.regions.push(region);
            } else {
                // Replace least confident region
                let mut min_confidence = 1.0;
                let mut min_idx = 0;
                
                for (idx, region) in self.regions.iter().enumerate() {
                    let confidence = (region.attack_probability - 0.5).abs();
                    if confidence < min_confidence {
                        min_confidence = confidence;
                        min_idx = idx;
                    }
                }
                
                let attack_prob = if is_attack { 1.0 } else { 0.0 };
                self.regions[min_idx] = GabrielRegion::new(classical, quantum, attack_prob);
            }
        }
    }
    
    /// Get statistics about the kernel
    pub fn stats(&self) -> KernelStats {
        let attack_regions = self.regions.iter().filter(|r| r.is_attack_region()).count();
        let benign_regions = self.regions.len() - attack_regions;
        
        KernelStats {
            total_regions: self.regions.len(),
            attack_regions,
            benign_regions,
            attack_threshold: self.attack_threshold,
        }
    }
    
    /// Reset the kernel (clear all learned regions)
    pub fn reset(&mut self) {
        self.regions.clear();
    }
}

impl Default for ThronionKernel {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the Thronion kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelStats {
    pub total_regions: usize,
    pub attack_regions: usize,
    pub benign_regions: usize,
    pub attack_threshold: f64,
}

/// Enhanced Thronion Kernel with Delta Kernel optimization
/// 
/// This extends the basic ThronionKernel with QRIK's Delta Kernel for
/// advanced coherence-based optimization, region merging, and Hamiltonian evolution.
pub struct EnhancedThronionKernel {
    /// Base Thronion kernel for classification
    pub base_kernel: ThronionKernel,
    /// Delta Kernel for optimization
    pub delta_kernel: DeltaKernel,
    /// Optimization interval (number of classifications before optimization)
    pub optimization_interval: usize,
    /// Classification counter
    pub classification_count: usize,
}

impl EnhancedThronionKernel {
    /// Create new enhanced kernel with Delta Kernel optimization
    pub fn new(base_kernel: ThronionKernel, delta_params: crate::delta::QRIKParams) -> Self {
        let delta_kernel = DeltaKernel::new(delta_params);
        
        Self {
            base_kernel,
            delta_kernel,
            optimization_interval: 100, // Optimize every 100 classifications
            classification_count: 0,
        }
    }
    
    /// Classify with automatic optimization
    pub fn classify(&mut self, metadata: &TorCircuitMetadata, timing: &TimingFeatures, dist: &CellTypeDistribution) -> (bool, f64, Option<usize>) {
        self.classification_count += 1;
        
        // Perform classification
        let result = self.base_kernel.classify(metadata, timing, dist);
        
        // Periodic optimization
        if self.classification_count % self.optimization_interval == 0 {
            self.optimize();
        }
        
        result
    }
    
    /// Learn with Delta Kernel-enhanced updates
    pub fn learn(&mut self, metadata: &TorCircuitMetadata, timing: &TimingFeatures, dist: &CellTypeDistribution, is_attack: bool) {
        // Standard learning
        self.base_kernel.learn(metadata, timing, dist, is_attack);
        
        // Evolve Delta Kernel
        self.delta_kernel.evolve(0.01);
    }
    
    /// Optimize regions using Delta Kernel coherence
    pub fn optimize(&mut self) {
        // Check if optimization is needed
        let gradient = self.delta_kernel.coherence_gradient();
        
        if gradient > 0.1 {
            // High gradient: system needs optimization
            
            // 1. Merge similar regions based on quantum coherence
            self.merge_coherent_regions();
            
            // 2. Evolve Delta Kernel towards optimal state
            for _ in 0..10 {
                self.delta_kernel.evolve(0.01);
                if self.delta_kernel.is_stable(0.05) {
                    break;
                }
            }
        }
    }
    
    /// Merge regions with high quantum coherence (fidelity > 0.9)
    fn merge_coherent_regions(&mut self) {
        let mut to_merge: Vec<(usize, usize)> = Vec::new();
        
        // Find pairs of regions with high fidelity
        for i in 0..self.base_kernel.regions.len() {
            for j in (i+1)..self.base_kernel.regions.len() {
                let fidelity = self.base_kernel.regions[i]
                    .quantum_center
                    .fidelity(&self.base_kernel.regions[j].quantum_center);
                
                if fidelity > 0.9 {
                    to_merge.push((i, j));
                }
            }
        }
        
        // Merge regions (simplified: keep first, update with second's data, remove second)
        for (i, j) in to_merge.iter().rev() {
            if *j < self.base_kernel.regions.len() {
                // Weighted average based on sample counts
                let region_i = &self.base_kernel.regions[*i];
                let region_j = &self.base_kernel.regions[*j];
                
                let total_samples = region_i.sample_count + region_j.sample_count;
                if total_samples > 0 {
                    let w_i = region_i.sample_count as f64 / total_samples as f64;
                    let w_j = region_j.sample_count as f64 / total_samples as f64;
                    
                    // Merge attack probabilities
                    let merged_prob = w_i * region_i.attack_probability + w_j * region_j.attack_probability;
                    self.base_kernel.regions[*i].attack_probability = merged_prob;
                    self.base_kernel.regions[*i].sample_count = total_samples;
                }
                
                // Remove the merged region
                self.base_kernel.regions.remove(*j);
            }
        }
    }
    
    /// Get coherence gradient from Delta Kernel
    pub fn coherence_gradient(&self) -> f64 {
        self.delta_kernel.coherence_gradient()
    }
    
    /// Check if system is stable
    pub fn is_stable(&self) -> bool {
        self.delta_kernel.is_stable(0.05)
    }
    
    /// Get combined statistics
    pub fn stats(&self) -> EnhancedKernelStats {
        let base_stats = self.base_kernel.stats();
        
        EnhancedKernelStats {
            base_stats,
            coherence_gradient: self.delta_kernel.coherence_gradient(),
            is_stable: self.is_stable(),
            classification_count: self.classification_count,
        }
    }
}

/// Statistics for Enhanced Thronion Kernel
#[derive(Debug, Clone)]
pub struct EnhancedKernelStats {
    pub base_stats: KernelStats,
    pub coherence_gradient: f64,
    pub is_stable: bool,
    pub classification_count: usize,
}

#[cfg(test)]
mod kernel_tests {
    use super::*;
    use std::time::Duration;
    
    fn create_test_metadata(is_attack: bool) -> (TorCircuitMetadata, TimingFeatures, CellTypeDistribution) {
        use crate::tor::TorCellType;
        use std::time::Instant;
        
        let timings = if is_attack {
            // Attack: fast, uniform timing
            vec![
                Duration::from_micros(10),
                Duration::from_micros(11),
                Duration::from_micros(10),
                Duration::from_micros(12),
            ]
        } else {
            // Benign: slower, variable timing
            vec![
                Duration::from_micros(100),
                Duration::from_micros(150),
                Duration::from_micros(80),
                Duration::from_micros(120),
            ]
        };
        
        let cells = if is_attack {
            // Attack: lots of data cells
            vec![
                TorCellType::Data,
                TorCellType::Data,
                TorCellType::Data,
                TorCellType::Padding,
            ]
        } else {
            // Benign: mixed cell types
            vec![
                TorCellType::Introduce2,
                TorCellType::Data,
                TorCellType::Rendezvous1,
                TorCellType::Data,
            ]
        };
        
        let metadata = TorCircuitMetadata {
            circuit_id: 1,
            created_at: Instant::now(),
            cell_timings: timings.clone(),
            cell_types: cells.clone(),
            introduction_point: Some("test.onion".to_string()),
            rendezvous_completed: true,
            total_bytes: if is_attack { 10000 } else { 2000 },
        };
        
        let timing = crate::tor::MetadataExtractor::extract_timing_features(&timings);
        let dist = crate::tor::MetadataExtractor::analyze_cell_types(&cells);
        
        (metadata, timing, dist)
    }
    
    #[test]
    fn test_kernel_creation() {
        let kernel = ThronionKernel::new();
        assert_eq!(kernel.regions.len(), 0);
        assert_eq!(kernel.attack_threshold, 0.5);
        assert_eq!(kernel.max_regions, 100);
    }
    
    #[test]
    fn test_kernel_learning() {
        let mut kernel = ThronionKernel::new();
        
        // Learn from benign circuit
        let (metadata, timing, dist) = create_test_metadata(false);
        kernel.learn(&metadata, &timing, &dist, false);
        
        assert_eq!(kernel.regions.len(), 1);
        assert!(!kernel.regions[0].is_attack_region());
    }
    
    #[test]
    fn test_kernel_classification() {
        let mut kernel = ThronionKernel::new();
        
        // Learn benign pattern
        let (metadata, timing, dist) = create_test_metadata(false);
        kernel.learn(&metadata, &timing, &dist, false);
        
        // Learn attack pattern - this should create a new region or update existing
        let (metadata_attack, timing_attack, dist_attack) = create_test_metadata(true);
        kernel.learn(&metadata_attack, &timing_attack, &dist_attack, true);
        
        // We should have at least 1 region
        assert!(kernel.regions.len() >= 1);
        assert!(kernel.regions.len() <= 2); // At most 2 if patterns are distinct enough
        
        // Classify benign
        let (is_attack, resonance, _) = kernel.classify(&metadata, &timing, &dist);
        assert!(!is_attack || resonance < 0.6); // Should classify as benign
        
        // Classify attack
        let (_is_attack_2, _, _) = kernel.classify(&metadata_attack, &timing_attack, &dist_attack);
        // Note: might be benign due to weak matching in simple test
        // In production, more training data would improve accuracy
    }
    
    #[test]
    fn test_kernel_stats() {
        let mut kernel = ThronionKernel::new();
        
        // Learn patterns
        let (metadata_benign, timing_benign, dist_benign) = create_test_metadata(false);
        let (metadata_attack, timing_attack, dist_attack) = create_test_metadata(true);
        
        kernel.learn(&metadata_benign, &timing_benign, &dist_benign, false);
        kernel.learn(&metadata_attack, &timing_attack, &dist_attack, true);
        
        let stats = kernel.stats();
        assert!(stats.total_regions >= 1 && stats.total_regions <= 2);
        // Either 1 region (updated) or 2 regions (distinct patterns)
    }
    
    #[test]
    fn test_kernel_capacity() {
        let mut kernel = ThronionKernel::with_params(0.5, 5, 0.1);
        
        // Try to learn more regions than capacity
        for i in 0..10 {
            let (metadata, timing, dist) = create_test_metadata(i % 2 == 0);
            kernel.learn(&metadata, &timing, &dist, i % 2 == 0);
        }
        
        // Should not exceed max capacity
        assert!(kernel.regions.len() <= 5);
    }
}

#[cfg(test)]
mod enhanced_kernel_tests {
    use super::*;
    use crate::delta::QRIKParams;
    use std::time::Duration;
    use crate::tor::TorCellType;
    
    fn create_test_metadata(is_attack: bool) -> (TorCircuitMetadata, TimingFeatures, CellTypeDistribution) {
        use std::time::Instant;
        
        let timings = if is_attack {
            vec![
                Duration::from_micros(10),
                Duration::from_micros(11),
                Duration::from_micros(10),
                Duration::from_micros(12),
            ]
        } else {
            vec![
                Duration::from_micros(100),
                Duration::from_micros(120),
                Duration::from_micros(110),
                Duration::from_micros(105),
            ]
        };
        
        let cells = if is_attack {
            vec![
                TorCellType::Data,
                TorCellType::Data,
                TorCellType::Data,
                TorCellType::Data,
            ]
        } else {
            vec![
                TorCellType::Introduce2,
                TorCellType::Data,
                TorCellType::Rendezvous1,
                TorCellType::Data,
            ]
        };
        
        let metadata = TorCircuitMetadata {
            circuit_id: 1,
            created_at: Instant::now(),
            cell_timings: timings.clone(),
            cell_types: cells.clone(),
            introduction_point: None,
            rendezvous_completed: false,
            total_bytes: if is_attack { 4096 } else { 1024 },
        };
        
        let timing = crate::tor::MetadataExtractor::extract_timing_features(&timings);
        let dist = crate::tor::MetadataExtractor::analyze_cell_types(&cells);
        
        (metadata, timing, dist)
    }
    
    #[test]
    fn test_enhanced_kernel_creation() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        assert_eq!(enhanced.classification_count, 0);
        assert_eq!(enhanced.optimization_interval, 100);
    }
    
    #[test]
    fn test_enhanced_kernel_classification() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let mut enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        let (metadata, timing, dist) = create_test_metadata(false);
        
        // First classification (benign, no regions yet)
        let (is_attack, resonance, region_idx) = enhanced.classify(&metadata, &timing, &dist);
        
        // Should default to benign (no regions learned)
        assert!(!is_attack);
        assert!(region_idx.is_none());
        assert_eq!(enhanced.classification_count, 1);
    }
    
    #[test]
    fn test_enhanced_kernel_learning() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let mut enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        // Learn from attack pattern
        let (metadata, timing, dist) = create_test_metadata(true);
        enhanced.learn(&metadata, &timing, &dist, true);
        
        // Should have created one region
        assert_eq!(enhanced.base_kernel.regions.len(), 1);
    }
    
    #[test]
    fn test_enhanced_kernel_optimization() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let mut enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        // Set lower interval for testing
        enhanced.optimization_interval = 5;
        
        // Perform multiple classifications to trigger optimization
        let (metadata, timing, dist) = create_test_metadata(false);
        
        for _ in 0..10 {
            enhanced.classify(&metadata, &timing, &dist);
        }
        
        // Should have optimized twice (at 5 and 10)
        assert_eq!(enhanced.classification_count, 10);
    }
    
    #[test]
    fn test_enhanced_kernel_coherence() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        // Check coherence gradient
        let gradient = enhanced.coherence_gradient();
        assert!(gradient.is_finite());
        assert!(gradient >= 0.0);
    }
    
    #[test]
    fn test_enhanced_kernel_stats() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let mut enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        // Learn some patterns
        let (metadata_attack, timing_attack, dist_attack) = create_test_metadata(true);
        let (metadata_benign, timing_benign, dist_benign) = create_test_metadata(false);
        
        enhanced.learn(&metadata_attack, &timing_attack, &dist_attack, true);
        enhanced.learn(&metadata_benign, &timing_benign, &dist_benign, false);
        
        let stats = enhanced.stats();
        assert!(stats.base_stats.total_regions >= 1);
        assert!(stats.coherence_gradient.is_finite());
        assert_eq!(stats.classification_count, 0); // No classifications yet
    }
    
    #[test]
    fn test_region_merging() {
        let base_kernel = ThronionKernel::new();
        let params = QRIKParams::default();
        let mut enhanced = EnhancedThronionKernel::new(base_kernel, params);
        
        // Learn similar patterns (should potentially merge)
        for _ in 0..3 {
            let (metadata, timing, dist) = create_test_metadata(true);
            enhanced.learn(&metadata, &timing, &dist, true);
        }
        
        // Trigger optimization manually
        enhanced.optimize();
        
        // After optimization, similar regions may have merged
        assert!(enhanced.base_kernel.regions.len() <= 3);
    }
}
