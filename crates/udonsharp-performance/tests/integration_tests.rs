//! Integration tests for UdonSharp performance monitoring system

use udonsharp_performance::*;
use udonsharp_performance::analysis::{RecommendationCategory, RecommendationPriority, EffortLevel};
use udonsharp_performance::metrics::VRChatMetrics;
use udonsharp_performance::optimizer::CodeLanguage;
use udonsharp_performance::profiler::{OptimizationCategory, CompletedOperation};
use udonsharp_performance::reporter::{PerformanceReport, ReportMetadata, ReportSummary};
use std::time::Duration;

#[tokio::test]
async fn test_performance_monitoring_session() {
    let mut monitor = UdonPerformanceMonitor::new().expect("Failed to create monitor");
    
    // Start a monitoring session
    let session_id = monitor.start_session("test_session").expect("Failed to start session");
    
    // Simulate some compilation steps
    monitor.start_step(&session_id, "rust_compilation").expect("Failed to start step");
    tokio::time::sleep(Duration::from_millis(100)).await;
    monitor.end_step(&session_id, "rust_compilation", true, None).expect("Failed to end step");
    
    monitor.start_step(&session_id, "wasm_generation").expect("Failed to start step");
    tokio::time::sleep(Duration::from_millis(50)).await;
    monitor.end_step(&session_id, "wasm_generation", true, None).expect("Failed to end step");
    
    // End the session
    monitor.end_session(&session_id).expect("Failed to end session");
    
    // Get metrics
    let metrics = monitor.get_session_metrics(&session_id).expect("Failed to get metrics");
    
    assert!(metrics.session_duration > Duration::from_millis(100));
    assert_eq!(metrics.step_count, 2);
    assert_eq!(metrics.failed_steps, 0);
}

#[test]
fn test_compilation_profiler() {
    let mut profiler = CompilationProfiler::new();
    
    profiler.start_session();
    
    // Profile a simple operation
    let result = profiler.profile_operation("test_operation", || {
        std::thread::sleep(Duration::from_millis(10));
        Ok("success")
    });
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
    
    let metrics = profiler.end_session().expect("Failed to end session");
    assert!(metrics.total_compilation_time >= Duration::from_millis(10));
}

#[test]
fn test_code_optimizer() {
    let optimizer = CodeOptimizer::new();
    
    let test_code = r#"
        fn test_function() {
            let _unused_var = 42;
            println!("Hello, world!");
        }
    "#;
    
    let optimized = optimizer.optimize(test_code, OptimizationStrategy::Development)
        .expect("Failed to optimize code");
    
    // Should remove unused variable
    assert!(!optimized.contains("_unused_var"));
    assert!(optimized.contains("println!"));
}

#[test]
fn test_performance_analyzer() {
    let analyzer = PerformanceAnalyzer::new();
    
    let mut metrics = PerformanceMetrics {
        compilation: CompilationMetrics {
            total_compilation_time: Duration::from_secs(60), // Slow compilation
            ..Default::default()
        },
        memory: MemoryMetrics {
            peak_usage: 1024 * 1024 * 1024, // 1GB - high usage
            ..Default::default()
        },
        session_duration: Duration::from_secs(60),
        step_count: 10,
        failed_steps: 0,
    };
    
    let recommendations = analyzer.analyze_metrics(&metrics)
        .expect("Failed to analyze metrics");
    
    // Should generate recommendations for slow compilation and high memory usage
    assert!(!recommendations.is_empty());
    
    let has_compilation_rec = recommendations.iter()
        .any(|r| r.category == RecommendationCategory::CompilationSpeed);
    let has_memory_rec = recommendations.iter()
        .any(|r| r.category == RecommendationCategory::MemoryUsage);
    
    assert!(has_compilation_rec || has_memory_rec);
}

#[test]
fn test_performance_reporter() {
    let reporter = PerformanceReporter::new();
    
    let metrics = PerformanceMetrics {
        compilation: CompilationMetrics {
            total_compilation_time: Duration::from_secs(30),
            rust_lines_of_code: 1000,
            compilation_speed_loc_per_second: 33.3,
            ..Default::default()
        },
        memory: MemoryMetrics {
            peak_usage: 100 * 1024 * 1024, // 100MB
            average_usage: 80 * 1024 * 1024, // 80MB
            ..Default::default()
        },
        session_duration: Duration::from_secs(35),
        step_count: 5,
        failed_steps: 0,
    };
    
    let recommendations = vec![
        OptimizationRecommendation {
            category: RecommendationCategory::CompilationSpeed,
            priority: RecommendationPriority::Medium,
            title: "Test Recommendation".to_string(),
            description: "Test description".to_string(),
            rationale: "Test rationale".to_string(),
            implementation_steps: vec!["Step 1".to_string()],
            estimated_improvement: 20.0,
            estimated_effort: EffortLevel::Low,
            prerequisites: vec![],
            related_metrics: vec![],
        }
    ];
    
    // Test console report generation - simplified test
    let report = reporter.create_performance_report("test_session", &metrics, &recommendations, &[])
        .expect("Failed to create report");
    
    assert_eq!(report.metadata.session_id, "test_session");
    assert!(report.summary.overall_score > 0.0);
}

#[test]
fn test_vrchat_metrics_analysis() {
    let mut vrchat_metrics = VRChatMetrics {
        estimated_instruction_count: 750_000, // High instruction count
        estimated_memory_footprint: 75_000_000, // 75MB
        network_sync_variables: 150,
        udon_event_count: 20,
        performance_rank_estimate: crate::metrics::PerformanceRank::Unknown,
        vrchat_compatibility_score: 0.0,
    };
    
    // Test performance rank estimation
    vrchat_metrics.estimate_performance_rank();
    
    assert_ne!(vrchat_metrics.performance_rank_estimate, crate::metrics::PerformanceRank::Unknown);
    assert!(vrchat_metrics.vrchat_compatibility_score > 0.0);
    
    // Test limit checking
    let issues = vrchat_metrics.check_vrchat_limits();
    
    // Should have issues due to high instruction count
    assert!(!issues.is_empty());
    
    let has_instruction_issue = issues.iter()
        .any(|issue| issue.description.contains("instruction count"));
    assert!(has_instruction_issue);
}

#[test]
fn test_optimization_opportunities() {
    let optimizer = CodeOptimizer::new();
    
    let test_code = r#"
        for i in 0..1000 {
            let vec = Vec::new();
            vec.push(i);
        }
    "#;
    
    let opportunities = optimizer.analyze_optimization_opportunities(
        test_code, 
        CodeLanguage::Rust
    ).expect("Failed to analyze opportunities");
    
    assert!(!opportunities.is_empty());
    
    // Should identify loop and memory optimization opportunities
    let has_loop_opt = opportunities.iter()
        .any(|op| op.category == OptimizationCategory::ComputationSpeed);
    let has_memory_opt = opportunities.iter()
        .any(|op| op.category == OptimizationCategory::MemoryUsage);
    
    assert!(has_loop_opt || has_memory_opt);
}

