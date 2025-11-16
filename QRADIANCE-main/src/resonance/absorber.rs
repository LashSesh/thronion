//! Resonant Absorber Layer (RAL)
//!
//! Implementiert adaptive spektrale Filterung für DDoS-Resistenz
//!
//! Rᵢ(t) = σ(⟨Fᵢ(t), S(T_in)⟩ - θᵢ)

use crate::core::NUM_NODES;
use crate::resonance::spectrum::SpectralFingerprint;
use serde::{Deserialize, Serialize};

/// Resonant Absorber Layer
///
/// Filtert eingehende Signale basierend auf spektraler Resonanz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResonantAbsorber {
    /// Lokale Feldprofile Fᵢ(t) für jeden Knoten
    pub local_fields: [Vec<f64>; NUM_NODES],
    /// Adaptive Schwellwerte θᵢ
    pub thresholds: [f64; NUM_NODES],
    /// Lernrate für Schwellwert-Anpassung
    pub learning_rate: f64,
    /// Spektral-Fingerprint-Größe
    pub spectrum_size: usize,
    /// Absorptions-Statistiken
    pub stats: AbsorberStats,
}

impl ResonantAbsorber {
    /// Erstellt neuen Resonant Absorber
    ///
    /// # Arguments
    /// * `spectrum_size` - Größe des Spektrums (FFT-Bins)
    /// * `learning_rate` - Lernrate für adaptive Schwellwerte
    pub fn new(spectrum_size: usize, learning_rate: f64) -> Self {
        let local_fields: [Vec<f64>; NUM_NODES] = std::array::from_fn(|_| vec![0.0; spectrum_size]);
        let thresholds = [0.5; NUM_NODES]; // Initial neutral

        Self {
            local_fields,
            thresholds,
            learning_rate,
            spectrum_size,
            stats: AbsorberStats::default(),
        }
    }

    /// Initialisiert lokale Felder mit Zufallswerten
    pub fn initialize_random_fields(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for field in &mut self.local_fields {
            for value in field.iter_mut() {
                *value = rng.gen_range(-1.0..1.0);
            }
        }
    }

    /// Berechnet Resonanz-Score für Paket an Knoten i
    ///
    /// Rᵢ = σ(⟨Fᵢ, S⟩ - θᵢ)
    ///
    /// # Arguments
    /// * `packet` - Eingehende Daten
    /// * `node` - Knoten-Index
    pub fn resonance_score(&self, packet: &[u8], node: usize) -> f64 {
        assert!(node < NUM_NODES);

        // Berechne Spektrum des Pakets
        let spectrum = SpectralFingerprint::compute(packet, self.spectrum_size);

        // Inneres Produkt mit lokalem Feld
        let inner_product: f64 = self.local_fields[node]
            .iter()
            .zip(spectrum.power_spectrum.iter())
            .map(|(f, s)| f * s)
            .sum();

        // Sigmoid-Aktivierung
        self.sigmoid(inner_product - self.thresholds[node])
    }

    /// Sigmoid-Funktion σ(x) = 1/(1 + e^(-x))
    fn sigmoid(&self, x: f64) -> f64 {
        1.0 / (1.0 + (-x).exp())
    }

    /// Entscheidet ob Paket absorbiert werden soll
    ///
    /// Absorb ⟺ R < ε_res
    pub fn should_absorb(&self, packet: &[u8], node: usize, epsilon_res: f64) -> bool {
        let score = self.resonance_score(packet, node);
        score < epsilon_res
    }

    /// Verarbeitet Paket und entscheidet über Absorption
    ///
    /// Returns: (absorbed, score)
    pub fn process_packet(&mut self, packet: &[u8], node: usize, epsilon_res: f64) -> (bool, f64) {
        let score = self.resonance_score(packet, node);
        let absorbed = score < epsilon_res;

        if absorbed {
            self.stats.packets_absorbed += 1;
            // Update Schwellwert basierend auf absorbierter Energie
            self.update_threshold(node, packet.len() as f64);
        } else {
            self.stats.packets_forwarded += 1;
        }

        self.stats.total_packets += 1;

        (absorbed, score)
    }

    /// Update adaptiver Schwellwert
    ///
    /// dθᵢ/dt = -λ ∂E_abs/∂θᵢ
    pub fn update_threshold(&mut self, node: usize, absorbed_energy: f64) {
        assert!(node < NUM_NODES);

        // Gradient descent: Verringere Schwellwert um mehr zu absorbieren
        self.thresholds[node] -= self.learning_rate * absorbed_energy / 1000.0;

        // Clamp to reasonable range
        self.thresholds[node] = self.thresholds[node].clamp(-10.0, 10.0);
    }

    /// Update lokales Feld basierend auf legitimen Traffic
    ///
    /// Lernt normale Verkehrsmuster
    pub fn learn_legitimate_pattern(&mut self, packet: &[u8], node: usize, weight: f64) {
        assert!(node < NUM_NODES);

        let spectrum = SpectralFingerprint::compute(packet, self.spectrum_size);

        // Exponentieller gleitender Durchschnitt
        for (field_val, &spectrum_val) in self.local_fields[node]
            .iter_mut()
            .zip(spectrum.power_spectrum.iter())
        {
            *field_val = (1.0 - weight) * (*field_val) + weight * spectrum_val;
        }
    }

    /// Berechnet Absorptions-Effizienz η_RAL
    pub fn absorption_efficiency(&self) -> f64 {
        if self.stats.total_packets == 0 {
            return 0.0;
        }
        self.stats.packets_absorbed as f64 / self.stats.total_packets as f64
    }

    /// Berechnet False-Positive-Rate
    pub fn false_positive_rate(&self, legitimate_count: usize) -> f64 {
        if self.stats.packets_absorbed == 0 {
            return 0.0;
        }
        legitimate_count as f64 / self.stats.packets_absorbed as f64
    }

    /// Reset Statistiken
    pub fn reset_stats(&mut self) {
        self.stats = AbsorberStats::default();
    }
}

impl Default for ResonantAbsorber {
    fn default() -> Self {
        Self::new(256, 0.01)
    }
}

/// Absorber-Statistiken
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AbsorberStats {
    /// Gesamtanzahl verarbeiteter Pakete
    pub total_packets: usize,
    /// Anzahl absorbierter Pakete
    pub packets_absorbed: usize,
    /// Anzahl weitergeleiteter Pakete
    pub packets_forwarded: usize,
}

impl std::fmt::Display for AbsorberStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Absorber-Statistiken:")?;
        writeln!(f, "  Total Packets:    {}", self.total_packets)?;
        writeln!(f, "  Absorbed:         {}", self.packets_absorbed)?;
        writeln!(f, "  Forwarded:        {}", self.packets_forwarded)?;
        if self.total_packets > 0 {
            writeln!(
                f,
                "  Absorption Rate:  {:.2}%",
                100.0 * self.packets_absorbed as f64 / self.total_packets as f64
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_absorber_creation() {
        let absorber = ResonantAbsorber::new(256, 0.01);
        assert_eq!(absorber.thresholds.len(), NUM_NODES);
        assert_eq!(absorber.spectrum_size, 256);
    }

    #[test]
    fn test_sigmoid() {
        let absorber = ResonantAbsorber::default();

        assert!((absorber.sigmoid(0.0) - 0.5).abs() < 1e-10);
        assert!(absorber.sigmoid(10.0) > 0.9);
        assert!(absorber.sigmoid(-10.0) < 0.1);
    }

    #[test]
    fn test_resonance_score() {
        let mut absorber = ResonantAbsorber::default();
        absorber.initialize_random_fields();

        let packet = b"test packet data";
        let score = absorber.resonance_score(packet, 0);

        // Score sollte in [0,1] liegen
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_absorption_decision() {
        let mut absorber = ResonantAbsorber::default();
        absorber.initialize_random_fields();

        let packet = b"malicious traffic";
        let epsilon_res = 0.3;

        let (absorbed, score) = absorber.process_packet(packet, 0, epsilon_res);

        // Entweder absorbiert oder weitergeleitet
        assert_eq!(absorbed, score < epsilon_res);
    }

    #[test]
    fn test_threshold_update() {
        let mut absorber = ResonantAbsorber::default();
        let initial_threshold = absorber.thresholds[0];

        absorber.update_threshold(0, 100.0);

        // Schwellwert sollte sich geändert haben
        assert_ne!(absorber.thresholds[0], initial_threshold);
    }

    #[test]
    fn test_pattern_learning() {
        let mut absorber = ResonantAbsorber::default();
        absorber.initialize_random_fields();

        let legitimate_packet = b"normal traffic pattern";

        // Lerne Pattern mehrfach
        for _ in 0..10 {
            absorber.learn_legitimate_pattern(legitimate_packet, 0, 0.1);
        }

        // Feld sollte sich dem Pattern angenähert haben
        assert!(absorber.local_fields[0].iter().any(|&x| x != 0.0));
    }

    #[test]
    fn test_statistics() {
        let mut absorber = ResonantAbsorber::default();
        absorber.initialize_random_fields();

        let packet1 = b"packet1";
        let packet2 = b"packet2";

        absorber.process_packet(packet1, 0, 0.5);
        absorber.process_packet(packet2, 0, 0.5);

        assert_eq!(absorber.stats.total_packets, 2);
        assert_eq!(
            absorber.stats.packets_absorbed + absorber.stats.packets_forwarded,
            2
        );
    }

    #[test]
    fn test_absorption_efficiency() {
        let mut absorber = ResonantAbsorber::default();

        // Simuliere Pakete
        absorber.stats.total_packets = 100;
        absorber.stats.packets_absorbed = 95;
        absorber.stats.packets_forwarded = 5;

        let efficiency = absorber.absorption_efficiency();
        assert!((efficiency - 0.95).abs() < 1e-10);
    }
}
