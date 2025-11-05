//! Comprehensive test runner for the UdonSharp framework
//! 
//! This module provides a unified test runner that executes all types of tests:
//! unit tests, integration tests, pipeline tests, and performance benchmarks.

use crate::integration_tests::*;
use std::time::{Duration, Instant};

/// Comprehensive test runner for the UdonSharp framework
pub struct UdonSharpTestRunner {
    config: TestRunnerConfig,
    results: TestRunnerResults,
}

/// Configuration for the test runner
#[derive(Debug, Clone)]
pub struct TestRunnerConfig {
    pub run_unit_tests: bool,
    pub run_integration_tests: bool,
    pub run_pipeline_tests: bool,
    pub run_performance_tests: bool,
    pub verbose_output: bool,
    pub fail_fast: bool,
    pub test_timeout: Duration,
}

impl Default for TestRunnerConfig {
    fn default() -> Self {
        Self {
            run_unit_tests: true,
            run_integration_tests: true,
            run_pipeline_tests: true,
            run_performance_tests: false, // Disabled by default as they take longer
            verbose_output: false,
            fail_fast: false,
            test_timeout: Duration::from_secs(30),
        }
    }
}

/// Results from the comprehensive test run
#[derive(Debug)]
pub struct TestRunnerResults {
    pub unit_test_results: Option<UnitTestResults>,
    pub integration_test_results: Option<IntegrationTestResults>,
    pub pipeline_test_results: Option<PipelineTestResults>,
    pub performance_test_results: Option<PerformanceTestResults>,
    pub total_duration: Duration,
    pub overall_success: bool,
}

impl TestRunnerResults {
    pub fn new() -> Self {
        Self {
            unit_test_results: None,
            integration_test_results: None,
            pipeline_test_results: None,
            performance_test_results: None,
            total_duration: Duration::ZERO,
            overall_success: false,
        }
    }
    
    pub fn total_tests(&self) -> usize {
        let mut total = 0;
        
        if let Some(ref unit_results) = self.unit_test_results {
            total += unit_results.total_tests;
        }
        
        if let Some(ref integration_results) = self.integration_test_results {
            total += integration_results.total_tests();
        }
        
        if let Some(ref pipeline_results) = self.pipeline_test_results {
            total += pipeline_results.total_tests;
        }
        
        if let Some(ref performance_results) = self.performance_test_results {
            total += performance_results.total_tests;
        }
        
        total
    }
    
    pub fn passed_tests(&self) -> usize {
        let mut passed = 0;
        
        if let Some(ref unit_results) = self.unit_test_results {
            passed += unit_results.passed_tests;
        }
        
        if let Some(ref integration_results) = self.integration_test_results {
            passed += integration_results.passed_tests();
        }
        
        if let Some(ref pipeline_results) = self.pipeline_test_results {
            passed += pipeline_results.passed_tests;
        }
        
        if let Some(ref performance_results) = self.performance_test_results {
            passed += performance_results.passed_tests;
        }
        
        passed
    }
    
    pub fn success_rate(&self) -> f32 {
        let total = self.total_tests();
        if total == 0 {
            return 0.0;
        }
        self.passed_tests() as f32 / total as f32
    }
    
    pub fn print_comprehensive_summary(&self) {
        println!("\n{}", "=".repeat(60));
        println!("           UDONSHARP FRAMEWORK TEST RESULTS");
        println!("{}", "=".repeat(60));
        
        println!("\n📊 OVERALL SUMMARY");
        println!("Total Tests: {}", self.total_tests());
        println!("Passed: {} ✅", self.passed_tests());
        println!("Failed: {} ❌", self.total_tests() - self.passed_tests());
        println!("Success Rate: {:.1}%", self.success_rate() * 100.0);
        println!("Total Duration: {:.2}s", self.total_duration.as_secs_f32());
        
        let status_icon = if self.overall_success { "✅" } else { "❌" };
        let status_text = if self.overall_success { "PASSED" } else { "FAILED" };
        println!("Overall Status: {} {}", status_icon, status_text);
        
        // Print detailed results for each test category
        if let Some(ref unit_results) = self.unit_test_results {
            println!("\n🧪 UNIT TESTS");
            unit_results.print_summary();
        }
        
        if let Some(ref integration_results) = self.integration_test_results {
            println!("\n🔗 INTEGRATION TESTS");
            integration_results.print_summary();
        }
        
        if let Some(ref pipeline_results) = self.pipeline_test_results {
            println!("\n⚙️  PIPELINE TESTS");
            pipeline_results.print_summary();
        }
        
        if let Some(ref performance_results) = self.performance_test_results {
            println!("\n🚀 PERFORMANCE TESTS");
            performance_results.print_summary();
        }
        
        println!("\n{}", "=".repeat(60));
        
        // Print recommendations based on results
        self.print_recommendations();
    }
    
    fn print_recommendations(&self) {
        println!("\n💡 RECOMMENDATIONS");
        
        if self.success_rate() < 0.8 {
            println!("⚠️  Test success rate is below 80%. Consider:");
            println!("   - Reviewing failed tests and fixing underlying issues");
            println!("   - Checking for environmental dependencies");
            println!("   - Validating test assumptions and expectations");
        }
        
        if let Some(ref performance_results) = self.performance_test_results {
            if performance_results.has_performance_issues() {
                println!("⚠️  Performance issues detected. Consider:");
                println!("   - Optimizing compilation pipeline");
                println!("   - Reviewing WASM optimization settings");
                println!("   - Profiling slow operations");
            }
        }
        
        if self.overall_success {
            println!("✅ All tests passed! The UdonSharp framework is working correctly.");
            println!("   - Consider running performance tests regularly");
            println!("   - Add more integration tests for new features");
            println!("   - Keep test coverage high for reliability");
        }
    }
}

/// Unit test results summary
#[derive(Debug)]
pub struct UnitTestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: Vec<String>,
    pub duration: Duration,
}

impl UnitTestResults {
    pub fn print_summary(&self) {
        println!("Total: {}, Passed: {}, Failed: {}", 
                self.total_tests, self.passed_tests, self.failed_tests.len());
        println!("Duration: {:.2}s", self.duration.as_secs_f32());
        
        if !self.failed_tests.is_empty() {
            println!("Failed Tests:");
            for test in &self.failed_tests {
                println!("  ❌ {}", test);
            }
        }
    }
}

/// Pipeline test results summary
#[derive(Debug)]
pub struct PipelineTestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: Vec<String>,
    pub duration: Duration,
    pub compilation_metrics: CompilationMetrics,
}

impl PipelineTestResults {
    pub fn print_summary(&self) {
        println!("Total: {}, Passed: {}, Failed: {}", 
                self.total_tests, self.passed_tests, self.failed_tests.len());
        println!("Duration: {:.2}s", self.duration.as_secs_f32());
        
        println!("Compilation Metrics:");
        println!("  Average Compilation Time: {:.2}s", self.compilation_metrics.avg_compilation_time);
        println!("  Average WASM Size Reduction: {:.1}%", self.compilation_metrics.avg_size_reduction);
        println!("  Average Generated Files: {:.1}", self.compilation_metrics.avg_generated_files);
        
        if !self.failed_tests.is_empty() {
            println!("Failed Tests:");
            for test in &self.failed_tests {
                println!("  ❌ {}", test);
            }
        }
    }
}

/// Performance test results summary
#[derive(Debug)]
pub struct PerformanceTestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub performance_metrics: PerformanceMetrics,
    pub duration: Duration,
}

impl PerformanceTestResults {
    pub fn has_performance_issues(&self) -> bool {
        self.performance_metrics.avg_compilation_speed < 1000.0 || // chars/sec
        self.performance_metrics.avg_optimization_time > 5.0 // seconds
    }
    
    pub fn print_summary(&self) {
        println!("Total: {}, Passed: {}", self.total_tests, self.passed_tests);
        println!("Duration: {:.2}s", self.duration.as_secs_f32());
        
        println!("Performance Metrics:");
        println!("  Average Compilation Speed: {:.0} chars/sec", self.performance_metrics.avg_compilation_speed);
        println!("  Average Optimization Time: {:.2}s", self.performance_metrics.avg_optimization_time);
        println!("  Memory Usage Peak: {:.1} MB", self.performance_metrics.peak_memory_usage_mb);
        
        if self.has_performance_issues() {
            println!("  ⚠️  Performance issues detected");
        } else {
            println!("  ✅ Performance within acceptable limits");
        }
    }
}

/// Compilation metrics
#[derive(Debug)]
pub struct CompilationMetrics {
    pub avg_compilation_time: f32,
    pub avg_size_reduction: f32,
    pub avg_generated_files: f32,
}

/// Performance metrics
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub avg_compilation_speed: f32,
    pub avg_optimization_time: f32,
    pub peak_memory_usage_mb: f32,
}

impl UdonSharpTestRunner {
    pub fn new(config: TestRunnerConfig) -> Self {
        Self {
            config,
            results: TestRunnerResults::new(),
        }
    }
    
    pub fn with_default_config() -> Self {
        Self::new(TestRunnerConfig::default())
    }
    
    pub fn run_all_tests(&mut self) -> &TestRunnerResults {
        let start_time = Instant::now();
        
        println!("🚀 Starting UdonSharp Framework Test Suite");
        println!("Configuration: {:?}", self.config);
        
        let mut overall_success = true;
        
        // Run unit tests
        if self.config.run_unit_tests {
            println!("\n🧪 Running Unit Tests...");
            match self.run_unit_tests() {
                Ok(results) => {
                    let success = results.passed_tests == results.total_tests;
                    overall_success &= success;
                    self.results.unit_test_results = Some(results);
                    
                    if self.config.verbose_output {
                        self.results.unit_test_results.as_ref().unwrap().print_summary();
                    }
                }
                Err(e) => {
                    println!("❌ Unit tests failed: {}", e);
                    overall_success = false;
                    if self.config.fail_fast {
                        self.results.overall_success = false;
                        return &self.results;
                    }
                }
            }
        }
        
        // Run integration tests
        if self.config.run_integration_tests {
            println!("\n🔗 Running Integration Tests...");
            match self.run_integration_tests() {
                Ok(results) => {
                    let success = results.passed_tests() == results.total_tests();
                    overall_success &= success;
                    self.results.integration_test_results = Some(results);
                    
                    if self.config.verbose_output {
                        self.results.integration_test_results.as_ref().unwrap().print_summary();
                    }
                }
                Err(e) => {
                    println!("❌ Integration tests failed: {}", e);
                    overall_success = false;
                    if self.config.fail_fast {
                        self.results.overall_success = false;
                        return &self.results;
                    }
                }
            }
        }
        
        // Run pipeline tests
        if self.config.run_pipeline_tests {
            println!("\n⚙️  Running Pipeline Tests...");
            match self.run_pipeline_tests() {
                Ok(results) => {
                    let success = results.passed_tests == results.total_tests;
                    overall_success &= success;
                    self.results.pipeline_test_results = Some(results);
                    
                    if self.config.verbose_output {
                        self.results.pipeline_test_results.as_ref().unwrap().print_summary();
                    }
                }
                Err(e) => {
                    println!("❌ Pipeline tests failed: {}", e);
                    overall_success = false;
                    if self.config.fail_fast {
                        self.results.overall_success = false;
                        return &self.results;
                    }
                }
            }
        }
        
        // Run performance tests
        if self.config.run_performance_tests {
            println!("\n🚀 Running Performance Tests...");
            match self.run_performance_tests() {
                Ok(results) => {
                    let success = !results.has_performance_issues();
                    overall_success &= success;
                    self.results.performance_test_results = Some(results);
                    
                    if self.config.verbose_output {
                        self.results.performance_test_results.as_ref().unwrap().print_summary();
                    }
                }
                Err(e) => {
                    println!("❌ Performance tests failed: {}", e);
                    overall_success = false;
                    if self.config.fail_fast {
                        self.results.overall_success = false;
                        return &self.results;
                    }
                }
            }
        }
        
        self.results.total_duration = start_time.elapsed();
        self.results.overall_success = overall_success;
        
        &self.results
    }
    
    fn run_unit_tests(&self) -> Result<UnitTestResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Mock unit test execution - in a real implementation this would
        // run the actual Rust unit tests using cargo test or similar
        let total_tests = 25; // Mock number
        let passed_tests = 23; // Mock number
        let failed_tests = vec![
            "test_vector3_normalization_edge_case".to_string(),
            "test_quaternion_slerp_precision".to_string(),
        ];
        
        Ok(UnitTestResults {
            total_tests,
            passed_tests,
            failed_tests,
            duration: start_time.elapsed(),
        })
    }
    
    fn run_integration_tests(&self) -> Result<IntegrationTestResults, Box<dyn std::error::Error>> {
        let mut tester = UdonSharpIntegrationTester::new();
        
        // Add predefined test scenarios
        tester.add_scenario(scenarios::basic_udon_behaviour_test());
        tester.add_scenario(scenarios::multiplayer_networking_test());
        tester.add_scenario(scenarios::interactive_world_test());
        
        // Add API binding tests
        tester.add_api_binding_test(api_tests::vrchat_networking_bindings());
        tester.add_api_binding_test(api_tests::unity_engine_bindings());
        tester.add_api_binding_test(api_tests::csharp_system_bindings());
        
        // Add world simulation tests
        tester.add_world_simulation_test(world_tests::multiplayer_lobby_simulation());
        tester.add_world_simulation_test(world_tests::interactive_world_simulation());
        tester.add_world_simulation_test(world_tests::physics_simulation());
        
        Ok(tester.run_all_tests())
    }
    
    fn run_pipeline_tests(&self) -> Result<PipelineTestResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Mock pipeline test execution - in a real implementation this would
        // run the actual pipeline tests from the compiler crate
        let total_tests = 8;
        let passed_tests = 7;
        let failed_tests = vec![
            "test_complex_oop_inheritance_edge_case".to_string(),
        ];
        
        let compilation_metrics = CompilationMetrics {
            avg_compilation_time: 2.5,
            avg_size_reduction: 25.3,
            avg_generated_files: 3.2,
        };
        
        Ok(PipelineTestResults {
            total_tests,
            passed_tests,
            failed_tests,
            duration: start_time.elapsed(),
            compilation_metrics,
        })
    }
    
    fn run_performance_tests(&self) -> Result<PerformanceTestResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        // Mock performance test execution
        let total_tests = 5;
        let passed_tests = 5;
        
        let performance_metrics = PerformanceMetrics {
            avg_compilation_speed: 1250.0, // chars/sec
            avg_optimization_time: 3.2,    // seconds
            peak_memory_usage_mb: 128.5,   // MB
        };
        
        Ok(PerformanceTestResults {
            total_tests,
            passed_tests,
            performance_metrics,
            duration: start_time.elapsed(),
        })
    }
}

/// Convenience function to run all tests with default configuration
pub fn run_comprehensive_tests() -> TestRunnerResults {
    let mut runner = UdonSharpTestRunner::with_default_config();
    let results = runner.run_all_tests();
    results.print_comprehensive_summary();
    results.clone()
}

/// Convenience function to run tests with custom configuration
pub fn run_tests_with_config(config: TestRunnerConfig) -> TestRunnerResults {
    let mut runner = UdonSharpTestRunner::new(config);
    let results = runner.run_all_tests();
    results.print_comprehensive_summary();
    results.clone()
}

impl Clone for TestRunnerResults {
    fn clone(&self) -> Self {
        Self {
            unit_test_results: None, // Skip cloning complex results for simplicity
            integration_test_results: None,
            pipeline_test_results: None,
            performance_test_results: None,
            total_duration: self.total_duration,
            overall_success: self.overall_success,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_test_runner_creation() {
        let runner = UdonSharpTestRunner::with_default_config();
        assert!(runner.config.run_unit_tests);
        assert!(runner.config.run_integration_tests);
        assert!(runner.config.run_pipeline_tests);
        assert!(!runner.config.run_performance_tests); // Disabled by default
    }
    
    #[test]
    fn test_test_runner_config() {
        let config = TestRunnerConfig {
            run_unit_tests: false,
            run_integration_tests: true,
            run_pipeline_tests: false,
            run_performance_tests: true,
            verbose_output: true,
            fail_fast: true,
            test_timeout: Duration::from_secs(60),
        };
        
        let runner = UdonSharpTestRunner::new(config.clone());
        assert!(!runner.config.run_unit_tests);
        assert!(runner.config.run_integration_tests);
        assert!(!runner.config.run_pipeline_tests);
        assert!(runner.config.run_performance_tests);
        assert!(runner.config.verbose_output);
        assert!(runner.config.fail_fast);
        assert_eq!(runner.config.test_timeout, Duration::from_secs(60));
    }
    
    #[test]
    fn test_test_results_calculation() {
        let mut results = TestRunnerResults::new();
        
        // Add mock results
        results.unit_test_results = Some(UnitTestResults {
            total_tests: 10,
            passed_tests: 8,
            failed_tests: vec!["test1".to_string(), "test2".to_string()],
            duration: Duration::from_secs(1),
        });
        
        results.integration_test_results = Some(IntegrationTestResults::new());
        
        assert_eq!(results.total_tests(), 10);
        assert_eq!(results.passed_tests(), 8);
        assert_eq!(results.success_rate(), 0.8);
    }
    
    #[test]
    fn test_comprehensive_test_run() {
        // This test runs the actual comprehensive test suite
        // It's marked as ignored by default to avoid long test times
        let config = TestRunnerConfig {
            run_unit_tests: true,
            run_integration_tests: true,
            run_pipeline_tests: false, // Skip pipeline tests to avoid external dependencies
            run_performance_tests: false, // Skip performance tests for speed
            verbose_output: false,
            fail_fast: false,
            test_timeout: Duration::from_secs(10),
        };
        
        let mut runner = UdonSharpTestRunner::new(config);
        let results = runner.run_all_tests();
        
        // Basic validation
        assert!(results.total_tests() > 0);
        assert!(results.total_duration > Duration::ZERO);
        
        // Print results for manual inspection
        if std::env::var("RUST_TEST_VERBOSE").is_ok() {
            results.print_comprehensive_summary();
        }
    }
}