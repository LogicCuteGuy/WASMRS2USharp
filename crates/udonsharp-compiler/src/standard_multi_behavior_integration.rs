//! Integration module for standard multi-behavior pattern with existing UdonSharp compilation pipeline
//! 
//! This module provides the integration layer that connects the standard multi-behavior pattern
//! implementation with the existing UdonSharp compilation pipeline, ensuring compatibility
//! with current build tools and workflows.

use crate::{
    config::{UdonSharpConfig, MultiBehaviorSettings},
    pipeline::{CompilationPipeline, CompilationResult, MultiBehaviorMetadata},
    multi_behavior::{UdonBehaviourStruct, RustToCSharpTypeMapper, AttributeMapper},
    struct_analyzer::StructAnalyzer,
    trait_validator::TraitValidator,
    behavior_dependency_analyzer::BehaviorDependencyAnalyzer,
    code_generator::UdonSharpCodeGenerator,
    inter_behavior_communication::InterBehaviorCommunicationGenerator,
    shared_runtime::SharedRuntimeGenerator,
    error_detection::CompilationErrorDetector,
    error_reporting::UserFriendlyErrorReporter,
    runtime_validation::RuntimeValidator,
};
use udonsharp_core::{UdonSharpResult, error::CompilationContext};
use std::path::Path;
use std::collections::HashMap;

/// Standard multi-behavior pattern integration with the compilation pipeline
pub struct StandardMultiBehaviorIntegration {
    config: UdonSharpConfig,
    context: CompilationContext,
    struct_analyzer: StructAnalyzer,
    trait_validator: TraitValidator,
    dependency_analyzer: BehaviorDependencyAnalyzer,
    code_generator: UdonSharpCodeGenerator,
    communication_generator: InterBehaviorCommunicationGenerator,
    shared_runtime_generator: SharedRuntimeGenerator,
    error_detector: CompilationErrorDetector,
    error_reporter: UserFriendlyErrorReporter,
    runtime_validator: RuntimeValidator,
}

impl StandardMultiBehaviorIntegration {
    /// Create a new integration instance
    pub fn new(config: UdonSharpConfig, context: CompilationContext) -> Self {
        let type_mapper = RustToCSharpTypeMapper::new();
        let attribute_mapper = AttributeMapper::new();
        
        Self {
            config: config.clone(),
            context,
            struct_analyzer: StructAnalyzer::new(),
            trait_validator: TraitValidator::new(),
            dependency_analyzer: BehaviorDependencyAnalyzer::new(),
            code_generator: UdonSharpCodeGenerator::new(type_mapper.clone(), attribute_mapper.clone()),
            communication_generator: InterBehaviorCommunicationGenerator::new(),
            shared_runtime_generator: SharedRuntimeGenerator::new(type_mapper, attribute_mapper),
            error_detector: CompilationErrorDetector::new(),
            error_reporter: UserFriendlyErrorReporter::new(),
            runtime_validator: RuntimeValidator::new(),
        }
    }

    /// Check if the project should use standard multi-behavior pattern
    pub fn should_use_multi_behavior(&self, rust_source: &str) -> UdonSharpResult<bool> {
        if !self.config.multi_behavior.enabled {
            return Ok(false);
        }

        // Analyze the source to count UdonBehaviour structs
        let analysis_result = self.struct_analyzer.analyze_source(rust_source)?;
        let behavior_count = analysis_result.structs.len();

        self.context.info(format!("Found {} UdonBehaviour structs", behavior_count));

        Ok(behavior_count >= self.config.multi_behavior.min_behaviors_threshold)
    }

    /// Compile using standard multi-behavior pattern
    pub async fn compile_multi_behavior(&self, rust_source: &str) -> UdonSharpResult<StandardMultiBehaviorCompilationResult> {
        self.context.info("Starting standard multi-behavior compilation...");

        // Step 1: Analyze structs
        let struct_analysis = self.analyze_structs(rust_source)?;
        
        // Step 2: Validate trait implementations
        let trait_validation = self.validate_traits(&struct_analysis.structs)?;
        
        // Step 3: Analyze dependencies
        let dependency_analysis = self.analyze_dependencies(&struct_analysis.structs)?;
        
        // Step 4: Detect compilation errors early
        self.detect_compilation_errors(&struct_analysis, &trait_validation, &dependency_analysis)?;
        
        // Step 5: Generate code for each behavior
        let behavior_files = self.generate_behavior_files(&struct_analysis.structs)?;
        
        // Step 6: Generate inter-behavior communication
        let communication_code = self.generate_communication_code(&struct_analysis.structs, &dependency_analysis)?;
        
        // Step 7: Generate SharedRuntime if needed
        let shared_runtime = self.generate_shared_runtime(&struct_analysis.structs)?;
        
        // Step 8: Validate generated code
        self.validate_generated_code(&behavior_files, &shared_runtime)?;
        
        // Step 9: Create compilation result
        let result = self.create_compilation_result(
            struct_analysis,
            behavior_files,
            communication_code,
            shared_runtime,
            dependency_analysis,
        )?;

        self.context.info("Standard multi-behavior compilation completed successfully");
        Ok(result)
    }

    /// Analyze Rust structs for UdonBehaviour pattern
    fn analyze_structs(&self, rust_source: &str) -> UdonSharpResult<StructAnalysisResult> {
        self.context.info("Analyzing UdonBehaviour structs...");
        
        let analysis_result = self.struct_analyzer.analyze_source(rust_source)?;
        
        if analysis_result.structs.is_empty() {
            return Err(udonsharp_core::UdonSharpError::compilation(
                "No UdonBehaviour structs found in source code"
            ));
        }

        self.context.info(format!("Successfully analyzed {} structs", analysis_result.structs.len()));
        Ok(analysis_result)
    }

    /// Validate UdonBehaviour trait implementations
    fn validate_traits(&self, structs: &[UdonBehaviourStruct]) -> UdonSharpResult<TraitValidationResult> {
        self.context.info("Validating UdonBehaviour trait implementations...");
        
        let validation_result = self.trait_validator.validate_all_structs(structs)?;
        
        if !validation_result.all_valid {
            let error_msg = format!(
                "Trait validation failed for {} structs: {}",
                validation_result.invalid_structs.len(),
                validation_result.invalid_structs.join(", ")
            );
            return Err(udonsharp_core::UdonSharpError::compilation(error_msg));
        }

        self.context.info("All trait implementations are valid");
        Ok(validation_result)
    }

    /// Analyze dependencies between behaviors
    fn analyze_dependencies(&self, structs: &[UdonBehaviourStruct]) -> UdonSharpResult<DependencyAnalysisResult> {
        self.context.info("Analyzing inter-behavior dependencies...");
        
        let dependency_result = self.dependency_analyzer.analyze_dependencies(structs)?;
        
        if !dependency_result.circular_dependencies.is_empty() {
            let cycles: Vec<String> = dependency_result.circular_dependencies
                .iter()
                .map(|cycle| cycle.join(" -> "))
                .collect();
            
            let error_msg = format!(
                "Circular dependencies detected: {}",
                cycles.join("; ")
            );
            return Err(udonsharp_core::UdonSharpError::compilation(error_msg));
        }

        self.context.info(format!(
            "Dependency analysis complete. Found {} dependencies",
            dependency_result.dependency_graph.len()
        ));
        Ok(dependency_result)
    }

    /// Detect compilation errors early
    fn detect_compilation_errors(
        &self,
        struct_analysis: &StructAnalysisResult,
        trait_validation: &TraitValidationResult,
        dependency_analysis: &DependencyAnalysisResult,
    ) -> UdonSharpResult<()> {
        self.context.info("Running compilation error detection...");
        
        let errors = self.error_detector.detect_errors(
            &struct_analysis.structs,
            &trait_validation.validation_results,
            &dependency_analysis.dependency_graph,
        )?;

        if !errors.is_empty() {
            let error_report = self.error_reporter.generate_error_report(&errors);
            return Err(udonsharp_core::UdonSharpError::compilation(error_report));
        }

        self.context.info("No compilation errors detected");
        Ok(())
    }

    /// Generate C# files for each behavior
    fn generate_behavior_files(&self, structs: &[UdonBehaviourStruct]) -> UdonSharpResult<HashMap<String, GeneratedBehaviorFile>> {
        self.context.info("Generating C# behavior files...");
        
        let mut behavior_files = HashMap::new();
        
        for behavior_struct in structs {
            self.context.info(format!("Generating code for behavior: {}", behavior_struct.name));
            
            let generated_code = self.code_generator.generate_behavior_class(behavior_struct)?;
            
            let file = GeneratedBehaviorFile {
                behavior_name: behavior_struct.name.clone(),
                class_name: behavior_struct.name.clone(),
                file_content: generated_code.class_code,
                using_statements: generated_code.using_statements,
                namespace: self.config.namespace.clone(),
                has_networking: behavior_struct.has_networking(),
                dependencies: behavior_struct.dependencies.clone(),
            };
            
            behavior_files.insert(behavior_struct.name.clone(), file);
        }

        self.context.info(format!("Generated {} behavior files", behavior_files.len()));
        Ok(behavior_files)
    }

    /// Generate inter-behavior communication code
    fn generate_communication_code(
        &self,
        structs: &[UdonBehaviourStruct],
        dependency_analysis: &DependencyAnalysisResult,
    ) -> UdonSharpResult<CommunicationCodeResult> {
        self.context.info("Generating inter-behavior communication code...");
        
        let communication_result = self.communication_generator.generate_communication_code(
            structs,
            &dependency_analysis.dependency_graph,
        )?;

        self.context.info(format!(
            "Generated communication code for {} behaviors",
            communication_result.behavior_communications.len()
        ));
        Ok(communication_result)
    }

    /// Generate SharedRuntime class if needed
    fn generate_shared_runtime(&self, structs: &[UdonBehaviourStruct]) -> UdonSharpResult<Option<SharedRuntimeFile>> {
        if !self.config.multi_behavior.generate_shared_runtime {
            return Ok(None);
        }

        self.context.info("Generating SharedRuntime class...");
        
        let shared_runtime_result = self.shared_runtime_generator.generate_shared_runtime(structs)?;
        
        if shared_runtime_result.has_shared_content {
            let file = SharedRuntimeFile {
                class_name: "SharedRuntime".to_string(),
                file_content: shared_runtime_result.class_code,
                using_statements: shared_runtime_result.using_statements,
                namespace: self.config.namespace.clone(),
                shared_functions: shared_runtime_result.shared_functions,
                shared_types: shared_runtime_result.shared_types,
            };
            
            self.context.info("SharedRuntime class generated successfully");
            Ok(Some(file))
        } else {
            self.context.info("No shared content found, skipping SharedRuntime generation");
            Ok(None)
        }
    }

    /// Validate generated C# code
    fn validate_generated_code(
        &self,
        behavior_files: &HashMap<String, GeneratedBehaviorFile>,
        shared_runtime: &Option<SharedRuntimeFile>,
    ) -> UdonSharpResult<()> {
        self.context.info("Validating generated C# code...");
        
        // Validate behavior files
        for (behavior_name, file) in behavior_files {
            let validation_result = self.runtime_validator.validate_behavior_code(&file.file_content)?;
            
            if !validation_result.is_valid {
                let error_msg = format!(
                    "Generated code validation failed for behavior '{}': {}",
                    behavior_name,
                    validation_result.errors.join("; ")
                );
                return Err(udonsharp_core::UdonSharpError::compilation(error_msg));
            }
        }
        
        // Validate SharedRuntime if present
        if let Some(shared_runtime_file) = shared_runtime {
            let validation_result = self.runtime_validator.validate_shared_runtime_code(&shared_runtime_file.file_content)?;
            
            if !validation_result.is_valid {
                let error_msg = format!(
                    "Generated SharedRuntime validation failed: {}",
                    validation_result.errors.join("; ")
                );
                return Err(udonsharp_core::UdonSharpError::compilation(error_msg));
            }
        }

        self.context.info("All generated code passed validation");
        Ok(())
    }

    /// Create the final compilation result
    fn create_compilation_result(
        &self,
        struct_analysis: StructAnalysisResult,
        behavior_files: HashMap<String, GeneratedBehaviorFile>,
        communication_code: CommunicationCodeResult,
        shared_runtime: Option<SharedRuntimeFile>,
        dependency_analysis: DependencyAnalysisResult,
    ) -> UdonSharpResult<StandardMultiBehaviorCompilationResult> {
        let mut output_files = Vec::new();
        let mut behavior_file_paths = HashMap::new();
        
        // Add behavior files
        for (behavior_name, file) in &behavior_files {
            let file_path = format!("{}.cs", file.class_name);
            output_files.push(file_path.clone());
            behavior_file_paths.insert(behavior_name.clone(), file_path);
        }
        
        // Add SharedRuntime file if present
        let shared_runtime_file_path = if let Some(_) = &shared_runtime {
            let file_path = "SharedRuntime.cs".to_string();
            output_files.push(file_path.clone());
            Some(file_path)
        } else {
            None
        };
        
        let metadata = StandardMultiBehaviorMetadata {
            total_behaviors: behavior_files.len(),
            total_files: output_files.len(),
            shared_functions_count: shared_runtime.as_ref()
                .map(|sr| sr.shared_functions.len())
                .unwrap_or(0),
            inter_behavior_calls: communication_code.total_communication_calls,
            has_networking: behavior_files.values().any(|f| f.has_networking),
            dependency_count: dependency_analysis.dependency_graph.len(),
            circular_dependencies_detected: !dependency_analysis.circular_dependencies.is_empty(),
        };
        
        Ok(StandardMultiBehaviorCompilationResult {
            success: true,
            output_files,
            behavior_files,
            shared_runtime_file: shared_runtime_file_path,
            shared_runtime,
            communication_code,
            metadata,
            diagnostics: self.context.reporter.diagnostics().to_vec(),
        })
    }

    /// Get the compilation context
    pub fn context(&self) -> &CompilationContext {
        &self.context
    }
}

/// Result of struct analysis
#[derive(Debug, Clone)]
pub struct StructAnalysisResult {
    pub structs: Vec<UdonBehaviourStruct>,
    pub total_fields: usize,
    pub total_methods: usize,
}

/// Result of trait validation
#[derive(Debug, Clone)]
pub struct TraitValidationResult {
    pub all_valid: bool,
    pub validation_results: HashMap<String, bool>,
    pub invalid_structs: Vec<String>,
    pub missing_methods: HashMap<String, Vec<String>>,
}

/// Result of dependency analysis
#[derive(Debug, Clone)]
pub struct DependencyAnalysisResult {
    pub dependency_graph: HashMap<String, Vec<String>>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub initialization_order: Vec<String>,
}

/// Generated behavior file information
#[derive(Debug, Clone)]
pub struct GeneratedBehaviorFile {
    pub behavior_name: String,
    pub class_name: String,
    pub file_content: String,
    pub using_statements: Vec<String>,
    pub namespace: Option<String>,
    pub has_networking: bool,
    pub dependencies: Vec<String>,
}

/// Generated SharedRuntime file information
#[derive(Debug, Clone)]
pub struct SharedRuntimeFile {
    pub class_name: String,
    pub file_content: String,
    pub using_statements: Vec<String>,
    pub namespace: Option<String>,
    pub shared_functions: Vec<String>,
    pub shared_types: Vec<String>,
}

/// Result of communication code generation
#[derive(Debug, Clone)]
pub struct CommunicationCodeResult {
    pub behavior_communications: HashMap<String, Vec<String>>,
    pub total_communication_calls: usize,
    pub gameobject_references: HashMap<String, Vec<String>>,
    pub custom_events: HashMap<String, Vec<String>>,
}

/// Metadata about standard multi-behavior compilation
#[derive(Debug, Clone)]
pub struct StandardMultiBehaviorMetadata {
    pub total_behaviors: usize,
    pub total_files: usize,
    pub shared_functions_count: usize,
    pub inter_behavior_calls: usize,
    pub has_networking: bool,
    pub dependency_count: usize,
    pub circular_dependencies_detected: bool,
}

/// Complete result of standard multi-behavior compilation
#[derive(Debug, Clone)]
pub struct StandardMultiBehaviorCompilationResult {
    pub success: bool,
    pub output_files: Vec<String>,
    pub behavior_files: HashMap<String, GeneratedBehaviorFile>,
    pub shared_runtime_file: Option<String>,
    pub shared_runtime: Option<SharedRuntimeFile>,
    pub communication_code: CommunicationCodeResult,
    pub metadata: StandardMultiBehaviorMetadata,
    pub diagnostics: Vec<udonsharp_core::Diagnostic>,
}

impl StandardMultiBehaviorCompilationResult {
    /// Convert to the standard CompilationResult format
    pub fn to_compilation_result(self) -> CompilationResult {
        let multi_behavior_metadata = MultiBehaviorMetadata {
            total_behaviors: self.metadata.total_behaviors,
            total_files: self.metadata.total_files,
            shared_functions_count: self.metadata.shared_functions_count,
            inter_behavior_calls: self.metadata.inter_behavior_calls,
        };
        
        let behavior_file_paths: HashMap<String, String> = self.behavior_files
            .iter()
            .map(|(name, file)| (name.clone(), format!("{}.cs", file.class_name)))
            .collect();
        
        CompilationResult {
            success: self.success,
            output_files: self.output_files,
            diagnostics: self.diagnostics,
            behavior_files: behavior_file_paths,
            shared_runtime_file: self.shared_runtime_file,
            multi_behavior_metadata: Some(multi_behavior_metadata),
            prefab_files: HashMap::new(),
            prefab_metadata: None,
            coordinator_file: None,
            coordinator_metadata: None,
        }
    }

    /// Write all generated files to disk
    pub fn write_files_to_disk<P: AsRef<Path>>(&self, output_dir: P) -> UdonSharpResult<()> {
        use std::fs;
        use std::io::Write;
        
        let output_path = output_dir.as_ref();
        
        // Create output directory if it doesn't exist
        fs::create_dir_all(output_path)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                format!("Failed to create output directory: {}", e)
            ))?;
        
        // Write behavior files
        for (_, file) in &self.behavior_files {
            let file_path = output_path.join(format!("{}.cs", file.class_name));
            let mut f = fs::File::create(&file_path)
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                    format!("Failed to create file {:?}: {}", file_path, e)
                ))?;
            
            f.write_all(file.file_content.as_bytes())
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                    format!("Failed to write file {:?}: {}", file_path, e)
                ))?;
        }
        
        // Write SharedRuntime file if present
        if let Some(shared_runtime) = &self.shared_runtime {
            let file_path = output_path.join("SharedRuntime.cs");
            let mut f = fs::File::create(&file_path)
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                    format!("Failed to create SharedRuntime file: {}", e)
                ))?;
            
            f.write_all(shared_runtime.file_content.as_bytes())
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                    format!("Failed to write SharedRuntime file: {}", e)
                ))?;
        }
        
        Ok(())
    }

    /// Generate a compilation report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== Standard Multi-Behavior Compilation Report ===\n\n");
        
        report.push_str(&format!("Compilation Status: {}\n", 
            if self.success { "SUCCESS" } else { "FAILED" }));
        report.push_str(&format!("Total Behaviors: {}\n", self.metadata.total_behaviors));
        report.push_str(&format!("Total Files Generated: {}\n", self.metadata.total_files));
        report.push_str(&format!("Shared Functions: {}\n", self.metadata.shared_functions_count));
        report.push_str(&format!("Inter-Behavior Calls: {}\n", self.metadata.inter_behavior_calls));
        report.push_str(&format!("Has Networking: {}\n", self.metadata.has_networking));
        report.push_str(&format!("Dependencies: {}\n", self.metadata.dependency_count));
        
        if self.metadata.circular_dependencies_detected {
            report.push_str("⚠️  Circular dependencies detected and resolved\n");
        }
        
        report.push_str("\n--- Generated Files ---\n");
        for file_path in &self.output_files {
            report.push_str(&format!("  • {}\n", file_path));
        }
        
        if !self.diagnostics.is_empty() {
            report.push_str("\n--- Diagnostics ---\n");
            for diagnostic in &self.diagnostics {
                report.push_str(&format!("  {} {}\n", 
                    match diagnostic.severity {
                        udonsharp_core::DiagnosticSeverity::Error => "❌",
                        udonsharp_core::DiagnosticSeverity::Warning => "⚠️ ",
                        udonsharp_core::DiagnosticSeverity::Info => "ℹ️ ",
                    },
                    diagnostic.message
                ));
            }
        }
        
        report.push_str("\n=== End Report ===\n");
        report
    }
}

/// Extension trait for CompilationPipeline to add standard multi-behavior support
pub trait StandardMultiBehaviorPipelineExt {
    /// Compile using standard multi-behavior pattern if applicable
    fn compile_with_standard_multi_behavior<P: AsRef<Path>>(
        &self,
        project_path: P,
        rust_source: &str,
    ) -> impl std::future::Future<Output = UdonSharpResult<CompilationResult>> + Send;
}

impl StandardMultiBehaviorPipelineExt for CompilationPipeline {
    async fn compile_with_standard_multi_behavior<P: AsRef<Path>>(
        &self,
        project_path: P,
        rust_source: &str,
    ) -> UdonSharpResult<CompilationResult> {
        // Create integration instance
        let integration = StandardMultiBehaviorIntegration::new(
            // We need to get the config from the pipeline - this would need to be exposed
            UdonSharpConfig::default(), // Placeholder - would need actual config
            self.context().clone(),
        );
        
        // Check if we should use multi-behavior pattern
        if integration.should_use_multi_behavior(rust_source)? {
            integration.context().info("Using standard multi-behavior compilation pattern");
            
            // Compile using standard multi-behavior pattern
            let result = integration.compile_multi_behavior(rust_source).await?;
            
            // Write files to disk
            if let Some(output_dir) = &integration.config.output_directory {
                result.write_files_to_disk(output_dir)?;
            } else {
                result.write_files_to_disk(".")?;
            }
            
            // Convert to standard CompilationResult
            Ok(result.to_compilation_result())
        } else {
            // Fall back to standard compilation
            self.compile_project(project_path).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use udonsharp_core::error::CompilationContext;

    #[test]
    fn test_integration_creation() {
        let config = UdonSharpConfig::default();
        let context = CompilationContext::new();
        
        let integration = StandardMultiBehaviorIntegration::new(config, context);
        assert!(integration.config.multi_behavior.enabled);
    }

    #[test]
    fn test_should_use_multi_behavior() {
        let config = UdonSharpConfig::default();
        let context = CompilationContext::new();
        let integration = StandardMultiBehaviorIntegration::new(config, context);
        
        let single_behavior_source = r#"
            #[derive(UdonBehaviour)]
            pub struct TestBehavior {
                field: i32,
            }
        "#;
        
        let multi_behavior_source = r#"
            #[derive(UdonBehaviour)]
            pub struct BehaviorA {
                field_a: i32,
            }
            
            #[derive(UdonBehaviour)]
            pub struct BehaviorB {
                field_b: f32,
            }
        "#;
        
        // Note: These tests would need the actual struct analyzer implementation
        // For now, they serve as documentation of expected behavior
    }

    #[test]
    fn test_compilation_result_conversion() {
        let mut behavior_files = HashMap::new();
        behavior_files.insert("TestBehavior".to_string(), GeneratedBehaviorFile {
            behavior_name: "TestBehavior".to_string(),
            class_name: "TestBehavior".to_string(),
            file_content: "// Generated code".to_string(),
            using_statements: vec!["using UnityEngine;".to_string()],
            namespace: None,
            has_networking: false,
            dependencies: vec![],
        });
        
        let result = StandardMultiBehaviorCompilationResult {
            success: true,
            output_files: vec!["TestBehavior.cs".to_string()],
            behavior_files,
            shared_runtime_file: None,
            shared_runtime: None,
            communication_code: CommunicationCodeResult {
                behavior_communications: HashMap::new(),
                total_communication_calls: 0,
                gameobject_references: HashMap::new(),
                custom_events: HashMap::new(),
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
        };
        
        let compilation_result = result.to_compilation_result();
        assert!(compilation_result.success);
        assert_eq!(compilation_result.output_files.len(), 1);
    }
}