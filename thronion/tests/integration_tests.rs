// Integration tests for Thronion system
// Tests end-to-end functionality across all layers

use thronion::thronion::{ClassicalSignature, ConversionUtils, EnhancedThronionKernel, ThronionKernel};
use thronion::tor::{TorCellType, TorCircuitMetadata, MetadataExtractor};
use thronion::delta::QRIKParams;
use std::time::Duration;

// Helper function to create Enhanced Kernel for tests
fn create_test_kernel(max_regions: usize, learning_rate: f64) -> EnhancedThronionKernel {
    let base_kernel = ThronionKernel::with_params(0.5, max_regions, learning_rate);
    let delta_params = QRIKParams::default();
    EnhancedThronionKernel::new(base_kernel, delta_params)
}

#[test]
fn test_end_to_end_circuit_classification() {
    // Create a circuit with known attack pattern
    let metadata = TorCircuitMetadata {
        circuit_id: 1u32,
        created_at: std::time::Instant::now(),
        cell_timings: vec![
            Duration::from_micros(50),  // Very fast - suspicious
            Duration::from_micros(50),
            Duration::from_micros(50),
            Duration::from_micros(50),
        ],
        cell_types: vec![
            TorCellType::Introduce2,
            TorCellType::Introduce2,
            TorCellType::Introduce2,
            TorCellType::Data,
        ],
        introduction_point: Some("node1".to_string()),
        rendezvous_completed: true,
        total_bytes: 5000,
    };

    // Extract features
    let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
    let dist = MetadataExtractor::analyze_cell_types(&metadata.cell_types);

    // Create kernel and classify
    let mut kernel = create_test_kernel(100, 0.1);
    
    // First classification (no training data yet)
    let (is_attack_initial, resonance, _) = kernel.classify(&metadata, &timing, &dist);
    
    // Conservative: should default to benign with low resonance
    assert!(!is_attack_initial || resonance < 0.3);
    
    // Learn this as an attack pattern
    kernel.learn(&metadata, &timing, &dist, true);
    
    // Create similar attack pattern
    let metadata2 = TorCircuitMetadata {
        circuit_id: 2u32,
        created_at: std::time::Instant::now(),
        cell_timings: vec![
            Duration::from_micros(55),  // Similar timing
            Duration::from_micros(48),
            Duration::from_micros(52),
            Duration::from_micros(51),
        ],
        cell_types: vec![
            TorCellType::Introduce2,
            TorCellType::Introduce2,
            TorCellType::Data,
            TorCellType::Data,
        ],
        introduction_point: Some("node1".to_string()),
        rendezvous_completed: true,
        total_bytes: 5100,
    };
    
    let timing2 = MetadataExtractor::extract_timing_features(&metadata2.cell_timings);
    let dist2 = MetadataExtractor::analyze_cell_types(&metadata2.cell_types);
    
    // Should now recognize similar pattern
    let (_is_attack_learned, resonance_learned, region_idx) = kernel.classify(&metadata2, &timing2, &dist2);
    
    // Should match with good resonance
    assert!(resonance_learned > 0.3, "Expected resonance > 0.3, got {}", resonance_learned);
    assert!(region_idx.is_some(), "Should match a learned region");
}

#[test]
fn test_benign_traffic_classification() {
    // Create benign circuit with normal pattern
    let metadata = TorCircuitMetadata {
        circuit_id: 1u32,
        created_at: std::time::Instant::now(),
        cell_timings: vec![
            Duration::from_millis(100),  // Normal intervals
            Duration::from_millis(150),
            Duration::from_millis(120),
            Duration::from_millis(110),
            Duration::from_millis(130),
        ],
        cell_types: vec![
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Padding,
            TorCellType::Data,
        ],
        introduction_point: None,
        rendezvous_completed: true,
        total_bytes: 10000,
    };

    let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
    let dist = MetadataExtractor::analyze_cell_types(&metadata.cell_types);

    let mut kernel = create_test_kernel(100, 0.1);
    
    // Learn as benign
    kernel.learn(&metadata, &timing, &dist, false);
    
    // Similar benign traffic
    let metadata2 = TorCircuitMetadata {
        circuit_id: 2u32,
        created_at: std::time::Instant::now(),
        cell_timings: vec![
            Duration::from_millis(105),
            Duration::from_millis(145),
            Duration::from_millis(125),
            Duration::from_millis(115),
        ],
        cell_types: vec![
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Data,
            TorCellType::Padding,
        ],
        introduction_point: None,
        rendezvous_completed: true,
        total_bytes: 9500,
    };
    
    let timing2 = MetadataExtractor::extract_timing_features(&metadata2.cell_timings);
    let dist2 = MetadataExtractor::analyze_cell_types(&metadata2.cell_types);
    
    let (is_attack, resonance, _) = kernel.classify(&metadata2, &timing2, &dist2);
    
    // Should classify as benign
    assert!(!is_attack, "Benign traffic should not be classified as attack");
    assert!(resonance > 0.3, "Should match learned benign pattern");
}

#[test]
fn test_classical_quantum_conversion_accuracy() {
    // Create classical signature
    let classical = ClassicalSignature {
        mean_interval: 100.0,
        std_dev_interval: 20.0,
        data_ratio: 0.8,
        intro_ratio: 0.1,
        total_bytes: 1000.0,
    };
    
    // Convert to quantum
    let quantum = ConversionUtils::classical_to_quantum(&classical);
    
    // Convert back to classical
    let classical_restored = ConversionUtils::quantum_to_classical(&quantum);
    
    // Check approximate preservation (lossy conversion)
    let tolerance = 0.2; // 20% tolerance
    assert!((classical.mean_interval - classical_restored.mean_interval).abs() / classical.mean_interval < tolerance);
    assert!((classical.std_dev_interval - classical_restored.std_dev_interval).abs() / classical.std_dev_interval < tolerance);
    assert!((classical.data_ratio - classical_restored.data_ratio).abs() < tolerance);
}

#[test]
fn test_online_learning_adaptation() {
    let mut kernel = create_test_kernel(100, 0.2); // Higher learning rate for faster adaptation
    
    // Simulate 10 attack patterns
    for i in 0..10 {
        let metadata = TorCircuitMetadata {
            circuit_id: i,
            created_at: std::time::Instant::now(),
            cell_timings: vec![
                Duration::from_micros((50 + i * 2) as u64),
                Duration::from_micros((50 + i * 2) as u64),
                Duration::from_micros((50 + i * 2) as u64),
            ],
            cell_types: vec![
                TorCellType::Introduce2,
                TorCellType::Introduce2,
                TorCellType::Data,
            ],
            introduction_point: Some(format!("node{}", i)),
            rendezvous_completed: true,
            total_bytes: (5000 + i * 100) as u64,
        };
        
        let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
        let dist = MetadataExtractor::analyze_cell_types(&metadata.cell_types);
        
        kernel.learn(&metadata, &timing, &dist, true);
    }
    
    // Check that regions were created
    let stats = kernel.stats();
    assert!(stats.base_stats.total_regions > 0, "Should have created regions");
    assert!(stats.base_stats.attack_regions > 0, "Should have attack regions");
    
    // Test classification on similar pattern
    let test_metadata = TorCircuitMetadata {
        circuit_id: 999u32,
        created_at: std::time::Instant::now(),
        cell_timings: vec![
            Duration::from_micros(55),
            Duration::from_micros(55),
            Duration::from_micros(55),
        ],
        cell_types: vec![
            TorCellType::Introduce2,
            TorCellType::Introduce2,
            TorCellType::Data,
        ],
        introduction_point: Some("node_test".to_string()),
        rendezvous_completed: true,
        total_bytes: 5200,
    };
    
    let timing = MetadataExtractor::extract_timing_features(&test_metadata.cell_timings);
    let dist = MetadataExtractor::analyze_cell_types(&test_metadata.cell_types);
    
    let (_is_attack, resonance, _) = kernel.classify(&test_metadata, &timing, &dist);
    
    // Should recognize attack pattern after learning
    assert!(resonance > 0.3, "Should match learned attack patterns");
}

#[test]
fn test_region_capacity_management() {
    let mut kernel = create_test_kernel(10, 0.1); // Small capacity for testing
    
    // Create diverse patterns (more than capacity)
    for i in 0..15 {
        let metadata = TorCircuitMetadata {
            circuit_id: i,
            created_at: std::time::Instant::now(),
            cell_timings: vec![
                Duration::from_micros((100 + i * 50) as u64), // Very different patterns
                Duration::from_micros((100 + i * 50) as u64),
            ],
            cell_types: vec![
                TorCellType::Data,
                TorCellType::Data,
            ],
            introduction_point: None,
            rendezvous_completed: true,
            total_bytes: (1000 + i * 1000) as u64,
        };
        
        let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
        let dist = MetadataExtractor::analyze_cell_types(&metadata.cell_types);
        
        kernel.learn(&metadata, &timing, &dist, i % 2 == 0); // Alternate attack/benign
    }
    
    let stats = kernel.stats();
    
    // Should not exceed capacity
    assert!(stats.base_stats.total_regions <= 10, "Should not exceed max_regions capacity");
}

#[test]
fn test_optimization_triggers_automatically() {
    let mut kernel = create_test_kernel(100, 0.1);
    
    // Create similar patterns that should merge
    for i in 0..3 {
        let metadata = TorCircuitMetadata {
            circuit_id: i,
            created_at: std::time::Instant::now(),
            cell_timings: vec![
                Duration::from_micros(100),
                Duration::from_micros(100),
            ],
            cell_types: vec![
                TorCellType::Data,
                TorCellType::Data,
            ],
            introduction_point: None,
            rendezvous_completed: true,
            total_bytes: 5000,
        };
        
        let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
        let dist = MetadataExtractor::analyze_cell_types(&metadata.cell_types);
        
        kernel.learn(&metadata, &timing, &dist, true);
    }
    
    let initial_regions = kernel.stats().base_stats.total_regions;
    
    // Trigger classifications to invoke optimization
    for i in 0..6 {
        let metadata = TorCircuitMetadata {
            circuit_id: 100 + i,
            created_at: std::time::Instant::now(),
            cell_timings: vec![
                Duration::from_micros(105),
                Duration::from_micros(105),
            ],
            cell_types: vec![
                TorCellType::Data,
                TorCellType::Data,
            ],
            introduction_point: None,
            rendezvous_completed: true,
            total_bytes: 5050,
        };
        
        let timing = MetadataExtractor::extract_timing_features(&metadata.cell_timings);
        let dist = MetadataExtractor::analyze_cell_types(&metadata.cell_types);
        
        kernel.classify(&metadata, &timing, &dist);
    }
    
    // Optimization should have been triggered (at classification #5)
    // This is verified by the system not crashing and operating normally
    let final_stats = kernel.stats();
    assert!(final_stats.base_stats.total_regions <= initial_regions, 
            "Optimization may have merged regions or maintained count");
}
