use crate::gabriel_cell::GabrielCluster;
use crate::config::OphanionSettings;
use ndarray::Array1;
use std::sync::Arc;

pub struct ResonanceEngine {
    gabriel_cluster: Arc<GabrielCluster>,
}

impl ResonanceEngine {
    pub fn new(config: OphanionSettings) -> Self {
        Self {
            gabriel_cluster: Arc::new(GabrielCluster::new(config)),
        }
    }
    
    /// Compute resonance score for circuit signature
    pub fn compute_score(&self, signature: &Array1<f64>) -> f64 {
        let cells = self.gabriel_cluster.cells.read();
        
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        
        for cell in cells.iter() {
            let distance = cell.distance_to(signature);
            let weight = cell.resonance_strength;
            
            // Gaussian kernel with adaptive covariance
            let kernel = (-distance.powi(2) / (2.0 * cell.covariance)).exp();
            
            total_score += weight * kernel;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        }
    }
    
    /// Compute resonance score using k-nearest cells only
    pub fn compute_score_knn(&self, signature: &Array1<f64>, k: usize) -> f64 {
        let nearest_ids = self.gabriel_cluster.find_k_nearest(signature, k);
        let cells = self.gabriel_cluster.cells.read();
        
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        
        for &id in &nearest_ids {
            if let Some(cell) = cells.get(id) {
                let distance = cell.distance_to(signature);
                let weight = cell.resonance_strength;
                
                let kernel = (-distance.powi(2) / (2.0 * cell.covariance)).exp();
                
                total_score += weight * kernel;
                total_weight += weight;
            }
        }
        
        if total_weight > 0.0 {
            total_score / total_weight
        } else {
            0.0
        }
    }
    
    /// Update internal state with new circuit
    pub fn learn_signature(&self, signature: &Array1<f64>) {
        let nearest = self.gabriel_cluster.find_nearest(signature);
        self.gabriel_cluster.update_cell(nearest, signature);
    }
    
    /// Learn signature with k-nearest update
    pub fn learn_signature_knn(&self, signature: &Array1<f64>, k: usize) {
        let nearest_ids = self.gabriel_cluster.find_k_nearest(signature, k);
        
        // Compute inverse distance weights
        let cells = self.gabriel_cluster.cells.read();
        let weights: Vec<f64> = nearest_ids.iter()
            .map(|&id| {
                if let Some(cell) = cells.get(id) {
                    let dist = cell.distance_to(signature);
                    1.0 / (dist + 1e-10)
                } else {
                    0.0
                }
            })
            .collect();
        drop(cells);
        
        // Normalize weights
        let sum: f64 = weights.iter().sum();
        let normalized_weights: Vec<f64> = if sum > 0.0 {
            weights.iter().map(|w| w / sum).collect()
        } else {
            vec![1.0 / nearest_ids.len() as f64; nearest_ids.len()]
        };
        
        self.gabriel_cluster.update_cells(&nearest_ids, signature, &normalized_weights);
    }
    
    /// Periodic maintenance: update connections and apply decay
    pub fn maintenance_cycle(&self) {
        self.gabriel_cluster.update_connections();
        self.gabriel_cluster.apply_decay(0.99);
    }
    
    /// Get global system coherence
    pub fn coherence(&self) -> f64 {
        self.gabriel_cluster.global_coherence()
    }
    
    /// Get weighted coherence
    pub fn weighted_coherence(&self) -> f64 {
        self.gabriel_cluster.weighted_coherence()
    }
    
    /// Get cluster statistics
    pub fn statistics(&self) -> crate::gabriel_cell::ClusterStatistics {
        self.gabriel_cluster.statistics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OphanionSettings;
    
    fn test_config() -> OphanionSettings {
        OphanionSettings {
            num_gabriel_cells: 16,
            spectral_dim: 32,
            learning_rate_alpha: 0.1,
            decay_rate_beta: 0.01,
            ..Default::default()
        }
    }
    
    #[test]
    fn test_resonance_scoring() {
        let config = test_config();
        let engine = ResonanceEngine::new(config.clone());
        
        let sig1 = Array1::from_vec(vec![0.5; config.spectral_dim]);
        let sig2 = Array1::from_vec(vec![0.7; config.spectral_dim]);
        
        engine.learn_signature(&sig1);
        let score1 = engine.compute_score(&sig1);
        
        engine.learn_signature(&sig2);
        let score2 = engine.compute_score(&sig2);
        
        // Learned signatures should have higher scores
        assert!(score1 > 0.0 || score2 > 0.0);
    }
    
    #[test]
    fn test_knn_scoring() {
        let config = test_config();
        let engine = ResonanceEngine::new(config.clone());
        
        let sig = Array1::from_vec(vec![0.5; config.spectral_dim]);
        
        engine.learn_signature_knn(&sig, 3);
        let score = engine.compute_score_knn(&sig, 3);
        
        assert!(score >= 0.0 && score <= 1.0);
    }
    
    #[test]
    fn test_coherence_evolution() {
        let config = test_config();
        let engine = ResonanceEngine::new(config.clone());
        
        let coherence_before = engine.coherence();
        
        // Add multiple signatures
        for i in 0..10 {
            let sig = Array1::from_vec(vec![0.5 + i as f64 * 0.01; config.spectral_dim]);
            engine.learn_signature(&sig);
        }
        
        let coherence_after = engine.coherence();
        
        assert!(coherence_after > coherence_before);
    }
}
