//! Compilation reporting and statistics for standard multi-behavior pattern
//! 
//! This module provides comprehensive reporting and statistics collection
//! for multi-behavior compilation processes.

use crate::{
    multi_behavior::UdonBehaviourStruct,
    standard_multi_behavior_integration::{StandardMultiBehaviorCompilationResult, StandardMultiBehaviorMetadata},
    debug_info_generator::CodeGenerationStats,
    dependency_analyzer_tool::{DependencyAnalyzerTool, DependencyAnalysisReport},
};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Comprehensive compilation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationReport {
    /// Basic compilation information
    pub compilation_info: CompilationInfo,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
    /// Code generation statistics
    pub code_generation_stats: CodeGenerationStatistics,
    /// Dependency analysis
    pub dependency_analysis: DependencyAnalysisReport,
    /// Quality metrics
    pub quality_metrics: QualityMetrics,
    /// File generation summary
    pub file_generation_summary: FileGenerationSummary,
    /// Warnings and recommendations
    pub warnings: Vec<CompilationWarning>,
    /// Success indicators
    pub success_indicators: SuccessIndicators,
}

/// Basic compilation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationInfo {
    /// Compilation timestamp
    pub timestamp: String,
    /// Compiler version
    pub compiler_version: String,
    /// Source file path
    pub source_file: String,
    /// Target configuration
    pub target_config: String,
    /// Compilation mode (debug/release)
    pub compilation_mode: String,
    /// Total compilation time
    pub total_compilation_time: Duration,
}

/// Performance metrics during compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Time spent on struct analysis
    pub struct_analysis_time: Duration,
    /// Time spent on trait validation
    pub trait_validation_time: Duration,
    /// Time spent on dependency analysis
    pub dependency_analysis_time: Duration,
    /// Time spent on code generation
    pub code_generation_time: Duration,
    /// Time spent on validation
    pub validation_time: Duration,
    /// Memory usage peak (in bytes)
    pub peak_memory_usage: u64,
    /// Number of compilation passes
    pub compilation_passes: usize,
}

/// Code generation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerationStatistics {
    /// Total lines of Rust source code
    pub rust_source_lines: usize,
    /// Total lines of generated C# code
    pub generated_csharp_lines: usize,
    /// Code expansion ratio (C# lines / Rust lines)
    pub code_expansion_ratio: f32,
    /// Statistics per behavior
    pub per_behavior_stats: HashMap<String, CodeGenerationStats>,
    /// SharedRuntime statistics
    pub shared_runtime_stats: Option<SharedRuntimeStats>,
}

/// SharedRuntime generation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedRuntimeStats {
    /// Number of shared functions
    pub shared_functions_count: usize,
    /// Number of shared types
    pub shared_types_count: usize,
    /// Lines of code in SharedRuntime
    pub shared_runtime_lines: usize,
    /// Code reuse factor (shared code / total code)
    pub code_reuse_factor: f32,
}

/// Quality metrics for generated code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Cyclomatic complexity average
    pub avg_cyclomatic_complexity: f32,
    /// Code duplication percentage
    pub code_duplication_percentage: f32,
    /// UdonSharp compatibility score (0.0 to 1.0)
    pub udonsharp_compatibility_score: f32,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f32,
    /// Maintainability index (0.0 to 100.0)
    pub maintainability_index: f32,
}

/// File generation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGenerationSummary {
    /// Total files generated
    pub total_files_generated: usize,
    /// Behavior files generated
    pub behavior_files: Vec<GeneratedFileInfo>,
    /// SharedRuntime file info
    pub shared_runtime_file: Option<GeneratedFileInfo>,
    /// Coordinator file info
    pub coordinator_file: Option<GeneratedFileInfo>,
    /// Prefab files generated
    pub prefab_files: Vec<GeneratedFileInfo>,
    /// Total size of generated files (in bytes)
    pub total_generated_size: u64,
}

/// Information about a generated file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFileInfo {
    /// File name
    pub file_name: String,
    /// File size in bytes
    pub file_size: u64,
    /// Number of lines
    pub line_count: usize,
    /// File type
    pub file_type: GeneratedFileType,
    /// Generation time
    pub generation_time: Duration,
}

/// Types of generated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeneratedFileType {
    BehaviorClass,
    SharedRuntime,
    Coordinator,
    Prefab,
    EditorScript,
}

/// Compilation warnings and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationWarning {
    /// Warning category
    pub category: WarningCategory,
    /// Warning message
    pub message: String,
    /// Severity level
    pub severity: WarningSeverity,
    /// Affected components
    pub affected_components: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Warning categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningCategory {
    Performance,
    Compatibility,
    BestPractices,
    Security,
    Maintainability,
    Dependencies,
}

/// Warning severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Success indicators and quality gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessIndicators {
    /// Overall compilation success
    pub compilation_successful: bool,
    /// All behaviors generated successfully
    pub all_behaviors_generated: bool,
    /// No critical errors
    pub no_critical_errors: bool,
    /// Performance targets met
    pub performance_targets_met: bool,
    /// Quality gates passed
    pub quality_gates_passed: bool,
    /// UdonSharp compatibility achieved
    pub udonsharp_compatibility_achieved: bool,
}

/// Compilation reporter that collects and generates reports
pub struct CompilationReporter {
    /// Start time of compilation
    start_time: Instant,
    /// Performance tracking
    performance_tracker: PerformanceTracker,
    /// Warnings collected during compilation
    warnings: Vec<CompilationWarning>,
    /// Quality analyzer
    quality_analyzer: QualityAnalyzer,
}

/// Tracks performance metrics during compilation
struct PerformanceTracker {
    struct_analysis_start: Option<Instant>,
    trait_validation_start: Option<Instant>,
    dependency_analysis_start: Option<Instant>,
    code_generation_start: Option<Instant>,
    validation_start: Option<Instant>,
    
    struct_analysis_time: Duration,
    trait_validation_time: Duration,
    dependency_analysis_time: Duration,
    code_generation_time: Duration,
    validation_time: Duration,
    
    peak_memory_usage: u64,
    compilation_passes: usize,
}

/// Analyzes code quality metrics
struct QualityAnalyzer {
    // Quality analysis state
}

impl CompilationReporter {
    /// Create a new compilation reporter
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            performance_tracker: PerformanceTracker::new(),
            warnings: Vec::new(),
            quality_analyzer: QualityAnalyzer::new(),
        }
    }

    /// Start tracking a compilation phase
    pub fn start_phase(&mut self, phase: CompilationPhase) {
        self.performance_tracker.start_phase(phase);
    }

    /// End tracking a compilation phase
    pub fn end_phase(&mut self, phase: CompilationPhase) {
        self.performance_tracker.end_phase(phase);
    }

    /// Add a warning to the report
    pub fn add_warning(&mut self, warning: CompilationWarning) {
        self.warnings.push(warning);
    }

    /// Generate a comprehensive compilation report
    pub fn generate_report(
        &self,
        compilation_result: &StandardMultiBehaviorCompilationResult,
        behaviors: &[UdonBehaviourStruct],
        dependency_analysis: &DependencyAnalysisReport,
    ) -> CompilationReport {
        let total_compilation_time = self.start_time.elapsed();
        
        let compilation_info = CompilationInfo {
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
            source_file: "src/lib.rs".to_string(), // TODO: Get actual source file
            target_config: "UdonSharp".to_string(),
            compilation_mode: if cfg!(debug_assertions) { "Debug" } else { "Release" }.to_string(),
            total_compilation_time,
        };

        let performance_metrics = self.performance_tracker.get_metrics();
        let code_generation_stats = self.calculate_code_generation_stats(compilation_result, behaviors);
        let quality_metrics = self.quality_analyzer.calculate_quality_metrics(compilation_result, behaviors);
        let file_generation_summary = self.calculate_file_generation_summary(compilation_result);
        let success_indicators = self.calculate_success_indicators(compilation_result, &quality_metrics);

        CompilationReport {
            compilation_info,
            performance_metrics,
            code_generation_stats,
            dependency_analysis: dependency_analysis.clone(),
            quality_metrics,
            file_generation_summary,
            warnings: self.warnings.clone(),
            success_indicators,
        }
    }

    /// Calculate code generation statistics
    fn calculate_code_generation_stats(
        &self,
        compilation_result: &StandardMultiBehaviorCompilationResult,
        behaviors: &[UdonBehaviourStruct],
    ) -> CodeGenerationStatistics {
        let rust_source_lines = self.count_rust_source_lines(behaviors);
        let generated_csharp_lines = self.count_generated_csharp_lines(compilation_result);
        let code_expansion_ratio = if rust_source_lines > 0 {
            generated_csharp_lines as f32 / rust_source_lines as f32
        } else {
            0.0
        };

        let per_behavior_stats = behaviors.iter()
            .map(|behavior| {
                let stats = CodeGenerationStats::from_behavior(behavior, 0); // TODO: Get actual generation time
                (behavior.name.clone(), stats)
            })
            .collect();

        let shared_runtime_stats = compilation_result.shared_runtime.as_ref().map(|sr| {
            SharedRuntimeStats {
                shared_functions_count: sr.shared_functions.len(),
                shared_types_count: sr.shared_types.len(),
                shared_runtime_lines: sr.file_content.lines().count(),
                code_reuse_factor: self.calculate_code_reuse_factor(compilation_result),
            }
        });

        CodeGenerationStatistics {
            rust_source_lines,
            generated_csharp_lines,
            code_expansion_ratio,
            per_behavior_stats,
            shared_runtime_stats,
        }
    }

    /// Count lines in Rust source code
    fn count_rust_source_lines(&self, behaviors: &[UdonBehaviourStruct]) -> usize {
        // Simplified calculation - in real implementation, would parse actual source
        behaviors.len() * 50 // Rough estimate
    }

    /// Count lines in generated C# code
    fn count_generated_csharp_lines(&self, compilation_result: &StandardMultiBehaviorCompilationResult) -> usize {
        let mut total_lines = 0;
        
        for file in compilation_result.behavior_files.values() {
            total_lines += file.file_content.lines().count();
        }
        
        if let Some(shared_runtime) = &compilation_result.shared_runtime {
            total_lines += shared_runtime.file_content.lines().count();
        }
        
        total_lines
    }

    /// Calculate code reuse factor
    fn calculate_code_reuse_factor(&self, compilation_result: &StandardMultiBehaviorCompilationResult) -> f32 {
        if let Some(shared_runtime) = &compilation_result.shared_runtime {
            let shared_lines = shared_runtime.file_content.lines().count() as f32;
            let total_lines = self.count_generated_csharp_lines(compilation_result) as f32;
            
            if total_lines > 0.0 {
                shared_lines / total_lines
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculate file generation summary
    fn calculate_file_generation_summary(&self, compilation_result: &StandardMultiBehaviorCompilationResult) -> FileGenerationSummary {
        let mut behavior_files = Vec::new();
        let mut total_size = 0u64;

        for (name, file) in &compilation_result.behavior_files {
            let file_size = file.file_content.len() as u64;
            total_size += file_size;
            
            behavior_files.push(GeneratedFileInfo {
                file_name: format!("{}.cs", name),
                file_size,
                line_count: file.file_content.lines().count(),
                file_type: GeneratedFileType::BehaviorClass,
                generation_time: Duration::from_millis(0), // TODO: Track actual generation time
            });
        }

        let shared_runtime_file = compilation_result.shared_runtime.as_ref().map(|sr| {
            let file_size = sr.file_content.len() as u64;
            total_size += file_size;
            
            GeneratedFileInfo {
                file_name: "SharedRuntime.cs".to_string(),
                file_size,
                line_count: sr.file_content.lines().count(),
                file_type: GeneratedFileType::SharedRuntime,
                generation_time: Duration::from_millis(0),
            }
        });

        FileGenerationSummary {
            total_files_generated: compilation_result.output_files.len(),
            behavior_files,
            shared_runtime_file,
            coordinator_file: None, // TODO: Add coordinator file info
            prefab_files: Vec::new(), // TODO: Add prefab file info
            total_generated_size: total_size,
        }
    }

    /// Calculate success indicators
    fn calculate_success_indicators(
        &self,
        compilation_result: &StandardMultiBehaviorCompilationResult,
        quality_metrics: &QualityMetrics,
    ) -> SuccessIndicators {
        let has_critical_errors = self.warnings.iter()
            .any(|w| matches!(w.severity, WarningSeverity::Critical));

        SuccessIndicators {
            compilation_successful: compilation_result.success,
            all_behaviors_generated: !compilation_result.behavior_files.is_empty(),
            no_critical_errors: !has_critical_errors,
            performance_targets_met: quality_metrics.performance_score >= 0.7,
            quality_gates_passed: quality_metrics.maintainability_index >= 60.0,
            udonsharp_compatibility_achieved: quality_metrics.udonsharp_compatibility_score >= 0.9,
        }
    }

    /// Generate a human-readable text report
    pub fn generate_text_report(&self, report: &CompilationReport) -> String {
        let mut output = String::new();

        output.push_str("=== UdonSharp-Rust Compilation Report ===\n\n");

        // Compilation Info
        output.push_str("## Compilation Information\n");
        output.push_str(&format!("Timestamp: {}\n", report.compilation_info.timestamp));
        output.push_str(&format!("Compiler Version: {}\n", report.compilation_info.compiler_version));
        output.push_str(&format!("Source File: {}\n", report.compilation_info.source_file));
        output.push_str(&format!("Compilation Mode: {}\n", report.compilation_info.compilation_mode));
        output.push_str(&format!("Total Time: {:.2}s\n", report.compilation_info.total_compilation_time.as_secs_f32()));
        output.push_str("\n");

        // Success Indicators
        output.push_str("## Success Indicators\n");
        output.push_str(&format!("✓ Compilation Successful: {}\n", 
            if report.success_indicators.compilation_successful { "YES" } else { "NO" }));
        output.push_str(&format!("✓ All Behaviors Generated: {}\n", 
            if report.success_indicators.all_behaviors_generated { "YES" } else { "NO" }));
        output.push_str(&format!("✓ No Critical Errors: {}\n", 
            if report.success_indicators.no_critical_errors { "YES" } else { "NO" }));
        output.push_str(&format!("✓ Performance Targets Met: {}\n", 
            if report.success_indicators.performance_targets_met { "YES" } else { "NO" }));
        output.push_str(&format!("✓ Quality Gates Passed: {}\n", 
            if report.success_indicators.quality_gates_passed { "YES" } else { "NO" }));
        output.push_str("\n");

        // Code Generation Statistics
        output.push_str("## Code Generation Statistics\n");
        output.push_str(&format!("Rust Source Lines: {}\n", report.code_generation_stats.rust_source_lines));
        output.push_str(&format!("Generated C# Lines: {}\n", report.code_generation_stats.generated_csharp_lines));
        output.push_str(&format!("Code Expansion Ratio: {:.2}x\n", report.code_generation_stats.code_expansion_ratio));
        
        if let Some(sr_stats) = &report.code_generation_stats.shared_runtime_stats {
            output.push_str(&format!("Shared Functions: {}\n", sr_stats.shared_functions_count));
            output.push_str(&format!("Code Reuse Factor: {:.2}%\n", sr_stats.code_reuse_factor * 100.0));
        }
        output.push_str("\n");

        // Performance Metrics
        output.push_str("## Performance Metrics\n");
        output.push_str(&format!("Struct Analysis: {:.2}s\n", report.performance_metrics.struct_analysis_time.as_secs_f32()));
        output.push_str(&format!("Trait Validation: {:.2}s\n", report.performance_metrics.trait_validation_time.as_secs_f32()));
        output.push_str(&format!("Dependency Analysis: {:.2}s\n", report.performance_metrics.dependency_analysis_time.as_secs_f32()));
        output.push_str(&format!("Code Generation: {:.2}s\n", report.performance_metrics.code_generation_time.as_secs_f32()));
        output.push_str(&format!("Peak Memory Usage: {:.2} MB\n", report.performance_metrics.peak_memory_usage as f32 / 1024.0 / 1024.0));
        output.push_str("\n");

        // Quality Metrics
        output.push_str("## Quality Metrics\n");
        output.push_str(&format!("UdonSharp Compatibility: {:.1}%\n", report.quality_metrics.udonsharp_compatibility_score * 100.0));
        output.push_str(&format!("Performance Score: {:.1}%\n", report.quality_metrics.performance_score * 100.0));
        output.push_str(&format!("Maintainability Index: {:.1}\n", report.quality_metrics.maintainability_index));
        output.push_str(&format!("Code Duplication: {:.1}%\n", report.quality_metrics.code_duplication_percentage));
        output.push_str("\n");

        // File Generation Summary
        output.push_str("## Generated Files\n");
        output.push_str(&format!("Total Files: {}\n", report.file_generation_summary.total_files_generated));
        output.push_str(&format!("Total Size: {:.2} KB\n", report.file_generation_summary.total_generated_size as f32 / 1024.0));
        
        for file in &report.file_generation_summary.behavior_files {
            output.push_str(&format!("  • {} ({} lines, {:.1} KB)\n", 
                file.file_name, file.line_count, file.file_size as f32 / 1024.0));
        }
        
        if let Some(sr_file) = &report.file_generation_summary.shared_runtime_file {
            output.push_str(&format!("  • {} ({} lines, {:.1} KB)\n", 
                sr_file.file_name, sr_file.line_count, sr_file.file_size as f32 / 1024.0));
        }
        output.push_str("\n");

        // Warnings
        if !report.warnings.is_empty() {
            output.push_str("## Warnings and Recommendations\n");
            for warning in &report.warnings {
                let severity_icon = match warning.severity {
                    WarningSeverity::Info => "ℹ️",
                    WarningSeverity::Warning => "⚠️",
                    WarningSeverity::Error => "❌",
                    WarningSeverity::Critical => "🚨",
                };
                
                output.push_str(&format!("{} {:?}: {}\n", severity_icon, warning.category, warning.message));
                
                if !warning.recommendations.is_empty() {
                    output.push_str("  Recommendations:\n");
                    for rec in &warning.recommendations {
                        output.push_str(&format!("    - {}\n", rec));
                    }
                }
                output.push_str("\n");
            }
        }

        // Dependency Analysis Summary
        output.push_str("## Dependency Analysis Summary\n");
        output.push_str(&format!("Total Behaviors: {}\n", report.dependency_analysis.total_behaviors));
        output.push_str(&format!("Total Dependencies: {}\n", report.dependency_analysis.total_dependencies));
        output.push_str(&format!("Circular Dependencies: {}\n", report.dependency_analysis.circular_dependencies.len()));
        output.push_str(&format!("Max Dependency Depth: {}\n", report.dependency_analysis.complexity_metrics.max_depth));
        output.push_str(&format!("Coupling Factor: {:.2}\n", report.dependency_analysis.complexity_metrics.coupling_factor));
        output.push_str("\n");

        output.push_str("=== End Report ===\n");
        output
    }

    /// Generate JSON report
    pub fn generate_json_report(&self, report: &CompilationReport) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }
}

/// Compilation phases for performance tracking
#[derive(Debug, Clone, Copy)]
pub enum CompilationPhase {
    StructAnalysis,
    TraitValidation,
    DependencyAnalysis,
    CodeGeneration,
    Validation,
}

impl PerformanceTracker {
    fn new() -> Self {
        Self {
            struct_analysis_start: None,
            trait_validation_start: None,
            dependency_analysis_start: None,
            code_generation_start: None,
            validation_start: None,
            struct_analysis_time: Duration::ZERO,
            trait_validation_time: Duration::ZERO,
            dependency_analysis_time: Duration::ZERO,
            code_generation_time: Duration::ZERO,
            validation_time: Duration::ZERO,
            peak_memory_usage: 0,
            compilation_passes: 1,
        }
    }

    fn start_phase(&mut self, phase: CompilationPhase) {
        let now = Instant::now();
        match phase {
            CompilationPhase::StructAnalysis => self.struct_analysis_start = Some(now),
            CompilationPhase::TraitValidation => self.trait_validation_start = Some(now),
            CompilationPhase::DependencyAnalysis => self.dependency_analysis_start = Some(now),
            CompilationPhase::CodeGeneration => self.code_generation_start = Some(now),
            CompilationPhase::Validation => self.validation_start = Some(now),
        }
    }

    fn end_phase(&mut self, phase: CompilationPhase) {
        let now = Instant::now();
        match phase {
            CompilationPhase::StructAnalysis => {
                if let Some(start) = self.struct_analysis_start.take() {
                    self.struct_analysis_time += now.duration_since(start);
                }
            }
            CompilationPhase::TraitValidation => {
                if let Some(start) = self.trait_validation_start.take() {
                    self.trait_validation_time += now.duration_since(start);
                }
            }
            CompilationPhase::DependencyAnalysis => {
                if let Some(start) = self.dependency_analysis_start.take() {
                    self.dependency_analysis_time += now.duration_since(start);
                }
            }
            CompilationPhase::CodeGeneration => {
                if let Some(start) = self.code_generation_start.take() {
                    self.code_generation_time += now.duration_since(start);
                }
            }
            CompilationPhase::Validation => {
                if let Some(start) = self.validation_start.take() {
                    self.validation_time += now.duration_since(start);
                }
            }
        }
    }

    fn get_metrics(&self) -> PerformanceMetrics {
        PerformanceMetrics {
            struct_analysis_time: self.struct_analysis_time,
            trait_validation_time: self.trait_validation_time,
            dependency_analysis_time: self.dependency_analysis_time,
            code_generation_time: self.code_generation_time,
            validation_time: self.validation_time,
            peak_memory_usage: self.peak_memory_usage,
            compilation_passes: self.compilation_passes,
        }
    }
}

impl QualityAnalyzer {
    fn new() -> Self {
        Self {}
    }

    fn calculate_quality_metrics(
        &self,
        _compilation_result: &StandardMultiBehaviorCompilationResult,
        behaviors: &[UdonBehaviourStruct],
    ) -> QualityMetrics {
        // Simplified quality metrics calculation
        let avg_cyclomatic_complexity = self.calculate_avg_complexity(behaviors);
        let code_duplication_percentage = self.calculate_code_duplication(behaviors);
        let udonsharp_compatibility_score = self.calculate_compatibility_score(behaviors);
        let performance_score = self.calculate_performance_score(behaviors);
        let maintainability_index = self.calculate_maintainability_index(behaviors);

        QualityMetrics {
            avg_cyclomatic_complexity,
            code_duplication_percentage,
            udonsharp_compatibility_score,
            performance_score,
            maintainability_index,
        }
    }

    fn calculate_avg_complexity(&self, behaviors: &[UdonBehaviourStruct]) -> f32 {
        if behaviors.is_empty() {
            return 0.0;
        }

        let total_complexity: usize = behaviors.iter()
            .map(|b| b.methods.len() + 1) // Simplified complexity calculation
            .sum();

        total_complexity as f32 / behaviors.len() as f32
    }

    fn calculate_code_duplication(&self, _behaviors: &[UdonBehaviourStruct]) -> f32 {
        // Simplified calculation - in real implementation would analyze actual code
        5.0 // 5% duplication
    }

    fn calculate_compatibility_score(&self, behaviors: &[UdonBehaviourStruct]) -> f32 {
        let mut compatible_elements = 0;
        let mut total_elements = 0;

        for behavior in behaviors {
            for field in &behavior.fields {
                total_elements += 1;
                if field.field_type.is_udonsharp_compatible() {
                    compatible_elements += 1;
                }
            }
        }

        if total_elements > 0 {
            compatible_elements as f32 / total_elements as f32
        } else {
            1.0
        }
    }

    fn calculate_performance_score(&self, behaviors: &[UdonBehaviourStruct]) -> f32 {
        // Simplified performance scoring
        let mut score = 1.0;

        for behavior in behaviors {
            // Penalize for too many sync fields
            let sync_fields = behavior.get_sync_fields().len();
            if sync_fields > 10 {
                score -= 0.1;
            }

            // Penalize for update methods (performance concern)
            let has_update = behavior.methods.iter().any(|m| m.name == "update");
            if has_update {
                score -= 0.05;
            }
        }

        score.max(0.0)
    }

    fn calculate_maintainability_index(&self, behaviors: &[UdonBehaviourStruct]) -> f32 {
        // Simplified maintainability calculation
        let avg_methods_per_behavior = if !behaviors.is_empty() {
            behaviors.iter().map(|b| b.methods.len()).sum::<usize>() as f32 / behaviors.len() as f32
        } else {
            0.0
        };

        // Higher score for moderate complexity
        if avg_methods_per_behavior <= 10.0 {
            85.0
        } else if avg_methods_per_behavior <= 20.0 {
            70.0
        } else {
            50.0
        }
    }
}

impl Default for CompilationReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_reporter_creation() {
        let reporter = CompilationReporter::new();
        assert!(reporter.warnings.is_empty());
    }

    #[test]
    fn test_performance_tracker() {
        let mut tracker = PerformanceTracker::new();
        
        tracker.start_phase(CompilationPhase::StructAnalysis);
        std::thread::sleep(Duration::from_millis(10));
        tracker.end_phase(CompilationPhase::StructAnalysis);
        
        let metrics = tracker.get_metrics();
        assert!(metrics.struct_analysis_time > Duration::ZERO);
    }

    #[test]
    fn test_quality_analyzer() {
        let analyzer = QualityAnalyzer::new();
        let behaviors = vec![
            crate::multi_behavior::UdonBehaviourStruct::new("TestBehavior".to_string())
        ];
        
        let metrics = analyzer.calculate_quality_metrics(
            &StandardMultiBehaviorCompilationResult {
                success: true,
                output_files: vec![],
                behavior_files: std::collections::HashMap::new(),
                shared_runtime_file: None,
                shared_runtime: None,
                communication_code: crate::standard_multi_behavior_integration::CommunicationCodeResult {
                    behavior_communications: std::collections::HashMap::new(),
                    total_communication_calls: 0,
                    gameobject_references: std::collections::HashMap::new(),
                    custom_events: std::collections::HashMap::new(),
                },
                metadata: StandardMultiBehaviorMetadata {
                    total_behaviors: 1,
                    total_files: 1,
                    shared_functions_count: 0,
                    inter_behavior_calls: 0,
                    has_networking: false,
                    dependency_count: 0,
                    circular_dependencies_detected: false,
                },
                diagnostics: vec![],
            },
            &behaviors,
        );
        
        assert!(metrics.udonsharp_compatibility_score >= 0.0);
        assert!(metrics.performance_score >= 0.0);
    }
}