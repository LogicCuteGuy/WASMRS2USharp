//! Profiling tools for analyzing compilation performance

use crate::metrics::{CompilationMetrics, CodeQualityMetrics, PerformanceIssue};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Profiler for compilation pipeline operations
#[derive(Debug)]
pub struct CompilationProfiler {
    active_operations: HashMap<String, ProfilingOperation>,
    completed_operations: Vec<CompletedOperation>,
    start_time: Option<Instant>,
}

/// Profiler for analyzing generated code performance
#[derive(Debug)]
pub struct CodeProfiler {
    analysis_cache: HashMap<String, CodeAnalysis>,
}

/// Active profiling operation
#[derive(Debug, Clone)]
struct ProfilingOperation {
    name: String,
    start_time: Instant,
    memory_before: u64,
    parent: Option<String>,
}

/// Completed profiling operation with results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedOperation {
    pub name: String,
    pub duration: Duration,
    pub memory_used: u64,
    pub memory_peak: u64,
    pub success: bool,
    pub error_message: Option<String>,
    pub sub_operations: Vec<CompletedOperation>,
    pub metadata: HashMap<String, String>,
}

/// Code analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub file_path: String,
    pub lines_of_code: u32,
    pub complexity_score: f64,
    pub performance_issues: Vec<PerformanceIssue>,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub udonsharp_compatibility: CompatibilityAnalysis,
}

/// Optimization opportunity identified in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    pub category: OptimizationCategory,
    pub description: String,
    pub location: Option<crate::metrics::CodeLocation>,
    pub estimated_improvement: f64, // percentage improvement
    pub difficulty: OptimizationDifficulty,
    pub suggested_changes: Vec<String>,
}

/// Category of optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationCategory {
    MemoryUsage,
    ComputationSpeed,
    NetworkEfficiency,
    CodeStructure,
    UdonSharpSpecific,
    VRChatOptimization,
}

/// Difficulty level of implementing an optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationDifficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

/// UdonSharp compatibility analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityAnalysis {
    pub compatibility_score: f64,
    pub unsupported_features: Vec<String>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

impl CompilationProfiler {
    /// Create a new compilation profiler
    pub fn new() -> Self {
        Self {
            active_operations: HashMap::new(),
            completed_operations: Vec::new(),
            start_time: None,
        }
    }

    /// Start profiling the entire compilation session
    pub fn start_session(&mut self) {
        self.start_time = Some(Instant::now());
        self.active_operations.clear();
        self.completed_operations.clear();
    }

    /// End the profiling session and return metrics
    pub fn end_session(&mut self) -> Result<CompilationMetrics> {
        if self.start_time.is_none() {
            return Err(anyhow!("No active profiling session"));
        }

        let total_time = self.start_time.unwrap().elapsed();
        let mut metrics = CompilationMetrics::default();
        metrics.total_compilation_time = total_time;

        // Aggregate metrics from completed operations
        for operation in &self.completed_operations {
            self.update_metrics_from_operation(&mut metrics, operation);
        }

        metrics.calculate_compilation_speed();
        
        Ok(metrics)
    }

    /// Profile a specific operation
    pub fn profile_operation<F, R>(&mut self, name: &str, operation: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        let start_time = Instant::now();
        let memory_before = self.get_memory_usage();
        
        let profiling_op = ProfilingOperation {
            name: name.to_string(),
            start_time,
            memory_before,
            parent: None,
        };

        self.active_operations.insert(name.to_string(), profiling_op);

        let result = operation();
        
        let duration = start_time.elapsed();
        let memory_after = self.get_memory_usage();
        let memory_used = memory_after.saturating_sub(memory_before);

        let completed_op = CompletedOperation {
            name: name.to_string(),
            duration,
            memory_used,
            memory_peak: memory_after,
            success: result.is_ok(),
            error_message: result.as_ref().err().map(|e| e.to_string()),
            sub_operations: Vec::new(),
            metadata: HashMap::new(),
        };

        self.completed_operations.push(completed_op);
        self.active_operations.remove(name);

        result
    }

    /// Start profiling a nested operation
    pub fn start_operation(&mut self, name: &str, parent: Option<&str>) -> Result<()> {
        let profiling_op = ProfilingOperation {
            name: name.to_string(),
            start_time: Instant::now(),
            memory_before: self.get_memory_usage(),
            parent: parent.map(|p| p.to_string()),
        };

        self.active_operations.insert(name.to_string(), profiling_op);
        Ok(())
    }

    /// End profiling a nested operation
    pub fn end_operation(&mut self, name: &str, success: bool, error: Option<String>) -> Result<()> {
        if let Some(op) = self.active_operations.remove(name) {
            let duration = op.start_time.elapsed();
            let memory_after = self.get_memory_usage();
            let memory_used = memory_after.saturating_sub(op.memory_before);

            let completed_op = CompletedOperation {
                name: name.to_string(),
                duration,
                memory_used,
                memory_peak: memory_after,
                success,
                error_message: error,
                sub_operations: Vec::new(),
                metadata: HashMap::new(),
            };

            // If this operation has a parent, add it as a sub-operation
            if let Some(parent_name) = &op.parent {
                if let Some(parent_completed) = self.completed_operations.iter_mut()
                    .find(|co| co.name == *parent_name) {
                    parent_completed.sub_operations.push(completed_op);
                } else {
                    self.completed_operations.push(completed_op);
                }
            } else {
                self.completed_operations.push(completed_op);
            }

            Ok(())
        } else {
            Err(anyhow!("Operation not found: {}", name))
        }
    }

    /// Get profiling results for a specific operation
    pub fn get_operation_results(&self, name: &str) -> Option<&CompletedOperation> {
        self.completed_operations.iter().find(|op| op.name == name)
    }

    /// Get all completed operations
    pub fn get_all_results(&self) -> &[CompletedOperation] {
        &self.completed_operations
    }

    /// Get the slowest operations
    pub fn get_slowest_operations(&self, count: usize) -> Vec<&CompletedOperation> {
        let mut operations = self.completed_operations.iter().collect::<Vec<_>>();
        operations.sort_by(|a, b| b.duration.cmp(&a.duration));
        operations.into_iter().take(count).collect()
    }

    /// Get operations that used the most memory
    pub fn get_memory_intensive_operations(&self, count: usize) -> Vec<&CompletedOperation> {
        let mut operations = self.completed_operations.iter().collect::<Vec<_>>();
        operations.sort_by(|a, b| b.memory_used.cmp(&a.memory_used));
        operations.into_iter().take(count).collect()
    }

    /// Update compilation metrics from a completed operation
    fn update_metrics_from_operation(&self, metrics: &mut CompilationMetrics, operation: &CompletedOperation) {
        match operation.name.as_str() {
            "rust_compilation" => metrics.rust_compilation_time = operation.duration,
            "wasm_generation" => metrics.wasm_generation_time = operation.duration,
            "wasm_optimization" => metrics.wasm_optimization_time = operation.duration,
            "csharp_generation" => metrics.csharp_generation_time = operation.duration,
            "file_io" => metrics.file_io_time = operation.duration,
            "binding_generation" => metrics.binding_generation_time = operation.duration,
            _ => {}
        }

        // Update code metrics from metadata if available
        if let Some(loc_str) = operation.metadata.get("lines_of_code") {
            if let Ok(loc) = loc_str.parse::<u32>() {
                metrics.rust_lines_of_code = loc;
            }
        }

        if let Some(wasm_size_str) = operation.metadata.get("wasm_size") {
            if let Ok(size) = wasm_size_str.parse::<u64>() {
                metrics.wasm_size_bytes = size;
            }
        }

        if let Some(csharp_lines_str) = operation.metadata.get("csharp_lines") {
            if let Ok(lines) = csharp_lines_str.parse::<u32>() {
                metrics.generated_csharp_lines = lines;
            }
        }
    }

    /// Get current memory usage (placeholder implementation)
    fn get_memory_usage(&self) -> u64 {
        // In a real implementation, this would use system APIs
        std::process::id() as u64 * 1024 // Placeholder
    }
}

impl CodeProfiler {
    /// Create a new code profiler
    pub fn new() -> Self {
        Self {
            analysis_cache: HashMap::new(),
        }
    }

    /// Analyze Rust source code for performance characteristics
    pub fn analyze_rust_code(&mut self, file_path: &str, source_code: &str) -> Result<CodeAnalysis> {
        // Check cache first
        if let Some(cached) = self.analysis_cache.get(file_path) {
            return Ok(cached.clone());
        }

        let analysis = self.perform_code_analysis(file_path, source_code)?;
        self.analysis_cache.insert(file_path.to_string(), analysis.clone());
        
        Ok(analysis)
    }

    /// Analyze generated C# code for UdonSharp compatibility
    pub fn analyze_csharp_code(&mut self, file_path: &str, source_code: &str) -> Result<CodeAnalysis> {
        let analysis = self.perform_csharp_analysis(file_path, source_code)?;
        Ok(analysis)
    }

    /// Get code quality metrics for a project
    pub fn get_code_quality_metrics(&self, analyses: &[CodeAnalysis]) -> CodeQualityMetrics {
        let total_loc = analyses.iter().map(|a| a.lines_of_code).sum::<u32>();
        let avg_complexity = analyses.iter().map(|a| a.complexity_score).sum::<f64>() / analyses.len().max(1) as f64;
        
        let all_issues: Vec<_> = analyses.iter()
            .flat_map(|a| &a.performance_issues)
            .collect();

        let all_opportunities: Vec<_> = analyses.iter()
            .flat_map(|a| &a.optimization_opportunities)
            .collect();

        let avg_compatibility = analyses.iter()
            .map(|a| a.udonsharp_compatibility.compatibility_score)
            .sum::<f64>() / analyses.len().max(1) as f64;

        CodeQualityMetrics {
            cyclomatic_complexity: avg_complexity,
            code_duplication_percentage: self.calculate_duplication_percentage(analyses),
            optimization_opportunities: all_opportunities.len() as u32,
            potential_performance_issues: all_issues.into_iter().cloned().collect(),
            udonsharp_best_practices_score: avg_compatibility,
        }
    }

    /// Perform detailed code analysis
    fn perform_code_analysis(&self, file_path: &str, source_code: &str) -> Result<CodeAnalysis> {
        let lines_of_code = source_code.lines().filter(|line| !line.trim().is_empty()).count() as u32;
        let complexity_score = self.calculate_complexity(source_code);
        let performance_issues = self.identify_performance_issues(source_code);
        let optimization_opportunities = self.identify_optimizations(source_code);
        let udonsharp_compatibility = self.analyze_udonsharp_compatibility(source_code);

        Ok(CodeAnalysis {
            file_path: file_path.to_string(),
            lines_of_code,
            complexity_score,
            performance_issues,
            optimization_opportunities,
            udonsharp_compatibility,
        })
    }

    /// Perform C# specific analysis
    fn perform_csharp_analysis(&self, file_path: &str, source_code: &str) -> Result<CodeAnalysis> {
        let lines_of_code = source_code.lines().filter(|line| !line.trim().is_empty()).count() as u32;
        let complexity_score = self.calculate_csharp_complexity(source_code);
        let performance_issues = self.identify_csharp_performance_issues(source_code);
        let optimization_opportunities = self.identify_csharp_optimizations(source_code);
        let udonsharp_compatibility = self.analyze_csharp_udonsharp_compatibility(source_code);

        Ok(CodeAnalysis {
            file_path: file_path.to_string(),
            lines_of_code,
            complexity_score,
            performance_issues,
            optimization_opportunities,
            udonsharp_compatibility,
        })
    }

    /// Calculate cyclomatic complexity (simplified)
    fn calculate_complexity(&self, source_code: &str) -> f64 {
        let mut complexity = 1.0; // Base complexity
        
        // Count decision points
        for line in source_code.lines() {
            let line = line.trim();
            if line.contains("if ") || line.contains("else if ") {
                complexity += 1.0;
            }
            if line.contains("match ") || line.contains("=>") {
                complexity += 0.5;
            }
            if line.contains("while ") || line.contains("for ") {
                complexity += 1.0;
            }
            if line.contains("&&") || line.contains("||") {
                complexity += 0.5;
            }
        }

        complexity
    }

    /// Calculate C# cyclomatic complexity
    fn calculate_csharp_complexity(&self, source_code: &str) -> f64 {
        let mut complexity = 1.0;
        
        for line in source_code.lines() {
            let line = line.trim();
            if line.contains("if (") || line.contains("else if (") {
                complexity += 1.0;
            }
            if line.contains("switch (") || line.contains("case ") {
                complexity += 0.5;
            }
            if line.contains("while (") || line.contains("for (") || line.contains("foreach (") {
                complexity += 1.0;
            }
            if line.contains("&&") || line.contains("||") {
                complexity += 0.5;
            }
        }

        complexity
    }

    /// Identify performance issues in Rust code
    fn identify_performance_issues(&self, source_code: &str) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        // Check for common performance anti-patterns
        for (line_num, line) in source_code.lines().enumerate() {
            let line = line.trim();
            
            // Check for excessive allocations
            if line.contains("Vec::new()") && line.contains("loop") {
                issues.push(PerformanceIssue {
                    severity: crate::metrics::IssueSeverity::Medium,
                    category: crate::metrics::IssueCategory::Memory,
                    description: "Potential excessive allocations in loop".to_string(),
                    location: Some(crate::metrics::CodeLocation {
                        file: "current".to_string(),
                        line: line_num as u32 + 1,
                        column: 0,
                        function: None,
                    }),
                    suggested_fix: Some("Consider pre-allocating or reusing collections".to_string()),
                    estimated_impact: crate::metrics::ImpactLevel::Medium,
                });
            }

            // Check for string concatenation in loops
            if line.contains("push_str") && source_code.contains("loop") {
                issues.push(PerformanceIssue {
                    severity: crate::metrics::IssueSeverity::Low,
                    category: crate::metrics::IssueCategory::CPU,
                    description: "String concatenation in loop may be inefficient".to_string(),
                    location: Some(crate::metrics::CodeLocation {
                        file: "current".to_string(),
                        line: line_num as u32 + 1,
                        column: 0,
                        function: None,
                    }),
                    suggested_fix: Some("Consider using format! or pre-calculating string size".to_string()),
                    estimated_impact: crate::metrics::ImpactLevel::Low,
                });
            }
        }

        issues
    }

    /// Identify performance issues in C# code
    fn identify_csharp_performance_issues(&self, source_code: &str) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        for (line_num, line) in source_code.lines().enumerate() {
            let line = line.trim();
            
            // Check for GameObject.Find usage
            if line.contains("GameObject.Find") {
                issues.push(PerformanceIssue {
                    severity: crate::metrics::IssueSeverity::High,
                    category: crate::metrics::IssueCategory::UdonSharpSpecific,
                    description: "GameObject.Find is expensive in UdonSharp".to_string(),
                    location: Some(crate::metrics::CodeLocation {
                        file: "current".to_string(),
                        line: line_num as u32 + 1,
                        column: 0,
                        function: None,
                    }),
                    suggested_fix: Some("Cache GameObject references or use direct assignment".to_string()),
                    estimated_impact: crate::metrics::ImpactLevel::High,
                });
            }

            // Check for excessive Update() usage
            if line.contains("void Update()") {
                issues.push(PerformanceIssue {
                    severity: crate::metrics::IssueSeverity::Medium,
                    category: crate::metrics::IssueCategory::UdonSharpSpecific,
                    description: "Update() method runs every frame - use sparingly".to_string(),
                    location: Some(crate::metrics::CodeLocation {
                        file: "current".to_string(),
                        line: line_num as u32 + 1,
                        column: 0,
                        function: None,
                    }),
                    suggested_fix: Some("Consider using events or coroutines for non-frame-dependent logic".to_string()),
                    estimated_impact: crate::metrics::ImpactLevel::Medium,
                });
            }
        }

        issues
    }

    /// Identify optimization opportunities
    fn identify_optimizations(&self, _source_code: &str) -> Vec<OptimizationOpportunity> {
        // Placeholder implementation
        Vec::new()
    }

    /// Identify C# optimization opportunities
    fn identify_csharp_optimizations(&self, _source_code: &str) -> Vec<OptimizationOpportunity> {
        // Placeholder implementation
        Vec::new()
    }

    /// Analyze UdonSharp compatibility
    fn analyze_udonsharp_compatibility(&self, _source_code: &str) -> CompatibilityAnalysis {
        CompatibilityAnalysis {
            compatibility_score: 0.9, // Placeholder
            unsupported_features: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Analyze C# UdonSharp compatibility
    fn analyze_csharp_udonsharp_compatibility(&self, _source_code: &str) -> CompatibilityAnalysis {
        CompatibilityAnalysis {
            compatibility_score: 0.95, // Placeholder
            unsupported_features: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    /// Calculate code duplication percentage
    fn calculate_duplication_percentage(&self, _analyses: &[CodeAnalysis]) -> f64 {
        // Placeholder implementation
        0.0
    }
}

impl Default for CompilationProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CodeProfiler {
    fn default() -> Self {
        Self::new()
    }
}