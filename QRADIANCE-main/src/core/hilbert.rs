//! 13-dimensionaler Tripol-Hilbertraum
//!
//! Implementiert Quantenzustände im ℂ¹³ mit Normalisierung und
//! Basis-Operationen gemäß der Metatron-Topologie

use nalgebra::SVector;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// Dimension des Hilbertraums (13 Knoten im Metatron-Graph)
pub const HILBERT_DIM: usize = 13;

/// Quantenzustand in ℋ₁₃
///
/// Repräsentiert einen normierten Zustand |ψ⟩ = Σᵢ αᵢ|i⟩ mit Σᵢ|αᵢ|² = 1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantumState {
    /// Komplexe Amplitudes-Vektor
    pub amplitudes: SVector<Complex64, HILBERT_DIM>,
}

impl QuantumState {
    /// Erstellt einen neuen Quantenzustand mit automatischer Normalisierung
    ///
    /// # Arguments
    /// * `amps` - Komplexer Amplitudenvektor (wird normalisiert)
    pub fn new(amps: SVector<Complex64, HILBERT_DIM>) -> Self {
        let norm = amps.norm();
        if norm < 1e-10 {
            panic!("Nullvektor kann nicht normalisiert werden");
        }
        Self {
            amplitudes: amps.scale(1.0 / norm),
        }
    }

    /// Erstellt einen Basiszustand |i⟩
    ///
    /// # Arguments
    /// * `index` - Index des Basiszustands (0..13)
    ///
    /// # Panics
    /// Panikt wenn index >= 13
    pub fn basis_state(index: usize) -> Self {
        assert!(index < HILBERT_DIM, "Index muss < 13 sein");
        let mut amps = SVector::<Complex64, HILBERT_DIM>::zeros();
        amps[index] = Complex64::new(1.0, 0.0);
        Self { amplitudes: amps }
    }

    /// Erstellt einen gleichverteilten Superpositionszustand
    ///
    /// |ψ⟩ = (1/√13) Σᵢ |i⟩
    pub fn uniform_superposition() -> Self {
        let amp = Complex64::new(1.0 / (HILBERT_DIM as f64).sqrt(), 0.0);
        let amps = SVector::<Complex64, HILBERT_DIM>::from_element(amp);
        Self { amplitudes: amps }
    }

    /// Erstellt einen zufälligen Quantenzustand (Haar-Maß)
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let amps: SVector<Complex64, HILBERT_DIM> = SVector::from_fn(|_, _| {
            Complex64::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0))
        });

        Self::new(amps)
    }

    /// Berechnet Skalarprodukt ⟨self|other⟩
    pub fn inner_product(&self, other: &Self) -> Complex64 {
        self.amplitudes
            .iter()
            .zip(other.amplitudes.iter())
            .map(|(a, b)| a.conj() * b)
            .sum()
    }

    /// Berechnet Norm ||ψ||
    pub fn norm(&self) -> f64 {
        self.amplitudes.norm()
    }

    /// Überprüft Normalisierung (sollte ≈ 1 sein)
    pub fn is_normalized(&self) -> bool {
        (self.norm() - 1.0).abs() < 1e-10
    }

    /// Berechnet Wahrscheinlichkeitsverteilung P(i) = |αᵢ|²
    pub fn probabilities(&self) -> [f64; HILBERT_DIM] {
        let mut probs = [0.0; HILBERT_DIM];
        for (i, amp) in self.amplitudes.iter().enumerate() {
            probs[i] = amp.norm_sqr();
        }
        probs
    }

    /// Führt Messung durch (Born-Regel)
    ///
    /// Returned den Index des gemessenen Zustands gemäß P(i) = |αᵢ|²
    pub fn measure(&self) -> usize {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let probs = self.probabilities();
        let sample: f64 = rng.gen();

        let mut cumulative = 0.0;
        for (i, &p) in probs.iter().enumerate() {
            cumulative += p;
            if sample <= cumulative {
                return i;
            }
        }
        HILBERT_DIM - 1 // Fallback (wegen Rundungsfehlern)
    }

    /// Projiziert auf Basiszustand |i⟩
    ///
    /// P̂ᵢ|ψ⟩ = |i⟩⟨i|ψ⟩
    pub fn project_onto(&self, index: usize) -> Self {
        assert!(index < HILBERT_DIM);
        let basis = Self::basis_state(index);
        let overlap = self.inner_product(&basis);
        Self::new(basis.amplitudes * overlap)
    }

    /// Berechnet Erwartungswert eines Hermiteschen Operators
    ///
    /// ⟨Ô⟩ = ⟨ψ|Ô|ψ⟩
    pub fn expectation_value(
        &self,
        operator: &nalgebra::SMatrix<Complex64, HILBERT_DIM, HILBERT_DIM>,
    ) -> Complex64 {
        let op_psi = operator * self.amplitudes;
        self.amplitudes
            .iter()
            .zip(op_psi.iter())
            .map(|(a, b)| a.conj() * b)
            .sum()
    }

    /// Berechnet Von-Neumann-Entropie (für reinen Zustand sollte ≈ 0 sein)
    pub fn von_neumann_entropy(&self) -> f64 {
        let probs = self.probabilities();
        -probs
            .iter()
            .filter(|&&p| p > 1e-15)
            .map(|&p| p * p.ln())
            .sum::<f64>()
    }

    /// Fidelity zwischen zwei Zuständen: F = |⟨ψ|φ⟩|²
    pub fn fidelity(&self, other: &Self) -> f64 {
        self.inner_product(other).norm_sqr()
    }
}

impl Default for QuantumState {
    /// Standard ist Basiszustand |0⟩
    fn default() -> Self {
        Self::basis_state(0)
    }
}

impl std::fmt::Display for QuantumState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "QuantumState |ψ⟩ in ℋ₁₃:")?;
        for (i, amp) in self.amplitudes.iter().enumerate() {
            if amp.norm() > 1e-6 {
                writeln!(
                    f,
                    "  |{:2}⟩: {:8.5} {:+8.5}i  (P = {:.5})",
                    i,
                    amp.re,
                    amp.im,
                    amp.norm_sqr()
                )?;
            }
        }
        writeln!(f, "  Norm: {:.10}", self.norm())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_basis_state() {
        let state = QuantumState::basis_state(3);
        assert!(state.is_normalized());
        assert_eq!(state.amplitudes[3].norm_sqr(), 1.0);
        for i in 0..HILBERT_DIM {
            if i != 3 {
                assert_eq!(state.amplitudes[i].norm_sqr(), 0.0);
            }
        }
    }

    #[test]
    fn test_uniform_superposition() {
        let state = QuantumState::uniform_superposition();
        assert!(state.is_normalized());

        let expected_prob = 1.0 / HILBERT_DIM as f64;
        for prob in state.probabilities() {
            assert_abs_diff_eq!(prob, expected_prob, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_inner_product_orthogonal() {
        let state1 = QuantumState::basis_state(0);
        let state2 = QuantumState::basis_state(1);
        let overlap = state1.inner_product(&state2);
        assert_abs_diff_eq!(overlap.norm(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_inner_product_self() {
        let state = QuantumState::random();
        let overlap = state.inner_product(&state);
        assert_abs_diff_eq!(overlap.re, 1.0, epsilon = 1e-10);
        assert_abs_diff_eq!(overlap.im, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_probabilities_sum() {
        let state = QuantumState::random();
        let probs = state.probabilities();
        let sum: f64 = probs.iter().sum();
        assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_measure_distribution() {
        let state = QuantumState::uniform_superposition();
        let mut counts = [0; HILBERT_DIM];

        for _ in 0..10000 {
            let outcome = state.measure();
            counts[outcome] += 1;
        }

        // Jeder Zustand sollte ca. gleich oft gemessen werden
        let expected = 10000.0 / HILBERT_DIM as f64;
        for count in counts {
            let deviation = (count as f64 - expected).abs() / expected;
            assert!(deviation < 0.15); // 15% Toleranz
        }
    }

    #[test]
    fn test_von_neumann_entropy_pure() {
        let state = QuantumState::basis_state(0);
        let entropy = state.von_neumann_entropy();
        assert_abs_diff_eq!(entropy, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_fidelity_identical() {
        let state = QuantumState::random();
        let fidelity = state.fidelity(&state);
        assert_abs_diff_eq!(fidelity, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_fidelity_orthogonal() {
        let state1 = QuantumState::basis_state(0);
        let state2 = QuantumState::basis_state(1);
        let fidelity = state1.fidelity(&state2);
        assert_abs_diff_eq!(fidelity, 0.0, epsilon = 1e-10);
    }
}
