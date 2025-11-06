//! Main compilation pipeline for Rust to UdonSharp
//! 
//! This module orchestrates the entire compilation process from Rust source
//! to UdonSharp-compatible C# code.

use crate::config::UdonSharpConfig;
use crate::prefab_generator::{UnityPrefabGenerator, PrefabGenerationResult};
use crate::initialization_coordinator::{InitializationCoordinator, CoordinatorGenerationResult};
use crate::standard_multi_behavior_integration::StandardMultiBehaviorPipelineExt;
use udonsharp_core::{UdonSharpResult, error::CompilationContext};
use wasm2usharp_enhanced::{
    EnhancedWasm2USharpPipeline, 
    MultiBehaviorFileGenerator, 
    MultiBehaviorConfig,
    OopBehaviorAnalyzer,
    BehaviorUnit
};
use std::path::Path;
use std::collections::HashMap;

/// Main compilation pipeline
pub struct CompilationPipeline {
    config: UdonSharpConfig,
    context: CompilationContext,
}

impl CompilationPipeline {
    /// Create a new compilation pipeline with the given configuration
    pub fn new(config: UdonSharpConfig) -> Self {
        let context = if config.generate_debug_info {
            CompilationContext::debug()
        } else {
            CompilationContext::new()
        };
        
        Self { config, context }
    }
    
    /// Create a pipeline with a custom context
    pub fn with_context(config: UdonSharpConfig, context: CompilationContext) -> Self {
        Self { config, context }
    }
    
    /// Compile a Rust project to UdonSharp
    pub async fn compile_project<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<CompilationResult> {
        self.context.info(format!("Starting compilation of project: {:?}", project_path.as_ref()));
        
        // Initialize logging for this compilation
        if let Err(e) = self.context.init_logging() {
            self.context.warning(format!("Failed to initialize logging: {}", e));
        }
        
        // Step 1: Read Rust source code
        let rust_source = self.read_rust_source(&project_path)?;
        
        // Step 2: Check if we should use standard multi-behavior pattern
        if self.should_use_standard_multi_behavior(&rust_source)? {
            self.context.info("Using standard multi-behavior compilation pattern");
            return self.compile_with_standard_multi_behavior(&rust_source).await;
        }
        
        // Step 3: Fall back to WASM-based compilation for legacy support
        self.context.info("Using WASM-based compilation (legacy mode)");
        
        // Parse Rust source code and compile to WASM
        let wasm_bytes = self.compile_rust_to_wasm(&project_path).await?;
        
        // Analyze WASM for multi-behavior patterns
        let behavior_analysis = self.analyze_multi_behavior_patterns(&wasm_bytes)?;
        
        // Generate UdonSharp code
        let compilation_result = if behavior_analysis.behavior_units.len() > 1 {
            // Multi-behavior compilation
            self.compile_multi_behavior(&wasm_bytes, &behavior_analysis).await?
        } else {
            // Single behavior compilation (legacy path)
            self.compile_single_behavior(&wasm_bytes).await?
        };
        
        if !self.context.should_continue() {
            return Err(udonsharp_core::UdonSharpError::compilation("Compilation failed due to errors"));
        }
        
        self.context.info("Compilation completed successfully");
        Ok(compilation_result)
    }
    
    /// Read Rust source code from project
    fn read_rust_source<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<String> {
        use std::fs;
        
        let project_path = project_path.as_ref();
        
        // Try to find the main source file
        let possible_paths = [
            project_path.join("src/lib.rs"),
            project_path.join("src/main.rs"),
            project_path.join("lib.rs"),
            project_path.join("main.rs"),
        ];
        
        for path in &possible_paths {
            if path.exists() {
                self.context.info(format!("Reading source from: {:?}", path));
                return fs::read_to_string(path)
                    .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                        format!("Failed to read source file {:?}: {}", path, e)
                    ));
            }
        }
        
        Err(udonsharp_core::UdonSharpError::compilation(
            format!("Could not find Rust source file in project: {:?}", project_path)
        ))
    }
    
    /// Check if we should use standard multi-behavior pattern
    fn should_use_standard_multi_behavior(&self, rust_source: &str) -> UdonSharpResult<bool> {
        // Check if standard multi-behavior pattern is enabled
        if !self.config.multi_behavior.enabled {
            return Ok(false);
        }
        
        // Quick check for multiple #[derive(UdonBehaviour)] annotations
        let udon_behaviour_count = rust_source.matches("#[derive(UdonBehaviour)]").count();
        
        self.context.info(format!("Found {} UdonBehaviour derive annotations", udon_behaviour_count));
        
        Ok(udon_behaviour_count >= self.config.multi_behavior.min_behaviors_threshold)
    }

    /// Compile Rust source to WASM
    async fn compile_rust_to_wasm<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<Vec<u8>> {
        self.context.info("Compiling Rust to WASM...");
        
        // TODO: Implement actual Rust to WASM compilation
        // For now, return a placeholder
        // In a real implementation, this would:
        // 1. Run cargo build --target wasm32-unknown-unknown
        // 2. Read the generated .wasm file
        // 3. Return the bytes
        
        // Placeholder implementation
        Ok(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]) // WASM magic number + version
    }
    
    /// Analyze WASM for multi-behavior patterns
    fn analyze_multi_behavior_patterns(&self, wasm_bytes: &[u8]) -> UdonSharpResult<MultiBehaviorAnalysis> {
        self.context.info("Analyzing WASM for multi-behavior patterns...");
        
        let mut analyzer = OopBehaviorAnalyzer::new();
        let analysis_result = analyzer.analyze(wasm_bytes)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("WASM analysis failed: {}", e)))?;
        
        // Extract behavior units from analysis result
        let behavior_units = analysis_result.behavior_units;
        let shared_functions = analysis_result.shared_functions;
        let call_graph = analysis_result.call_graph.unwrap_or_else(|| {
            // Create empty call graph if none provided
            wasm2usharp_enhanced::CallGraph {
                nodes: HashMap::new(),
                edges: HashMap::new(),
                reverse_edges: HashMap::new(),
            }
        });
        
        self.context.info(format!("Found {} behavior units", behavior_units.len()));
        
        Ok(MultiBehaviorAnalysis {
            behavior_units,
            shared_functions,
            call_graph,
        })
    }
    
    /// Compile using multi-behavior approach
    async fn compile_multi_behavior(&self, wasm_bytes: &[u8], analysis: &MultiBehaviorAnalysis) -> UdonSharpResult<CompilationResult> {
        self.context.info("Compiling with multi-behavior support...");
        
        // Configure multi-behavior file generator
        let naming_convention = match &self.config.multi_behavior.naming_convention {
            crate::config::BehaviorNamingConvention::PascalCase => 
                wasm2usharp_enhanced::BehaviorNamingConvention::PascalCase,
            crate::config::BehaviorNamingConvention::PascalCaseWithSuffix => 
                wasm2usharp_enhanced::BehaviorNamingConvention::PascalCaseWithSuffix,
            crate::config::BehaviorNamingConvention::Custom(format) => 
                wasm2usharp_enhanced::BehaviorNamingConvention::Custom(format.clone()),
        };
        
        let multi_behavior_config = MultiBehaviorConfig {
            namespace: self.config.namespace.clone(),
            generate_shared_runtime: self.config.multi_behavior.generate_shared_runtime && !analysis.shared_functions.is_empty(),
            naming_convention,
            include_debug_info: self.config.generate_debug_info,
            custom_templates: HashMap::new(),
        };
        
        let file_generator = MultiBehaviorFileGenerator::with_config(multi_behavior_config);
        
        // Generate behavior files
        let generation_result = file_generator.generate_all_files(&analysis.behavior_units, &analysis.shared_functions)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("File generation failed: {}", e)))?;
        
        // Generate initialization coordinator if enabled
        let mut coordinator_file = None;
        let mut coordinator_metadata = None;
        
        if self.config.multi_behavior.initialization_order.generate_coordinator {
            let coordinator = InitializationCoordinator::new(
                self.config.multi_behavior.initialization_order.clone()
            );
            
            let coordinator_result = coordinator.generate_coordinator(&analysis.behavior_units, self.config.namespace.as_deref())
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("Coordinator generation failed: {}", e)))?;
            
            // Write coordinator file
            let coordinator_file_name = format!("{}.cs", self.config.multi_behavior.initialization_order.coordinator_class_name);
            self.write_generated_file(&coordinator_file_name, &coordinator_result.coordinator_code)?;
            coordinator_file = Some(coordinator_file_name);
            
            // Write script execution order editor script if enabled
            if let Some(script_execution_order) = &coordinator_result.script_execution_order {
                let editor_script_content = script_execution_order.generate_editor_script(self.config.namespace.as_deref());
                let editor_script_path = "Editor/MultiBehaviorExecutionOrderSetup.cs".to_string();
                self.write_generated_file(&editor_script_path, &editor_script_content)?;
            }
            
            coordinator_metadata = Some(coordinator_result);
        }
        
        // Generate prefabs if enabled
        let mut prefab_files = HashMap::new();
        let mut prefab_metadata = None;
        
        if self.config.multi_behavior.generate_prefabs {
            let prefab_generator = UnityPrefabGenerator::new(
                self.config.multi_behavior.prefab_settings.clone(),
                self.config.multi_behavior.initialization_order.clone(),
            );
            
            let prefab_result = prefab_generator.generate_prefabs(&analysis.behavior_units, self.config.namespace.as_deref())
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("Prefab generation failed: {}", e)))?;
            
            // Write individual prefab files
            for (prefab_name, prefab_content) in &prefab_result.individual_prefabs {
                let prefab_path = if let Some(output_dir) = &self.config.multi_behavior.prefab_settings.output_directory {
                    format!("{}/{}", output_dir, prefab_name)
                } else {
                    prefab_name.clone()
                };
                
                self.write_generated_file(&prefab_path, prefab_content)?;
                prefab_files.insert(prefab_name.clone(), prefab_path);
            }
            
            // Write master prefab if generated
            if let Some(master_prefab_content) = &prefab_result.master_prefab {
                let master_prefab_path = if let Some(output_dir) = &self.config.multi_behavior.prefab_settings.output_directory {
                    format!("{}/MultiBehaviorSystem.prefab", output_dir)
                } else {
                    "MultiBehaviorSystem.prefab".to_string()
                };
                
                self.write_generated_file(&master_prefab_path, master_prefab_content)?;
                prefab_files.insert("MultiBehaviorSystem.prefab".to_string(), master_prefab_path);
            }
            
            // Write example scene if generated
            if let Some(example_scene_content) = &prefab_result.example_scene {
                let scene_path = if let Some(output_dir) = &self.config.multi_behavior.prefab_settings.output_directory {
                    format!("{}/ExampleScene.unity", output_dir)
                } else {
                    "ExampleScene.unity".to_string()
                };
                
                self.write_generated_file(&scene_path, example_scene_content)?;
                prefab_files.insert("ExampleScene.unity".to_string(), scene_path);
            }
            
            prefab_metadata = Some(prefab_result);
        }
        
        // Write files to disk
        let mut output_files = Vec::new();
        let mut behavior_files = HashMap::new();
        let mut shared_runtime_file = None;
        
        // Write behavior files
        for (behavior_name, generated_file) in generation_result.behavior_files {
            let file_path = format!("{}.cs", generated_file.name.trim_end_matches(".cs"));
            self.write_generated_file(&file_path, &generated_file.content)?;
            output_files.push(file_path.clone());
            behavior_files.insert(behavior_name, file_path);
        }
        
        // Write SharedRuntime file if generated
        if let Some(shared_runtime) = generation_result.shared_runtime_file {
            let file_path = "SharedRuntime.cs".to_string();
            self.write_generated_file(&file_path, &shared_runtime.content)?;
            output_files.push(file_path.clone());
            shared_runtime_file = Some(file_path);
        }
        
        // Add prefab files to output
        output_files.extend(prefab_files.values().cloned());
        
        // Add coordinator file to output
        if let Some(coord_file) = &coordinator_file {
            output_files.push(coord_file.clone());
        }
        
        let metadata = MultiBehaviorMetadata {
            total_behaviors: generation_result.metadata.total_behaviors,
            total_files: generation_result.metadata.total_files,
            shared_functions_count: generation_result.metadata.shared_functions_count,
            inter_behavior_calls: generation_result.metadata.inter_behavior_calls,
        };
        
        Ok(CompilationResult {
            success: true,
            output_files,
            diagnostics: self.context.reporter.diagnostics().to_vec(),
            behavior_files,
            shared_runtime_file,
            multi_behavior_metadata: Some(metadata),
            prefab_files,
            prefab_metadata,
            coordinator_file,
            coordinator_metadata,
        })
    }
    
    /// Compile using single behavior approach (legacy)
    async fn compile_single_behavior(&self, wasm_bytes: &[u8]) -> UdonSharpResult<CompilationResult> {
        self.context.info("Compiling with single behavior (legacy mode)...");
        
        // Use the enhanced pipeline for single behavior
        let mut pipeline = EnhancedWasm2USharpPipeline::new();
        let conversion_result = pipeline.convert(wasm_bytes)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("WASM conversion failed: {}", e)))?;
        
        // Write the main file
        let main_file_path = "Main.cs".to_string();
        self.write_generated_file(&main_file_path, &conversion_result.main_class)?;
        
        Ok(CompilationResult {
            success: true,
            output_files: vec![main_file_path],
            diagnostics: self.context.reporter.diagnostics().to_vec(),
            behavior_files: HashMap::new(),
            shared_runtime_file: None,
            multi_behavior_metadata: None,
            prefab_files: HashMap::new(),
            prefab_metadata: None,
            coordinator_file: None,
            coordinator_metadata: None,
        })
    }
    
    /// Write a generated file to disk
    fn write_generated_file(&self, file_path: &str, content: &str) -> UdonSharpResult<()> {
        use std::fs;
        use std::io::Write;
        
        // Create output directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            fs::create_dir_all(parent)
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("Failed to create directory: {}", e)))?;
        }
        
        // Write the file
        let mut file = fs::File::create(file_path)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("Failed to create file {}: {}", file_path, e)))?;
        
        file.write_all(content.as_bytes())
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("Failed to write file {}: {}", file_path, e)))?;
        
        self.context.info(format!("Generated file: {}", file_path));
        Ok(())
    }
    
    /// Get the compilation context
    pub fn context(&self) -> &CompilationContext {
        &self.context
    }
}

/// Result of a compilation operation
#[derive(Debug)]
pub struct CompilationResult {
    pub success: bool,
    pub output_files: Vec<String>,
    pub diagnostics: Vec<udonsharp_core::Diagnostic>,
    /// Generated behavior files (behavior_name -> file_path)
    pub behavior_files: HashMap<String, String>,
    /// Generated SharedRuntime file path (if any)
    pub shared_runtime_file: Option<String>,
    /// Multi-behavior metadata
    pub multi_behavior_metadata: Option<MultiBehaviorMetadata>,
    /// Generated prefab files (filename -> file_path)
    pub prefab_files: HashMap<String, String>,
    /// Generated prefab metadata
    pub prefab_metadata: Option<PrefabGenerationResult>,
    /// Generated coordinator file path (if any)
    pub coordinator_file: Option<String>,
    /// Coordinator generation metadata
    pub coordinator_metadata: Option<CoordinatorGenerationResult>,
}

/// Metadata about multi-behavior compilation
#[derive(Debug, Clone)]
pub struct MultiBehaviorMetadata {
    /// Total number of behaviors generated
    pub total_behaviors: usize,
    /// Total number of files generated
    pub total_files: usize,
    /// Number of shared functions
    pub shared_functions_count: usize,
    /// Inter-behavior calls count
    pub inter_behavior_calls: usize,
}

/// Analysis result for multi-behavior compilation
#[derive(Debug)]
struct MultiBehaviorAnalysis {
    /// Detected behavior units
    behavior_units: Vec<BehaviorUnit>,
    /// Shared functions across behaviors
    shared_functions: Vec<String>,
    /// Function call graph
    call_graph: wasm2usharp_enhanced::CallGraph,
}

impl CompilationPipeline {
    /// Check a project for errors without building
    pub async fn check_project<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<CompilationResult> {
        self.context.info(format!("Checking project: {:?}", project_path.as_ref()));
        
        // TODO: Implement project checking logic
        // This would parse and validate Rust code without generating output
        
        Ok(CompilationResult {
            success: true,
            output_files: vec![],
            diagnostics: self.context.reporter.diagnostics().to_vec(),
            behavior_files: HashMap::new(),
            shared_runtime_file: None,
            multi_behavior_metadata: None,
            prefab_files: HashMap::new(),
            prefab_metadata: None,
            coordinator_file: None,
            coordinator_metadata: None,
        })
    }
    
    /// Run tests for a project
    pub async fn run_tests<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<CompilationResult> {
        self.context.info(format!("Running tests for project: {:?}", project_path.as_ref()));
        
        // TODO: Implement test running logic
        // This would compile and run UdonSharp tests
        
        Ok(CompilationResult {
            success: true,
            output_files: vec![],
            diagnostics: self.context.reporter.diagnostics().to_vec(),
            behavior_files: HashMap::new(),
            shared_runtime_file: None,
            multi_behavior_metadata: None,
            prefab_files: HashMap::new(),
            prefab_metadata: None,
            coordinator_file: None,
            coordinator_metadata: None,
        })
    }
}