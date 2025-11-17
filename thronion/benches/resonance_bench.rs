//! Benchmarks für Resonanz-Module

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qrik::prelude::*;

fn kuramoto_evolution(c: &mut Criterion) {
    c.bench_function("kuramoto_evolution_100steps", |b| {
        b.iter(|| {
            let mut network = KuramotoNetwork::uniform(1.0, 2.0);
            for _ in 0..100 {
                network.evolve_rk4(black_box(0.01));
            }
        })
    });
}

fn resonance_score(c: &mut Criterion) {
    let mut absorber = ResonantAbsorber::default();
    absorber.initialize_random_fields();
    let packet = b"test packet data";

    c.bench_function("resonance_score_computation", |b| {
        b.iter(|| {
            absorber.resonance_score(black_box(packet), black_box(0));
        })
    });
}

fn spectral_fingerprint(c: &mut Criterion) {
    let data = b"benchmark data for spectral fingerprinting test";

    c.bench_function("spectral_fingerprint_256", |b| {
        b.iter(|| {
            SpectralFingerprint::compute(black_box(data), black_box(256));
        })
    });
}

criterion_group!(
    benches,
    kuramoto_evolution,
    resonance_score,
    spectral_fingerprint
);
criterion_main!(benches);
