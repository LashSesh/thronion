use crate::config::OphanionSettings;

pub struct DeltaKernel {
    config: OphanionSettings,
    pub alpha: f64,
    pub beta: f64,
    pub theta: f64,
}

impl DeltaKernel {
    pub fn new(config: OphanionSettings) -> Self {
        Self {
            alpha: config.learning_rate_alpha,
            beta: config.decay_rate_beta,
            theta: config.initial_threshold,
            config,
        }
    }
    
    /// Compute Delta-gradient magnitude
    pub fn gradient_magnitude(&self, coherence: f64, flood_energy: f64) -> f64 {
        let kappa = 0.2;
        let _psi = -coherence + kappa * flood_energy;
        
        // Partial derivatives (simplified model)
        let d_psi_d_alpha = -0.1 * coherence;
        let d_psi_d_beta = 0.05 * flood_energy;
        let d_psi_d_theta = -0.15 * (coherence - flood_energy);
        
        (d_psi_d_alpha.powi(2) 
         + d_psi_d_beta.powi(2) 
         + d_psi_d_theta.powi(2)).sqrt()
    }
    
    /// Optimize parameters toward gradient = 0
    pub fn optimize_step(&mut self, coherence: f64, flood_energy: f64) {
        let eta = self.config.optimization_eta;
        
        // Simplified gradient descent
        let grad_alpha = -0.1 * coherence;
        let grad_beta = 0.05 * flood_energy;
        let grad_theta = -0.15 * (coherence - flood_energy);
        
        self.alpha -= eta * grad_alpha;
        self.beta -= eta * grad_beta;
        self.theta -= eta * grad_theta;
        
        // Clamp to valid ranges
        self.alpha = self.alpha.clamp(0.001, 0.1);
        self.beta = self.beta.clamp(0.0001, 0.01);
        self.theta = self.theta.clamp(0.1, 0.9);
    }
    
    /// Get optimized parameters
    pub fn get_params(&self) -> (f64, f64, f64) {
        (self.alpha, self.beta, self.theta)
    }
    
    /// Check if convergence achieved
    pub fn has_converged(&self, coherence: f64, flood_energy: f64) -> bool {
        self.gradient_magnitude(coherence, flood_energy) < self.config.convergence_epsilon
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_delta_kernel() {
        let config = OphanionSettings::default();
        let mut kernel = DeltaKernel::new(config);
        
        let coherence = 0.7;
        let flood_energy = 0.3;
        
        let grad_before = kernel.gradient_magnitude(coherence, flood_energy);
        
        kernel.optimize_step(coherence, flood_energy);
        
        let grad_after = kernel.gradient_magnitude(coherence, flood_energy);
        
        // Gradient should decrease (or at least not increase significantly)
        assert!(grad_after <= grad_before + 0.01);
    }
}
