# Thronion Usage Guide

This guide provides comprehensive examples and best practices for using Thronion to protect Tor Hidden Services from DDoS attacks.

## Table of Contents

- [Quick Start](#quick-start)
- [Library Usage](#library-usage)
- [Configuration Examples](#configuration-examples)
- [Monitoring & Metrics](#monitoring--metrics)
- [Advanced Usage](#advanced-usage)
- [Best Practices](#best-practices)
- [Examples](#examples)

---

## Quick Start

### Basic Library Integration

```rust
use thronion::prelude::*;
use thronion::thronion::{ThronionKernel, ConversionUtils, ClassicalSignature};
use thronion::tor::{TorCircuitMetadata, MetadataExtractor};

fn main() {
    // Create Thronion kernel with default parameters
    let mut kernel = ThronionKernel::new();

    println!("Thronion v{} initialized", thronion::VERSION);
    println!("Tripolar information capacity: {} bits/symbol",
             thronion::TRIPOLAR_CAPACITY);

    // Ready to process circuits!
}
```

### Processing Tor Circuits

```rust
use thronion::thronion::ThronionKernel;
use thronion::tor::{TorCircuitMetadata, MetadataExtractor};

fn classify_circuit(
    kernel: &ThronionKernel,
    metadata: &TorCircuitMetadata
) -> bool {
    // Extract features from circuit metadata
    let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
    let distribution = MetadataExtractor::analyze_cell_types(&metadata.cell_types);

    // Classify the circuit
    let (is_attack, resonance, region_idx) = kernel.classify(
        metadata,
        &timing,
        &distribution
    );

    if is_attack {
        println!("⚠️  ATTACK DETECTED!");
        println!("   Circuit ID: {}", metadata.circuit_id);
        println!("   Resonance: {:.3}", resonance);
        println!("   Matching region: {:?}", region_idx);

        // Take protective action
        // - Drop circuit
        // - Rate limit source
        // - Log for analysis
    } else {
        println!("✓ Circuit {} classified as benign (resonance: {:.3})",
                 metadata.circuit_id, resonance);
    }

    is_attack
}
```

---

## Library Usage

### 1. Basic ThronionKernel

```rust
use thronion::thronion::ThronionKernel;

// Create kernel with default parameters
let mut kernel = ThronionKernel::new();

// Or with custom parameters
let mut kernel = ThronionKernel::with_params(
    0.5,   // attack_threshold
    100,   // max_regions
    0.1    // learning_rate
);

// Check kernel state
let stats = kernel.stats();
println!("Total regions: {}", stats.total_regions);
println!("Attack regions: {}", stats.attack_regions);
println!("Benign regions: {}", stats.benign_regions);
```

### 2. Online Learning

```rust
use thronion::thronion::ThronionKernel;

let mut kernel = ThronionKernel::new();

// Process and learn from circuits
for circuit in incoming_circuits {
    let metadata = extract_metadata(&circuit);
    let timing = extract_timing(&circuit);
    let distribution = analyze_types(&circuit);

    // Classify
    let (is_attack, resonance, _) = kernel.classify(
        &metadata, &timing, &distribution
    );

    // Learn from labeled data (if available)
    // This could come from manual review, external threat intel, etc.
    if let Some(ground_truth) = get_ground_truth(&circuit) {
        kernel.learn(&metadata, &timing, &distribution, ground_truth);
    }

    // Or learn from high-confidence classifications
    if resonance > 0.8 {
        kernel.learn(&metadata, &timing, &distribution, is_attack);
    }
}
```

### 3. EnhancedThronionKernel with Delta Optimization

```rust
use thronion::thronion::{ThronionKernel, EnhancedThronionKernel};
use thronion::delta::QRIKParams;

// Create base kernel
let base_kernel = ThronionKernel::new();

// Create enhanced kernel with Delta optimization
let params = QRIKParams::default();
let mut enhanced = EnhancedThronionKernel::new(base_kernel, params);

// Process circuits with automatic optimization
for circuit in incoming_circuits {
    let metadata = extract_metadata(&circuit);
    let timing = extract_timing(&circuit);
    let distribution = analyze_types(&circuit);

    // Classify with automatic periodic optimization
    let (is_attack, resonance, _) = enhanced.classify(
        &metadata, &timing, &distribution
    );

    // Check system stability
    if !enhanced.is_stable() {
        println!("⚠️  System undergoing optimization...");
        println!("   Coherence gradient: {:.6}", enhanced.coherence_gradient());
    }

    // Get enhanced statistics
    let stats = enhanced.stats();
    println!("Processed {} circuits", stats.classification_count);
    println!("System stable: {}", stats.is_stable);
}
```

### 4. Classical-Quantum Conversion

```rust
use thronion::thronion::{ClassicalSignature, ConversionUtils};

// Create classical signature from circuit features
let classical = ClassicalSignature {
    mean_interval: 100.0,      // microseconds
    std_dev_interval: 20.0,    // microseconds
    data_ratio: 0.75,          // 75% data cells
    intro_ratio: 0.10,         // 10% intro cells
    total_bytes: 4096.0,       // bytes transferred
};

// Convert to quantum state (5D → 13D Hilbert space)
let quantum = ConversionUtils::classical_to_quantum(&classical);
println!("Quantum state normalized: {}", quantum.is_normalized());

// Convert back to classical (13D → 5D projection)
let classical_recovered = ConversionUtils::quantum_to_classical(&quantum);
println!("Recovered mean interval: {:.2} μs", classical_recovered.mean_interval);
```

### 5. Working with QRIK Core

```rust
use thronion::prelude::*;
use thronion::core::{QuantumState, DTLState, DTLClass};

// Create quantum state in 13D Hilbert space
let state = QuantumState::zero();  // |0⟩ state
println!("State purity: {:.6}", state.purity());

// Create DTL (Dynamic Tripolar Logic) state
let dtl = DTLState::new(
    0.5,  // ψ: L1 (One) probability
    0.0,  // ρ: L0 (Null) probability
    0.5   // δ: LD (Dynamic) probability
);

// Classify DTL state
match dtl.classify() {
    DTLClass::L0 => println!("NULL: Benign/padding traffic"),
    DTLClass::L1 => println!("ONE: Confirmed legitimate"),
    DTLClass::LD => println!("DYNAMIC: Suspicious pattern"),
}

// Information capacity advantage
println!("Tripolar vs Binary: +{:.1}% capacity",
         thronion::BINARY_ADVANTAGE * 100.0);
```

### 6. Delta Kernel Operations

```rust
use thronion::delta::{DeltaKernel, QRIKParams};

// Create Delta Kernel with custom parameters
let params = QRIKParams {
    alpha: 0.1,
    beta: 0.5,
    gamma: 0.3,
    lambda: 0.8,
    mu: 0.2,
    eta: 0.05,
};

let mut kernel = DeltaKernel::new(params);

// Evolve system over time
for step in 0..1000 {
    kernel.evolve(0.01);  // dt = 0.01

    if step % 100 == 0 {
        let gradient = kernel.coherence_gradient();
        println!("Step {}: gradient = {:.6}", step, gradient);

        if kernel.is_stable(0.05) {
            println!("✓ System converged!");
            break;
        }
    }
}
```

---

## Configuration Examples

### Development Configuration

```toml
# thronion-dev.toml
[thronion]
max_regions = 50
learning_rate = 0.15
attack_threshold = 0.4
resonance_threshold = 0.25
optimization_interval = 50

[tor]
control_port = 9051
cookie_path = "/var/run/tor/control.authcookie"

[service]
bind_address = "127.0.0.1"
worker_threads = 2

[monitoring]
enable_metrics = true
metrics_port = 9090
verbose_logging = true
log_file = "./thronion-dev.log"

[performance]
max_tracked_circuits = 5000
metadata_retention_secs = 1800
```

### Production Configuration (High Traffic)

```toml
# thronion-prod.toml
[thronion]
max_regions = 200
learning_rate = 0.05
attack_threshold = 0.55
resonance_threshold = 0.35
optimization_interval = 200
coherence_threshold = 0.03
merge_threshold = 0.92

[tor]
control_port = 9051
cookie_path = "/var/run/tor/control.authcookie"

[service]
bind_address = "127.0.0.1"
worker_threads = 8

[monitoring]
enable_metrics = true
metrics_port = 9090
verbose_logging = false
log_file = "/var/log/thronion/thronion.log"

[performance]
max_tracked_circuits = 100000
metadata_retention_secs = 7200
```

### Resource-Constrained Configuration

```toml
# thronion-minimal.toml
[thronion]
max_regions = 30
learning_rate = 0.1
attack_threshold = 0.5
resonance_threshold = 0.3
optimization_interval = 200

[service]
worker_threads = 2

[performance]
max_tracked_circuits = 2000
metadata_retention_secs = 900
```

---

## Monitoring & Metrics

### Prometheus Metrics

Thronion exposes the following Prometheus metrics on port 9090 (configurable):

```
# Circuit metrics
thronion_circuits_total                # Total circuits processed
thronion_circuits_absorbed             # Circuits identified as attacks
thronion_circuits_forwarded            # Circuits forwarded (benign)

# Classification metrics
thronion_classifications_total         # Total classification operations
thronion_attacks_detected              # Number of attacks detected
thronion_false_positives               # Estimated false positives

# Timing metrics (histograms)
thronion_classification_latency_seconds  # Time to classify a circuit
thronion_learning_latency_seconds        # Time for learning updates
thronion_optimization_latency_seconds    # Time for Delta optimization

# System metrics (gauges)
thronion_active_regions                # Number of learned regions
thronion_coherence_gradient            # Delta Kernel gradient (stability)
thronion_attack_rate                   # Current attack rate (circuits/sec)
```

### Scraping Configuration (Prometheus)

Add to `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'thronion'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    scrape_timeout: 10s
```

### Example Queries

```promql
# Attack detection rate (last 5 minutes)
rate(thronion_attacks_detected[5m])

# False positive rate
rate(thronion_false_positives[5m]) / rate(thronion_classifications_total[5m])

# 99th percentile classification latency
histogram_quantile(0.99, thronion_classification_latency_seconds_bucket)

# System stability (lower is more stable)
thronion_coherence_gradient

# Active learning progress
thronion_active_regions
```

### Alerting Rules

```yaml
# prometheus-alerts.yml
groups:
  - name: thronion
    rules:
      - alert: HighAttackRate
        expr: rate(thronion_attacks_detected[5m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High attack rate detected"
          description: "Attack rate is {{ $value }} circuits/sec"

      - alert: SystemUnstable
        expr: thronion_coherence_gradient > 0.2
        for: 10m
        labels:
          severity: info
        annotations:
          summary: "Thronion system optimization in progress"

      - alert: HighLatency
        expr: histogram_quantile(0.99, thronion_classification_latency_seconds_bucket) > 0.1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Classification latency exceeds 100ms"
```

---

## Advanced Usage

### Custom Region Management

```rust
use thronion::thronion::{ThronionKernel, GabrielRegion, ClassicalSignature};
use thronion::core::QuantumState;

let mut kernel = ThronionKernel::new();

// Pre-seed kernel with known benign patterns
let benign_patterns = vec![
    (100.0, 20.0, 0.7, 0.15, 2048.0),  // Normal browsing
    (150.0, 30.0, 0.6, 0.20, 4096.0),  // File download
    (80.0, 15.0, 0.8, 0.10, 1024.0),   // Chat/messaging
];

for (mean, std, data_ratio, intro_ratio, bytes) in benign_patterns {
    let classical = ClassicalSignature {
        mean_interval: mean,
        std_dev_interval: std,
        data_ratio,
        intro_ratio,
        total_bytes: bytes,
    };

    let quantum = ConversionUtils::classical_to_quantum(&classical);

    // Create region directly
    let region = GabrielRegion::new(classical, quantum, 0.1);

    // Note: Direct region insertion not in public API
    // Use learning instead:
    // kernel.learn(&metadata, &timing, &dist, false);
}
```

### Batch Processing

```rust
use thronion::thronion::ThronionKernel;
use rayon::prelude::*;

let kernel = ThronionKernel::new();

// Process circuits in parallel (read-only classification)
let results: Vec<_> = circuits.par_iter()
    .map(|circuit| {
        let metadata = extract_metadata(circuit);
        let timing = extract_timing(circuit);
        let distribution = analyze_types(circuit);

        let (is_attack, resonance, _) = kernel.classify(
            &metadata, &timing, &distribution
        );

        (circuit.id, is_attack, resonance)
    })
    .collect();

// Analyze batch results
let attack_count = results.iter().filter(|(_, is_attack, _)| *is_attack).count();
let total = results.len();
println!("Batch analysis: {}/{} attacks detected ({:.1}%)",
         attack_count, total, (attack_count as f64 / total as f64) * 100.0);
```

### Integration with External Threat Intelligence

```rust
use thronion::thronion::ThronionKernel;
use std::collections::HashSet;

struct ThreatIntelligence {
    known_attack_ips: HashSet<String>,
    known_attack_patterns: Vec<ClassicalSignature>,
}

impl ThreatIntelligence {
    fn check_circuit(&self, circuit: &TorCircuit) -> Option<bool> {
        // Check against known indicators
        if let Some(ip) = circuit.source_ip() {
            if self.known_attack_ips.contains(ip) {
                return Some(true);  // Known attacker
            }
        }

        None  // Unknown, use Thronion classification
    }
}

fn classify_with_intel(
    kernel: &ThronionKernel,
    intel: &ThreatIntelligence,
    circuit: &TorCircuit
) -> bool {
    // First check threat intel
    if let Some(is_attack) = intel.check_circuit(circuit) {
        return is_attack;
    }

    // Fall back to Thronion
    let metadata = extract_metadata(circuit);
    let timing = extract_timing(circuit);
    let distribution = analyze_types(circuit);

    let (is_attack, _, _) = kernel.classify(&metadata, &timing, &distribution);
    is_attack
}
```

---

## Best Practices

### 1. Initialization

- ✅ Start with default parameters, tune based on observed performance
- ✅ Pre-seed kernel with known benign patterns if available
- ✅ Use EnhancedThronionKernel for automatic optimization
- ❌ Don't set learning rate too high (>0.3) - causes instability
- ❌ Don't set max_regions too low (<30) - limits pattern diversity

### 2. Classification

- ✅ Always check resonance score along with classification
- ✅ Use higher attack_threshold for critical services (fewer false positives)
- ✅ Log all high-resonance classifications for review
- ❌ Don't blindly trust low-resonance classifications (<0.3)
- ❌ Don't ignore the region index - it provides context

### 3. Learning

- ✅ Learn from manually reviewed samples when possible
- ✅ Use online learning for adaptation to evolving attacks
- ✅ Periodically review and retrain on labeled dataset
- ❌ Don't learn from every classification (risk of poisoning)
- ❌ Don't ignore false positives - they indicate tuning needed

### 4. Monitoring

- ✅ Monitor coherence gradient for system stability
- ✅ Track false positive rate and tune attack_threshold accordingly
- ✅ Set up alerts for high attack rates
- ✅ Review metrics regularly (daily/weekly)
- ❌ Don't ignore optimization events - they may cause brief latency spikes

### 5. Performance Tuning

- ✅ Start with worker_threads = CPU cores
- ✅ Increase max_regions if seeing many low-resonance matches
- ✅ Decrease optimization_interval if system is unstable
- ❌ Don't exceed max_tracked_circuits capacity
- ❌ Don't set worker_threads > 2x CPU cores

### 6. Security

- ✅ Run with minimal required privileges
- ✅ Use Tor cookie authentication (not password)
- ✅ Rotate logs regularly
- ✅ Monitor for adversarial poisoning attempts
- ❌ Don't expose metrics port publicly
- ❌ Don't log sensitive circuit data

---

## Examples

### Example 1: Simple CLI Monitor

```rust
use thronion::thronion::ThronionKernel;
use thronion::service::ThronionConfig;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = ThronionConfig::from_file("/etc/thronion/thronion.toml")?;
    config.validate()?;

    // Create kernel
    let mut kernel = ThronionKernel::with_params(
        config.thronion.attack_threshold,
        config.thronion.max_regions,
        config.thronion.learning_rate,
    );

    println!("Thronion Monitor started");
    println!("Monitoring Tor control port {}...", config.tor.control_port);

    // Connect to Tor and process circuits
    // (Simplified - see full example in examples/monitor.rs)

    loop {
        // Process incoming circuits
        // let circuit = receive_circuit()?;
        // let (is_attack, resonance, _) = classify(&kernel, &circuit);

        // if is_attack {
        //     println!("⚠️  ATTACK: Circuit {}", circuit.id);
        //     take_action(&circuit);
        // }

        // Periodic stats
        std::thread::sleep(Duration::from_secs(60));

        let stats = kernel.stats();
        println!("\n=== Stats ===");
        println!("Regions: {} (A:{}, B:{})",
                 stats.total_regions, stats.attack_regions, stats.benign_regions);
    }
}
```

### Example 2: Testing Mode

```rust
use thronion::thronion::ThronionKernel;

fn test_mode(labeled_data: Vec<(TorCircuitMetadata, bool)>) {
    let mut kernel = ThronionKernel::new();

    let mut tp = 0;  // True positives
    let mut fp = 0;  // False positives
    let mut tn = 0;  // True negatives
    let mut fn_ = 0; // False negatives

    // Split into training and testing
    let (train, test) = labeled_data.split_at(labeled_data.len() * 8 / 10);

    // Training phase
    println!("Training on {} samples...", train.len());
    for (metadata, ground_truth) in train {
        let timing = extract_timing(metadata);
        let dist = analyze_types(metadata);
        kernel.learn(metadata, &timing, &dist, *ground_truth);
    }

    // Testing phase
    println!("Testing on {} samples...", test.len());
    for (metadata, ground_truth) in test {
        let timing = extract_timing(metadata);
        let dist = analyze_types(metadata);
        let (is_attack, _, _) = kernel.classify(metadata, &timing, &dist);

        match (is_attack, *ground_truth) {
            (true, true) => tp += 1,
            (true, false) => fp += 1,
            (false, false) => tn += 1,
            (false, true) => fn_ += 1,
        }
    }

    // Calculate metrics
    let precision = tp as f64 / (tp + fp) as f64;
    let recall = tp as f64 / (tp + fn_) as f64;
    let fpr = fp as f64 / (fp + tn) as f64;

    println!("\n=== Results ===");
    println!("Precision: {:.2}%", precision * 100.0);
    println!("Recall (Detection Rate): {:.2}%", recall * 100.0);
    println!("False Positive Rate: {:.4}%", fpr * 100.0);
}
```

---

## Additional Resources

- **Architecture**: See [THRONION_ARCHITECTURE.md](THRONION_ARCHITECTURE.md)
- **API Docs**: Run `cargo doc --no-deps --open`
- **Installation**: See [INSTALL.md](INSTALL.md)
- **Examples**: Check `thronion/examples/` directory
- **Issues**: https://github.com/LashSesh/thronion/issues

---

**Happy protecting!** 🛡️🧅
