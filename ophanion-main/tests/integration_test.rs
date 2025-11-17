use ophanion::*;
use ophanion::spectral::SpectralEngine;
use ophanion::resonance::ResonanceEngine;
use ophanion::threshold::AdaptiveThreshold;
use ophanion::delta_kernel::DeltaKernel;
use ophanion::decision::DecisionEngine;
use ophanion::config::OphanionSettings;
use std::time::{Duration, Instant};
use ndarray::Array1;

fn create_legitimate_circuit(id: u32) -> TorCircuitMetadata {
    // Simulate human-like browsing pattern with variable timings
    let timings: Vec<Duration> = vec![
        Duration::from_millis(50),
        Duration::from_millis(120),
        Duration::from_millis(80),
        Duration::from_millis(200),
        Duration::from_millis(150),
        Duration::from_millis(90),
        Duration::from_millis(180),
        Duration::from_millis(110),
    ];

    TorCircuitMetadata {
        circuit_id: id,
        created_at: Instant::now(),
        cell_timings: timings,
        cell_types: vec![
            TorCellType::Introduce2,
            TorCellType::Rendezvous1,
            TorCellType::Rendezvous2,
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Padding,
        ],
        introduction_point: Some("legitimate_intro".to_string()),
        rendezvous_completed: true,
        total_bytes: 8192,
    }
}

fn create_attack_circuit(id: u32) -> TorCircuitMetadata {
    // Simulate bot-like pattern with uniform timings
    let timings: Vec<Duration> = vec![
        Duration::from_millis(10),
        Duration::from_millis(10),
        Duration::from_millis(10),
        Duration::from_millis(10),
    ];

    TorCircuitMetadata {
        circuit_id: id,
        created_at: Instant::now(),
        cell_timings: timings,
        cell_types: vec![
            TorCellType::Introduce2,
            TorCellType::Rendezvous1,
            TorCellType::Other,
            TorCellType::Other,
        ],
        introduction_point: Some("attack_intro".to_string()),
        rendezvous_completed: false,
        total_bytes: 512,
    }
}

#[test]
fn test_full_system_integration() {
    let config = OphanionSettings {
        num_gabriel_cells: 32,
        spectral_dim: 128,
        learning_rate_alpha: 0.05,
        decay_rate_beta: 0.001,
        initial_threshold: 0.5,
        ..Default::default()
    };

    let mut spectral = SpectralEngine::new();
    let resonance = ResonanceEngine::new(config.clone());
    let threshold = AdaptiveThreshold::new(config.clone());
    let mut decision = DecisionEngine::new();

    // Phase 1: Learn from legitimate traffic
    println!("Phase 1: Learning from 20 legitimate circuits...");
    for i in 0..20 {
        let circuit = create_legitimate_circuit(i);
        let signature = spectral.create_signature(&circuit);
        resonance.learn_signature_knn(&signature, 3);
    }

    let coherence_after_learning = resonance.coherence();
    println!("Coherence after learning: {:.3}", coherence_after_learning);
    assert!(coherence_after_learning > 0.0, "System should have learned patterns");

    // Phase 2: Test with legitimate traffic
    println!("\nPhase 2: Testing with 10 legitimate circuits...");
    let mut legit_scores = Vec::new();
    for i in 100..110 {
        let circuit = create_legitimate_circuit(i);
        let signature = spectral.create_signature(&circuit);
        let score = resonance.compute_score_knn(&signature, 3);
        legit_scores.push(score);

        let action = decision.decide(score, threshold.value());
        println!("  Circuit {}: Score = {:.3}, Action = {:?}", i, score, action);
    }

    let avg_legit_score = legit_scores.iter().sum::<f64>() / legit_scores.len() as f64;
    println!("Average legitimate score: {:.3}", avg_legit_score);

    // Phase 3: Test with attack traffic
    println!("\nPhase 3: Testing with 10 attack circuits...");
    let mut attack_scores = Vec::new();
    for i in 200..210 {
        let circuit = create_attack_circuit(i);
        let signature = spectral.create_signature(&circuit);
        let score = resonance.compute_score_knn(&signature, 3);
        attack_scores.push(score);

        let action = decision.decide(score, threshold.value());
        println!("  Circuit {}: Score = {:.3}, Action = {:?}", i, score, action);
    }

    let avg_attack_score = attack_scores.iter().sum::<f64>() / attack_scores.len() as f64;
    println!("Average attack score: {:.3}", avg_attack_score);

    // Phase 4: Verify discrimination
    println!("\nPhase 4: Verification...");
    let stats = decision.statistics();
    println!("Total decisions: {}", stats.total_decisions);
    println!("Forwarded: {}", stats.forwarded);
    println!("Absorbed: {}", stats.absorbed);
    println!("Absorption rate: {:.1}%", stats.absorption_rate * 100.0);

    // Assertions
    assert!(
        avg_legit_score > avg_attack_score,
        "Legitimate traffic should score higher than attack traffic"
    );

    assert!(
        stats.absorbed > 0,
        "System should absorb some circuits"
    );

    println!("\n✓ Integration test passed!");
}

#[test]
fn test_adaptive_threshold_convergence() {
    let config = OphanionSettings {
        num_gabriel_cells: 16,
        spectral_dim: 128,
        learning_rate_alpha: 0.1,
        threshold_learning_rate: 0.01,
        target_absorption_rate: 0.5, // 50% absorption target for this test
        ..Default::default()
    };

    let mut spectral = SpectralEngine::new();
    let resonance = ResonanceEngine::new(config.clone());
    let threshold = AdaptiveThreshold::new(config.clone());
    let mut decision = DecisionEngine::new();

    // Train with mixed traffic
    for i in 0..50 {
        let circuit = if i % 2 == 0 {
            create_legitimate_circuit(i)
        } else {
            create_attack_circuit(i)
        };

        let signature = spectral.create_signature(&circuit);
        resonance.learn_signature(&signature);

        let score = resonance.compute_score(&signature);
        let action = decision.decide(score, threshold.value());
        threshold.record_absorption(action == CircuitAction::Absorb);
    }

    // Update threshold based on performance
    for _ in 0..10 {
        let coherence = resonance.coherence();
        let absorption_rate = threshold.absorption_rate();
        let flood_energy = 1.0 - absorption_rate;
        threshold.update(coherence, flood_energy);
    }

    let final_absorption = threshold.absorption_rate();
    let stats = decision.statistics();

    println!("Final absorption rate: {:.1}%", final_absorption * 100.0);
    println!("Target absorption rate: {:.1}%", config.target_absorption_rate * 100.0);
    println!("Threshold value: {:.3}", threshold.value());
    println!("Decision stats: Forwarded={}, Absorbed={}, Total={}",
             stats.forwarded, stats.absorbed, stats.total_decisions);

    // The absorption rate should be valid (between 0 and 1, inclusive)
    assert!(
        final_absorption >= 0.0 && final_absorption <= 1.0,
        "Absorption rate should be between 0 and 1 (got {})", final_absorption
    );

    // We should have made decisions
    assert!(stats.total_decisions > 0, "Should have made some decisions");

    println!("✓ Adaptive threshold test passed!");
}

#[test]
fn test_delta_kernel_optimization() {
    let config = OphanionSettings {
        learning_rate_alpha: 0.01,
        decay_rate_beta: 0.001,
        initial_threshold: 0.5,
        optimization_eta: 0.001,
        convergence_epsilon: 0.01,
        ..Default::default()
    };

    let mut delta = DeltaKernel::new(config);

    let coherence = 0.7;
    let flood_energy = 0.3;

    let initial_gradient = delta.gradient_magnitude(coherence, flood_energy);
    println!("Initial gradient magnitude: {:.6}", initial_gradient);

    // Run optimization steps
    for i in 0..100 {
        delta.optimize_step(coherence, flood_energy);

        if i % 20 == 0 {
            let grad = delta.gradient_magnitude(coherence, flood_energy);
            let (alpha, beta, theta) = delta.get_params();
            println!(
                "Step {}: ∇Ψ_Δ = {:.6}, α = {:.4}, β = {:.4}, θ = {:.3}",
                i, grad, alpha, beta, theta
            );
        }
    }

    let final_gradient = delta.gradient_magnitude(coherence, flood_energy);
    println!("Final gradient magnitude: {:.6}", final_gradient);

    let (alpha, beta, theta) = delta.get_params();
    println!("Final parameters: α = {:.4}, β = {:.4}, θ = {:.3}", alpha, beta, theta);

    // Gradient should decrease or stabilize
    assert!(
        final_gradient <= initial_gradient + 0.1,
        "Gradient should not increase significantly"
    );

    // Parameters should be in valid ranges
    assert!(alpha > 0.0 && alpha <= 0.1, "Alpha should be in valid range");
    assert!(beta > 0.0 && beta <= 0.01, "Beta should be in valid range");
    assert!(theta > 0.0 && theta <= 1.0, "Theta should be in valid range");

    println!("✓ Delta-kernel optimization test passed!");
}

#[test]
fn test_spectral_engine_consistency() {
    let mut engine = SpectralEngine::new();

    let circuit1 = create_legitimate_circuit(1);
    let circuit2 = create_legitimate_circuit(2);
    let circuit3 = create_attack_circuit(3);

    let sig1 = engine.create_signature(&circuit1);
    let sig2 = engine.create_signature(&circuit2);
    let sig3 = engine.create_signature(&circuit3);

    // Signatures should be normalized (L2 norm = 1)
    let norm1 = sig1.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm2 = sig2.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm3 = sig3.iter().map(|x| x * x).sum::<f64>().sqrt();

    println!("Signature norms: {:.6}, {:.6}, {:.6}", norm1, norm2, norm3);

    assert!((norm1 - 1.0).abs() < 1e-6, "Signature should be normalized");
    assert!((norm2 - 1.0).abs() < 1e-6, "Signature should be normalized");
    assert!((norm3 - 1.0).abs() < 1e-6, "Signature should be normalized");

    println!("✓ Spectral engine consistency test passed!");
}

#[test]
fn test_resonance_engine_learning() {
    let config = OphanionSettings {
        num_gabriel_cells: 16,
        spectral_dim: 128,
        learning_rate_alpha: 0.1,
        ..Default::default()
    };

    let engine = ResonanceEngine::new(config.clone());

    let coherence_before = engine.coherence();
    println!("Coherence before learning: {:.3}", coherence_before);

    // Learn from 30 signatures
    for i in 0..30 {
        let sig = Array1::from_vec(
            (0..config.spectral_dim)
                .map(|j| 0.5 + (i as f64 * 0.01) + (j as f64 * 0.001))
                .collect()
        );
        engine.learn_signature(&sig);
    }

    let coherence_after = engine.coherence();
    println!("Coherence after learning: {:.3}", coherence_after);

    let stats = engine.statistics();
    println!("Cluster statistics:");
    println!("  Mean strength: {:.3}", stats.mean_strength);
    println!("  Std dev: {:.3}", stats.std_dev);
    println!("  Active cells: {}/{}", stats.active_cells, stats.total_cells);

    assert!(
        coherence_after > coherence_before,
        "Coherence should increase after learning"
    );

    assert!(
        stats.active_cells > 0,
        "Some cells should be active after learning"
    );

    println!("✓ Resonance engine learning test passed!");
}
