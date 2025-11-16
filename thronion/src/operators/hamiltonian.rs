//! Metatron-Hamilton-Operator
//!
//! Implementiert Ĥ_M = -J·L + Σᵢ εᵢ|i⟩⟨i|
//! für Zeitevolution auf dem Metatron-Graph

use crate::core::{MetatronGraph, QuantumState, HILBERT_DIM};
use nalgebra::{SMatrix, SVector};
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// Metatron-Hamilton-Operator
///
/// Ĥ_M = -J·L + Σᵢ εᵢ|i⟩⟨i|
///
/// wobei:
/// - J: Hopping-Konstante (Kopplungsstärke)
/// - L: Graph-Laplacian
/// - εᵢ: Lokale Energien (on-site energies)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HamiltonOperator {
    /// Hopping-Konstante J > 0
    pub hopping_strength: f64,
    /// Lokale Energien εᵢ
    pub local_energies: [f64; HILBERT_DIM],
    /// Vollständige Hamilton-Matrix
    pub matrix: SMatrix<Complex64, HILBERT_DIM, HILBERT_DIM>,
}

impl HamiltonOperator {
    /// Erstellt einen neuen Hamilton-Operator
    ///
    /// # Arguments
    /// * `hopping_strength` - Kopplungsstärke J > 0
    /// * `local_energies` - On-site Energien εᵢ für jeden Knoten
    /// * `graph` - Metatron-Graph für Laplacian
    pub fn new(
        hopping_strength: f64,
        local_energies: [f64; HILBERT_DIM],
        graph: &MetatronGraph,
    ) -> Self {
        assert!(hopping_strength > 0.0, "J muss positiv sein");

        // Konstruiere Hamilton-Matrix: Ĥ = -J·L + diag(ε)
        let mut matrix = SMatrix::<Complex64, HILBERT_DIM, HILBERT_DIM>::zeros();

        // Term: -J·L
        for i in 0..HILBERT_DIM {
            for j in 0..HILBERT_DIM {
                matrix[(i, j)] += Complex64::new(-hopping_strength * graph.laplacian[(i, j)], 0.0);
            }
        }

        // Term: diag(ε)
        for i in 0..HILBERT_DIM {
            matrix[(i, i)] += Complex64::new(local_energies[i], 0.0);
        }

        Self {
            hopping_strength,
            local_energies,
            matrix,
        }
    }

    /// Erstellt Hamilton-Operator mit uniformen lokalen Energien
    pub fn uniform(hopping_strength: f64, energy: f64, graph: &MetatronGraph) -> Self {
        Self::new(hopping_strength, [energy; HILBERT_DIM], graph)
    }

    /// Erstellt Hamilton-Operator mit zufälligen lokalen Energien
    pub fn random_disorder(
        hopping_strength: f64,
        disorder_strength: f64,
        graph: &MetatronGraph,
    ) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let local_energies: [f64; HILBERT_DIM] =
            std::array::from_fn(|_| rng.gen_range(-disorder_strength..disorder_strength));

        Self::new(hopping_strength, local_energies, graph)
    }

    /// Wendet Hamilton-Operator auf Zustand an: Ĥ|ψ⟩
    pub fn apply(&self, state: &QuantumState) -> SVector<Complex64, HILBERT_DIM> {
        self.matrix * state.amplitudes
    }

    /// Berechnet Energie-Erwartungswert ⟨ψ|Ĥ|ψ⟩
    pub fn expectation_value(&self, state: &QuantumState) -> f64 {
        state.expectation_value(&self.matrix).re
    }

    /// Zeitevolution: |ψ(t)⟩ = exp(-iĤt/ℏ)|ψ(0)⟩
    ///
    /// Verwendet Matrix-Exponentiation
    ///
    /// # Arguments
    /// * `state` - Initialzustand |ψ(0)⟩
    /// * `time` - Evolutionszeit t (in natürlichen Einheiten ℏ=1)
    pub fn time_evolution(&self, state: &QuantumState, time: f64) -> QuantumState {
        let evolution_matrix = self.evolution_operator(time);
        let evolved_amps = evolution_matrix * state.amplitudes;
        QuantumState::new(evolved_amps)
    }

    /// Berechnet Evolutionsoperator U(t) = exp(-iĤt)
    pub fn evolution_operator(&self, time: f64) -> SMatrix<Complex64, HILBERT_DIM, HILBERT_DIM> {
        // Für kleine Matrizen: Diagonalisierung verwenden
        use nalgebra::SymmetricEigen;

        // Extrahiere Realteil (Hamilton ist Hermitesch, also reell)
        let h_real = self.matrix.map(|c| c.re);

        // Eigenwertzerlegung
        let eigen = SymmetricEigen::new(h_real);
        let eigenvalues = eigen.eigenvalues;
        let eigenvectors = eigen.eigenvectors;

        // Konstruiere exp(-iĤt) = V·exp(-iΛt)·V†
        let mut exp_matrix = SMatrix::<Complex64, HILBERT_DIM, HILBERT_DIM>::zeros();

        for i in 0..HILBERT_DIM {
            let phase = Complex64::new(0.0, -eigenvalues[i] * time).exp();

            for j in 0..HILBERT_DIM {
                for k in 0..HILBERT_DIM {
                    exp_matrix[(j, k)] += phase
                        * Complex64::new(eigenvectors[(j, i)], 0.0)
                        * Complex64::new(eigenvectors[(k, i)], 0.0);
                }
            }
        }

        exp_matrix
    }

    /// Berechnet Eigenenergien und Eigenzustände
    pub fn eigenspectrum(&self) -> EigenSpectrum {
        use nalgebra::SymmetricEigen;

        let h_real = self.matrix.map(|c| c.re);
        let eigen = SymmetricEigen::new(h_real);

        let mut energies = eigen.eigenvalues.as_slice().to_vec();
        energies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        EigenSpectrum {
            energies: energies.try_into().unwrap(),
            eigenvectors: eigen.eigenvectors,
        }
    }

    /// Berechnet spektrale Lücke (Δε = ε₁ - ε₀)
    pub fn spectral_gap(&self) -> f64 {
        let spectrum = self.eigenspectrum();
        spectrum.energies[1] - spectrum.energies[0]
    }
}

impl Default for HamiltonOperator {
    fn default() -> Self {
        let graph = MetatronGraph::new();
        Self::uniform(1.0, 0.0, &graph)
    }
}

/// Eigenspektrum des Hamilton-Operators
#[derive(Debug, Clone)]
pub struct EigenSpectrum {
    /// Eigenenergien (sortiert)
    pub energies: [f64; HILBERT_DIM],
    /// Eigenvektoren (Spalten der Matrix)
    pub eigenvectors: SMatrix<f64, HILBERT_DIM, HILBERT_DIM>,
}

impl EigenSpectrum {
    /// Gibt Grundzustand-Energie zurück
    pub fn ground_state_energy(&self) -> f64 {
        self.energies[0]
    }

    /// Gibt angeregten Zustand zurück
    pub fn excited_state(&self, n: usize) -> QuantumState {
        assert!(n < HILBERT_DIM);
        let eigenvector = self.eigenvectors.column(n);
        let amps =
            SVector::<Complex64, HILBERT_DIM>::from_fn(|i, _| Complex64::new(eigenvector[i], 0.0));
        QuantumState::new(amps)
    }

    /// Berechnet Zustandsdichte bei Energie E
    pub fn density_of_states(&self, energy: f64, broadening: f64) -> f64 {
        self.energies
            .iter()
            .map(|&e| {
                let delta_e = energy - e;
                (broadening / std::f64::consts::PI) / (delta_e.powi(2) + broadening.powi(2))
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_hamilton_creation() {
        let graph = MetatronGraph::new();
        let ham = HamiltonOperator::uniform(1.0, 0.0, &graph);
        assert_eq!(ham.hopping_strength, 1.0);
    }

    #[test]
    fn test_hermiticity() {
        let graph = MetatronGraph::new();
        let ham = HamiltonOperator::uniform(1.0, 0.5, &graph);

        // Hamilton sollte Hermitesch sein: H† = H
        for i in 0..HILBERT_DIM {
            for j in 0..HILBERT_DIM {
                let h_ij = ham.matrix[(i, j)];
                let h_ji_conj = ham.matrix[(j, i)].conj();
                assert_abs_diff_eq!(h_ij.re, h_ji_conj.re, epsilon = 1e-10);
                assert_abs_diff_eq!(h_ij.im, h_ji_conj.im, epsilon = 1e-10);
            }
        }
    }

    #[test]
    fn test_energy_conservation() {
        let graph = MetatronGraph::new();
        let ham = HamiltonOperator::uniform(1.0, 0.0, &graph);
        let state = QuantumState::random();

        let e0 = ham.expectation_value(&state);

        // Evolution sollte Energie erhalten
        let evolved = ham.time_evolution(&state, 1.0);
        let e1 = ham.expectation_value(&evolved);

        assert_abs_diff_eq!(e0, e1, epsilon = 1e-6);
    }

    #[test]
    fn test_unitarity() {
        let graph = MetatronGraph::new();
        let ham = HamiltonOperator::uniform(1.0, 0.0, &graph);
        let u = ham.evolution_operator(1.0);

        // U sollte unitär sein: U†U = I
        let u_dagger_u = u.adjoint() * u;
        let identity = SMatrix::<Complex64, HILBERT_DIM, HILBERT_DIM>::identity();

        for i in 0..HILBERT_DIM {
            for j in 0..HILBERT_DIM {
                assert_abs_diff_eq!(u_dagger_u[(i, j)].re, identity[(i, j)].re, epsilon = 1e-8);
                assert_abs_diff_eq!(u_dagger_u[(i, j)].im, 0.0, epsilon = 1e-8);
            }
        }
    }

    #[test]
    fn test_eigenspectrum() {
        let graph = MetatronGraph::new();
        let ham = HamiltonOperator::uniform(1.0, 0.0, &graph);
        let spectrum = ham.eigenspectrum();

        // Eigenenergien sollten sortiert sein
        for i in 0..(HILBERT_DIM - 1) {
            assert!(spectrum.energies[i] <= spectrum.energies[i + 1]);
        }
    }

    #[test]
    fn test_spectral_gap() {
        let graph = MetatronGraph::new();
        let ham = HamiltonOperator::uniform(1.0, 0.0, &graph);
        let gap = ham.spectral_gap();

        // Spektrale Lücke sollte positiv sein
        assert!(gap > 0.0);
    }
}
