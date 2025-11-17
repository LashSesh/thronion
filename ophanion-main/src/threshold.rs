// threshold.rs
use crate::{Threshold, config::OphanionSettings};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct AdaptiveThreshold {
    threshold: Arc<RwLock<Threshold>>,
    config: OphanionSettings,
    absorption_history: Arc<RwLock<Vec<f64>>>,
    coherence_history: Arc<RwLock<Vec<f64>>>,
}

impl AdaptiveThreshold {
    pub fn new(config: OphanionSettings) -> Self {
        let threshold = Threshold::new(
            config.initial_threshold, 
            config.threshold_learning_rate
        );
        
        Self {
            threshold: Arc::new(RwLock::new(threshold)),
            config,
            absorption_history: Arc::new(RwLock::new(Vec::new())),
            coherence_history: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn value(&self) -> f64 {
        self.threshold.read().value
    }
    
    pub fn update(&self, coherence: f64, flood_energy: f64) {
        let mut threshold = self.threshold.write();
        
        let kappa = 0.2;
        let gradient = -coherence + kappa * flood_energy;
        
        threshold.update(gradient);
        
        self.coherence_history.write().push(coherence);
    }
    
    pub fn record_absorption(&self, was_absorbed: bool) {
        let mut history = self.absorption_history.write();
        history.push(if was_absorbed { 1.0 } else { 0.0 });
        
        if history.len() > 1000 {
            history.remove(0);
        }
    }
    
    pub fn absorption_rate(&self) -> f64 {
        let history = self.absorption_history.read();
        if history.is_empty() {
            return 0.0;
        }
        
        history.iter().sum::<f64>() / history.len() as f64
    }
    
    pub fn has_converged(&self) -> bool {
        let current_rate = self.absorption_rate();
        let target = self.config.target_absorption_rate;
        
        (current_rate - target).abs() < 0.05
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_threshold() {
        let config = OphanionSettings::default();
        let threshold = AdaptiveThreshold::new(config);
        
        threshold.update(0.8, 0.3);
        let value = threshold.value();
        
        assert!(value >= 0.0 && value <= 1.0);
    }
}
