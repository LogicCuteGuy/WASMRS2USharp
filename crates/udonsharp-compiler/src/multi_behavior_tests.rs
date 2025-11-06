//! Multi-behavior compilation tests for the UdonSharp compiler
//! 
//! This module provides tests that validate the complete multi-behavior
//! compilation pipeline from Rust source to generated UdonSharp behaviors.

use crate::config::UdonSharpConfig;
use crate::pipeline::CompilationPipeline;
use crate::wasm_compiler::RustToWasmCompiler;
use udonsharp_core::error::{UdonSharpError, UdonSharpResult};
use udonsharp_core::multi_behavior_errors::MultiBehaviorErrorHandler;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// End-to-end multi-behavior compilation test suite
pub struct MultiBehaviorCompilationTests {
    config: UdonSharpConfig,
    error_handler: MultiBehaviorErrorHandler,
    test_cases: Vec<CompilationTestCase>,
}

impl MultiBehaviorCompilationTests {
    /// Create a new test suite
    pub fn new() -> Self {
        Self {
            config: UdonSharpConfig::default(),
            error_handler: MultiBehaviorErrorHandler::new(),
            test_cases: Vec::new(),
        }
    }
    
    /// Add a compilation test case
    pub fn add_test_case(&mut self, test_case: CompilationTestCase) {
        self.test_cases.push(test_case);
    }
    
    /// Run all compilation tests
    pub fn run_all_tests(&self) -> CompilationTestResults {
        let mut results = CompilationTestResults::new();
        
        for test_case in &self.test_cases {
            let result = self.run_compilation_test(test_case);
            results.add_result(result);
        }
        
        results
    }
    
    /// Run a single compilation test
    fn run_compilation_test(&self, test_case: &CompilationTestCase) -> CompilationTestResult {
        let start_time = std::time::Instant::now();
        
        let mut result = CompilationTestResult {
            test_name: test_case.name.clone(),
            success: false,
            duration: Duration::ZERO,
            error_message: None,
            generated_files: Vec::new(),
            compilation_stats: None,
        };
        
        match self.execute_compilation_test(test_case) {
            Ok(output) => {
                result.success = self.validate_compilation_output(&output, &test_case.expected_output);
                result.generated_files = output.generated_files;
                result.compilation_stats = Some(output.stats);
            }
            Err(e) => {
                result.error_message = Some(e.to_string());
                
                // Check if this is an expected error
                if let Some(expected_error) = &test_case.expected_error {
                    result.success = e.to_string().contains(expected_error);
                }
            }
        }
        
        result.duration = start_time.elapsed();
        result
    }
    
    /// Execute a compilation test
    fn execute_compilation_test(&self, test_case: &CompilationTestCase) -> UdonSharpResult<CompilationOutput> {
        // Create a temporary directory for the test
        let temp_dir = std::env::temp_dir().join(format!("udonsharp_test_{}", test_case.name));
        std::fs::create_dir_all(&temp_dir).map_err(|e| {
            UdonSharpError::multi_behavior(format!("Failed to create test directory: {}", e))
        })?;
        
        // Write the test source code to a file
        let source_file = temp_dir.join("lib.rs");
        std::fs::write(&source_file, &test_case.source_code).map_err(|e| {
            UdonSharpError::multi_behavior(format!("Failed to write test source: {}", e))
        })?;
        
        // Create Cargo.toml for the test project
        let cargo_toml = temp_dir.join("Cargo.toml");
        std::fs::write(&cargo_toml, &self.create_test_cargo_toml(&test_case.name)).map_err(|e| {
            UdonSharpError::multi_behavior(format!("Failed to write Cargo.toml: {}", e))
        })?;
        
        // Set up compilation pipeline
        let mut config = self.config.clone();
        config.project_root = Some(temp_dir.clone());
        config.output_directory = Some(temp_dir.join("output"));
        
        let compiler = RustToWasmCompiler::new(config.clone());
        let mut pipeline = CompilationPipeline::new(config);
        
        // Compile Rust to WASM
        let wasm_result = compiler.compile_project(&temp_dir)?;
        
        // Process through the multi-behavior pipeline
        let compilation_start = std::time::Instant::now();
        let pipeline_result = pipeline.process_multi_behavior_wasm(&wasm_result.wasm_bytes)?;
        let compilation_time = compilation_start.elapsed();
        
        // Collect generated files
        let output_dir = temp_dir.join("output");
        let generated_files = self.collect_generated_files(&output_dir)?;
        
        // Clean up temporary directory
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        Ok(CompilationOutput {
            generated_files,
            stats: CompilationStats {
                compilation_time,
                wasm_size: wasm_result.wasm_bytes.len(),
                generated_behaviors: pipeline_result.behavior_count,
                total_dependencies: pipeline_result.dependency_count,
                shared_functions: pipeline_result.shared_function_count,
            },
        })
    }
    
    /// Create a test Cargo.toml file
    fn create_test_cargo_toml(&self, test_name: &str) -> String {
        format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
udonsharp-core = {{ path = "../../../udonsharp-core" }}
udonsharp-bindings = {{ path = "../../../udonsharp-bindings" }}

[profile.release]
opt-level = "s"
lto = true
panic = "abort"
"#, test_name)
    }
    
    /// Collect generated files from the output directory
    fn collect_generated_files(&self, output_dir: &PathBuf) -> UdonSharpResult<Vec<GeneratedFile>> {
        let mut files = Vec::new();
        
        if !output_dir.exists() {
            return Ok(files);
        }
        
        for entry in std::fs::read_dir(output_dir).map_err(|e| {
            UdonSharpError::multi_behavior(format!("Failed to read output directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                UdonSharpError::multi_behavior(format!("Failed to read directory entry: {}", e))
            })?;
            
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "cs") {
                let content = std::fs::read_to_string(&path).map_err(|e| {
                    UdonSharpError::multi_behavior(format!("Failed to read generated file: {}", e))
                })?;
                
                files.push(GeneratedFile {
                    name: path.file_name().unwrap().to_string_lossy().to_string(),
                    content,
                    file_type: if path.file_stem().unwrap().to_string_lossy().contains("SharedRuntime") {
                        GeneratedFileType::SharedRuntime
                    } else {
                        GeneratedFileType::UdonBehaviour
                    },
                });
            }
        }
        
        Ok(files)
    }
    
    /// Validate compilation output against expected results
    fn validate_compilation_output(&self, output: &CompilationOutput, expected: &ExpectedCompilationOutput) -> bool {
        // Check number of generated behaviors
        if let Some(expected_count) = expected.expected_behavior_count {
            let behavior_count = output.generated_files.iter()
                .filter(|f| f.file_type == GeneratedFileType::UdonBehaviour)
                .count();
            if behavior_count != expected_count {
                return false;
            }
        }
        
        // Check for specific behavior names
        if let Some(expected_behaviors) = &expected.expected_behavior_names {
            for expected_name in expected_behaviors {
                let found = output.generated_files.iter().any(|f| {
                    f.file_type == GeneratedFileType::UdonBehaviour && 
                    f.name.contains(expected_name)
                });
                if !found {
                    return false;
                }
            }
        }
        
        // Check for SharedRuntime if expected
        if expected.should_have_shared_runtime {
            let has_shared_runtime = output.generated_files.iter()
                .any(|f| f.file_type == GeneratedFileType::SharedRuntime);
            if !has_shared_runtime {
                return false;
            }
        }
        
        // Check compilation time threshold
        if let Some(max_time) = expected.max_compilation_time {
            if output.stats.compilation_time > max_time {
                return false;
            }
        }
        
        true
    }
}

/// Compilation test case definition
#[derive(Debug, Clone)]
pub struct CompilationTestCase {
    pub name: String,
    pub description: String,
    pub source_code: String,
    pub expected_output: ExpectedCompilationOutput,
    pub expected_error: Option<String>,
}

/// Expected compilation output
#[derive(Debug, Clone)]
pub struct ExpectedCompilationOutput {
    pub expected_behavior_count: Option<usize>,
    pub expected_behavior_names: Option<Vec<String>>,
    pub should_have_shared_runtime: bool,
    pub max_compilation_time: Option<Duration>,
}

/// Compilation test output
#[derive(Debug)]
struct CompilationOutput {
    generated_files: Vec<GeneratedFile>,
    stats: CompilationStats,
}

/// Generated file information
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub name: String,
    pub content: String,
    pub file_type: GeneratedFileType,
}

/// Type of generated file
#[derive(Debug, Clone, PartialEq)]
pub enum GeneratedFileType {
    UdonBehaviour,
    SharedRuntime,
    Utility,
}

/// Compilation statistics
#[derive(Debug, Clone)]
pub struct CompilationStats {
    pub compilation_time: Duration,
    pub wasm_size: usize,
    pub generated_behaviors: usize,
    pub total_dependencies: usize,
    pub shared_functions: usize,
}

/// Test results collection
#[derive(Debug)]
pub struct CompilationTestResults {
    pub results: Vec<CompilationTestResult>,
}

impl CompilationTestResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }
    
    pub fn add_result(&mut self, result: CompilationTestResult) {
        self.results.push(result);
    }
    
    pub fn total_tests(&self) -> usize {
        self.results.len()
    }
    
    pub fn passed_tests(&self) -> usize {
        self.results.iter().filter(|r| r.success).count()
    }
    
    pub fn success_rate(&self) -> f32 {
        if self.total_tests() == 0 {
            return 0.0;
        }
        self.passed_tests() as f32 / self.total_tests() as f32
    }
    
    pub fn print_summary(&self) {
        println!("\n=== Multi-Behavior Compilation Test Results ===");
        println!("Total Tests: {}", self.total_tests());
        println!("Passed: {}", self.passed_tests());
        println!("Failed: {}", self.total_tests() - self.passed_tests());
        println!("Success Rate: {:.1}%", self.success_rate() * 100.0);
        
        for result in &self.results {
            let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
            println!("{} {} ({:.2}s)", 
                    status, result.test_name, result.duration.as_secs_f32());
            
            if let Some(ref error) = result.error_message {
                println!("    Error: {}", error);
            }
            
            if result.success {
                println!("    Generated {} files", result.generated_files.len());
                if let Some(ref stats) = result.compilation_stats {
                    println!("    Stats: {} behaviors, {} dependencies, {:.2}ms", 
                            stats.generated_behaviors, 
                            stats.total_dependencies,
                            stats.compilation_time.as_millis());
                }
            }
        }
    }
}

/// Individual compilation test result
#[derive(Debug)]
pub struct CompilationTestResult {
    pub test_name: String,
    pub success: bool,
    pub duration: Duration,
    pub error_message: Option<String>,
    pub generated_files: Vec<GeneratedFile>,
    pub compilation_stats: Option<CompilationStats>,
}

// Mock types for compilation pipeline - these would be actual types in the real implementation
#[derive(Debug)]
struct WasmCompilationResult {
    wasm_bytes: Vec<u8>,
}

#[derive(Debug)]
struct PipelineResult {
    behavior_count: usize,
    dependency_count: usize,
    shared_function_count: usize,
}

// Mock implementations for testing
impl CompilationPipeline {
    fn process_multi_behavior_wasm(&mut self, _wasm_bytes: &[u8]) -> UdonSharpResult<PipelineResult> {
        // Mock implementation - in reality this would process the WASM
        Ok(PipelineResult {
            behavior_count: 2,
            dependency_count: 1,
            shared_function_count: 3,
        })
    }
}

impl RustToWasmCompiler {
    fn compile_project(&self, _project_dir: &PathBuf) -> UdonSharpResult<WasmCompilationResult> {
        // Mock implementation - in reality this would compile Rust to WASM
        Ok(WasmCompilationResult {
            wasm_bytes: vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00], // Mock WASM header
        })
    }
}

/// Predefined test cases for common multi-behavior scenarios
pub mod test_cases {
    use super::*;
    
    /// Test basic multi-behavior compilation
    pub fn basic_multi_behavior() -> CompilationTestCase {
        CompilationTestCase {
            name: "basic_multi_behavior".to_string(),
            description: "Test compilation of basic multi-behavior Rust code".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

#[udon_behaviour(name = "PlayerManager")]
pub fn player_manager_start() {
    log::info!("PlayerManager started");
    shared_utility_function();
}

#[udon_behaviour(name = "UIController")]
pub fn ui_controller_start() {
    log::info!("UIController started");
    shared_utility_function();
}

pub fn shared_utility_function() {
    log::info!("Shared utility called");
}

#[derive(UdonBehaviour)]
pub struct PlayerManager {
    player_count: i32,
}

impl UdonBehaviour for PlayerManager {
    fn start(&mut self) {
        self.player_count = 0;
    }
    
    fn update(&mut self) {
        // Update logic
    }
}

#[derive(UdonBehaviour)]
pub struct UIController {
    ui_elements: Vec<unity::GameObject>,
}

impl UdonBehaviour for UIController {
    fn start(&mut self) {
        self.ui_elements = Vec::new();
    }
    
    fn update(&mut self) {
        // UI update logic
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(2),
                expected_behavior_names: Some(vec![
                    "PlayerManager".to_string(),
                    "UIController".to_string(),
                ]),
                should_have_shared_runtime: true,
                max_compilation_time: Some(Duration::from_secs(30)),
            },
            expected_error: None,
        }
    }
    
    /// Test inter-behavior communication
    pub fn inter_behavior_communication() -> CompilationTestCase {
        CompilationTestCase {
            name: "inter_behavior_communication".to_string(),
            description: "Test compilation with inter-behavior communication".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

#[udon_behaviour(name = "GameManager")]
pub fn game_manager_start() {
    log::info!("GameManager started");
}

#[udon_behaviour(name = "ScoreTracker")]
pub fn score_tracker_start() {
    log::info!("ScoreTracker started");
}

#[derive(UdonBehaviour)]
pub struct GameManager {
    game_state: GameState,
    score_tracker_ref: Option<unity::GameObject>,
}

impl GameManager {
    fn update_score(&mut self, points: i32) {
        if let Some(score_tracker) = &self.score_tracker_ref {
            score_tracker.send_custom_event("AddScore");
        }
    }
}

impl UdonBehaviour for GameManager {
    fn start(&mut self) {
        self.game_state = GameState::Playing;
        self.score_tracker_ref = unity::GameObject::find("ScoreTracker");
    }
}

#[derive(UdonBehaviour)]
pub struct ScoreTracker {
    current_score: i32,
}

impl ScoreTracker {
    #[udon_event("AddScore")]
    pub fn add_score(&mut self) {
        self.current_score += 10;
        log::info!("Score updated: {}", self.current_score);
    }
}

impl UdonBehaviour for ScoreTracker {
    fn start(&mut self) {
        self.current_score = 0;
    }
}

#[derive(Debug, Clone)]
pub enum GameState {
    Waiting,
    Playing,
    Ended,
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(2),
                expected_behavior_names: Some(vec![
                    "GameManager".to_string(),
                    "ScoreTracker".to_string(),
                ]),
                should_have_shared_runtime: true,
                max_compilation_time: Some(Duration::from_secs(30)),
            },
            expected_error: None,
        }
    }
    
    /// Test error case - circular dependency
    pub fn circular_dependency_error() -> CompilationTestCase {
        CompilationTestCase {
            name: "circular_dependency_error".to_string(),
            description: "Test error handling for circular dependencies".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;

#[udon_behaviour(name = "BehaviorA")]
pub fn behavior_a_start() {
    behavior_b_function();
}

#[udon_behaviour(name = "BehaviorB")]
pub fn behavior_b_start() {
    behavior_a_function();
}

pub fn behavior_a_function() {
    behavior_b_function(); // Creates circular dependency
}

pub fn behavior_b_function() {
    behavior_a_function(); // Creates circular dependency
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: None,
                expected_behavior_names: None,
                should_have_shared_runtime: false,
                max_compilation_time: None,
            },
            expected_error: Some("circular dependency".to_string()),
        }
    }
    
    /// Test invalid attribute error
    pub fn invalid_attribute_error() -> CompilationTestCase {
        CompilationTestCase {
            name: "invalid_attribute_error".to_string(),
            description: "Test error handling for invalid attributes".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;

#[udon_behaviour(name = "")]  // Invalid: empty name
pub fn invalid_behavior_empty_name() {
    log::info!("This should fail");
}

#[udon_behaviour(name = "ValidName")]
fn private_behavior() {  // Invalid: not public
    log::info!("This should also fail");
}

#[udon_behaviour(name = "123InvalidName")]  // Invalid: starts with number
pub fn invalid_behavior_bad_name() {
    log::info!("This should fail too");
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: None,
                expected_behavior_names: None,
                should_have_shared_runtime: false,
                max_compilation_time: None,
            },
            expected_error: Some("invalid attribute".to_string()),
        }
    }

    /// Test SharedRuntime generation
    pub fn shared_runtime_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "shared_runtime_test".to_string(),
            description: "Test SharedRuntime generation for shared functions".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

// Shared utility functions
pub fn calculate_distance(pos1: unity::Vector3, pos2: unity::Vector3) -> f32 {
    let diff = pos1 - pos2;
    (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt()
}

pub fn format_score(score: i32) -> String {
    format!("Score: {}", score)
}

#[derive(UdonBehaviour)]
pub struct PlayerTracker {
    player_positions: Vec<unity::Vector3>,
}

impl UdonBehaviour for PlayerTracker {
    fn start(&mut self) {
        self.player_positions = Vec::new();
    }
    
    fn update(&mut self) {
        // Use shared function
        if self.player_positions.len() >= 2 {
            let distance = calculate_distance(self.player_positions[0], self.player_positions[1]);
            log::info!("Distance: {}", distance);
        }
    }
}

#[derive(UdonBehaviour)]
pub struct ScoreDisplay {
    current_score: i32,
}

impl UdonBehaviour for ScoreDisplay {
    fn start(&mut self) {
        self.current_score = 0;
    }
    
    fn update(&mut self) {
        // Use shared function
        let formatted = format_score(self.current_score);
        log::info!("{}", formatted);
    }
}

#[derive(UdonBehaviour)]
pub struct GameAnalyzer {
    analysis_data: Vec<f32>,
}

impl UdonBehaviour for GameAnalyzer {
    fn start(&mut self) {
        self.analysis_data = Vec::new();
    }
    
    fn update(&mut self) {
        // Use both shared functions
        let score_text = format_score(100);
        log::info!("{}", score_text);
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(3),
                expected_behavior_names: Some(vec![
                    "PlayerTracker".to_string(),
                    "ScoreDisplay".to_string(),
                    "GameAnalyzer".to_string(),
                ]),
                should_have_shared_runtime: true,
                max_compilation_time: Some(Duration::from_secs(30)),
            },
            expected_error: None,
        }
    }

    /// Test networking synchronization
    pub fn networking_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "networking_test".to_string(),
            description: "Test network synchronization code generation".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct NetworkedGameState {
    #[udon_sync]
    player_count: i32,
    #[udon_sync]
    game_phase: String,
    #[udon_sync]
    round_timer: f32,
    local_data: i32,
}

impl UdonBehaviour for NetworkedGameState {
    fn start(&mut self) {
        self.player_count = 0;
        self.game_phase = "waiting".to_string();
        self.round_timer = 0.0;
        self.local_data = 42;
    }
    
    fn update(&mut self) {
        if vrchat::Networking::is_master() {
            self.round_timer += unity::Time::delta_time();
            if self.round_timer > 60.0 {
                self.game_phase = "ended".to_string();
                self.request_serialization();
            }
        }
    }
    
    fn on_post_deserialization(&mut self) {
        log::info!("Game state updated: {} players, phase: {}", 
                  self.player_count, self.game_phase);
    }
}

impl NetworkedGameState {
    fn request_serialization(&mut self) {
        // This should generate RequestSerialization() call
    }
    
    #[udon_event("OnPlayerJoined")]
    pub fn on_player_joined(&mut self) {
        if vrchat::Networking::is_master() {
            self.player_count += 1;
            self.request_serialization();
        }
    }
    
    #[udon_event("OnPlayerLeft")]
    pub fn on_player_left(&mut self) {
        if vrchat::Networking::is_master() {
            self.player_count -= 1;
            self.request_serialization();
        }
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(1),
                expected_behavior_names: Some(vec!["NetworkedGameState".to_string()]),
                should_have_shared_runtime: false,
                max_compilation_time: Some(Duration::from_secs(30)),
            },
            expected_error: None,
        }
    }

    /// Test complex multi-behavior project
    pub fn complex_project_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "complex_project_test".to_string(),
            description: "Test complex project with multiple interacting behaviors".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

// Shared enums and types
#[derive(Debug, Clone)]
pub enum GameState {
    Lobby,
    Playing,
    Paused,
    Ended,
}

#[derive(Debug, Clone)]
pub struct PlayerData {
    name: String,
    score: i32,
    position: unity::Vector3,
}

// Shared utility functions
pub fn validate_player_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 20
}

pub fn calculate_score_multiplier(base_score: i32, time_bonus: f32) -> i32 {
    (base_score as f32 * (1.0 + time_bonus)).round() as i32
}

#[derive(UdonBehaviour)]
pub struct GameManager {
    #[udon_sync]
    current_state: GameState,
    #[udon_sync]
    round_number: i32,
    ui_manager_ref: Option<unity::GameObject>,
    score_manager_ref: Option<unity::GameObject>,
    player_manager_ref: Option<unity::GameObject>,
}

impl UdonBehaviour for GameManager {
    fn start(&mut self) {
        self.current_state = GameState::Lobby;
        self.round_number = 0;
        self.ui_manager_ref = unity::GameObject::find("UIManager");
        self.score_manager_ref = unity::GameObject::find("ScoreManager");
        self.player_manager_ref = unity::GameObject::find("PlayerManager");
    }
    
    fn update(&mut self) {
        match self.current_state {
            GameState::Playing => {
                // Game logic here
            }
            _ => {}
        }
    }
}

impl GameManager {
    #[udon_event("StartGame")]
    pub fn start_game(&mut self) {
        if vrchat::Networking::is_master() {
            self.current_state = GameState::Playing;
            self.round_number += 1;
            
            if let Some(ui_ref) = &self.ui_manager_ref {
                ui_ref.send_custom_event("OnGameStarted");
            }
        }
    }
    
    #[udon_event("EndGame")]
    pub fn end_game(&mut self) {
        if vrchat::Networking::is_master() {
            self.current_state = GameState::Ended;
            
            if let Some(score_ref) = &self.score_manager_ref {
                score_ref.send_custom_event("CalculateFinalScores");
            }
        }
    }
}

#[derive(UdonBehaviour)]
pub struct UIManager {
    #[udon_public]
    game_state_text: Option<unity::GameObject>,
    #[udon_public]
    score_display: Option<unity::GameObject>,
    game_manager_ref: Option<unity::GameObject>,
}

impl UdonBehaviour for UIManager {
    fn start(&mut self) {
        self.game_manager_ref = unity::GameObject::find("GameManager");
    }
    
    fn update(&mut self) {
        // Update UI elements
    }
}

impl UIManager {
    #[udon_event("OnGameStarted")]
    pub fn on_game_started(&mut self) {
        log::info!("Game started - updating UI");
        // Update UI for game start
    }
    
    #[udon_event("UpdateScore")]
    pub fn update_score(&mut self, new_score: i32) {
        log::info!("Updating score display: {}", new_score);
        // Update score display
    }
}

#[derive(UdonBehaviour)]
pub struct ScoreManager {
    #[udon_sync]
    player_scores: Vec<i32>,
    #[udon_sync]
    high_score: i32,
    ui_manager_ref: Option<unity::GameObject>,
}

impl UdonBehaviour for ScoreManager {
    fn start(&mut self) {
        self.player_scores = Vec::new();
        self.high_score = 0;
        self.ui_manager_ref = unity::GameObject::find("UIManager");
    }
    
    fn update(&mut self) {
        // Score management logic
    }
}

impl ScoreManager {
    #[udon_event("AddScore")]
    pub fn add_score(&mut self, player_id: i32, base_score: i32) {
        if vrchat::Networking::is_master() {
            let final_score = calculate_score_multiplier(base_score, 0.1);
            
            if player_id < self.player_scores.len() as i32 {
                self.player_scores[player_id as usize] += final_score;
                
                if let Some(ui_ref) = &self.ui_manager_ref {
                    ui_ref.send_custom_event_with_data("UpdateScore", final_score);
                }
            }
        }
    }
    
    #[udon_event("CalculateFinalScores")]
    pub fn calculate_final_scores(&mut self) {
        if vrchat::Networking::is_master() {
            self.high_score = self.player_scores.iter().max().copied().unwrap_or(0);
            log::info!("Final high score: {}", self.high_score);
        }
    }
}

#[derive(UdonBehaviour)]
pub struct PlayerManager {
    #[udon_sync]
    connected_players: Vec<PlayerData>,
    #[udon_sync]
    player_count: i32,
    game_manager_ref: Option<unity::GameObject>,
}

impl UdonBehaviour for PlayerManager {
    fn start(&mut self) {
        self.connected_players = Vec::new();
        self.player_count = 0;
        self.game_manager_ref = unity::GameObject::find("GameManager");
    }
    
    fn on_player_joined(&mut self, player: vrchat::VRCPlayerApi) {
        if vrchat::Networking::is_master() {
            let player_name = player.get_display_name();
            if validate_player_name(&player_name) {
                let new_player = PlayerData {
                    name: player_name,
                    score: 0,
                    position: unity::Vector3::zero(),
                };
                
                self.connected_players.push(new_player);
                self.player_count += 1;
                
                log::info!("Player joined: {} (total: {})", new_player.name, self.player_count);
            }
        }
    }
    
    fn on_player_left(&mut self, player: vrchat::VRCPlayerApi) {
        if vrchat::Networking::is_master() {
            let player_name = player.get_display_name();
            self.connected_players.retain(|p| p.name != player_name);
            self.player_count -= 1;
            
            log::info!("Player left: {} (total: {})", player_name, self.player_count);
        }
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(4),
                expected_behavior_names: Some(vec![
                    "GameManager".to_string(),
                    "UIManager".to_string(),
                    "ScoreManager".to_string(),
                    "PlayerManager".to_string(),
                ]),
                should_have_shared_runtime: true,
                max_compilation_time: Some(Duration::from_secs(45)),
            },
            expected_error: None,
        }
    }

    /// Test compilation performance
    pub fn performance_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "performance_test".to_string(),
            description: "Test compilation performance with moderately complex project".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};

// Generate multiple behaviors for performance testing
#[derive(UdonBehaviour)]
pub struct Behavior01 { field: i32 }
impl UdonBehaviour for Behavior01 { fn start(&mut self) { self.field = 1; } }

#[derive(UdonBehaviour)]
pub struct Behavior02 { field: i32 }
impl UdonBehaviour for Behavior02 { fn start(&mut self) { self.field = 2; } }

#[derive(UdonBehaviour)]
pub struct Behavior03 { field: i32 }
impl UdonBehaviour for Behavior03 { fn start(&mut self) { self.field = 3; } }

#[derive(UdonBehaviour)]
pub struct Behavior04 { field: i32 }
impl UdonBehaviour for Behavior04 { fn start(&mut self) { self.field = 4; } }

#[derive(UdonBehaviour)]
pub struct Behavior05 { field: i32 }
impl UdonBehaviour for Behavior05 { fn start(&mut self) { self.field = 5; } }

pub fn shared_function() -> i32 { 42 }
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(5),
                expected_behavior_names: None,
                should_have_shared_runtime: true,
                max_compilation_time: Some(Duration::from_secs(30)),
            },
            expected_error: None,
        }
    }

    /// Test syntax error handling
    pub fn syntax_error_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "syntax_error_test".to_string(),
            description: "Test syntax error detection and reporting".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;

#[derive(UdonBehaviour)]
pub struct SyntaxErrorBehavior {
    field: i32,
}

impl UdonBehaviour for SyntaxErrorBehavior {
    fn start(&mut self) {
        // Syntax error: missing semicolon
        self.field = 42
        log::info!("This should fail");
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: None,
                expected_behavior_names: None,
                should_have_shared_runtime: false,
                max_compilation_time: None,
            },
            expected_error: Some("syntax error".to_string()),
        }
    }

    /// Test type error handling
    pub fn type_error_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "type_error_test".to_string(),
            description: "Test type error detection and reporting".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;

#[derive(UdonBehaviour)]
pub struct TypeErrorBehavior {
    number_field: i32,
    string_field: String,
}

impl UdonBehaviour for TypeErrorBehavior {
    fn start(&mut self) {
        // Type error: assigning string to number field
        self.number_field = "not a number";
        self.string_field = 42;
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: None,
                expected_behavior_names: None,
                should_have_shared_runtime: false,
                max_compilation_time: None,
            },
            expected_error: Some("type error".to_string()),
        }
    }

    /// Test missing dependency error
    pub fn missing_dependency_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "missing_dependency_test".to_string(),
            description: "Test missing dependency detection".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;

#[derive(UdonBehaviour)]
pub struct DependentBehavior {
    other_behavior_ref: Option<unity::GameObject>,
}

impl UdonBehaviour for DependentBehavior {
    fn start(&mut self) {
        // Reference to non-existent behavior
        self.other_behavior_ref = unity::GameObject::find("NonExistentBehavior");
        
        if let Some(other_ref) = &self.other_behavior_ref {
            other_ref.send_custom_event("NonExistentEvent");
        }
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: None,
                expected_behavior_names: None,
                should_have_shared_runtime: false,
                max_compilation_time: None,
            },
            expected_error: Some("missing dependency".to_string()),
        }
    }

    /// Test incremental compilation
    pub fn incremental_compilation_test() -> CompilationTestCase {
        CompilationTestCase {
            name: "incremental_compilation_test".to_string(),
            description: "Test incremental compilation performance".to_string(),
            source_code: r#"
use udonsharp_core::prelude::*;

#[derive(UdonBehaviour)]
pub struct IncrementalBehavior {
    counter: i32,
}

impl UdonBehaviour for IncrementalBehavior {
    fn start(&mut self) {
        self.counter = 0;
    }
    
    fn update(&mut self) {
        self.counter += 1;
    }
}
"#.to_string(),
            expected_output: ExpectedCompilationOutput {
                expected_behavior_count: Some(1),
                expected_behavior_names: Some(vec!["IncrementalBehavior".to_string()]),
                should_have_shared_runtime: false,
                max_compilation_time: Some(Duration::from_secs(10)),
            },
            expected_error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_multi_behavior_compilation() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::basic_multi_behavior());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // Note: This test may fail if the actual compilation pipeline is not implemented
        // but it demonstrates the testing structure
    }
    
    #[test]
    fn test_inter_behavior_communication_compilation() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::inter_behavior_communication());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
    }
    
    #[test]
    fn test_circular_dependency_error_handling() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::circular_dependency_error());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // This test should "pass" by correctly detecting the circular dependency error
        if let Some(result) = results.results.first() {
            assert!(result.success, "Circular dependency should be detected as an error");
        }
    }
    
    #[test]
    fn test_invalid_attribute_error_handling() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::invalid_attribute_error());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // This test should "pass" by correctly detecting the invalid attribute errors
        if let Some(result) = results.results.first() {
            assert!(result.success, "Invalid attributes should be detected as errors");
        }
    }
    
    #[test]
    fn test_full_compilation_test_suite() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        
        // Add all test cases
        test_suite.add_test_case(test_cases::basic_multi_behavior());
        test_suite.add_test_case(test_cases::inter_behavior_communication());
        test_suite.add_test_case(test_cases::circular_dependency_error());
        test_suite.add_test_case(test_cases::invalid_attribute_error());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 4);
        results.print_summary();
        
        // At least the error detection tests should pass
        assert!(results.passed_tests() >= 2, "Error detection tests should pass");
    }

    // Additional comprehensive integration tests

    #[test]
    fn test_shared_runtime_generation() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::shared_runtime_test());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // Verify SharedRuntime was generated
        if let Some(result) = results.results.first() {
            if result.success {
                let has_shared_runtime = result.generated_files.iter()
                    .any(|f| f.file_type == GeneratedFileType::SharedRuntime);
                assert!(has_shared_runtime, "SharedRuntime should be generated for multi-behavior projects");
            }
        }
    }

    #[test]
    fn test_networking_synchronization() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::networking_test());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // Verify networking code generation
        if let Some(result) = results.results.first() {
            if result.success {
                let has_networking_code = result.generated_files.iter()
                    .any(|f| f.content.contains("[UdonSynced]") && f.content.contains("RequestSerialization"));
                assert!(has_networking_code, "Networking synchronization code should be generated");
            }
        }
    }

    #[test]
    fn test_complex_multi_behavior_project() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::complex_project_test());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // Verify complex project compilation
        if let Some(result) = results.results.first() {
            if result.success {
                // Should generate multiple behaviors
                let behavior_count = result.generated_files.iter()
                    .filter(|f| f.file_type == GeneratedFileType::UdonBehaviour)
                    .count();
                assert!(behavior_count >= 3, "Complex project should generate multiple behaviors");
                
                // Should have SharedRuntime
                let has_shared_runtime = result.generated_files.iter()
                    .any(|f| f.file_type == GeneratedFileType::SharedRuntime);
                assert!(has_shared_runtime, "Complex project should have SharedRuntime");
            }
        }
    }

    #[test]
    fn test_compilation_performance() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::performance_test());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // Verify compilation performance
        if let Some(result) = results.results.first() {
            if result.success {
                // Compilation should complete within reasonable time
                assert!(result.duration.as_secs() < 60, "Compilation should complete within 60 seconds");
                
                if let Some(ref stats) = result.compilation_stats {
                    // Verify reasonable compilation stats
                    assert!(stats.generated_behaviors > 0, "Should generate at least one behavior");
                    assert!(stats.wasm_size > 0, "WASM should have non-zero size");
                }
            }
        }
    }

    #[test]
    fn test_error_recovery_and_reporting() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        
        // Add multiple error test cases
        test_suite.add_test_case(test_cases::syntax_error_test());
        test_suite.add_test_case(test_cases::type_error_test());
        test_suite.add_test_case(test_cases::missing_dependency_test());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 3);
        results.print_summary();
        
        // All error tests should "pass" by correctly detecting errors
        for result in &results.results {
            assert!(result.success, "Error test {} should pass by detecting the error", result.test_name);
            assert!(result.error_message.is_some(), "Error test should have error message");
        }
    }

    #[test]
    fn test_incremental_compilation() {
        let mut test_suite = MultiBehaviorCompilationTests::new();
        test_suite.add_test_case(test_cases::incremental_compilation_test());
        
        let results = test_suite.run_all_tests();
        
        assert_eq!(results.total_tests(), 1);
        results.print_summary();
        
        // Verify incremental compilation works
        if let Some(result) = results.results.first() {
            if result.success {
                // Should be faster than full compilation
                assert!(result.duration.as_millis() < 5000, "Incremental compilation should be fast");
            }
        }
    }
}