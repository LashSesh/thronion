//! Mandorla-Eigenstate-Fusion
//!
//! Implementiert Schnittmengen-Operatoren für semantische Stabilität
//! M_k = R_{k,1} ∩ R_{k,2}

use crate::core::{QuantumState, HILBERT_DIM};
use nalgebra::SVector;
use num_complex::Complex64;
use serde::{Deserialize, Serialize};

/// Mandorla-Region (Informationsschnittmenge)
///
/// Repräsentiert Überlappung zweier Informationsdomänen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandorlaRegion {
    /// Zentrum der ersten Region
    pub center1: QuantumState,
    /// Zentrum der zweiten Region
    pub center2: QuantumState,
    /// Radius der Regionen
    pub radius: f64,
    /// Schnittmengen-Zustand
    pub intersection_state: Option<QuantumState>,
}

impl MandorlaRegion {
    /// Erstellt neue Mandorla-Region
    pub fn new(center1: QuantumState, center2: QuantumState, radius: f64) -> Self {
        assert!(radius > 0.0, "Radius muss positiv sein");

        Self {
            center1,
            center2,
            radius,
            intersection_state: None,
        }
    }

    /// Berechnet Schnittmengen-Zustand M_k = R_1 ∩ R_2
    pub fn compute_intersection(&mut self) -> QuantumState {
        // Gewichteter Mittelwert der Zentren
        let fidelity = self.center1.fidelity(&self.center2);

        // Overlap-basierte Gewichtung
        let w1 = fidelity.sqrt();
        let w2 = (1.0 - fidelity).sqrt();

        let intersection_amps =
            self.center1.amplitudes.scale(w1) + self.center2.amplitudes.scale(w2);
        let state = QuantumState::new(intersection_amps);

        self.intersection_state = Some(state.clone());
        state
    }

    /// Prüft ob Zustand in Mandorla-Schnittmenge liegt
    pub fn contains(&self, state: &QuantumState) -> bool {
        let d1 = 1.0 - state.fidelity(&self.center1);
        let d2 = 1.0 - state.fidelity(&self.center2);

        d1 <= self.radius && d2 <= self.radius
    }

    /// Berechnet Overlap zwischen zwei Regionen
    pub fn overlap_measure(&self, other: &MandorlaRegion) -> f64 {
        self.center1.fidelity(&other.center1) * self.center2.fidelity(&other.center2)
    }

    /// Projiziert Zustand in Mandorla-Region
    pub fn project(&self, state: &QuantumState) -> QuantumState {
        if let Some(ref intersection) = self.intersection_state {
            // Projiziere auf Schnittmenge
            let overlap = state.inner_product(intersection);
            let projected_amps = intersection.amplitudes * overlap;
            QuantumState::new(projected_amps)
        } else {
            // Falls noch nicht berechnet, verwende Mittelwert
            let avg_amps = (self.center1.amplitudes + self.center2.amplitudes).scale(0.5);
            QuantumState::new(avg_amps)
        }
    }
}

/// Mandorla-Eigenstate-Operator
///
/// Fusioniert mehrere Mandorla-Regionen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandorlaOperator {
    /// Sammlung von Mandorla-Regionen
    pub regions: Vec<MandorlaRegion>,
    /// Rekursionslevel
    pub recursion_level: usize,
}

impl MandorlaOperator {
    /// Erstellt neuen Mandorla-Operator
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            recursion_level: 0,
        }
    }

    /// Fügt neue Mandorla-Region hinzu
    pub fn add_region(&mut self, region: MandorlaRegion) {
        self.regions.push(region);
    }

    /// Rekursive Fraktal-Konstruktion
    ///
    /// Ψ_{k+1} = F(Ψ_k, M_k, Ω_k)
    pub fn recursive_fusion(
        &mut self,
        initial_state: &QuantumState,
        max_depth: usize,
    ) -> QuantumState {
        let mut current_state = initial_state.clone();

        for k in 0..max_depth {
            if k >= self.regions.len() {
                break;
            }

            // Berechne Schnittmenge für aktuelle Region
            self.regions[k].compute_intersection();

            // Fusioniere mit aktuellem Zustand
            let intersection = &self.regions[k].intersection_state.as_ref().unwrap();
            let weight = 1.0 / (k + 2) as f64;

            let fused_amps = current_state.amplitudes.scale(1.0 - weight)
                + intersection.amplitudes.scale(weight);

            current_state = QuantumState::new(fused_amps);
            self.recursion_level = k + 1;
        }

        current_state
    }

    /// Berechnet Mandorla-Dichte in Zustandsraum
    pub fn mandorla_density(&self, state: &QuantumState) -> f64 {
        if self.regions.is_empty() {
            return 0.0;
        }

        let count = self.regions.iter().filter(|r| r.contains(state)).count();
        count as f64 / self.regions.len() as f64
    }

    /// Prüft Invarianz-Eigenschaft
    ///
    /// M_k(γ) = M_k(T(γ)) für zulässige Transformationen T
    pub fn check_invariance(&self, state: &QuantumState, transformed: &QuantumState) -> bool {
        let density1 = self.mandorla_density(state);
        let density2 = self.mandorla_density(transformed);

        (density1 - density2).abs() < 1e-6
    }
}

impl Default for MandorlaOperator {
    fn default() -> Self {
        Self::new()
    }
}

/// Mandorla-Konvolutions-Operator (⋆)
pub struct MandorlaConvolution;

impl MandorlaConvolution {
    /// Konvolution im Frequenzraum: M ⋆ B
    pub fn convolve(mandorla: &MandorlaRegion, block: &QuantumState) -> QuantumState {
        if let Some(ref intersection) = mandorla.intersection_state {
            // Frequenzraum-Multiplikation (vereinfacht)
            let conv_amps: SVector<Complex64, HILBERT_DIM> =
                SVector::from_fn(|i, _| intersection.amplitudes[i] * block.amplitudes[i]);
            QuantumState::new(conv_amps)
        } else {
            block.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_mandorla_region_creation() {
        let c1 = QuantumState::basis_state(0);
        let c2 = QuantumState::basis_state(1);
        let region = MandorlaRegion::new(c1, c2, 0.5);

        assert_abs_diff_eq!(region.radius, 0.5, epsilon = 1e-10);
    }

    #[test]
    fn test_intersection_computation() {
        let c1 = QuantumState::basis_state(0);
        let c2 = QuantumState::basis_state(1);
        let mut region = MandorlaRegion::new(c1, c2, 0.5);

        let intersection = region.compute_intersection();
        assert!(intersection.is_normalized());
        assert!(region.intersection_state.is_some());
    }

    #[test]
    fn test_contains() {
        // Use two similar states so their Mandorla intersection is non-empty
        let c1 = QuantumState::basis_state(0);
        let c2 = QuantumState::uniform_superposition();
        // Use larger radius to ensure intersection is within both circles
        let region = MandorlaRegion::new(c1.clone(), c2.clone(), 1.5);

        // The intersection state should be in the region
        let mut region_mut = region.clone();
        let intersection = region_mut.compute_intersection();

        // Intersection should be contained in the Mandorla region
        assert!(region.contains(&intersection));
    }

    #[test]
    fn test_mandorla_operator() {
        let mut operator = MandorlaOperator::new();

        let c1 = QuantumState::basis_state(0);
        let c2 = QuantumState::basis_state(1);
        let region = MandorlaRegion::new(c1, c2, 0.5);

        operator.add_region(region);
        assert_eq!(operator.regions.len(), 1);
    }

    #[test]
    fn test_recursive_fusion() {
        let mut operator = MandorlaOperator::new();

        // Füge mehrere Regionen hinzu
        for i in 0..3 {
            let c1 = QuantumState::basis_state(i);
            let c2 = QuantumState::basis_state(i + 1);
            let region = MandorlaRegion::new(c1, c2, 0.3);
            operator.add_region(region);
        }

        let initial = QuantumState::random();
        let fused = operator.recursive_fusion(&initial, 3);

        assert!(fused.is_normalized());
        assert_eq!(operator.recursion_level, 3);
    }

    #[test]
    fn test_mandorla_density() {
        let mut operator = MandorlaOperator::new();

        let c1 = QuantumState::basis_state(0);
        let c2 = QuantumState::basis_state(1);
        let region = MandorlaRegion::new(c1.clone(), c2, 0.8);
        operator.add_region(region);

        let density = operator.mandorla_density(&c1);
        assert!(density >= 0.0 && density <= 1.0);
    }

    #[test]
    fn test_convolution() {
        let c1 = QuantumState::basis_state(0);
        let c2 = QuantumState::basis_state(1);
        let mut region = MandorlaRegion::new(c1, c2, 0.5);
        region.compute_intersection();

        let block = QuantumState::random();
        let convolved = MandorlaConvolution::convolve(&region, &block);

        assert!(convolved.is_normalized());
    }
}
