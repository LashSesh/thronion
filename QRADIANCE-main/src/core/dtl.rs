//! Dynamic Tripolar Logic (DTL) - Drei-Zustands-Logik-System
//!
//! Dieses Modul implementiert die fundamentale DTL-Struktur mit drei Zustandsklassen:
//! - L0: Null-Pol (ρ=0, ω=0)
//! - L1: Eins-Pol (ρ=1, ω=0)
//! - LD: Dynamischer Pol (ρ∈(0,1), ω≠0)

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// DTL-Zustand als Tripel (ψ, ρ, ω)
///
/// # Komponenten
/// - `psi`: Kohärenzamplitude ∈ [0,1]
/// - `rho`: Populationsdichte ∈ [0,1]
/// - `omega`: Instantane Kreisfrequenz ∈ ℝ
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DTLState {
    /// Kohärenzamplitude [0,1]
    pub psi: f64,
    /// Populationsdichte [0,1]
    pub rho: f64,
    /// Kreisfrequenz [rad/s]
    pub omega: f64,
}

impl DTLState {
    /// Erstellt einen neuen DTL-Zustand mit Validierung
    ///
    /// # Panics
    /// Panikt wenn psi oder rho nicht in [0,1] liegen
    pub fn new(psi: f64, rho: f64, omega: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&psi),
            "psi muss in [0,1] liegen, ist aber {}",
            psi
        );
        assert!(
            (0.0..=1.0).contains(&rho),
            "rho muss in [0,1] liegen, ist aber {}",
            rho
        );
        Self { psi, rho, omega }
    }

    /// Erstellt L0 Zustand (Null-Pol)
    ///
    /// Null-Pol: ρ = 0, ω = 0
    pub fn l0() -> Self {
        Self {
            psi: 0.0,
            rho: 0.0,
            omega: 0.0,
        }
    }

    /// Erstellt L1 Zustand (Eins-Pol)
    ///
    /// Eins-Pol: ρ = 1, ω = 0
    pub fn l1() -> Self {
        Self {
            psi: 1.0,
            rho: 1.0,
            omega: 0.0,
        }
    }

    /// Erstellt LD Zustand (Dynamischer Pol)
    ///
    /// Dynamischer Pol: ρ ∈ (0,1), ω ≠ 0
    ///
    /// # Arguments
    /// * `freq` - Oszillationsfrequenz in Hz
    /// * `amp` - Amplitude der Populationsdichte ∈ (0,1)
    pub fn ld_oscillatory(freq: f64, amp: f64) -> Self {
        assert!((0.0..1.0).contains(&amp), "Amplitude muss in (0,1) liegen");
        assert!(freq.abs() > 1e-10, "Frequenz muss ≠ 0 sein");

        Self {
            psi: 0.5,
            rho: amp,
            omega: 2.0 * PI * freq,
        }
    }

    /// Klassifiziert den Zustand
    pub fn classify(&self) -> DTLClass {
        const EPSILON: f64 = 1e-10;

        if self.rho < EPSILON && self.omega.abs() < EPSILON {
            DTLClass::L0
        } else if (self.rho - 1.0).abs() < EPSILON && self.omega.abs() < EPSILON {
            DTLClass::L1
        } else {
            DTLClass::LD
        }
    }

    /// Berechnet die Informationskapazität dieses Zustands
    ///
    /// Für tripolares System: C_DTL = log₂(3) ≈ 1.585 Bit/Symbol
    pub fn information_capacity() -> f64 {
        3.0_f64.log2()
    }

    /// Zeitevolution des dynamischen Pols
    ///
    /// Für LD-Zustände: ρ(t) = ρ₀ + A·sin(ωt)
    pub fn evolve(&self, dt: f64) -> Self {
        match self.classify() {
            DTLClass::LD => {
                // Oszillatorische Evolution
                let phase_shift = self.omega * dt;
                Self {
                    psi: self.psi,
                    rho: (self.rho * (1.0 + (phase_shift).sin())).clamp(0.0, 1.0),
                    omega: self.omega,
                }
            }
            _ => *self, // L0 und L1 sind stationär
        }
    }

    /// Berechnet Hamming-Distanz zwischen zwei DTL-Zuständen
    pub fn hamming_distance(&self, other: &Self) -> f64 {
        ((self.psi - other.psi).powi(2)
            + (self.rho - other.rho).powi(2)
            + ((self.omega - other.omega) / (2.0 * PI)).powi(2))
        .sqrt()
    }
}

/// DTL-Zustandsklassifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DTLClass {
    /// Null-Pol: ρ = 0, ω = 0
    L0,
    /// Eins-Pol: ρ = 1, ω = 0
    L1,
    /// Dynamischer Pol: ρ ∈ (0,1), ω ≠ 0
    LD,
}

impl std::fmt::Display for DTLClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DTLClass::L0 => write!(f, "L₀ (Null-Pol)"),
            DTLClass::L1 => write!(f, "L₁ (Eins-Pol)"),
            DTLClass::LD => write!(f, "L_D (Dynamischer Pol)"),
        }
    }
}

impl Default for DTLState {
    fn default() -> Self {
        Self::l0()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l0_classification() {
        let l0 = DTLState::l0();
        assert_eq!(l0.classify(), DTLClass::L0);
        assert_eq!(l0.psi, 0.0);
        assert_eq!(l0.rho, 0.0);
        assert_eq!(l0.omega, 0.0);
    }

    #[test]
    fn test_l1_classification() {
        let l1 = DTLState::l1();
        assert_eq!(l1.classify(), DTLClass::L1);
        assert_eq!(l1.psi, 1.0);
        assert_eq!(l1.rho, 1.0);
        assert_eq!(l1.omega, 0.0);
    }

    #[test]
    fn test_ld_classification() {
        let ld = DTLState::ld_oscillatory(10.0, 0.5);
        assert_eq!(ld.classify(), DTLClass::LD);
        assert!(ld.omega.abs() > 0.0);
        assert!(ld.rho > 0.0 && ld.rho < 1.0);
    }

    #[test]
    fn test_information_capacity() {
        let capacity = DTLState::information_capacity();
        assert!((capacity - 1.585).abs() < 0.001);
    }

    #[test]
    fn test_evolution() {
        let ld = DTLState::ld_oscillatory(1.0, 0.5);
        let evolved = ld.evolve(0.1);
        assert_eq!(evolved.classify(), DTLClass::LD);
    }

    #[test]
    #[should_panic]
    fn test_invalid_psi() {
        DTLState::new(1.5, 0.5, 0.0);
    }

    #[test]
    #[should_panic]
    fn test_invalid_rho() {
        DTLState::new(0.5, -0.1, 0.0);
    }
}
