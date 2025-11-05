//! Performance monitoring and optimization tools for UdonSharp compilation pipeline
//!
//! This crate provides comprehensive performance monitoring, profiling, and optimization
//! capabilities for the Rust-to-UdonSharp compilation pipeline.

pub mod monitor;
pub mod profiler;
pub mod optimizer;
pub mod reporter;
pub mod metrics;
pub mod analysis;
pub mod world_analyzer;
pub mod multi_behavior_metrics;

pub use monitor::UdonPerformanceMonitor;
pub use profiler::{CompilationProfiler, CodeProfiler};
pub use optimizer::{CodeOptimizer, OptimizationPass, OptimizationStrategy};
pub use reporter::{PerformanceReporter, ReportFormat};
pub use metrics::{CompilationMetrics, PerformanceMetrics, MemoryMetrics};
pub use analysis::{PerformanceAnalyzer, OptimizationRecommendation};
pub use world_analyzer::{VRChatWorldAnalyzer, WorldPerformanceAnalysis, VRChatWorldMetrics};
pub use multi_behavior_metrics::{
    MultiBehaviorMetrics, MultiBehaviorAnalyzer, MultiBehaviorReport, 
    BehaviorMetrics, OptimizationOpportunity, SharingRecommendation
};

use anyhow::Result;
use std::time::Duration;

/// Main performance monitoring and optimization facade
pub struct UdonSharpPerformance {
    monitor: UdonPerformanceMonitor,
    profiler: CompilationProfiler,
    optimizer: CodeOptimizer,
    reporter: PerformanceReporter,
    analyzer: PerformanceAnalyzer,
    world_analyzer: VRChatWorldAnalyzer,
    multi_behavior_analyzer: multi_behavior_metrics::MultiBehaviorAnalyzer,
}

impl UdonSharpPerformance {
    /// Create a new performance monitoring system
    pub fn new() -> Result<Self> {
        Ok(Self {
            monitor: UdonPerformanceMonitor::new()?,
            profiler: CompilationProfiler::new(),
            optimizer: CodeOptimizer::new(),
            reporter: PerformanceReporter::new(),
            analyzer: PerformanceAnalyzer::new(),
            world_analyzer: VRChatWorldAnalyzer::new(),
            multi_behavior_analyzer: multi_behavior_metrics::MultiBehaviorAnalyzer::new(),
        })
    }

    /// Start monitoring a compilation session
    pub fn start_session(&mut self, session_name: &str) -> Result<String> {
        self.monitor.start_session(session_name)
    }

    /// End a monitoring session and generate report
    pub fn end_session(&mut self, session_id: &str) -> Result<()> {
        self.monitor.end_session(session_id)?;
        let metrics = self.monitor.get_session_metrics(session_id)?;
        let recommendations = self.analyzer.analyze_metrics(&metrics)?;
        self.reporter.generate_session_report(session_id, &metrics, &recommendations)?;
        Ok(())
    }

    /// Profile a compilation step
    pub fn profile_step<F, R>(&mut self, step_name: &str, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        self.profiler.profile_operation(step_name, operation)
    }

    /// Optimize generated code
    pub fn optimize_code(&self, code: &str, strategy: OptimizationStrategy) -> Result<String> {
        self.optimizer.optimize(code, strategy)
    }

    /// Get performance recommendations
    pub fn get_recommendations(&self, metrics: &PerformanceMetrics) -> Result<Vec<OptimizationRecommendation>> {
        self.analyzer.analyze_metrics(metrics)
    }

    /// Analyze VRChat world performance
    pub fn analyze_vrchat_world(&self, world_metrics: &VRChatWorldMetrics) -> Result<WorldPerformanceAnalysis> {
        self.world_analyzer.analyze_world_performance(world_metrics)
    }

    /// Optimize code for VRChat constraints
    pub fn optimize_for_vrchat(&self, code: &str) -> Result<String> {
        self.optimizer.optimize(code, OptimizationStrategy::VRChatOptimal)
    }

    /// Optimize C# code for VRChat
    pub fn optimize_csharp_for_vrchat(&self, code: &str) -> Result<String> {
        self.optimizer.optimize_csharp(code, OptimizationStrategy::VRChatOptimal)
    }

    /// Analyze multi-behavior compilation performance
    pub fn analyze_multi_behavior_compilation(
        &mut self,
        behavior_count: usize,
        shared_functions: &[String],
        inter_behavior_calls: usize,
        dependency_analysis_time: Duration,
        generation_times: &multi_behavior_metrics::FileGenerationMetrics,
    ) -> &multi_behavior_metrics::MultiBehaviorMetrics {
        self.multi_behavior_analyzer.analyze_compilation_result(
            behavior_count,
            shared_functions,
            inter_behavior_calls,
            dependency_analysis_time,
            generation_times,
        )
    }

    /// Add behavior-specific metrics
    pub fn add_behavior_metrics(&mut self, name: String, metrics: multi_behavior_metrics::BehaviorMetrics) {
        self.multi_behavior_analyzer.add_behavior_metrics(name, metrics);
    }

    /// Generate multi-behavior performance report
    pub fn generate_multi_behavior_report(&self) -> multi_behavior_metrics::MultiBehaviorReport {
        self.multi_behavior_analyzer.generate_report()
    }

    /// Get multi-behavior optimization recommendations
    pub fn get_multi_behavior_recommendations(&self) -> Vec<multi_behavior_metrics::OptimizationOpportunity> {
        self.multi_behavior_analyzer.get_metrics().generate_optimization_recommendations()
    }
}

impl Default for UdonSharpPerformance {
    fn default() -> Self {
        Self::new().expect("Failed to create UdonSharpPerformance")
    }
}