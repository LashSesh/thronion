//! Service Runtime Layer
//!
//! This module provides the production service infrastructure including
//! configuration, metrics, and the main async runtime.
//!
//! ## Components
//!
//! - **Configuration**: Unified config loading from TOML
//! - **Metrics**: Prometheus metrics collection
//! - **Runtime**: Async tokio-based service loop
//! - **Main**: Service entry point and initialization

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use anyhow::{Context, Result};
use tokio::sync::RwLock;
use prometheus::{IntCounter, Histogram, Gauge, Registry, HistogramOpts, Opts};

/// Thronion Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThronionConfig {
    #[serde(default)]
    pub thronion: ThronionSettings,
    
    #[serde(default)]
    pub tor: TorSettings,
    
    #[serde(default)]
    pub service: ServiceSettings,
    
    #[serde(default)]
    pub monitoring: MonitoringSettings,
    
    #[serde(default)]
    pub performance: PerformanceSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThronionSettings {
    #[serde(default = "default_max_regions")]
    pub max_regions: usize,
    
    #[serde(default = "default_learning_rate")]
    pub learning_rate: f64,
    
    #[serde(default = "default_attack_threshold")]
    pub attack_threshold: f64,
    
    #[serde(default = "default_resonance_threshold")]
    pub resonance_threshold: f64,
    
    #[serde(default = "default_optimization_interval")]
    pub optimization_interval: usize,
    
    #[serde(default = "default_coherence_threshold")]
    pub coherence_threshold: f64,
    
    #[serde(default = "default_merge_threshold")]
    pub merge_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorSettings {
    #[serde(default = "default_control_port")]
    pub control_port: u16,
    
    #[serde(default = "default_cookie_path")]
    pub cookie_path: String,
    
    pub control_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSettings {
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    
    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSettings {
    #[serde(default = "default_true")]
    pub enable_metrics: bool,
    
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
    
    #[serde(default)]
    pub verbose_logging: bool,
    
    pub log_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    #[serde(default = "default_max_circuits")]
    pub max_tracked_circuits: usize,
    
    #[serde(default = "default_retention")]
    pub metadata_retention_secs: u64,
}

// Default value functions
fn default_max_regions() -> usize { 100 }
fn default_learning_rate() -> f64 { 0.1 }
fn default_attack_threshold() -> f64 { 0.5 }
fn default_resonance_threshold() -> f64 { 0.3 }
fn default_optimization_interval() -> usize { 100 }
fn default_coherence_threshold() -> f64 { 0.05 }
fn default_merge_threshold() -> f64 { 0.9 }
fn default_control_port() -> u16 { 9051 }
fn default_cookie_path() -> String { "/var/run/tor/control.authcookie".to_string() }
fn default_bind_address() -> String { "127.0.0.1".to_string() }
fn default_worker_threads() -> usize { 4 }
fn default_true() -> bool { true }
fn default_metrics_port() -> u16 { 9090 }
fn default_max_circuits() -> usize { 10000 }
fn default_retention() -> u64 { 3600 }

impl Default for ThronionConfig {
    fn default() -> Self {
        Self {
            thronion: ThronionSettings::default(),
            tor: TorSettings::default(),
            service: ServiceSettings::default(),
            monitoring: MonitoringSettings::default(),
            performance: PerformanceSettings::default(),
        }
    }
}

impl Default for ThronionSettings {
    fn default() -> Self {
        Self {
            max_regions: default_max_regions(),
            learning_rate: default_learning_rate(),
            attack_threshold: default_attack_threshold(),
            resonance_threshold: default_resonance_threshold(),
            optimization_interval: default_optimization_interval(),
            coherence_threshold: default_coherence_threshold(),
            merge_threshold: default_merge_threshold(),
        }
    }
}

impl Default for TorSettings {
    fn default() -> Self {
        Self {
            control_port: default_control_port(),
            cookie_path: default_cookie_path(),
            control_password: None,
        }
    }
}

impl Default for ServiceSettings {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            worker_threads: default_worker_threads(),
        }
    }
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            enable_metrics: default_true(),
            metrics_port: default_metrics_port(),
            verbose_logging: false,
            log_file: Some("/var/log/thronion/thronion.log".to_string()),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            max_tracked_circuits: default_max_circuits(),
            metadata_retention_secs: default_retention(),
        }
    }
}

impl ThronionConfig {
    /// Load configuration from TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;
        
        let config: ThronionConfig = toml::from_str(&content)
            .context("Failed to parse TOML config")?;
        
        config.validate()?;
        
        Ok(config)
    }
    
    /// Validate configuration parameters
    pub fn validate(&self) -> Result<()> {
        if self.thronion.max_regions == 0 {
            anyhow::bail!("max_regions must be > 0");
        }
        
        if !(0.0..=1.0).contains(&self.thronion.attack_threshold) {
            anyhow::bail!("attack_threshold must be in range [0, 1]");
        }
        
        if !(0.0..=1.0).contains(&self.thronion.resonance_threshold) {
            anyhow::bail!("resonance_threshold must be in range [0, 1]");
        }
        
        if !(0.0..=1.0).contains(&self.thronion.merge_threshold) {
            anyhow::bail!("merge_threshold must be in range [0, 1]");
        }
        
        Ok(())
    }
}

/// Prometheus Metrics Collector
pub struct ThronionMetrics {
    // Circuit metrics
    pub circuits_total: IntCounter,
    pub circuits_absorbed: IntCounter,
    pub circuits_forwarded: IntCounter,
    
    // Classification metrics
    pub classifications_total: IntCounter,
    pub attacks_detected: IntCounter,
    pub false_positives: IntCounter,
    
    // Timing metrics
    pub classification_latency: Histogram,
    pub learning_latency: Histogram,
    pub optimization_latency: Histogram,
    
    // System metrics
    pub active_regions: Gauge,
    pub coherence_gradient: Gauge,
    pub attack_rate: Gauge,
    
    registry: Registry,
}

impl ThronionMetrics {
    /// Create new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        let circuits_total = IntCounter::new("thronion_circuits_total", "Total circuits processed")?;
        let circuits_absorbed = IntCounter::new("thronion_circuits_absorbed", "Circuits absorbed (attacks)")?;
        let circuits_forwarded = IntCounter::new("thronion_circuits_forwarded", "Circuits forwarded (benign)")?;
        
        let classifications_total = IntCounter::new("thronion_classifications_total", "Total classifications")?;
        let attacks_detected = IntCounter::new("thronion_attacks_detected", "Attacks detected")?;
        let false_positives = IntCounter::new("thronion_false_positives", "False positives")?;
        
        let classification_latency = Histogram::with_opts(
            HistogramOpts::new("thronion_classification_latency_seconds", "Classification latency")
        )?;
        let learning_latency = Histogram::with_opts(
            HistogramOpts::new("thronion_learning_latency_seconds", "Learning latency")
        )?;
        let optimization_latency = Histogram::with_opts(
            HistogramOpts::new("thronion_optimization_latency_seconds", "Optimization latency")
        )?;
        
        let active_regions = Gauge::with_opts(
            Opts::new("thronion_active_regions", "Number of active regions")
        )?;
        let coherence_gradient = Gauge::with_opts(
            Opts::new("thronion_coherence_gradient", "Delta Kernel coherence gradient")
        )?;
        let attack_rate = Gauge::with_opts(
            Opts::new("thronion_attack_rate", "Current attack rate")
        )?;
        
        registry.register(Box::new(circuits_total.clone()))?;
        registry.register(Box::new(circuits_absorbed.clone()))?;
        registry.register(Box::new(circuits_forwarded.clone()))?;
        registry.register(Box::new(classifications_total.clone()))?;
        registry.register(Box::new(attacks_detected.clone()))?;
        registry.register(Box::new(false_positives.clone()))?;
        registry.register(Box::new(classification_latency.clone()))?;
        registry.register(Box::new(learning_latency.clone()))?;
        registry.register(Box::new(optimization_latency.clone()))?;
        registry.register(Box::new(active_regions.clone()))?;
        registry.register(Box::new(coherence_gradient.clone()))?;
        registry.register(Box::new(attack_rate.clone()))?;
        
        Ok(Self {
            circuits_total,
            circuits_absorbed,
            circuits_forwarded,
            classifications_total,
            attacks_detected,
            false_positives,
            classification_latency,
            learning_latency,
            optimization_latency,
            active_regions,
            coherence_gradient,
            attack_rate,
            registry,
        })
    }
    
    /// Get registry for HTTP endpoint
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for ThronionMetrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}

/// Service Runtime State
pub struct ThronionService {
    config: ThronionConfig,
    metrics: Arc<ThronionMetrics>,
    running: Arc<RwLock<bool>>,
}

impl ThronionService {
    /// Create new service instance
    pub fn new(config: ThronionConfig) -> Result<Self> {
        let metrics = Arc::new(ThronionMetrics::new()?);
        let running = Arc::new(RwLock::new(false));
        
        Ok(Self {
            config,
            metrics,
            running,
        })
    }
    
    /// Get configuration
    pub fn config(&self) -> &ThronionConfig {
        &self.config
    }
    
    /// Get metrics
    pub fn metrics(&self) -> Arc<ThronionMetrics> {
        Arc::clone(&self.metrics)
    }
    
    /// Check if service is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
    
    /// Start the service
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = true;
        Ok(())
    }
    
    /// Stop the service
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = ThronionConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.thronion.max_regions, 100);
        assert_eq!(config.tor.control_port, 9051);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = ThronionConfig::default();
        config.thronion.max_regions = 0;
        assert!(config.validate().is_err());
        
        let mut config = ThronionConfig::default();
        config.thronion.attack_threshold = 1.5;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_metrics_creation() {
        let metrics = ThronionMetrics::new();
        assert!(metrics.is_ok());
    }
    
    #[tokio::test]
    async fn test_service_lifecycle() {
        let config = ThronionConfig::default();
        let service = ThronionService::new(config).unwrap();
        
        assert!(!service.is_running().await);
        
        service.start().await.unwrap();
        assert!(service.is_running().await);
        
        service.stop().await.unwrap();
        assert!(!service.is_running().await);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = ThronionConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("max_regions"));
        assert!(toml_str.contains("control_port"));
    }
}
