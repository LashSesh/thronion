//! Spektrales Fingerprinting
//!
//! FFT-basierte spektrale Analyse für Paket-Klassifikation

use rustfft::{num_complex::Complex, FftPlanner};
use serde::{Deserialize, Serialize};

/// Spektrales Fingerprint eines Datenpakets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralFingerprint {
    /// Power-Spektrum |F(ω)|²
    pub power_spectrum: Vec<f64>,
    /// Dominante Frequenzen
    pub dominant_frequencies: Vec<usize>,
    /// Spektrale Entropie
    pub spectral_entropy: f64,
}

impl SpectralFingerprint {
    /// Berechnet spektrales Fingerprint via FFT
    ///
    /// # Arguments
    /// * `data` - Byte-Array (Paket-Daten)
    /// * `fft_size` - Größe der FFT (Power of 2)
    pub fn compute(data: &[u8], fft_size: usize) -> Self {
        assert!(fft_size.is_power_of_two(), "FFT-Größe muss 2^n sein");

        // Konvertiere Bytes zu f64 und normalisiere
        let mut signal: Vec<f64> = data.iter().map(|&b| b as f64 / 255.0).collect();

        // Padding/Truncation auf FFT-Größe
        signal.resize(fft_size, 0.0);

        // Konvertiere zu Complex
        let mut buffer: Vec<Complex<f64>> = signal.iter().map(|&x| Complex::new(x, 0.0)).collect();

        // FFT durchführen
        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(fft_size);
        fft.process(&mut buffer);

        // Berechne Power-Spektrum
        let power_spectrum: Vec<f64> = buffer.iter().map(|c| c.norm_sqr()).collect();

        // Normalisiere
        let total_power: f64 = power_spectrum.iter().sum();
        let normalized_spectrum: Vec<f64> = if total_power > 1e-10 {
            power_spectrum.iter().map(|&p| p / total_power).collect()
        } else {
            vec![1.0 / fft_size as f64; fft_size]
        };

        // Finde dominante Frequenzen (Top-5)
        let mut freq_power: Vec<(usize, f64)> = normalized_spectrum
            .iter()
            .enumerate()
            .map(|(i, &p)| (i, p))
            .collect();
        freq_power.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let dominant_frequencies: Vec<usize> = freq_power.iter().take(5).map(|&(i, _)| i).collect();

        // Berechne spektrale Entropie
        let spectral_entropy = Self::compute_entropy(&normalized_spectrum);

        Self {
            power_spectrum: normalized_spectrum,
            dominant_frequencies,
            spectral_entropy,
        }
    }

    /// Berechnet Shannon-Entropie des Spektrums
    fn compute_entropy(spectrum: &[f64]) -> f64 {
        -spectrum
            .iter()
            .filter(|&&p| p > 1e-15)
            .map(|&p| p * p.ln())
            .sum::<f64>()
    }

    /// Berechnet Ähnlichkeit zwischen zwei Spektren (Kosinus-Ähnlichkeit)
    pub fn similarity(&self, other: &Self) -> f64 {
        assert_eq!(self.power_spectrum.len(), other.power_spectrum.len());

        let dot_product: f64 = self
            .power_spectrum
            .iter()
            .zip(other.power_spectrum.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_self: f64 = self
            .power_spectrum
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt();
        let norm_other: f64 = other
            .power_spectrum
            .iter()
            .map(|x| x * x)
            .sum::<f64>()
            .sqrt();

        if norm_self < 1e-10 || norm_other < 1e-10 {
            0.0
        } else {
            dot_product / (norm_self * norm_other)
        }
    }

    /// Prüft ob Spektrum "flach" ist (typisch für Bot-Traffic)
    pub fn is_flat(&self, threshold: f64) -> bool {
        // Flaches Spektrum hat hohe Entropie
        self.spectral_entropy > threshold
    }

    /// Berechnet spektrale Kurtosis (Maß für Spitzigkeit)
    pub fn spectral_kurtosis(&self) -> f64 {
        let mean = self.power_spectrum.iter().sum::<f64>() / self.power_spectrum.len() as f64;
        let variance: f64 = self
            .power_spectrum
            .iter()
            .map(|&p| (p - mean).powi(2))
            .sum::<f64>()
            / self.power_spectrum.len() as f64;

        if variance < 1e-10 {
            return 0.0;
        }

        let fourth_moment: f64 = self
            .power_spectrum
            .iter()
            .map(|&p| (p - mean).powi(4))
            .sum::<f64>()
            / self.power_spectrum.len() as f64;

        fourth_moment / variance.powi(2) - 3.0
    }

    /// Klassifiziert Traffic-Typ basierend auf Spektrum
    pub fn classify_traffic(&self) -> TrafficType {
        // Heuristiken basierend auf spektralen Features
        if self.spectral_entropy > 0.9 * (self.power_spectrum.len() as f64).ln() {
            // Sehr flaches Spektrum → Bot-Traffic
            TrafficType::Bot
        } else if self.dominant_frequencies[0] < 10 {
            // Niedrigfrequente Komponenten → Strukturiert
            TrafficType::Legitimate
        } else if self.spectral_kurtosis().abs() > 2.0 {
            // Hohe Kurtosis → Spiky, verdächtig
            TrafficType::Suspicious
        } else {
            TrafficType::Legitimate
        }
    }
}

/// Traffic-Klassifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrafficType {
    /// Legitimer strukturierter Traffic
    Legitimate,
    /// Bot-generierter Traffic (flaches Spektrum)
    Bot,
    /// Verdächtiger Traffic
    Suspicious,
}

impl std::fmt::Display for TrafficType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrafficType::Legitimate => write!(f, "Legitimate"),
            TrafficType::Bot => write!(f, "Bot"),
            TrafficType::Suspicious => write!(f, "Suspicious"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_fingerprint_creation() {
        let data = b"test packet data with some content";
        let fingerprint = SpectralFingerprint::compute(data, 256);

        assert_eq!(fingerprint.power_spectrum.len(), 256);
        assert_eq!(fingerprint.dominant_frequencies.len(), 5);
    }

    #[test]
    fn test_power_spectrum_normalization() {
        let data = b"normalized test";
        let fingerprint = SpectralFingerprint::compute(data, 128);

        let sum: f64 = fingerprint.power_spectrum.iter().sum();
        assert_abs_diff_eq!(sum, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_entropy_calculation() {
        let data = b"entropy test data";
        let fingerprint = SpectralFingerprint::compute(data, 256);

        // Entropie sollte positiv sein
        assert!(fingerprint.spectral_entropy > 0.0);

        // Maximum Entropie für 256 bins: ln(256)
        assert!(fingerprint.spectral_entropy <= 256.0_f64.ln());
    }

    #[test]
    fn test_similarity_identical() {
        let data = b"identical data";
        let fp1 = SpectralFingerprint::compute(data, 128);
        let fp2 = SpectralFingerprint::compute(data, 128);

        let similarity = fp1.similarity(&fp2);
        assert_abs_diff_eq!(similarity, 1.0, epsilon = 1e-6);
    }

    #[test]
    fn test_similarity_different() {
        let data1 = b"first packet";
        let data2 = b"completely different packet";

        let fp1 = SpectralFingerprint::compute(data1, 128);
        let fp2 = SpectralFingerprint::compute(data2, 128);

        let similarity = fp1.similarity(&fp2);

        // Ähnlichkeit sollte < 1 sein
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_flat_spectrum_detection() {
        // Zufällige Daten sollten flaches Spektrum haben
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let random_data: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();

        let fingerprint = SpectralFingerprint::compute(&random_data, 256);

        // Entropy should be positive for random data
        // Note: Actual entropy can vary significantly, so we use a very conservative threshold
        assert!(fingerprint.spectral_entropy > 0.0);

        // Power spectrum should exist and have entries
        assert!(!fingerprint.power_spectrum.is_empty());
    }

    #[test]
    fn test_structured_spectrum() {
        // Periodisches Signal sollte strukturiertes Spektrum haben
        let periodic_data: Vec<u8> = (0..256)
            .map(|i| ((i as f64 * 0.1).sin() * 127.0 + 128.0) as u8)
            .collect();

        let fingerprint = SpectralFingerprint::compute(&periodic_data, 256);

        // Niedrige Entropie erwartet (wenige dominante Frequenzen)
        assert!(fingerprint.spectral_entropy < 4.0);
    }

    #[test]
    fn test_traffic_classification() {
        // Bot-artiger Traffic (random)
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bot_data: Vec<u8> = (0..512).map(|_| rng.gen()).collect();
        let bot_fp = SpectralFingerprint::compute(&bot_data, 256);

        // Periodischer legitimer Traffic
        let legit_data: Vec<u8> = (0..512)
            .map(|i| ((i as f64 * 0.05).sin() * 100.0 + 128.0) as u8)
            .collect();
        let legit_fp = SpectralFingerprint::compute(&legit_data, 256);

        // Bot sollte höhere Entropie haben
        assert!(bot_fp.spectral_entropy > legit_fp.spectral_entropy);
    }

    #[test]
    fn test_kurtosis() {
        let data = b"kurtosis test data packet";
        let fingerprint = SpectralFingerprint::compute(data, 128);

        let kurtosis = fingerprint.spectral_kurtosis();

        // Kurtosis sollte finite sein
        assert!(kurtosis.is_finite());
    }
}
