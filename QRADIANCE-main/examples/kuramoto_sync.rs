//! Kuramoto-Synchronisations-Demo
//!
//! Demonstriert emergente Synchronisation im Resonanznetzwerk

use qrik::prelude::*;

fn main() {
    println!("=== Kuramoto-Synchronisations-Demo ===\n");

    // Erstelle Netzwerk mit Frequenz-Dispersion
    let mut network = KuramotoNetwork::with_frequency_disorder(1.0, 0.5, 2.5);

    println!("Netzwerk-Konfiguration:");
    println!("  Knoten: {}", HILBERT_DIM);
    println!("  Kopplung: 2.5");
    println!("  Frequenz-Dispersion: σ=0.5\n");

    // Randomisiere Initialphasen
    network.randomize_phases();

    let initial_sync = network.synchronization();
    println!("Initiale Synchronisation: r = {:.4}\n", initial_sync);

    // Evolution
    let dt = 0.01;
    let total_time = 20.0;
    let steps = (total_time / dt) as usize;

    println!("Starte Zeitevolution...\n");
    println!("{:>8} {:>10} {:>10}", "Zeit", "r", "Θ");
    println!("{:-<30}", "");

    for step in 0..steps {
        network.evolve_rk4(dt);

        let t = step as f64 * dt;
        if step % 100 == 0 {
            let (r, theta) = network.order_parameter();
            println!("{:8.2} {:10.6} {:10.4}", t, r, theta);

            // Prüfe Konvergenz
            if r > 0.99 && step > 100 {
                println!("\n✓ Synchronisation erreicht bei t = {:.2}", t);
                break;
            }
        }
    }

    let (r_final, theta_final) = network.order_parameter();

    println!("\n=== Finale Metriken ===");
    println!("Ordnungsparameter r: {:.6}", r_final);
    println!("Globale Phase Θ:     {:.4} rad", theta_final);
    println!(
        "Synchronisiert:      {}",
        if network.is_synchronized(0.95) {
            "✓"
        } else {
            "✗"
        }
    );

    // Berechne kritische Kopplung
    let kappa_c = network.critical_coupling();
    println!("\nKritische Kopplung: κ_c = {:.4}", kappa_c);

    // Lokale Ordnungsparameter
    println!("\n=== Lokale Synchronisation ===");
    let local_r = network.local_order_parameters();
    for (i, &r_local) in local_r.iter().enumerate() {
        if i % 2 == 0 {
            // Zeige nur jeden 2. Knoten
            println!("Knoten {:2}: r_local = {:.4}", i, r_local);
        }
    }

    println!("\n=== Phasen-Kohärenz ===");
    let coherence_matrix = network.phase_coherence_matrix();

    // Durchschnittliche Kohärenz
    let mut total_coherence = 0.0;
    let mut count = 0;
    for i in 0..HILBERT_DIM {
        for j in (i + 1)..HILBERT_DIM {
            total_coherence += coherence_matrix[i][j];
            count += 1;
        }
    }
    let avg_coherence = total_coherence / count as f64;
    println!("Durchschnittliche Kohärenz: {:.4}", avg_coherence);
}
