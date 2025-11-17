use crate::{Spectrum, TorCircuitMetadata};
use num_complex::Complex64;
use rustfft::{FftPlanner, num_complex};
use ndarray::Array1;

pub struct SpectralEngine {
    planner: FftPlanner<f64>,
}

impl SpectralEngine {
    pub fn new() -> Self {
        Self {
            planner: FftPlanner::new(),
        }
    }
    
    /// Compute spectral fingerprint from circuit timing data
    pub fn compute_fingerprint(&mut self, circuit: &TorCircuitMetadata) -> Spectrum {
        let timings: Vec<f64> = circuit.cell_timings
            .iter()
            .map(|d| d.as_secs_f64())
            .collect();
        
        if timings.is_empty() {
            return Spectrum {
                frequencies: vec![0.0],
                amplitudes: vec![0.0],
            };
        }
        
        // Zero-padding to next power of 2
        let n = timings.len();
        let padded_len = n.next_power_of_two();
        
        let mut buffer: Vec<Complex64> = timings
            .iter()
            .map(|&x| Complex64::new(x, 0.0))
            .collect();
        
        buffer.resize(padded_len, Complex64::new(0.0, 0.0));
        
        let fft = self.planner.plan_fft_forward(padded_len);
        fft.process(&mut buffer);
        
        let frequencies: Vec<f64> = (0..padded_len)
            .map(|i| i as f64 / padded_len as f64)
            .collect();
        
        let amplitudes: Vec<f64> = buffer
            .iter()
            .map(|c| c.norm())
            .collect();
        
        Spectrum {
            frequencies,
            amplitudes,
        }
    }
    
    /// Extract additional statistical features
    pub fn extract_features(&self, circuit: &TorCircuitMetadata) -> Vec<f64> {
        let timings: Vec<f64> = circuit.cell_timings
            .iter()
            .map(|d| d.as_secs_f64())
            .collect();
        
        if timings.is_empty() {
            return vec![0.0; 8];
        }
        
        let n = timings.len() as f64;
        let mean = timings.iter().sum::<f64>() / n;
        let variance = timings.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / n;
        let std_dev = variance.sqrt();
        
        let min = timings.iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let max = timings.iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        
        // Inter-quartile range
        let mut sorted = timings.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q1_idx = sorted.len() / 4;
        let q3_idx = 3 * sorted.len() / 4;
        let q1 = sorted.get(q1_idx).copied().unwrap_or(0.0);
        let q3 = sorted.get(q3_idx).copied().unwrap_or(0.0);
        let iqr = q3 - q1;
        
        // Circuit-specific features
        let duration = circuit.created_at.elapsed().as_secs_f64();
        let bytes_per_sec = circuit.total_bytes as f64 / duration.max(1.0);
        
        vec![mean, std_dev, min, max, iqr, variance, duration, bytes_per_sec]
    }
    
    /// Combine spectral and statistical features into unified signature
    pub fn create_signature(&mut self, circuit: &TorCircuitMetadata) -> Array1<f64> {
        let spectrum = self.compute_fingerprint(circuit);
        let features = self.extract_features(circuit);
        
        // Take dominant frequency components + statistical features
        let n_freq = 120; // Use first 120 frequency bins
        let mut signature = Vec::with_capacity(n_freq + features.len());
        
        signature.extend(
            spectrum.amplitudes
                .iter()
                .take(n_freq)
                .cloned()
        );
        signature.extend(features);
        
        // Pad to standard dimension if needed
        while signature.len() < 128 {
            signature.push(0.0);
        }
        
        // Normalize signature
        let norm = signature.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for x in &mut signature {
                *x /= norm;
            }
        }
        
        Array1::from_vec(signature)
    }
    
    /// Compute entropy of timing distribution
    pub fn timing_entropy(&self, circuit: &TorCircuitMetadata) -> f64 {
        let timings: Vec<f64> = circuit.cell_timings
            .iter()
            .map(|d| d.as_secs_f64())
            .collect();
        
        if timings.is_empty() {
            return 0.0;
        }
        
        // Create histogram
        let mut hist = vec![0usize; 10];
        let min_time = timings.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_time = timings.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = (max_time - min_time).max(1e-10);
        
        for &t in &timings {
            let bin = (((t - min_time) / range) * 9.99).floor() as usize;
            let bin = bin.min(9);
            hist[bin] += 1;
        }
        
        // Compute entropy
        let total = timings.len() as f64;
        hist.iter()
            .filter(|&&count| count > 0)
            .map(|&count| {
                let p = count as f64 / total;
                -p * p.log2()
            })
            .sum()
    }
}

impl Default for SpectralEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    
    fn create_test_circuit() -> TorCircuitMetadata {
        TorCircuitMetadata {
            circuit_id: 1,
            created_at: Instant::now(),
            cell_timings: vec![
                Duration::from_millis(10),
                Duration::from_millis(15),
                Duration::from_millis(12),
                Duration::from_millis(18),
                Duration::from_millis(14),
                Duration::from_millis(16),
            ],
            cell_types: vec![],
            introduction_point: None,
            rendezvous_completed: false,
            total_bytes: 1000,
        }
    }
    
    #[test]
    fn test_spectral_fingerprint() {
        let mut engine = SpectralEngine::new();
        let circuit = create_test_circuit();
        
        let spectrum = engine.compute_fingerprint(&circuit);
        
        assert!(!spectrum.frequencies.is_empty());
        assert_eq!(spectrum.frequencies.len(), spectrum.amplitudes.len());
        assert!(spectrum.energy() > 0.0);
    }
    
    #[test]
    fn test_feature_extraction() {
        let engine = SpectralEngine::new();
        let circuit = create_test_circuit();
        
        let features = engine.extract_features(&circuit);
        
        assert_eq!(features.len(), 8);
        assert!(features.iter().all(|&f| f.is_finite()));
    }
    
    #[test]
    fn test_signature_creation() {
        let mut engine = SpectralEngine::new();
        let circuit = create_test_circuit();
        
        let signature = engine.create_signature(&circuit);
        
        assert_eq!(signature.len(), 128);
        
        // Check normalization
        let norm = signature.iter().map(|&x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }
    
    #[test]
    fn test_timing_entropy() {
        let engine = SpectralEngine::new();
        let circuit = create_test_circuit();
        
        let entropy = engine.timing_entropy(&circuit);
        
        assert!(entropy >= 0.0);
        assert!(entropy <= 10.0_f64.log2());
    }
}
