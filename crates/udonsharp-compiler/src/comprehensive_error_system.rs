//! Comprehensive error handling and validation system integration
//! 
//! This module integrates all error handling components into a unified system
//! for multi-behavior compilation with detailed reporting and validation.

use crate::error_detection::{CompilationErrorDetector, CompilationErrorReport};
use crate::error_reporting::{ErrorReporter, QuickFix};
use crate::runtime_validation::{RuntimeValidator, ValidationResult, MultiClassValidationResult};
use crate::code_generator::GeneratedClass;
use crate::multi_behavior::UdonBehaviourStruct;
use udonsharp_core::error::{UdonSharpError, DiagnosticCollector, Diagnostic, DiagnosticLevel};
use std::path::PathBuf;

/// Comprehensive error handling system that integrates detection, reporting, and validation
pub struct ComprehensiveErrorSystem {
    /// Error detection system
    error_detector: CompilationErrorDetector,
    /// Error reporting system
    error_reporter: ErrorReporter,
    /// Runtime validation system
    runtime_validator: RuntimeValidator,
    /// Configuration options
    config: ErrorSystemConfig,
}

impl ComprehensiveErrorSystem {
    /// Create a new comprehensive error system
    pub fn new() -> Self {
        Self {
            error_detector: CompilationErrorDetector::new(),
            error_reporter: ErrorReporter::new(),
            runtime_validator: RuntimeValidator::new(),
            config: ErrorSystemConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ErrorSystemConfig) -> Self {
        Self {
            error_detector: CompilationErrorDetector::new(),
            error_reporter: ErrorReporter::with_config(config.use_colors, config.verbose),
            runtime_validator: RuntimeValidator::new(),
            config,
        }
    }
    
    /// Perform comprehensive analysis of UdonBehaviour structs
    pub fn analyze_structs(&self, structs: &[UdonBehaviourStruct]) -> ComprehensiveAnalysisResult {
        // Phase 1: Compilation error detection
        let error_report = self.error_detector.generate_error_report(structs);
        
        // Phase 2: Generate user-friendly error report
        let formatted_report = self.error_reporter.generate_report(&error_report);
        
        // Phase 3: Generate quick fixes
        let quick_fixes = self.error_reporter.generate_quick_fixes(&error_report.errors);
        
        // Phase 4: Generate IDE diagnostics
        let diagnostics = self.error_reporter.generate_diagnostics(&error_report.errors);
        
        let should_block_compilation = error_report.has_blocking_errors;
        let analysis_summary = self.generate_analysis_summary(&error_report);
        
        ComprehensiveAnalysisResult {
            error_report,
            formatted_report,
            quick_fixes,
            diagnostics,
            should_block_compilation,
            analysis_summary,
        }
    }
    
    /// Validate generated C# classes
    pub fn validate_generated_classes(&self, classes: &[GeneratedClass]) -> ComprehensiveValidationResult {
        // Phase 1: Runtime validation
        let validation_result = self.runtime_validator.validate_multiple_classes(classes);
        
        // Phase 2: Convert validation issues to compilation errors for consistent reporting
        let compilation_errors = self.convert_validation_issues_to_errors(&validation_result);
        
        // Phase 3: Generate formatted report
        let formatted_report = if !compilation_errors.is_empty() {
            let temp_report = CompilationErrorReport {
                errors: compilation_errors.clone(),
                errors_by_category: std::collections::HashMap::new(),
                error_count: compilation_errors.iter().filter(|e| e.severity == DiagnosticLevel::Error).count(),
                warning_count: compilation_errors.iter().filter(|e| e.severity == DiagnosticLevel::Warning).count(),
                structs_analyzed: classes.len(),
                has_blocking_errors: compilation_errors.iter().any(|e| e.severity == DiagnosticLevel::Error),
            };
            self.error_reporter.generate_report(&temp_report)
        } else {
            "✅ All generated classes passed validation successfully!".to_string()
        };
        
        // Phase 4: Generate improvement suggestions
        let improvement_suggestions = validation_result.class_results.iter()
            .flat_map(|result| result.suggestions.clone())
            .collect();
        
        let is_valid = validation_result.is_valid;
        let validation_summary = self.generate_validation_summary(&validation_result);
        
        ComprehensiveValidationResult {
            validation_result,
            compilation_errors,
            formatted_report,
            improvement_suggestions,
            is_valid,
            validation_summary,
        }
    }
    
    /// Perform end-to-end analysis and validation
    pub fn full_analysis(&self, structs: &[UdonBehaviourStruct], generated_classes: &[GeneratedClass]) -> FullAnalysisResult {
        let struct_analysis = self.analyze_structs(structs);
        let class_validation = self.validate_generated_classes(generated_classes);
        
        // Combine results
        let total_errors = struct_analysis.error_report.error_count + class_validation.validation_result.total_error_count;
        let total_warnings = struct_analysis.error_report.warning_count + class_validation.validation_result.total_warning_count;
        
        let should_proceed = !struct_analysis.should_block_compilation && class_validation.is_valid;
        
        let combined_report = self.generate_combined_report(&struct_analysis, &class_validation);
        
        FullAnalysisResult {
            struct_analysis,
            class_validation,
            total_errors,
            total_warnings,
            should_proceed_with_compilation: should_proceed,
            combined_report,
            recommendations: self.generate_recommendations(total_errors, total_warnings),
        }
    }
    
    /// Generate analysis summary
    fn generate_analysis_summary(&self, error_report: &CompilationErrorReport) -> String {
        if error_report.has_blocking_errors {
            format!(
                "❌ Analysis failed: {} error(s) found in {} struct(s). Compilation blocked.",
                error_report.error_count, error_report.structs_analyzed
            )
        } else if error_report.warning_count > 0 {
            format!(
                "⚠️ Analysis completed with {} warning(s) in {} struct(s). Compilation can proceed.",
                error_report.warning_count, error_report.structs_analyzed
            )
        } else {
            format!(
                "✅ Analysis completed successfully for {} struct(s). No issues found.",
                error_report.structs_analyzed
            )
        }
    }
    
    /// Generate validation summary
    fn generate_validation_summary(&self, validation_result: &MultiClassValidationResult) -> String {
        if !validation_result.is_valid {
            format!(
                "❌ Validation failed: {} error(s), {} warning(s) in {} class(es).",
                validation_result.total_error_count,
                validation_result.total_warning_count,
                validation_result.class_results.len()
            )
        } else if validation_result.total_warning_count > 0 {
            format!(
                "⚠️ Validation completed with {} warning(s) in {} class(es).",
                validation_result.total_warning_count,
                validation_result.class_results.len()
            )
        } else {
            format!(
                "✅ Validation completed successfully for {} class(es).",
                validation_result.class_results.len()
            )
        }
    }
    
    /// Convert validation issues to compilation errors for consistent reporting
    fn convert_validation_issues_to_errors(&self, validation_result: &MultiClassValidationResult) -> Vec<crate::error_detection::CompilationError> {
        let mut errors = Vec::new();
        
        for class_result in &validation_result.class_results {
            for issue in &class_result.issues {
                errors.push(crate::error_detection::CompilationError {
                    error_type: self.map_validation_issue_to_error_type(&issue.issue_type),
                    struct_name: Some(class_result.class_name.clone()),
                    field_name: None,
                    method_name: None,
                    message: issue.message.clone(),
                    suggestion: issue.suggestion.clone(),
                    code_example: issue.code_example.clone(),
                    severity: issue.severity,
                    source_location: issue.line_number.map(|line| crate::error_detection::SourceLocation {
                        file: format!("{}.cs", class_result.class_name),
                        line: line as u32,
                        column: 1,
                    }),
                });
            }
        }
        
        // Add cross-class issues
        for issue in &validation_result.cross_class_issues {
            errors.push(crate::error_detection::CompilationError {
                error_type: self.map_validation_issue_to_error_type(&issue.issue_type),
                struct_name: None,
                field_name: None,
                method_name: None,
                message: issue.message.clone(),
                suggestion: issue.suggestion.clone(),
                code_example: issue.code_example.clone(),
                severity: issue.severity,
                source_location: None,
            });
        }
        
        errors
    }
    
    /// Map validation issue types to compilation error types
    fn map_validation_issue_to_error_type(&self, issue_type: &crate::runtime_validation::ValidationIssueType) -> crate::error_detection::ErrorType {
        use crate::runtime_validation::ValidationIssueType;
        use crate::error_detection::ErrorType;
        
        match issue_type {
            ValidationIssueType::SyntaxError => ErrorType::UnsupportedFeature,
            ValidationIssueType::NamingConvention => ErrorType::InvalidAttributeUsage,
            ValidationIssueType::AccessModifier => ErrorType::InvalidAttributeUsage,
            ValidationIssueType::UdonSharpCompatibility => ErrorType::UnsupportedFeature,
            ValidationIssueType::NetworkingSafety => ErrorType::InvalidAttributeUsage,
            ValidationIssueType::CustomEventSafety => ErrorType::InvalidAttributeUsage,
            ValidationIssueType::NullSafety => ErrorType::UnsupportedFeature,
            ValidationIssueType::BoundsSafety => ErrorType::UnsupportedFeature,
            ValidationIssueType::ErrorHandling => ErrorType::UnsupportedFeature,
            ValidationIssueType::Performance => ErrorType::UnsupportedFeature,
            ValidationIssueType::Documentation => ErrorType::InvalidAttributeUsage,
            ValidationIssueType::CircularDependency => ErrorType::UnsupportedFeature,
        }
    }
    
    /// Generate combined report for full analysis
    fn generate_combined_report(&self, struct_analysis: &ComprehensiveAnalysisResult, class_validation: &ComprehensiveValidationResult) -> String {
        let mut report = Vec::new();
        
        report.push("🔍 UdonSharp Multi-Behavior Compilation Analysis".to_string());
        report.push("═".repeat(50));
        report.push("".to_string());
        
        // Struct analysis section
        report.push("📋 Phase 1: Struct Analysis".to_string());
        report.push(struct_analysis.analysis_summary.clone());
        if struct_analysis.should_block_compilation {
            report.push("".to_string());
            report.push("❌ Compilation cannot proceed due to struct analysis errors.".to_string());
            report.push("Please fix the issues above before continuing.".to_string());
        }
        report.push("".to_string());
        
        // Class validation section
        report.push("🔧 Phase 2: Generated Code Validation".to_string());
        report.push(class_validation.validation_summary.clone());
        report.push("".to_string());
        
        // Overall status
        let overall_status = if struct_analysis.should_block_compilation || !class_validation.is_valid {
            "❌ COMPILATION BLOCKED"
        } else {
            "✅ READY FOR COMPILATION"
        };
        
        report.push("🎯 Overall Status".to_string());
        report.push(overall_status.to_string());
        report.push("".to_string());
        
        // Next steps
        report.push("📝 Next Steps:".to_string());
        if struct_analysis.should_block_compilation {
            report.push("   1. Fix struct analysis errors listed above".to_string());
            report.push("   2. Re-run the analysis".to_string());
        } else if !class_validation.is_valid {
            report.push("   1. Review generated code validation issues".to_string());
            report.push("   2. Check code generation logic".to_string());
        } else {
            report.push("   1. Proceed with UdonSharp compilation".to_string());
            report.push("   2. Deploy to VRChat world".to_string());
        }
        
        report.join("\n")
    }
    
    /// Generate recommendations based on analysis results
    fn generate_recommendations(&self, total_errors: usize, total_warnings: usize) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if total_errors > 0 {
            recommendations.push("🔧 Fix all errors before proceeding with compilation".to_string());
            recommendations.push("📚 Review UdonSharp documentation for proper usage patterns".to_string());
        }
        
        if total_warnings > 5 {
            recommendations.push("⚠️ Consider addressing warnings to improve code quality".to_string());
        }
        
        if total_errors == 0 && total_warnings == 0 {
            recommendations.push("🎉 Excellent! Your code follows UdonSharp best practices".to_string());
            recommendations.push("🚀 Ready for deployment to VRChat".to_string());
        }
        
        // Always add general recommendations
        recommendations.push("💡 Use #[udon_public] for Inspector-visible fields".to_string());
        recommendations.push("🔄 Use #[udon_sync] for networked multiplayer data".to_string());
        recommendations.push("🎯 Keep behaviors focused on single responsibilities".to_string());
        
        recommendations
    }
    
    /// Create diagnostic collector from analysis results
    pub fn create_diagnostic_collector(&self, analysis_result: &ComprehensiveAnalysisResult) -> DiagnosticCollector {
        let mut collector = DiagnosticCollector::new();
        
        for diagnostic in &analysis_result.diagnostics {
            collector.add(diagnostic.clone());
        }
        
        collector
    }
    
    /// Export analysis results to JSON for IDE integration
    pub fn export_to_json(&self, analysis_result: &FullAnalysisResult) -> Result<String, serde_json::Error> {
        let export_data = AnalysisExport {
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_errors: analysis_result.total_errors,
            total_warnings: analysis_result.total_warnings,
            should_proceed: analysis_result.should_proceed_with_compilation,
            struct_count: analysis_result.struct_analysis.error_report.structs_analyzed,
            class_count: analysis_result.class_validation.validation_result.class_results.len(),
            recommendations: analysis_result.recommendations.clone(),
            summary: analysis_result.combined_report.clone(),
        };
        
        serde_json::to_string_pretty(&export_data)
    }
}

impl Default for ComprehensiveErrorSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the error system
#[derive(Debug, Clone)]
pub struct ErrorSystemConfig {
    /// Use colors in terminal output
    pub use_colors: bool,
    /// Verbose output mode
    pub verbose: bool,
    /// Maximum number of errors to display
    pub max_errors_displayed: usize,
    /// Include suggestions in output
    pub include_suggestions: bool,
    /// Include code examples in output
    pub include_code_examples: bool,
}

impl Default for ErrorSystemConfig {
    fn default() -> Self {
        Self {
            use_colors: true,
            verbose: false,
            max_errors_displayed: 10,
            include_suggestions: true,
            include_code_examples: true,
        }
    }
}

/// Result of comprehensive struct analysis
#[derive(Debug, Clone)]
pub struct ComprehensiveAnalysisResult {
    pub error_report: CompilationErrorReport,
    pub formatted_report: String,
    pub quick_fixes: Vec<QuickFix>,
    pub diagnostics: Vec<Diagnostic>,
    pub should_block_compilation: bool,
    pub analysis_summary: String,
}

/// Result of comprehensive class validation
#[derive(Debug, Clone)]
pub struct ComprehensiveValidationResult {
    pub validation_result: MultiClassValidationResult,
    pub compilation_errors: Vec<crate::error_detection::CompilationError>,
    pub formatted_report: String,
    pub improvement_suggestions: Vec<crate::runtime_validation::ImprovementSuggestion>,
    pub is_valid: bool,
    pub validation_summary: String,
}

/// Result of full analysis (structs + classes)
#[derive(Debug, Clone)]
pub struct FullAnalysisResult {
    pub struct_analysis: ComprehensiveAnalysisResult,
    pub class_validation: ComprehensiveValidationResult,
    pub total_errors: usize,
    pub total_warnings: usize,
    pub should_proceed_with_compilation: bool,
    pub combined_report: String,
    pub recommendations: Vec<String>,
}

/// Export format for IDE integration
#[derive(Debug, Clone, serde::Serialize)]
struct AnalysisExport {
    timestamp: String,
    total_errors: usize,
    total_warnings: usize,
    should_proceed: bool,
    struct_count: usize,
    class_count: usize,
    recommendations: Vec<String>,
    summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::*;
    use crate::code_generator::*;

    fn create_test_struct() -> UdonBehaviourStruct {
        UdonBehaviourStruct::new("TestBehavior".to_string())
    }

    fn create_test_class() -> GeneratedClass {
        GeneratedClass {
            class_name: "TestBehavior".to_string(),
            namespace: None,
            using_statements: vec!["using UnityEngine;".to_string()],
            class_attributes: vec![],
            fields: vec![],
            methods: vec![],
            custom_events: vec![],
            source_code: "public class TestBehavior : UdonSharpBehaviour { }".to_string(),
        }
    }

    #[test]
    fn test_comprehensive_system_creation() {
        let system = ComprehensiveErrorSystem::new();
        // System should be created successfully
        assert!(true);
    }

    #[test]
    fn test_struct_analysis() {
        let system = ComprehensiveErrorSystem::new();
        let test_struct = create_test_struct();
        
        let result = system.analyze_structs(&[test_struct]);
        assert!(result.should_block_compilation); // Missing trait implementation
        assert!(!result.error_report.errors.is_empty());
    }

    #[test]
    fn test_class_validation() {
        let system = ComprehensiveErrorSystem::new();
        let test_class = create_test_class();
        
        let result = system.validate_generated_classes(&[test_class]);
        assert!(result.is_valid); // Basic class should be valid
    }

    #[test]
    fn test_full_analysis() {
        let system = ComprehensiveErrorSystem::new();
        let test_struct = create_test_struct();
        let test_class = create_test_class();
        
        let result = system.full_analysis(&[test_struct], &[test_class]);
        assert!(!result.should_proceed_with_compilation); // Should be blocked due to missing trait
        assert!(result.total_errors > 0);
    }

    #[test]
    fn test_config_customization() {
        let config = ErrorSystemConfig {
            use_colors: false,
            verbose: true,
            max_errors_displayed: 5,
            include_suggestions: false,
            include_code_examples: false,
        };
        
        let system = ComprehensiveErrorSystem::with_config(config);
        // System should be created with custom config
        assert!(true);
    }
}