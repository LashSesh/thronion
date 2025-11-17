use crate::{GabrielCell, config::OphanionSettings};
use ndarray::Array1;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct GabrielCluster {
    pub cells: Arc<RwLock<Vec<GabrielCell>>>,
    config: OphanionSettings,
}

impl GabrielCluster {
    pub fn new(config: OphanionSettings) -> Self {
        let cells = (0..config.num_gabriel_cells)
            .map(|id| GabrielCell::new(id, config.spectral_dim))
            .collect();
        
        Self {
            cells: Arc::new(RwLock::new(cells)),
            config,
        }
    }
    
    /// Find nearest Gabriel Cell to given signature
    pub fn find_nearest(&self, signature: &Array1<f64>) -> usize {
        let cells = self.cells.read();
        
        cells.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let dist_a = a.distance_to(signature);
                let dist_b = b.distance_to(signature);
                dist_a.partial_cmp(&dist_b).unwrap()
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
    
    /// Find k-nearest cells
    pub fn find_k_nearest(&self, signature: &Array1<f64>, k: usize) -> Vec<usize> {
        let cells = self.cells.read();
        
        let mut distances: Vec<(usize, f64)> = cells.iter()
            .enumerate()
            .map(|(idx, cell)| (idx, cell.distance_to(signature)))
            .collect();
        
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        distances.iter()
            .take(k.min(cells.len()))
            .map(|(idx, _)| *idx)
            .collect()
    }
    
    /// Update cell with new circuit signature
    pub fn update_cell(&self, cell_id: usize, signature: &Array1<f64>) {
        let mut cells = self.cells.write();
        
        if let Some(cell) = cells.get_mut(cell_id) {
            cell.update_centroid(signature, self.config.learning_rate_alpha);
            cell.resonance_strength += 0.01;
            cell.resonance_strength = cell.resonance_strength.min(1.0);
        }
    }
    
    /// Update multiple cells (for k-nearest)
    pub fn update_cells(&self, cell_ids: &[usize], signature: &Array1<f64>, weights: &[f64]) {
        let mut cells = self.cells.write();
        
        for (i, &cell_id) in cell_ids.iter().enumerate() {
            if let Some(cell) = cells.get_mut(cell_id) {
                let weight = weights.get(i).copied().unwrap_or(1.0);
                let alpha = self.config.learning_rate_alpha * weight;
                cell.update_centroid(signature, alpha);
                cell.resonance_strength += 0.01 * weight;
                cell.resonance_strength = cell.resonance_strength.min(1.0);
            }
        }
    }
    
    /// Update connection weights between all cells
    pub fn update_connections(&self) {
        let mut cells = self.cells.write();
        
        for i in 0..cells.len() {
            let (left, right) = cells.split_at_mut(i + 1);
            if let Some(cell) = left.last_mut() {
                cell.update_connections(
                    right, 
                    self.config.learning_rate_alpha, 
                    self.config.decay_rate_beta
                );
            }
        }
    }
    
    /// Decay all resonance strengths over time
    pub fn apply_decay(&self, decay_factor: f64) {
        let mut cells = self.cells.write();
        
        for cell in cells.iter_mut() {
            cell.resonance_strength *= decay_factor;
        }
    }
    
    /// Get current global coherence metric
    pub fn global_coherence(&self) -> f64 {
        let cells = self.cells.read();
        let total: f64 = cells.iter()
            .map(|c| c.resonance_strength)
            .sum();
        
        total / cells.len() as f64
    }
    
    /// Get weighted coherence (high-strength cells matter more)
    pub fn weighted_coherence(&self) -> f64 {
        let cells = self.cells.read();
        
        let total_strength: f64 = cells.iter()
            .map(|c| c.resonance_strength)
            .sum();
        
        if total_strength == 0.0 {
            return 0.0;
        }
        
        let weighted_sum: f64 = cells.iter()
            .map(|c| c.resonance_strength.powi(2))
            .sum();
        
        weighted_sum / total_strength
    }
    
    /// Get statistics about cell distribution
    pub fn statistics(&self) -> ClusterStatistics {
        let cells = self.cells.read();
        
        let strengths: Vec<f64> = cells.iter()
            .map(|c| c.resonance_strength)
            .collect();
        
        let mean = strengths.iter().sum::<f64>() / strengths.len() as f64;
        
        let variance = strengths.iter()
            .map(|&s| (s - mean).powi(2))
            .sum::<f64>() / strengths.len() as f64;
        
        let active_cells = strengths.iter()
            .filter(|&&s| s > 0.1)
            .count();
        
        ClusterStatistics {
            mean_strength: mean,
            variance: variance,
            std_dev: variance.sqrt(),
            active_cells,
            total_cells: cells.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClusterStatistics {
    pub mean_strength: f64,
    pub variance: f64,
    pub std_dev: f64,
    pub active_cells: usize,
    pub total_cells: usize,
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
    fn test_gabriel_cluster_creation() {
        let config = test_config();
        let cluster = GabrielCluster::new(config);
        
        let cells = cluster.cells.read();
        assert_eq!(cells.len(), 16);
    }
    
    #[test]
    fn test_find_nearest() {
        let config = test_config();
        let cluster = GabrielCluster::new(config.clone());
        
        let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);
        let nearest = cluster.find_nearest(&signature);
        
        assert!(nearest < config.num_gabriel_cells);
    }
    
    #[test]
    fn test_update_cell() {
        let config = test_config();
        let cluster = GabrielCluster::new(config.clone());
        
        let signature = Array1::from_vec(vec![0.7; config.spectral_dim]);
        let nearest = cluster.find_nearest(&signature);
        
        cluster.update_cell(nearest, &signature);
        
        let coherence = cluster.global_coherence();
        assert!(coherence > 0.0);
    }
    
    #[test]
    fn test_decay() {
        let config = test_config();
        let cluster = GabrielCluster::new(config.clone());
        
        // Add some resonance
        let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);
        cluster.update_cell(0, &signature);
        
        let coherence_before = cluster.global_coherence();
        
        // Apply decay
        cluster.apply_decay(0.9);
        
        let coherence_after = cluster.global_coherence();
        assert!(coherence_after < coherence_before);
    }
    
    #[test]
    fn test_statistics() {
        let config = test_config();
        let cluster = GabrielCluster::new(config.clone());
        
        let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);
        for i in 0..5 {
            cluster.update_cell(i, &signature);
        }
        
        let stats = cluster.statistics();
        assert_eq!(stats.total_cells, 16);
        assert!(stats.active_cells <= 5);
    }
}
