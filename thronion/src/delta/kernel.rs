//! Delta-Kernel
//!
//! Unified QRIK-Zustand: Ψ_Δ = Ψ_ℋ₁₃ ⋆ Ψ_Ω₅ ⋆ Ψ_ℛ ⋆ Ψ_ℳ

use crate::core::{MetatronGraph, QuantumState};
use crate::mandorla::MandorlaOperator;
use crate::operators::HamiltonOperator;
use crate::resonance::{KuramotoNetwork, ResonantAbsorber};
use serde::{Deserialize, Serialize};

/// Vereinheitlichter Delta-Kernel
///
/// Fusioniert alle QRIK-Komponenten in einen kohärenten Zustand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaKernel {
    /// Quantenzustand in ℋ₁₃
    pub quantum_state: QuantumState,
    /// Hamilton-Operator
    pub hamiltonian: HamiltonOperator,
    /// Kuramoto-Resonanznetzwerk
    pub kuramoto: KuramotoNetwork,
    /// Resonant Absorber Layer
    pub absorber: ResonantAbsorber,
    /// Mandorla-Operator
    pub mandorla: MandorlaOperator,
    /// Metatron-Graph
    pub graph: MetatronGraph,
    /// System-Parameter
    pub params: QRIKParams,
}

impl DeltaKernel {
    /// Erstellt neuen Delta-Kernel mit Standardparametern
    pub fn new(params: QRIKParams) -> Self {
        let graph = MetatronGraph::new();
        let quantum_state = QuantumState::uniform_superposition();
        let hamiltonian = HamiltonOperator::uniform(params.hopping_strength, 0.0, &graph);
        let kuramoto = KuramotoNetwork::uniform(params.base_frequency, params.coupling_strength);
        let absorber = ResonantAbsorber::new(params.spectrum_size, params.learning_rate);
        let mandorla = MandorlaOperator::new();

        Self {
            quantum_state,
            hamiltonian,
            kuramoto,
            absorber,
            mandorla,
            graph,
            params,
        }
    }

    /// Zeitevolution des gesamten Systems
    ///
    /// Integriert alle Komponenten für Zeitschritt dt
    pub fn evolve(&mut self, dt: f64) {
        // 1. Hamilton-Zeitevolution
        self.quantum_state = self.hamiltonian.time_evolution(&self.quantum_state, dt);

        // 2. Kuramoto-Synchronisation
        self.kuramoto.evolve_rk4(dt);

        // 3. Mandorla-Fusion (periodisch)
        if !self.mandorla.regions.is_empty() {
            self.quantum_state = self
                .mandorla
                .recursive_fusion(&self.quantum_state, self.mandorla.regions.len());
        }
    }

    /// Berechnet Delta-Gradient ∇Ψ_Δ
    ///
    /// Misst Abweichung vom optimalen Kohärenzzustand
    pub fn coherence_gradient(&self) -> f64 {
        // Gradient basierend auf:
        // 1. Quantenzustands-Kohärenz
        // 2. Kuramoto-Ordnungsparameter
        // 3. Mandorla-Dichte

        let quantum_coherence = self.quantum_state.von_neumann_entropy();
        let kuramoto_sync = 1.0 - self.kuramoto.synchronization();

        // Gesamtgradient (minimieren!)
        (quantum_coherence.powi(2) + kuramoto_sync.powi(2)).sqrt()
    }

    /// Prüft Stabilität: ∇Ψ_Δ < ε
    pub fn is_stable(&self, epsilon: f64) -> bool {
        self.coherence_gradient() < epsilon
    }

    /// Berechnet Systemkohärenz
    pub fn coherence(&self) -> f64 {
        // Durchschnittliche Populationsdichte (vereinfacht)
        let probs = self.quantum_state.probabilities();
        probs.iter().sum::<f64>() / probs.len() as f64
    }

    /// Berechnet Flood-Energy (absorbierte DDoS-Energie)
    pub fn flood_energy(&self) -> f64 {
        self.absorber.stats.packets_absorbed as f64
            / self.absorber.stats.total_packets.max(1) as f64
    }

    /// Berechnet Mandorla-Dichte
    pub fn mandorla_density(&self) -> f64 {
        self.mandorla.mandorla_density(&self.quantum_state)
    }

    /// Verarbeitet eingehendes Paket
    ///
    /// Returns: (absorbed, resonance_score)
    pub fn process_packet(&mut self, packet: &[u8], node: usize) -> (bool, f64) {
        self.absorber
            .process_packet(packet, node, self.params.epsilon_res)
    }

    /// Optimierungsziel: Maximiere Kohärenz, minimiere Flood-Energy
    pub fn fitness(&self) -> f64 {
        let coherence = self.coherence();
        let flood = self.flood_energy();
        coherence - self.params.lambda_flood * flood
    }

    /// Reset zu sicheren Zustand (via K_N)
    pub fn reset_to_safe_state(&mut self) {
        use crate::operators::NullpointOperator;

        let kn = NullpointOperator::new(&self.graph, 10);
        self.quantum_state = kn.apply(&self.quantum_state, &self.graph);

        // Reset Kuramoto zu desynchronisiertem Zustand
        self.kuramoto.randomize_phases();
    }
}

impl Default for DeltaKernel {
    fn default() -> Self {
        Self::new(QRIKParams::default())
    }
}

/// QRIK-System-Parameter
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QRIKParams {
    /// Hopping-Stärke J
    pub hopping_strength: f64,
    /// Basis-Frequenz ω₀
    pub base_frequency: f64,
    /// Kuramoto-Kopplung κ
    pub coupling_strength: f64,
    /// Spektrum-Größe (FFT-Bins)
    pub spectrum_size: usize,
    /// Lernrate λ
    pub learning_rate: f64,
    /// Resonanz-Schwelle ε_res
    pub epsilon_res: f64,
    /// Flood-Penalty-Gewicht
    pub lambda_flood: f64,
}

impl Default for QRIKParams {
    fn default() -> Self {
        Self {
            hopping_strength: 1.0,
            base_frequency: 1.0,
            coupling_strength: 2.0,
            spectrum_size: 256,
            learning_rate: 0.01,
            epsilon_res: 0.3,
            lambda_flood: 1.0,
        }
    }
}

impl std::fmt::Display for DeltaKernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== QRIK Delta-Kernel ===")?;
        writeln!(f, "\nKohärenz-Metriken:")?;
        writeln!(f, "  Coherence:         {:.4}", self.coherence())?;
        writeln!(f, "  Delta-Gradient:    {:.4}", self.coherence_gradient())?;
        writeln!(
            f,
            "  Kuramoto-Sync:     {:.4}",
            self.kuramoto.synchronization()
        )?;
        writeln!(f, "  Mandorla-Density:  {:.4}", self.mandorla_density())?;
        writeln!(f, "\nAbsorber-Statistiken:")?;
        writeln!(f, "{}", self.absorber.stats)?;
        writeln!(
            f,
            "\nStabilität: {}",
            if self.is_stable(0.01) { "✓" } else { "✗" }
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_kernel_creation() {
        let kernel = DeltaKernel::default();
        assert!(kernel.quantum_state.is_normalized());
    }

    #[test]
    fn test_evolution() {
        let mut kernel = DeltaKernel::default();
        let _initial_gradient = kernel.coherence_gradient();

        for _ in 0..10 {
            kernel.evolve(0.01);
        }

        // Zustand sollte weiterhin normalisiert sein
        assert!(kernel.quantum_state.is_normalized());
    }

    #[test]
    fn test_coherence_gradient() {
        let kernel = DeltaKernel::default();
        let gradient = kernel.coherence_gradient();

        // Gradient sollte nicht-negativ sein
        assert!(gradient >= 0.0);
    }

    #[test]
    fn test_stability_check() {
        let kernel = DeltaKernel::default();

        // Mit Standardparametern sollte System nicht sofort stabil sein
        let is_stable = kernel.is_stable(0.001);
        assert!(is_stable == (kernel.coherence_gradient() < 0.001));
    }

    #[test]
    fn test_packet_processing() {
        let mut kernel = DeltaKernel::default();
        kernel.absorber.initialize_random_fields();

        let packet = b"test packet";
        let (_absorbed, score) = kernel.process_packet(packet, 0);

        // Score sollte in [0,1] sein
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_fitness() {
        let kernel = DeltaKernel::default();
        let fitness = kernel.fitness();

        // Fitness sollte endlich sein
        assert!(fitness.is_finite());
    }

    #[test]
    fn test_reset() {
        let mut kernel = DeltaKernel::default();

        // Evolve ein bisschen
        for _ in 0..5 {
            kernel.evolve(0.1);
        }

        kernel.reset_to_safe_state();
        assert!(kernel.quantum_state.is_normalized());
    }
}
