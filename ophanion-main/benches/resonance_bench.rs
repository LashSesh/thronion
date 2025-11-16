use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use ophanion::*;
use ophanion::spectral::SpectralEngine;
use ophanion::resonance::ResonanceEngine;
use ophanion::config::OphanionSettings;
use std::time::{Duration, Instant};
use ndarray::Array1;

fn create_test_circuit(id: u32, num_cells: usize) -> TorCircuitMetadata {
    let timings: Vec<Duration> = (0..num_cells)
        .map(|i| Duration::from_micros(10000 + (i * 1500) as u64))
        .collect();

    TorCircuitMetadata {
        circuit_id: id,
        created_at: Instant::now(),
        cell_timings: timings,
        cell_types: vec![TorCellType::Data; num_cells],
        introduction_point: Some("test_intro_point".to_string()),
        rendezvous_completed: true,
        total_bytes: 10000,
    }
}

fn bench_spectral_fingerprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("spectral_fingerprint");

    for num_cells in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cells),
            num_cells,
            |b, &num_cells| {
                let mut engine = SpectralEngine::new();
                let circuit = create_test_circuit(1, num_cells);

                b.iter(|| {
                    black_box(engine.compute_fingerprint(&circuit))
                });
            },
        );
    }

    group.finish();
}

fn bench_signature_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_creation");

    for num_cells in [10, 50, 100, 200].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cells),
            num_cells,
            |b, &num_cells| {
                let mut engine = SpectralEngine::new();
                let circuit = create_test_circuit(1, num_cells);

                b.iter(|| {
                    black_box(engine.create_signature(&circuit))
                });
            },
        );
    }

    group.finish();
}

fn bench_resonance_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("resonance_scoring");

    for num_cells in [16, 32, 64, 128].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_cells),
            num_cells,
            |b, &num_cells| {
                let config = OphanionSettings {
                    num_gabriel_cells: num_cells,
                    spectral_dim: 128,
                    ..Default::default()
                };
                let engine = ResonanceEngine::new(config.clone());
                let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);

                // Train the engine first
                for _ in 0..10 {
                    engine.learn_signature(&signature);
                }

                b.iter(|| {
                    black_box(engine.compute_score(&signature))
                });
            },
        );
    }

    group.finish();
}

fn bench_knn_scoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("knn_scoring");

    let config = OphanionSettings {
        num_gabriel_cells: 64,
        spectral_dim: 128,
        ..Default::default()
    };
    let engine = ResonanceEngine::new(config.clone());
    let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);

    for k in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(k),
            k,
            |b, &k| {
                b.iter(|| {
                    black_box(engine.compute_score_knn(&signature, k))
                });
            },
        );
    }

    group.finish();
}

fn bench_learning(c: &mut Criterion) {
    let mut group = c.benchmark_group("learning");

    let config = OphanionSettings {
        num_gabriel_cells: 64,
        spectral_dim: 128,
        learning_rate_alpha: 0.01,
        ..Default::default()
    };
    let engine = ResonanceEngine::new(config.clone());

    group.bench_function("learn_signature", |b| {
        let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);

        b.iter(|| {
            black_box(engine.learn_signature(&signature))
        });
    });

    group.bench_function("learn_signature_knn_k3", |b| {
        let signature = Array1::from_vec(vec![0.5; config.spectral_dim]);

        b.iter(|| {
            black_box(engine.learn_signature_knn(&signature, 3))
        });
    });

    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    c.bench_function("full_decision_pipeline", |b| {
        let mut spectral = SpectralEngine::new();
        let config = OphanionSettings::default();
        let resonance = ResonanceEngine::new(config.clone());

        let circuit = create_test_circuit(1, 50);

        b.iter(|| {
            // Complete pipeline: Spectral -> Signature -> Resonance -> Decision
            let signature = spectral.create_signature(&circuit);
            let score = resonance.compute_score(&signature);
            let _decision = if score > 0.5 {
                CircuitAction::Forward
            } else {
                CircuitAction::Absorb
            };

            black_box(_decision)
        });
    });
}

criterion_group!(
    benches,
    bench_spectral_fingerprint,
    bench_signature_creation,
    bench_resonance_scoring,
    bench_knn_scoring,
    bench_learning,
    bench_full_pipeline
);

criterion_main!(benches);
