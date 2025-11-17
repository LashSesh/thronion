//! Kuramoto-Modell auf Metatron-Graph
//!
//! Implementiert Resonator-Dynamik und Synchronisationsphänomene
//!
//! dφᵢ/dt = ωᵢ + Σⱼ κᵢⱼ sin(φⱼ - φᵢ)

use crate::core::{MetatronGraph, NUM_NODES};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Kuramoto-Netzwerk auf Metatron-Topologie
///
/// Jeder Knoten trägt Phase φᵢ(t) ∈ S¹ mit gekoppelter Dynamik
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KuramotoNetwork {
    /// Phasen φᵢ(t) für jeden Knoten
    pub phases: [f64; NUM_NODES],
    /// Intrinsische Frequenzen ωᵢ
    pub frequencies: [f64; NUM_NODES],
    /// Kopplungsmatrix κᵢⱼ (symmetrisch, basierend auf Graph)
    pub coupling_matrix: [[f64; NUM_NODES]; NUM_NODES],
    /// Metatron-Graph-Struktur
    graph: MetatronGraph,
}

impl KuramotoNetwork {
    /// Erstellt neues Kuramoto-Netzwerk
    ///
    /// # Arguments
    /// * `frequencies` - Intrinsische Frequenzen ωᵢ
    /// * `coupling_strength` - Globale Kopplungsstärke κ
    pub fn new(frequencies: [f64; NUM_NODES], coupling_strength: f64) -> Self {
        let graph = MetatronGraph::new();

        // Konstruiere Kopplungsmatrix basierend auf Graph-Adjazenz
        let mut coupling_matrix = [[0.0; NUM_NODES]; NUM_NODES];
        for i in 0..NUM_NODES {
            for j in 0..NUM_NODES {
                if graph.adjacency[(i, j)] {
                    coupling_matrix[i][j] = coupling_strength;
                }
            }
        }

        // Zufällige Initialphasen
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let phases = std::array::from_fn(|_| rng.gen_range(0.0..2.0 * PI));

        Self {
            phases,
            frequencies,
            coupling_matrix,
            graph,
        }
    }

    /// Erstellt Netzwerk mit uniformen Frequenzen
    pub fn uniform(base_frequency: f64, coupling_strength: f64) -> Self {
        Self::new([base_frequency; NUM_NODES], coupling_strength)
    }

    /// Erstellt Netzwerk mit Frequenz-Dispersion
    ///
    /// ωᵢ ~ N(ω₀, σ²)
    pub fn with_frequency_disorder(
        base_frequency: f64,
        disorder_std: f64,
        coupling_strength: f64,
    ) -> Self {
        use rand_distr::{Distribution, Normal};
        let mut rng = rand::thread_rng();

        let normal = Normal::new(base_frequency, disorder_std).unwrap();
        let frequencies: [f64; NUM_NODES] = std::array::from_fn(|_| normal.sample(&mut rng));

        Self::new(frequencies, coupling_strength)
    }

    /// Randomisiert Phasen
    pub fn randomize_phases(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for phase in &mut self.phases {
            *phase = rng.gen_range(0.0..2.0 * PI);
        }
    }

    /// Zeitschritt: Euler-Integration
    ///
    /// φᵢ(t+dt) = φᵢ(t) + (dφᵢ/dt)·dt
    pub fn evolve(&mut self, dt: f64) {
        let mut derivatives = [0.0; NUM_NODES];

        // Berechne dφᵢ/dt für alle Knoten
        for i in 0..NUM_NODES {
            derivatives[i] = self.frequencies[i];

            for j in 0..NUM_NODES {
                if self.coupling_matrix[i][j] > 0.0 {
                    derivatives[i] +=
                        self.coupling_matrix[i][j] * (self.phases[j] - self.phases[i]).sin();
                }
            }
        }

        // Update Phasen
        for i in 0..NUM_NODES {
            self.phases[i] += derivatives[i] * dt;
            self.phases[i] = self.phases[i].rem_euclid(2.0 * PI);
        }
    }

    /// Runge-Kutta 4. Ordnung (genauer als Euler)
    pub fn evolve_rk4(&mut self, dt: f64) {
        let k1 = self.compute_derivatives(&self.phases);

        let mut phases_temp = self.phases;
        for i in 0..NUM_NODES {
            phases_temp[i] += 0.5 * dt * k1[i];
        }
        let k2 = self.compute_derivatives(&phases_temp);

        phases_temp = self.phases;
        for i in 0..NUM_NODES {
            phases_temp[i] += 0.5 * dt * k2[i];
        }
        let k3 = self.compute_derivatives(&phases_temp);

        phases_temp = self.phases;
        for i in 0..NUM_NODES {
            phases_temp[i] += dt * k3[i];
        }
        let k4 = self.compute_derivatives(&phases_temp);

        // Update
        for i in 0..NUM_NODES {
            self.phases[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            self.phases[i] = self.phases[i].rem_euclid(2.0 * PI);
        }
    }

    /// Berechnet Ableitungen dφᵢ/dt
    fn compute_derivatives(&self, phases: &[f64; NUM_NODES]) -> [f64; NUM_NODES] {
        let mut derivatives = [0.0; NUM_NODES];

        for i in 0..NUM_NODES {
            derivatives[i] = self.frequencies[i];

            for j in 0..NUM_NODES {
                if self.coupling_matrix[i][j] > 0.0 {
                    derivatives[i] += self.coupling_matrix[i][j] * (phases[j] - phases[i]).sin();
                }
            }
        }

        derivatives
    }

    /// Berechnet Kuramoto-Ordnungsparameter
    ///
    /// r(t)·e^(iΘ) = (1/N)Σᵢ e^(iφᵢ)
    ///
    /// Returns: (r, Θ)
    pub fn order_parameter(&self) -> (f64, f64) {
        let z: Complex64 = self
            .phases
            .iter()
            .map(|&phi| Complex64::from_polar(1.0, phi))
            .sum::<Complex64>()
            / (NUM_NODES as f64);

        (z.norm(), z.arg())
    }

    /// Gibt nur r zurück (Synchronisationsgrad)
    pub fn synchronization(&self) -> f64 {
        self.order_parameter().0
    }

    /// Berechnet kritische Kopplung für Synchronisation
    ///
    /// κ_c ≈ 2|ω_max|/λ₂
    pub fn critical_coupling(&self) -> f64 {
        let omega_max = self
            .frequencies
            .iter()
            .map(|&f| f.abs())
            .fold(0.0, f64::max);
        let lambda2 = self.graph.algebraic_connectivity();

        2.0 * omega_max / lambda2
    }

    /// Prüft ob Netzwerk synchronisiert ist
    pub fn is_synchronized(&self, threshold: f64) -> bool {
        self.synchronization() > threshold
    }

    /// Berechnet lokale Ordnungsparameter für jeden Knoten
    pub fn local_order_parameters(&self) -> [f64; NUM_NODES] {
        let mut local_r = [0.0; NUM_NODES];

        for i in 0..NUM_NODES {
            let neighbors = self.graph.neighbors(i);
            if neighbors.is_empty() {
                local_r[i] = 1.0;
                continue;
            }

            let z: Complex64 = neighbors
                .iter()
                .map(|&j| Complex64::from_polar(1.0, self.phases[j]))
                .sum::<Complex64>()
                / (neighbors.len() as f64);

            local_r[i] = z.norm();
        }

        local_r
    }

    /// Berechnet Phasen-Kohärenz-Matrix
    pub fn phase_coherence_matrix(&self) -> [[f64; NUM_NODES]; NUM_NODES] {
        let mut coherence = [[0.0; NUM_NODES]; NUM_NODES];

        for i in 0..NUM_NODES {
            for j in 0..NUM_NODES {
                // Kohärenz: cos(Δφ)
                coherence[i][j] = (self.phases[i] - self.phases[j]).cos();
            }
        }

        coherence
    }
}

impl Default for KuramotoNetwork {
    fn default() -> Self {
        Self::uniform(0.0, 1.0)
    }
}

impl std::fmt::Display for KuramotoNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (r, theta) = self.order_parameter();
        writeln!(f, "Kuramoto-Netzwerk (N={}):", NUM_NODES)?;
        writeln!(f, "  Ordnungsparameter r: {:.4}", r)?;
        writeln!(f, "  Globale Phase Θ: {:.4} rad", theta)?;
        writeln!(f, "  Synchronisiert: {}", self.is_synchronized(0.9))?;
        writeln!(f, "  Kritische Kopplung: {:.4}", self.critical_coupling())?;

        writeln!(f, "\nPhasen:")?;
        for (i, &phi) in self.phases.iter().enumerate() {
            writeln!(
                f,
                "  φ{:2} = {:.4} rad  (ω = {:.4})",
                i, phi, self.frequencies[i]
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_kuramoto_creation() {
        let network = KuramotoNetwork::uniform(1.0, 2.0);
        assert_eq!(network.phases.len(), NUM_NODES);
        assert_eq!(network.frequencies.len(), NUM_NODES);
    }

    #[test]
    fn test_order_parameter_desynchronized() {
        let mut network = KuramotoNetwork::uniform(0.0, 0.0);
        network.randomize_phases();

        let (r, _) = network.order_parameter();
        // Zufällige Phasen sollten r ≈ 0 haben
        assert!(r < 0.5);
    }

    #[test]
    fn test_order_parameter_synchronized() {
        let mut network = KuramotoNetwork::uniform(0.0, 0.0);
        // Alle Phasen gleich
        for phase in &mut network.phases {
            *phase = 0.0;
        }

        let (r, _) = network.order_parameter();
        assert_abs_diff_eq!(r, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_evolution_preserves_range() {
        let mut network = KuramotoNetwork::uniform(1.0, 1.0);

        for _ in 0..100 {
            network.evolve(0.01);
        }

        // Alle Phasen sollten in [0, 2π] bleiben
        for &phase in &network.phases {
            assert!(phase >= 0.0 && phase < 2.0 * PI);
        }
    }

    #[test]
    fn test_synchronization_convergence() {
        // Starke Kopplung sollte zu Synchronisation führen
        let mut network = KuramotoNetwork::uniform(1.0, 10.0);
        network.randomize_phases();

        let r_initial = network.synchronization();

        for _ in 0..1000 {
            network.evolve_rk4(0.01);
        }

        let r_final = network.synchronization();

        // r sollte zunehmen (Synchronisation)
        assert!(r_final > r_initial);
    }

    #[test]
    fn test_critical_coupling() {
        let network = KuramotoNetwork::uniform(1.0, 1.0);
        let kappa_c = network.critical_coupling();

        // κ_c sollte positiv sein
        assert!(kappa_c > 0.0);
    }

    #[test]
    fn test_local_order_parameters() {
        let network = KuramotoNetwork::uniform(0.0, 1.0);
        let local_r = network.local_order_parameters();

        // Alle lokalen r sollten in [0,1] sein
        for &r in &local_r {
            assert!(r >= 0.0 && r <= 1.0);
        }
    }

    #[test]
    fn test_phase_coherence() {
        let network = KuramotoNetwork::uniform(0.0, 1.0);
        let coherence = network.phase_coherence_matrix();

        // Kohärenz sollte symmetrisch sein
        for i in 0..NUM_NODES {
            for j in 0..NUM_NODES {
                assert_abs_diff_eq!(coherence[i][j], coherence[j][i], epsilon = 1e-10);
            }
        }

        // Diagonale sollte 1 sein
        for i in 0..NUM_NODES {
            assert_abs_diff_eq!(coherence[i][i], 1.0, epsilon = 1e-10);
        }
    }
}
