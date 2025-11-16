use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OphanionConfig {
    #[serde(default)]
    pub ophanion: OphanionSettings,
    
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
pub struct OphanionSettings {
    #[serde(default = "default_gabriel_cells")]
    pub num_gabriel_cells: usize,
    
    #[serde(default = "default_spectral_dim")]
    pub spectral_dim: usize,
    
    #[serde(default = "default_learning_rate")]
    pub learning_rate_alpha: f64,
    
    #[serde(default = "default_decay_rate")]
    pub decay_rate_beta: f64,
    
    #[serde(default = "default_initial_threshold")]
    pub initial_threshold: f64,
    
    #[serde(default = "default_threshold_learning_rate")]
    pub threshold_learning_rate: f64,
    
    #[serde(default = "default_optimization_eta")]
    pub optimization_eta: f64,
    
    #[serde(default = "default_target_absorption")]
    pub target_absorption_rate: f64,
    
    #[serde(default = "default_convergence_epsilon")]
    pub convergence_epsilon: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorSettings {
    #[serde(default = "default_control_port")]
    pub control_port: u16,
    
    pub cookie_path: Option<String>,
    pub control_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSettings {
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,
    
    #[serde(default = "default_backend_port")]
    pub backend_port: u16,
    
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
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
    #[serde(default)]
    pub worker_threads: usize,
    
    #[serde(default = "default_max_circuits")]
    pub max_tracked_circuits: usize,
    
    #[serde(default = "default_retention")]
    pub metadata_retention: u64,
}

// Default value functions
fn default_gabriel_cells() -> usize { 64 }
fn default_spectral_dim() -> usize { 128 }
fn default_learning_rate() -> f64 { 0.01 }
fn default_decay_rate() -> f64 { 0.001 }
fn default_initial_threshold() -> f64 { 0.5 }
fn default_threshold_learning_rate() -> f64 { 0.001 }
fn default_optimization_eta() -> f64 { 0.0001 }
fn default_target_absorption() -> f64 { 0.95 }
fn default_convergence_epsilon() -> f64 { 0.001 }
fn default_control_port() -> u16 { 9051 }
fn default_listen_port() -> u16 { 8080 }
fn default_backend_port() -> u16 { 8081 }
fn default_bind_address() -> String { "127.0.0.1".to_string() }
fn default_true() -> bool { true }
fn default_metrics_port() -> u16 { 9090 }
fn default_max_circuits() -> usize { 10000 }
fn default_retention() -> u64 { 3600 }

impl Default for OphanionConfig {
    fn default() -> Self {
        Self {
            ophanion: OphanionSettings::default(),
            tor: TorSettings::default(),
            service: ServiceSettings::default(),
            monitoring: MonitoringSettings::default(),
            performance: PerformanceSettings::default(),
        }
    }
}

impl Default for OphanionSettings {
    fn default() -> Self {
        Self {
            num_gabriel_cells: default_gabriel_cells(),
            spectral_dim: default_spectral_dim(),
            learning_rate_alpha: default_learning_rate(),
            decay_rate_beta: default_decay_rate(),
            initial_threshold: default_initial_threshold(),
            threshold_learning_rate: default_threshold_learning_rate(),
            optimization_eta: default_optimization_eta(),
            target_absorption_rate: default_target_absorption(),
            convergence_epsilon: default_convergence_epsilon(),
        }
    }
}

impl Default for TorSettings {
    fn default() -> Self {
        Self {
            control_port: default_control_port(),
            cookie_path: Some("/var/run/tor/control.authcookie".to_string()),
            control_password: None,
        }
    }
}

impl Default for ServiceSettings {
    fn default() -> Self {
        Self {
            listen_port: default_listen_port(),
            backend_port: default_backend_port(),
            bind_address: default_bind_address(),
        }
    }
}

impl Default for MonitoringSettings {
    fn default() -> Self {
        Self {
            enable_metrics: default_true(),
            metrics_port: default_metrics_port(),
            verbose_logging: false,
            log_file: Some("/var/log/ophanion/ophanion.log".to_string()),
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            worker_threads: 0,
            max_tracked_circuits: default_max_circuits(),
            metadata_retention: default_retention(),
        }
    }
}

impl OphanionConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;
        
        let config: OphanionConfig = toml::from_str(&content)
            .context("Failed to parse TOML config")?;
        
        config.validate()?;
        
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<()> {
        if self.ophanion.num_gabriel_cells == 0 {
            anyhow::bail!("num_gabriel_cells must be > 0");
        }
        
        if self.ophanion.spectral_dim == 0 {
            anyhow::bail!("spectral_dim must be > 0");
        }
        
        if !(0.0..=1.0).contains(&self.ophanion.initial_threshold) {
            anyhow::bail!("initial_threshold must be in range [0, 1]");
        }
        
        if !(0.0..=1.0).contains(&self.ophanion.target_absorption_rate) {
            anyhow::bail!("target_absorption_rate must be in range [0, 1]");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = OphanionConfig::default();
        assert!(config.validate().is_ok());
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = OphanionConfig::default();
        config.ophanion.num_gabriel_cells = 0;
        
        assert!(config.validate().is_err());
    }
}
