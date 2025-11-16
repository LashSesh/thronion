use ophanion::*;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, debug};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "ophanion")]
#[clap(about = "Resonant Monolith DDoS Protection for Tor Hidden Services", long_about = None)]
struct Args {
    /// Configuration file path
    #[clap(short, long, default_value = "config.toml")]
    config: String,
    
    /// Enable verbose logging
    #[clap(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    // Initialize tracing
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();
    
    info!("┌────────────────────────────────────────┐");
    info!("│     OPHANION v1.0                     │");
    info!("│  Resonant Monolith DDoS Protection     │");
    info!("└────────────────────────────────────────┘");
    info!("");
    
    // Load configuration
    info!("Loading configuration from: {}", args.config);
    let config = config::OphanionConfig::from_file(&args.config)?;
    info!("✓ Configuration loaded and validated");
    
    // Initialize components
    info!("Initializing OPHANION components...");
    
    let _spectral_engine = spectral::SpectralEngine::new();
    info!("✓ Spectral Engine initialized");

    let resonance_engine = Arc::new(
        resonance::ResonanceEngine::new(config.ophanion.clone())
    );
    info!("✓ Resonance Engine initialized ({} Gabriel Cells)", 
          config.ophanion.num_gabriel_cells);
    
    let adaptive_threshold = Arc::new(
        threshold::AdaptiveThreshold::new(config.ophanion.clone())
    );
    info!("✓ Adaptive Threshold initialized (θ₀ = {:.3})", 
          config.ophanion.initial_threshold);
    
    let mut delta_kernel = delta_kernel::DeltaKernel::new(config.ophanion.clone());
    info!("✓ Delta-Kernel Optimizer initialized");
    
    let decision_engine = Arc::new(parking_lot::Mutex::new(
        decision::DecisionEngine::new()
    ));
    info!("✓ Decision Engine initialized");
    
    info!("");
    info!("All components initialized successfully!");
    info!("Target absorption rate: {:.1}%", 
          config.ophanion.target_absorption_rate * 100.0);
    info!("");
    
    // Spawn maintenance task
    let resonance_clone = Arc::clone(&resonance_engine);
    let threshold_clone = Arc::clone(&adaptive_threshold);
    let decision_clone = Arc::clone(&decision_engine);
    
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(10));
        loop {
            tick.tick().await;
            
            // Run maintenance
            resonance_clone.maintenance_cycle();
            
            let coherence = resonance_clone.coherence();
            let absorption_rate = threshold_clone.absorption_rate();
            let stats = decision_clone.lock().statistics();
            
            info!(
                "Stats | Coherence: {:.3} | Absorption: {:.1}% | Decisions: {} | Forwarded: {} | Absorbed: {}",
                coherence, 
                absorption_rate * 100.0,
                stats.total_decisions,
                stats.forwarded,
                stats.absorbed
            );
            
            // Update threshold
            let flood_energy = 1.0 - absorption_rate;
            threshold_clone.update(coherence, flood_energy);
        }
    });
    
    // Spawn Delta-Kernel optimization task
    let resonance_clone2 = Arc::clone(&resonance_engine);
    let threshold_clone2 = Arc::clone(&adaptive_threshold);
    
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(30));
        loop {
            tick.tick().await;
            
            let coherence = resonance_clone2.coherence();
            let flood_energy = 1.0 - threshold_clone2.absorption_rate();
            
            delta_kernel.optimize_step(coherence, flood_energy);
            
            let gradient = delta_kernel.gradient_magnitude(coherence, flood_energy);
            let (alpha, beta, theta) = delta_kernel.get_params();
            
            debug!(
                "Δ-Optimization | ∇Ψ_Δ: {:.6} | α: {:.4} | β: {:.4} | θ: {:.3}",
                gradient, alpha, beta, theta
            );
            
            if delta_kernel.has_converged(coherence, flood_energy) {
                info!("★ CONVERGENCE ACHIEVED: ∇Ψ_Δ ≈ 0 ★");
            }
        }
    });
    
    // Main event loop (simplified demo - would integrate with actual Tor control port)
    info!("Entering main event loop...");
    info!("(In production: would connect to Tor control port at localhost:{})", 
          config.tor.control_port);
    info!("");
    warn!("NOTE: This is a demonstration version. Full Tor integration requires:");
    warn!("  1. Tor control port connection (tor-control-proto crate)");
    warn!("  2. Circuit event monitoring (SETEVENTS CIRC)");
    warn!("  3. Cell timing extraction from Tor daemon");
    info!("");
    
    // Keep running
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("");
            info!("Shutdown signal received");
            info!("OPHANION stopping gracefully...");
        }
    }
    
    info!("OPHANION stopped.");
    Ok(())
}
