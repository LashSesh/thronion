//! Nullpunkt-Operator (K_N)
//!
//! Implementiert den sterilen Naht-Operator:
//! K_N = lim_{k→∞} (∏ᵢ₌₁ᵏ Uᵢ·Dᵢ) · P_sterile

use crate::core::{MetatronGraph, QuantumState, HILBERT_DIM};
use crate::operators::omega5::{DampingOperator, ScalingOperator};
use nalgebra::SMatrix;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// Nullpunkt-Operator K_N
///
/// Resetzt System in stabilen "sterilen" Unterraum wenn Guards verletzt werden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullpointOperator {
    /// Projektions-Operator auf sterilen Unterraum
    pub sterile_projector: SMatrix<Complex64, HILBERT_DIM, HILBERT_DIM>,
    /// Anzahl der Iterationen für Konvergenz
    pub num_iterations: usize,
}

impl NullpointOperator {
    /// Erstellt Nullpunkt-Operator
    ///
    /// # Arguments
    /// * `graph` - Metatron-Graph
    /// * `num_iterations` - Anzahl der Iterationen k
    pub fn new(graph: &MetatronGraph, num_iterations: usize) -> Self {
        // Konstruiere sterilen Projektor: Projiziert auf Grundzustand
        let sterile_projector = Self::construct_sterile_projector(graph);

        Self {
            sterile_projector,
            num_iterations,
        }
    }

    /// Konstruiert sterilen Projektor
    ///
    /// Projiziert auf niedrigenergetischen invarianten Unterraum
    fn construct_sterile_projector(
        graph: &MetatronGraph,
    ) -> SMatrix<Complex64, HILBERT_DIM, HILBERT_DIM> {
        use nalgebra::SymmetricEigen;

        // Verwende Laplacian-Eigenvektoren
        let eigen = SymmetricEigen::new(graph.laplacian);

        // Projiziere auf die ersten 3 Eigenvektoren (niedrigste Energie)
        let mut projector = SMatrix::<Complex64, HILBERT_DIM, HILBERT_DIM>::zeros();

        for mode in 0..3 {
            for i in 0..HILBERT_DIM {
                for j in 0..HILBERT_DIM {
                    projector[(i, j)] += Complex64::from(
                        eigen.eigenvectors[(i, mode)] * eigen.eigenvectors[(j, mode)],
                    );
                }
            }
        }

        projector
    }

    /// Wendet K_N-Operator an
    ///
    /// Iteriert: ψ_{k+1} = D_k · U_k · ψ_k, dann projiziert auf sterilen Raum
    pub fn apply(&self, state: &QuantumState, graph: &MetatronGraph) -> QuantumState {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut current_state = state.clone();

        // Iteriere mit zufälligen Unitären und Dämpfungen
        for _ in 0..self.num_iterations {
            // Zufällige Dämpfung
            let mu = rng.gen_range(0.01..0.1);
            let damping = DampingOperator::new(mu, graph);
            current_state = damping.apply(&current_state);

            // Zufällige Skalierung
            let lambda = rng.gen_range(-0.1..0.1);
            let scaling = ScalingOperator::uniform(lambda, 0.0);
            current_state = scaling.apply(&current_state);
        }

        // Finale Projektion auf sterilen Unterraum
        let sterile_amps = self.sterile_projector * current_state.amplitudes;
        QuantumState::new(sterile_amps)
    }

    /// Prüft ob Zustand im sterilen Unterraum liegt
    pub fn is_sterile(&self, state: &QuantumState, tolerance: f64) -> bool {
        let projected = self.sterile_projector * state.amplitudes;
        let diff = (state.amplitudes - projected).norm();
        diff < tolerance
    }

    /// Berechnet Overlap mit sterilem Unterraum
    pub fn sterile_overlap(&self, state: &QuantumState) -> f64 {
        let projected = self.sterile_projector * state.amplitudes;
        let overlap = state
            .amplitudes
            .iter()
            .zip(projected.iter())
            .map(|(a, b)| (a.conj() * b).norm())
            .sum::<f64>();
        overlap / (HILBERT_DIM as f64)
    }
}

impl Default for NullpointOperator {
    fn default() -> Self {
        let graph = MetatronGraph::new();
        Self::new(&graph, 10)
    }
}

/// Guard-Validator
///
/// Prüft topologische Invarianten und triggert K_N bei Verletzung
#[derive(Debug, Clone)]
pub struct GuardValidator {
    /// Minimale Betti-Zahl
    pub min_betti: f64,
    /// Minimale spektrale Lücke
    pub min_gap: f64,
    /// Maximaler Kohärenzgradient
    pub max_gradient: f64,
    /// Nullpunkt-Operator für Reset
    pub nullpoint: NullpointOperator,
}

impl GuardValidator {
    /// Erstellt Guard-Validator mit Standardwerten
    pub fn new(graph: &MetatronGraph) -> Self {
        Self {
            min_betti: 1.0,
            min_gap: 0.1,
            max_gradient: 0.01,
            nullpoint: NullpointOperator::new(graph, 10),
        }
    }

    /// Validiert alle Guards
    pub fn validate(&self, betti: f64, spectral_gap: f64, coherence_gradient: f64) -> GuardStatus {
        let mut violations = Vec::new();

        if betti < self.min_betti {
            violations.push(GuardViolation::BettiTooLow(betti));
        }

        if spectral_gap < self.min_gap {
            violations.push(GuardViolation::SpectralGapTooSmall(spectral_gap));
        }

        if coherence_gradient > self.max_gradient {
            violations.push(GuardViolation::CoherenceGradientTooLarge(
                coherence_gradient,
            ));
        }

        if violations.is_empty() {
            GuardStatus::Valid
        } else {
            GuardStatus::Violated(violations)
        }
    }

    /// Führt bedingten Reset durch wenn Guards verletzt sind
    pub fn conditional_reset(
        &self,
        state: &QuantumState,
        betti: f64,
        spectral_gap: f64,
        coherence_gradient: f64,
        graph: &MetatronGraph,
    ) -> (QuantumState, bool) {
        match self.validate(betti, spectral_gap, coherence_gradient) {
            GuardStatus::Valid => (state.clone(), false),
            GuardStatus::Violated(_) => {
                let reset_state = self.nullpoint.apply(state, graph);
                (reset_state, true)
            }
        }
    }
}

/// Guard-Status
#[derive(Debug, Clone)]
pub enum GuardStatus {
    /// Alle Guards erfüllt
    Valid,
    /// Guards verletzt
    Violated(Vec<GuardViolation>),
}

/// Guard-Verletzungs-Typen
#[derive(Debug, Clone)]
pub enum GuardViolation {
    /// Betti-Zahl zu niedrig
    BettiTooLow(f64),
    /// Spektrale Lücke zu klein
    SpectralGapTooSmall(f64),
    /// Kohärenzgradient zu groß
    CoherenceGradientTooLarge(f64),
}

impl std::fmt::Display for GuardViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuardViolation::BettiTooLow(val) => {
                write!(f, "Betti-Zahl zu niedrig: {:.4}", val)
            }
            GuardViolation::SpectralGapTooSmall(val) => {
                write!(f, "Spektrale Lücke zu klein: {:.4}", val)
            }
            GuardViolation::CoherenceGradientTooLarge(val) => {
                write!(f, "Kohärenzgradient zu groß: {:.4}", val)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_nullpoint_creation() {
        let graph = MetatronGraph::new();
        let kn = NullpointOperator::new(&graph, 5);
        assert_eq!(kn.num_iterations, 5);
    }

    #[test]
    fn test_nullpoint_application() {
        let graph = MetatronGraph::new();
        let kn = NullpointOperator::new(&graph, 10);
        let state = QuantumState::random();

        let sterile_state = kn.apply(&state, &graph);

        // Zustand sollte normalisiert bleiben
        assert!(sterile_state.is_normalized());

        // Overlap mit sterilem Unterraum sollte hoch sein
        let overlap = kn.sterile_overlap(&sterile_state);
        assert!(overlap > 0.5);
    }

    #[test]
    fn test_sterile_check() {
        let graph = MetatronGraph::new();
        let kn = NullpointOperator::new(&graph, 10);
        let state = QuantumState::random();

        let sterile_state = kn.apply(&state, &graph);

        // Nach Anwendung sollte Zustand sterieler sein
        let overlap_before = kn.sterile_overlap(&state);
        let overlap_after = kn.sterile_overlap(&sterile_state);

        assert!(overlap_after >= overlap_before);
    }

    #[test]
    fn test_guard_validator_valid() {
        let graph = MetatronGraph::new();
        let validator = GuardValidator::new(&graph);

        let status = validator.validate(2.0, 0.5, 0.005);
        assert!(matches!(status, GuardStatus::Valid));
    }

    #[test]
    fn test_guard_validator_violations() {
        let graph = MetatronGraph::new();
        let validator = GuardValidator::new(&graph);

        // Verletzt Betti-Bedingung
        let status = validator.validate(0.5, 0.5, 0.005);
        assert!(matches!(status, GuardStatus::Violated(_)));

        // Verletzt spektrale Lücke
        let status = validator.validate(2.0, 0.05, 0.005);
        assert!(matches!(status, GuardStatus::Violated(_)));

        // Verletzt Kohärenzgradient
        let status = validator.validate(2.0, 0.5, 0.05);
        assert!(matches!(status, GuardStatus::Violated(_)));
    }

    #[test]
    fn test_conditional_reset() {
        let graph = MetatronGraph::new();
        let validator = GuardValidator::new(&graph);
        let state = QuantumState::random();

        // Gültige Guards: Kein Reset
        let (new_state, was_reset) = validator.conditional_reset(&state, 2.0, 0.5, 0.005, &graph);
        assert!(!was_reset);
        assert_abs_diff_eq!(new_state.fidelity(&state), 1.0, epsilon = 1e-10);

        // Verletzte Guards: Reset
        let (reset_state, was_reset) = validator.conditional_reset(&state, 0.5, 0.05, 0.05, &graph);
        assert!(was_reset);
        assert!(reset_state.is_normalized());
    }
}
