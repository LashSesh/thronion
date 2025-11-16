use crate::CircuitAction;

pub struct DecisionEngine {
    decisions_made: u64,
    circuits_forwarded: u64,
    circuits_absorbed: u64,
}

impl DecisionEngine {
    pub fn new() -> Self {
        Self {
            decisions_made: 0,
            circuits_forwarded: 0,
            circuits_absorbed: 0,
        }
    }
    
    /// Make decision: Forward or Absorb
    pub fn decide(&mut self, resonance_score: f64, threshold: f64) -> CircuitAction {
        self.decisions_made += 1;
        
        if resonance_score > threshold {
            self.circuits_forwarded += 1;
            CircuitAction::Forward
        } else {
            self.circuits_absorbed += 1;
            CircuitAction::Absorb
        }
    }
    
    /// Get current absorption rate
    pub fn absorption_rate(&self) -> f64 {
        if self.decisions_made == 0 {
            return 0.0;
        }
        
        self.circuits_absorbed as f64 / self.decisions_made as f64
    }
    
    /// Get statistics
    pub fn statistics(&self) -> DecisionStatistics {
        DecisionStatistics {
            total_decisions: self.decisions_made,
            forwarded: self.circuits_forwarded,
            absorbed: self.circuits_absorbed,
            absorption_rate: self.absorption_rate(),
        }
    }
    
    /// Reset counters
    pub fn reset(&mut self) {
        self.decisions_made = 0;
        self.circuits_forwarded = 0;
        self.circuits_absorbed = 0;
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct DecisionStatistics {
    pub total_decisions: u64,
    pub forwarded: u64,
    pub absorbed: u64,
    pub absorption_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_engine() {
        let mut engine = DecisionEngine::new();
        
        // Test forward decision
        let action1 = engine.decide(0.8, 0.5);
        assert_eq!(action1, CircuitAction::Forward);
        
        // Test absorb decision
        let action2 = engine.decide(0.3, 0.5);
        assert_eq!(action2, CircuitAction::Absorb);
        
        let stats = engine.statistics();
        assert_eq!(stats.total_decisions, 2);
        assert_eq!(stats.forwarded, 1);
        assert_eq!(stats.absorbed, 1);
    }
}
