//! DDoS-Defense-Szenario
//!
//! Demonstriert QRIK's Fähigkeit, DDoS-Attacken zu absorbieren

use qrik::prelude::*;
use rand::Rng;

fn main() {
    println!("=== QRIK DDoS-Defense Demo ===\n");

    // Erstelle QRIK-Kernel
    let mut kernel = DeltaKernel::default();
    kernel.absorber.initialize_random_fields();

    println!("System initialisiert:");
    println!("  Knoten: {}", HILBERT_DIM);
    println!("  Kohärenz: {:.4}\n", kernel.coherence());

    // Generiere legitimen Traffic
    let legitimate_packets = generate_legitimate_traffic(100);
    println!("Generiere {} legitime Pakete...", legitimate_packets.len());

    // Lerne normale Pattern
    for packet in &legitimate_packets {
        kernel.absorber.learn_legitimate_pattern(packet, 0, 0.1);
    }

    // Generiere DDoS-Attacke (bot-generiert, flaches Spektrum)
    let attack_packets = generate_ddos_traffic(1000);
    println!("Generiere {} DDoS-Pakete...\n", attack_packets.len());

    // Verarbeite alle Pakete
    let mut legitimate_absorbed = 0;
    let mut attack_forwarded = 0;

    println!("Verarbeite legitimen Traffic:");
    for packet in &legitimate_packets {
        let (absorbed, score) = kernel.process_packet(packet, 0);
        if absorbed {
            legitimate_absorbed += 1;
        }
    }

    println!("Verarbeite Attacke:");
    for packet in &attack_packets {
        let (absorbed, score) = kernel.process_packet(packet, 0);
        if !absorbed {
            attack_forwarded += 1;
        }
    }

    println!("\n=== Ergebnisse ===");
    println!("Total Pakete: {}", kernel.absorber.stats.total_packets);
    println!("Absorbiert:   {}", kernel.absorber.stats.packets_absorbed);
    println!("Weitergel.:   {}", kernel.absorber.stats.packets_forwarded);
    println!(
        "Absorption:   {:.2}%",
        kernel.absorber.absorption_efficiency() * 100.0
    );

    // Performance-Metriken
    let false_positive_rate = legitimate_absorbed as f64 / legitimate_packets.len() as f64;
    let attack_detection_rate =
        (attack_packets.len() - attack_forwarded) as f64 / attack_packets.len() as f64;

    println!("\n=== Performance ===");
    println!("False Positive Rate: {:.2}%", false_positive_rate * 100.0);
    println!("Attack Detection:    {:.2}%", attack_detection_rate * 100.0);

    // System-Status
    println!("\n=== System-Status ===");
    println!("Kohärenz:        {:.4}", kernel.coherence());
    println!("Delta-Gradient:  {:.4}", kernel.coherence_gradient());
    println!("Kuramoto-Sync:   {:.4}", kernel.kuramoto.synchronization());
    println!(
        "Stabil:          {}",
        if kernel.is_stable(0.01) { "✓" } else { "✗" }
    );
}

/// Generiert legitimen strukturierten Traffic
fn generate_legitimate_traffic(count: usize) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    for i in 0..count {
        // Strukturiertes Pattern (periodisch)
        let packet: Vec<u8> = (0..64)
            .map(|j| ((i + j) as f64 * 0.1).sin() as u8)
            .collect();
        packets.push(packet);
    }
    packets
}

/// Generiert DDoS-Traffic (zufällig, flaches Spektrum)
fn generate_ddos_traffic(count: usize) -> Vec<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let mut packets = Vec::new();

    for _ in 0..count {
        let packet: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
        packets.push(packet);
    }

    packets
}
