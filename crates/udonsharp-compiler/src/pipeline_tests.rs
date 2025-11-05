//! End-to-end compilation pipeline tests
//! 
//! This module contains tests for the complete Rust → WASM → UdonSharp pipeline,
//! including regression tests and performance benchmarks.

use crate::{
    config::{UdonSharpConfig, WasmTargetConfig},
    pipeline::UdonSharpCompilationPipeline,
    wasm_compiler::RustToWasmCompiler,
    optimizer::{WasmOptimizer, UdonSharpOptimizer, OptimizationLevel},
};
use wasm2usharp_enhanced::{
    EnhancedWasm2USharpPipeline, ConversionConfig, OopBehaviorAnalyzer,
    file_generator::MainClassGenerator,
};
use std::path::PathBuf;
use std::time::Instant;

/// Test configuration for pipeline tests
#[derive(Debug, Clone)]
pub struct PipelineTestConfig {
    pub test_name: String,
    pub rust_source: String,
    pub expected_csharp_patterns: Vec<String>,
    pub should_optimize: bool,
    pub target_config: WasmTargetConfig,
}

impl PipelineTestConfig {
    pub fn simple_test() -> Self {
        Self {
            test_name: "simple_udon_behaviour".to_string(),
            rust_source: r#"
                use udonsharp_core::prelude::*;
                
                #[derive(UdonBehaviour)]
                #[udon_sync_mode("Manual")]
                pub struct SimpleController {
                    #[udon_public]
                    pub message: String,
                    
                    #[udon_sync]
                    pub counter: i32,
                    
                    initialized: bool,
                }
                
                impl UdonBehaviour for SimpleController {
                    fn start(&mut self) {
                        self.initialized = true;
                        self.message = "Hello UdonSharp!".to_string();
                        self.counter = 0;
                    }
                    
                    fn update(&mut self) {
                        if self.initialized {
                            self.counter += 1;
                        }
                    }
                }
                
                impl SimpleController {
                    pub fn new() -> Self {
                        Self {
                            message: String::new(),
                            counter: 0,
                            initialized: false,
                        }
                    }
                    
                    pub fn reset_counter(&mut self) {
                        self.counter = 0;
                    }
                }
            "#.to_string(),
            expected_csharp_patterns: vec![
                "public class SimpleController : UdonSharpBehaviour".to_string(),
                "[UdonSynced]".to_string(),
                "public string message".to_string(),
                "public int counter".to_string(),
                "void Start()".to_string(),
                "void Update()".to_string(),
                "w2us_init()".to_string(),
                "w2us_start()".to_string(),
            ],
            should_optimize: true,
            target_config: WasmTargetConfig::development(),
        }
    }
    
    pub fn networking_test() -> Self {
        Self {
            test_name: "networking_behaviour".to_string(),
            rust_source: r#"
                use udonsharp_core::prelude::*;
                
                #[derive(UdonBehaviour)]
                #[udon_sync_mode("Manual")]
                pub struct NetworkController {
                    #[udon_sync]
                    pub player_count: i32,
                    
                    #[udon_sync]
                    pub world_state: String,
                    
                    players: Vec<VRCPlayerApi>,
                }
                
                impl UdonBehaviour for NetworkController {
                    fn start(&mut self) {
                        self.player_count = 1;
                        self.world_state = "Active".to_string();
                        self.players = Vec::new();
                    }
                    
                    fn on_player_joined(&mut self, player: VRCPlayerApi) {
                        self.players.push(player);
                        self.player_count = self.players.len() as i32;
                        self.sync_variables();
                    }
                    
                    fn on_player_left(&mut self, player: VRCPlayerApi) {
                        self.players.retain(|p| p.get_display_name() != player.get_display_name());
                        self.player_count = self.players.len() as i32;
                        self.sync_variables();
                    }
                }
                
                impl NetworkController {
                    pub fn new() -> Self {
                        Self {
                            player_count: 0,
                            world_state: String::new(),
                            players: Vec::new(),
                        }
                    }
                    
                    fn sync_variables(&mut self) {
                        // This would trigger UdonSharp networking sync
                    }
                }
            "#.to_string(),
            expected_csharp_patterns: vec![
                "public class NetworkController : UdonSharpBehaviour".to_string(),
                "[UdonSynced]".to_string(),
                "public int playerCount".to_string(),
                "public string worldState".to_string(),
                "public override void OnPlayerJoined(VRCPlayerApi player)".to_string(),
                "public override void OnPlayerLeft(VRCPlayerApi player)".to_string(),
            ],
            should_optimize: true,
            target_config: WasmTargetConfig::production(),
        }
    }
    
    pub fn complex_oop_test() -> Self {
        Self {
            test_name: "complex_oop_behaviour".to_string(),
            rust_source: r#"
                use udonsharp_core::prelude::*;
                
                pub trait Interactable {
                    fn interact(&mut self, player: VRCPlayerApi);
                    fn can_interact(&self, player: VRCPlayerApi) -> bool;
                }
                
                #[derive(UdonBehaviour)]
                pub struct InteractableButton {
                    #[udon_public]
                    pub button_text: String,
                    
                    #[udon_public]
                    pub interaction_count: i32,
                    
                    #[udon_sync]
                    pub is_enabled: bool,
                    
                    last_interactor: Option<String>,
                }
                
                impl UdonBehaviour for InteractableButton {
                    fn start(&mut self) {
                        self.button_text = "Click Me!".to_string();
                        self.interaction_count = 0;
                        self.is_enabled = true;
                        self.last_interactor = None;
                    }
                }
                
                impl Interactable for InteractableButton {
                    fn interact(&mut self, player: VRCPlayerApi) {
                        if self.can_interact(player) {
                            self.interaction_count += 1;
                            self.last_interactor = Some(player.get_display_name());
                            self.on_button_clicked(player);
                        }
                    }
                    
                    fn can_interact(&self, _player: VRCPlayerApi) -> bool {
                        self.is_enabled
                    }
                }
                
                impl InteractableButton {
                    pub fn new() -> Self {
                        Self {
                            button_text: String::new(),
                            interaction_count: 0,
                            is_enabled: true,
                            last_interactor: None,
                        }
                    }
                    
                    fn on_button_clicked(&mut self, player: VRCPlayerApi) {
                        // Custom button click logic
                        println!("Button clicked by: {}", player.get_display_name());
                    }
                    
                    pub fn reset(&mut self) {
                        self.interaction_count = 0;
                        self.last_interactor = None;
                    }
                    
                    pub fn set_enabled(&mut self, enabled: bool) {
                        self.is_enabled = enabled;
                    }
                }
            "#.to_string(),
            expected_csharp_patterns: vec![
                "public class InteractableButton : UdonSharpBehaviour".to_string(),
                "public string buttonText".to_string(),
                "public int interactionCount".to_string(),
                "[UdonSynced]".to_string(),
                "public bool isEnabled".to_string(),
                "public void Interact(".to_string(),
                "public bool CanInteract(".to_string(),
                "private void OnButtonClicked(".to_string(),
                "public void Reset()".to_string(),
                "public void SetEnabled(".to_string(),
            ],
            should_optimize: true,
            target_config: WasmTargetConfig::production(),
        }
    }
}

/// Performance metrics for pipeline tests
#[derive(Debug, Clone)]
pub struct PipelinePerformanceMetrics {
    pub test_name: String,
    pub rust_compilation_time: std::time::Duration,
    pub wasm_optimization_time: std::time::Duration,
    pub wasm_to_csharp_time: std::time::Duration,
    pub total_pipeline_time: std::time::Duration,
    pub rust_source_size: usize,
    pub wasm_size_before_optimization: usize,
    pub wasm_size_after_optimization: usize,
    pub generated_csharp_size: usize,
    pub generated_file_count: usize,
}

impl PipelinePerformanceMetrics {
    pub fn new(test_name: String) -> Self {
        Self {
            test_name,
            rust_compilation_time: std::time::Duration::ZERO,
            wasm_optimization_time: std::time::Duration::ZERO,
            wasm_to_csharp_time: std::time::Duration::ZERO,
            total_pipeline_time: std::time::Duration::ZERO,
            rust_source_size: 0,
            wasm_size_before_optimization: 0,
            wasm_size_after_optimization: 0,
            generated_csharp_size: 0,
            generated_file_count: 0,
        }
    }
    
    pub fn optimization_effectiveness(&self) -> f32 {
        if self.wasm_size_before_optimization == 0 {
            return 0.0;
        }
        
        let reduction = self.wasm_size_before_optimization - self.wasm_size_after_optimization;
        (reduction as f32 / self.wasm_size_before_optimization as f32) * 100.0
    }
    
    pub fn compilation_speed(&self) -> f32 {
        if self.total_pipeline_time.as_secs_f32() == 0.0 {
            return 0.0;
        }
        
        self.rust_source_size as f32 / self.total_pipeline_time.as_secs_f32()
    }
    
    pub fn summary(&self) -> String {
        format!(
            "Pipeline Test: {}\n\
             Total Time: {:.2}s\n\
             Rust Compilation: {:.2}s\n\
             WASM Optimization: {:.2}s\n\
             WASM to C#: {:.2}s\n\
             WASM Size Reduction: {:.1}%\n\
             Compilation Speed: {:.0} chars/sec\n\
             Generated Files: {}",
            self.test_name,
            self.total_pipeline_time.as_secs_f32(),
            self.rust_compilation_time.as_secs_f32(),
            self.wasm_optimization_time.as_secs_f32(),
            self.wasm_to_csharp_time.as_secs_f32(),
            self.optimization_effectiveness(),
            self.compilation_speed(),
            self.generated_file_count
        )
    }
}

/// End-to-end pipeline test runner
pub struct PipelineTestRunner {
    temp_dir: PathBuf,
    performance_metrics: Vec<PipelinePerformanceMetrics>,
}

impl PipelineTestRunner {
    pub fn new() -> std::io::Result<Self> {
        let temp_dir = std::env::temp_dir().join("udonsharp_pipeline_tests");
        std::fs::create_dir_all(&temp_dir)?;
        
        Ok(Self {
            temp_dir,
            performance_metrics: Vec::new(),
        })
    }
    
    pub fn run_pipeline_test(&mut self, config: PipelineTestConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut metrics = PipelinePerformanceMetrics::new(config.test_name.clone());
        let total_start = Instant::now();
        
        // Set up test project structure
        let project_dir = self.temp_dir.join(&config.test_name);
        std::fs::create_dir_all(&project_dir)?;
        
        // Create Cargo.toml
        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
udonsharp-core = {{ path = "../../../udonsharp-core" }}
udonsharp-macros = {{ path = "../../../udonsharp-macros" }}

[profile.release]
opt-level = "s"
lto = true
panic = "abort"
"#,
            config.test_name
        );
        
        std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
        
        // Create src directory and lib.rs
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(src_dir.join("lib.rs"), &config.rust_source)?;
        
        metrics.rust_source_size = config.rust_source.len();
        
        // Step 1: Rust to WASM compilation
        let rust_start = Instant::now();
        let udon_config = UdonSharpConfig::default();
        let compiler = RustToWasmCompiler::with_wasm_config(udon_config, config.target_config.clone());
        
        let wasm_output = compiler.compile_project(&project_dir)?;
        metrics.rust_compilation_time = rust_start.elapsed();
        metrics.wasm_size_before_optimization = wasm_output.len();
        
        // Step 2: WASM optimization
        let opt_start = Instant::now();
        let optimizer = if config.should_optimize {
            WasmOptimizer::new(OptimizationLevel::UdonSharp)
        } else {
            WasmOptimizer::new(OptimizationLevel::None)
        };
        
        let optimized_wasm = optimizer.optimize(&wasm_output)?;
        metrics.wasm_optimization_time = opt_start.elapsed();
        metrics.wasm_size_after_optimization = optimized_wasm.len();
        
        // Step 3: WASM to UdonSharp conversion
        let conversion_start = Instant::now();
        let conversion_config = ConversionConfig {
            class_name: format!("{}Generated", config.test_name),
            namespace: Some("UdonSharp.Generated".to_string()),
            generate_separate_files: true,
            udonsharp_attributes: true,
            inheritance_support: true,
        };
        
        let pipeline = EnhancedWasm2USharpPipeline::new();
        let conversion_result = pipeline.convert_with_config(&optimized_wasm, conversion_config)?;
        
        metrics.wasm_to_csharp_time = conversion_start.elapsed();
        metrics.generated_file_count = conversion_result.files.len();
        metrics.generated_csharp_size = conversion_result.files.iter()
            .map(|f| f.content.len())
            .sum();
        
        // Step 4: Validate generated C# contains expected patterns
        self.validate_generated_csharp(&conversion_result.files, &config.expected_csharp_patterns)?;
        
        metrics.total_pipeline_time = total_start.elapsed();
        self.performance_metrics.push(metrics);
        
        println!("Pipeline test '{}' completed successfully", config.test_name);
        Ok(())
    }
    
    fn validate_generated_csharp(
        &self,
        files: &[wasm2usharp_enhanced::file_generator::CSharpFile],
        expected_patterns: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let combined_content = files.iter()
            .map(|f| f.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        for pattern in expected_patterns {
            if !combined_content.contains(pattern) {
                return Err(format!(
                    "Expected pattern '{}' not found in generated C# code",
                    pattern
                ).into());
            }
        }
        
        Ok(())
    }
    
    pub fn run_regression_tests(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running regression tests for compilation pipeline...");
        
        // Test basic UdonBehaviour generation
        self.run_pipeline_test(PipelineTestConfig::simple_test())?;
        
        // Test networking features
        self.run_pipeline_test(PipelineTestConfig::networking_test())?;
        
        // Test complex OOP patterns
        self.run_pipeline_test(PipelineTestConfig::complex_oop_test())?;
        
        println!("All regression tests passed!");
        Ok(())
    }
    
    pub fn run_performance_benchmarks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running performance benchmarks...");
        
        // Run each test multiple times for consistent metrics
        for _ in 0..3 {
            self.run_pipeline_test(PipelineTestConfig::simple_test())?;
            self.run_pipeline_test(PipelineTestConfig::networking_test())?;
            self.run_pipeline_test(PipelineTestConfig::complex_oop_test())?;
        }
        
        self.print_performance_summary();
        Ok(())
    }
    
    pub fn print_performance_summary(&self) {
        println!("\n=== Pipeline Performance Summary ===");
        
        // Group metrics by test name
        let mut grouped_metrics: std::collections::HashMap<String, Vec<&PipelinePerformanceMetrics>> = 
            std::collections::HashMap::new();
        
        for metric in &self.performance_metrics {
            grouped_metrics.entry(metric.test_name.clone())
                .or_insert_with(Vec::new)
                .push(metric);
        }
        
        for (test_name, metrics) in grouped_metrics {
            if metrics.is_empty() {
                continue;
            }
            
            let avg_total_time = metrics.iter()
                .map(|m| m.total_pipeline_time.as_secs_f32())
                .sum::<f32>() / metrics.len() as f32;
            
            let avg_optimization = metrics.iter()
                .map(|m| m.optimization_effectiveness())
                .sum::<f32>() / metrics.len() as f32;
            
            let avg_speed = metrics.iter()
                .map(|m| m.compilation_speed())
                .sum::<f32>() / metrics.len() as f32;
            
            println!("\nTest: {}", test_name);
            println!("  Average Total Time: {:.2}s", avg_total_time);
            println!("  Average WASM Size Reduction: {:.1}%", avg_optimization);
            println!("  Average Compilation Speed: {:.0} chars/sec", avg_speed);
            println!("  Runs: {}", metrics.len());
        }
        
        // Performance thresholds
        let max_acceptable_time = 10.0; // seconds
        let min_optimization_effectiveness = 10.0; // percent
        let min_compilation_speed = 1000.0; // chars/sec
        
        println!("\n=== Performance Analysis ===");
        for (test_name, metrics) in grouped_metrics {
            let avg_time = metrics.iter()
                .map(|m| m.total_pipeline_time.as_secs_f32())
                .sum::<f32>() / metrics.len() as f32;
            
            let avg_optimization = metrics.iter()
                .map(|m| m.optimization_effectiveness())
                .sum::<f32>() / metrics.len() as f32;
            
            let avg_speed = metrics.iter()
                .map(|m| m.compilation_speed())
                .sum::<f32>() / metrics.len() as f32;
            
            println!("\n{}: ", test_name);
            
            if avg_time > max_acceptable_time {
                println!("  ⚠️  SLOW: Compilation time {:.2}s exceeds threshold of {:.2}s", 
                        avg_time, max_acceptable_time);
            } else {
                println!("  ✅ FAST: Compilation time {:.2}s is acceptable", avg_time);
            }
            
            if avg_optimization < min_optimization_effectiveness {
                println!("  ⚠️  LOW OPTIMIZATION: {:.1}% reduction below threshold of {:.1}%", 
                        avg_optimization, min_optimization_effectiveness);
            } else {
                println!("  ✅ GOOD OPTIMIZATION: {:.1}% size reduction", avg_optimization);
            }
            
            if avg_speed < min_compilation_speed {
                println!("  ⚠️  SLOW SPEED: {:.0} chars/sec below threshold of {:.0} chars/sec", 
                        avg_speed, min_compilation_speed);
            } else {
                println!("  ✅ GOOD SPEED: {:.0} chars/sec compilation speed", avg_speed);
            }
        }
    }
    
    pub fn cleanup(&self) -> std::io::Result<()> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
}

impl Drop for PipelineTestRunner {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_test_config_creation() {
        let config = PipelineTestConfig::simple_test();
        assert_eq!(config.test_name, "simple_udon_behaviour");
        assert!(!config.rust_source.is_empty());
        assert!(!config.expected_csharp_patterns.is_empty());
        assert!(config.should_optimize);
    }
    
    #[test]
    fn test_performance_metrics() {
        let mut metrics = PipelinePerformanceMetrics::new("test".to_string());
        metrics.wasm_size_before_optimization = 1000;
        metrics.wasm_size_after_optimization = 800;
        metrics.rust_source_size = 500;
        metrics.total_pipeline_time = std::time::Duration::from_secs(2);
        
        assert_eq!(metrics.optimization_effectiveness(), 20.0);
        assert_eq!(metrics.compilation_speed(), 250.0);
        
        let summary = metrics.summary();
        assert!(summary.contains("test"));
        assert!(summary.contains("20.0%"));
    }
    
    #[test]
    fn test_pipeline_runner_creation() {
        let runner = PipelineTestRunner::new();
        assert!(runner.is_ok());
        
        let runner = runner.unwrap();
        assert!(runner.temp_dir.exists() || runner.temp_dir.parent().map_or(false, |p| p.exists()));
        assert_eq!(runner.performance_metrics.len(), 0);
    }
    
    #[test]
    fn test_csharp_validation() {
        let runner = PipelineTestRunner::new().unwrap();
        
        let files = vec![
            wasm2usharp_enhanced::file_generator::CSharpFile {
                name: "Test.cs".to_string(),
                content: "public class Test : UdonSharpBehaviour { void Start() {} }".to_string(),
            }
        ];
        
        let patterns = vec![
            "public class Test".to_string(),
            "UdonSharpBehaviour".to_string(),
            "void Start()".to_string(),
        ];
        
        let result = runner.validate_generated_csharp(&files, &patterns);
        assert!(result.is_ok());
        
        // Test with missing pattern
        let missing_patterns = vec!["missing_pattern".to_string()];
        let result = runner.validate_generated_csharp(&files, &missing_patterns);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_networking_config() {
        let config = PipelineTestConfig::networking_test();
        assert_eq!(config.test_name, "networking_behaviour");
        assert!(config.rust_source.contains("on_player_joined"));
        assert!(config.rust_source.contains("on_player_left"));
        assert!(config.expected_csharp_patterns.iter().any(|p| p.contains("OnPlayerJoined")));
        assert!(config.expected_csharp_patterns.iter().any(|p| p.contains("OnPlayerLeft")));
    }
    
    #[test]
    fn test_complex_oop_config() {
        let config = PipelineTestConfig::complex_oop_test();
        assert_eq!(config.test_name, "complex_oop_behaviour");
        assert!(config.rust_source.contains("trait Interactable"));
        assert!(config.rust_source.contains("impl Interactable"));
        assert!(config.expected_csharp_patterns.iter().any(|p| p.contains("Interact(")));
        assert!(config.expected_csharp_patterns.iter().any(|p| p.contains("CanInteract(")));
    }
}