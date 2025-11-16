pub mod config;
pub mod spectral;
pub mod gabriel_cell;
pub mod resonance;
pub mod threshold;
pub mod delta_kernel;
pub mod tor_interface;
pub mod circuit_monitor;
pub mod decision;

use ndarray::Array1;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Spectral fingerprint of circuit behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spectrum {
    pub frequencies: Vec<f64>,
    pub amplitudes: Vec<f64>,
}

impl Spectrum {
    pub fn dominant_frequency(&self) -> f64 {
        let max_idx = self.amplitudes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        
        self.frequencies.get(max_idx).copied().unwrap_or(0.0)
    }
    
    pub fn energy(&self) -> f64 {
        self.amplitudes.iter().map(|a| a * a).sum()
    }
}

/// Tor circuit metadata extracted from control port
#[derive(Debug, Clone)]
pub struct TorCircuitMetadata {
    pub circuit_id: u32,
    pub created_at: Instant,
    pub cell_timings: Vec<Duration>,
    pub cell_types: Vec<TorCellType>,
    pub introduction_point: Option<String>,
    pub rendezvous_completed: bool,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TorCellType {
    Introduce2,
    Rendezvous1,
    Rendezvous2,
    Data,
    Padding,
    Other,
}

/// Resonant field state vector
#[derive(Debug, Clone)]
pub struct ResonantField {
    pub state: Array1<f64>,
    pub gradient: f64,
}

impl ResonantField {
    pub fn new(dim: usize) -> Self {
        Self {
            state: Array1::zeros(dim),
            gradient: 1.0,
        }
    }
    
    pub fn update_gradient(&mut self) {
        self.gradient *= 0.95;
    }
}

/// Adaptive threshold with learning rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold {
    pub value: f64,
    pub lambda: f64,
}

impl Threshold {
    pub fn new(initial: f64, lambda: f64) -> Self {
        Self {
            value: initial.clamp(0.0, 1.0),
            lambda: lambda.max(0.0),
        }
    }
    
    pub fn update(&mut self, gradient: f64) {
        self.value -= self.lambda * gradient;
        self.value = self.value.clamp(0.0, 1.0);
    }
}

/// Gabriel Cell - proto-intelligent learning unit
#[derive(Debug, Clone)]
pub struct GabrielCell {
    pub id: usize,
    pub centroid: Array1<f64>,
    pub covariance: f64,
    pub resonance_strength: f64,
    pub connections: Vec<(usize, f64)>,
}

impl GabrielCell {
    pub fn new(id: usize, dim: usize) -> Self {
        Self {
            id,
            centroid: Array1::zeros(dim),
            covariance: 1.0,
            resonance_strength: 0.0,
            connections: Vec::new(),
        }
    }
    
    pub fn distance_to(&self, signature: &Array1<f64>) -> f64 {
        (&self.centroid - signature).mapv(|x| x * x).sum().sqrt()
    }
    
    pub fn update_centroid(&mut self, signature: &Array1<f64>, alpha: f64) {
        self.centroid = &self.centroid * (1.0 - alpha) + signature * alpha;
    }
    
    pub fn update_connections(&mut self, other_cells: &[GabrielCell], alpha: f64, beta: f64) {
        for other in other_cells {
            if other.id == self.id {
                continue;
            }
            
            let delta_w = alpha * self.resonance_strength * other.resonance_strength;
            
            if let Some(conn) = self.connections.iter_mut().find(|(id, _)| *id == other.id) {
                conn.1 = conn.1 * (1.0 - beta) + delta_w;
            } else {
                self.connections.push((other.id, delta_w));
            }
        }
    }
}

/// Circuit decision outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitAction {
    Forward,
    Absorb,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spectrum_dominant_frequency() {
        let spectrum = Spectrum {
            frequencies: vec![0.0, 0.1, 0.2, 0.3],
            amplitudes: vec![1.0, 5.0, 2.0, 1.5],
        };
        
        assert_eq!(spectrum.dominant_frequency(), 0.1);
    }
    
    #[test]
    fn test_threshold_update() {
        let mut threshold = Threshold::new(0.5, 0.1);
        threshold.update(0.2);
        
        assert!(threshold.value < 0.5);
        assert!(threshold.value >= 0.0);
    }
    
    #[test]
    fn test_gabriel_cell_distance() {
        let cell = GabrielCell::new(0, 3);
        let sig = Array1::from_vec(vec![1.0, 1.0, 1.0]);
        
        let dist = cell.distance_to(&sig);
        assert!(dist > 0.0);
    }
}
