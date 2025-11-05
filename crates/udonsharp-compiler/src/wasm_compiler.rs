//! Rust to WASM compilation wrapper
//! 
//! This module provides a wrapper around the Rust compiler for generating
//! UdonSharp-compatible WASM output with comprehensive error handling and diagnostics.

use crate::config::{UdonSharpConfig, WasmTargetConfig};
use udonsharp_core::{UdonSharpResult, UdonSharpError, error::Diagnostic};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::env;
use std::fs;
use std::collections::HashMap;

/// Compilation result with detailed information
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// The compiled WASM bytes
    pub wasm_bytes: Vec<u8>,
    /// Path to the generated WASM file
    pub wasm_path: PathBuf,
    /// Compilation diagnostics and warnings
    pub diagnostics: Vec<Diagnostic>,
    /// Compilation time in milliseconds
    pub compilation_time_ms: u64,
}

/// Rust to WASM compiler wrapper with UdonSharp-specific optimizations
pub struct RustToWasmCompiler {
    config: UdonSharpConfig,
    wasm_config: WasmTargetConfig,
    rustc_path: Option<PathBuf>,
    cargo_path: Option<PathBuf>,
}

impl RustToWasmCompiler {
    /// Create a new Rust to WASM compiler with default configuration
    pub fn new(config: UdonSharpConfig) -> Self {
        Self {
            config,
            wasm_config: WasmTargetConfig::default(),
            rustc_path: None,
            cargo_path: None,
        }
    }
    
    /// Create a new compiler with custom WASM target configuration
    pub fn with_wasm_config(config: UdonSharpConfig, wasm_config: WasmTargetConfig) -> Self {
        Self {
            config,
            wasm_config,
            rustc_path: None,
            cargo_path: None,
        }
    }
    
    /// Set custom rustc path
    pub fn with_rustc_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.rustc_path = Some(path.into());
        self
    }
    
    /// Set custom cargo path
    pub fn with_cargo_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.cargo_path = Some(path.into());
        self
    }
    
    /// Compile Rust project to WASM with comprehensive error handling
    pub fn compile<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<CompilationResult> {
        let start_time = std::time::Instant::now();
        let project_path = project_path.as_ref();
        
        log::info!("Starting Rust to WASM compilation for project: {:?}", project_path);
        
        // Validate project structure
        self.validate_project_structure(project_path)?;
        
        // Set up environment variables
        let env_vars = self.setup_environment_variables()?;
        
        // Build the cargo command
        let mut cmd = self.build_cargo_command(project_path)?;
        
        // Apply environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        
        // Configure stdio for capturing output
        cmd.stdout(Stdio::piped())
           .stderr(Stdio::piped());
        
        log::debug!("Executing cargo command: {:?}", cmd);
        
        // Execute compilation
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to execute cargo: {}", e)))?;
        
        let compilation_time = start_time.elapsed().as_millis() as u64;
        
        // Parse compilation output for diagnostics
        let diagnostics = self.parse_compilation_output(&output.stdout, &output.stderr)?;
        
        // Check compilation success
        if !output.status.success() {
            let error_msg = self.format_compilation_error(&output.stderr, &diagnostics);
            return Err(UdonSharpError::compilation(error_msg));
        }
        
        // Find and read the generated WASM file
        let wasm_path = self.find_wasm_output(project_path)?;
        let wasm_bytes = fs::read(&wasm_path)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read WASM file: {}", e)))?;
        
        log::info!("WASM compilation completed successfully in {}ms", compilation_time);
        
        Ok(CompilationResult {
            wasm_bytes,
            wasm_path,
            diagnostics,
            compilation_time_ms: compilation_time,
        })
    }
    
    /// Validate that the project has the correct structure for UdonSharp compilation
    fn validate_project_structure<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<()> {
        let project_path = project_path.as_ref();
        
        // Check for Cargo.toml
        let cargo_toml = project_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(UdonSharpError::compilation(
                "Project must contain a Cargo.toml file".to_string()
            ));
        }
        
        // Check for src directory
        let src_dir = project_path.join("src");
        if !src_dir.exists() {
            return Err(UdonSharpError::compilation(
                "Project must contain a src directory".to_string()
            ));
        }
        
        // Check for lib.rs or main.rs
        let lib_rs = src_dir.join("lib.rs");
        let main_rs = src_dir.join("main.rs");
        if !lib_rs.exists() && !main_rs.exists() {
            return Err(UdonSharpError::compilation(
                "Project must contain either src/lib.rs or src/main.rs".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Set up environment variables for UdonSharp-compatible WASM compilation
    fn setup_environment_variables(&self) -> UdonSharpResult<HashMap<String, String>> {
        let mut env_vars = HashMap::new();
        
        // Set RUSTFLAGS for UdonSharp compatibility
        let rustflags = self.get_rustflags().join(" ");
        env_vars.insert("RUSTFLAGS".to_string(), rustflags);
        
        // Set cargo target directory to avoid conflicts
        if let Some(target_dir) = self.get_target_directory() {
            env_vars.insert("CARGO_TARGET_DIR".to_string(), target_dir);
        }
        
        // Disable incremental compilation for reproducible builds
        env_vars.insert("CARGO_INCREMENTAL".to_string(), "0".to_string());
        
        // Set optimization level based on configuration
        if self.config.optimize_for_performance {
            env_vars.insert("CARGO_PROFILE_RELEASE_OPT_LEVEL".to_string(), "s".to_string());
            env_vars.insert("CARGO_PROFILE_RELEASE_LTO".to_string(), "true".to_string());
        }
        
        Ok(env_vars)
    }
    
    /// Build the cargo command with appropriate arguments
    fn build_cargo_command<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<Command> {
        let cargo_path = self.cargo_path.as_ref()
            .map(|p| p.as_os_str())
            .unwrap_or_else(|| "cargo".as_ref());
        
        let mut cmd = Command::new(cargo_path);
        
        cmd.arg("build")
           .arg("--target")
           .arg("wasm32-unknown-unknown")
           .current_dir(project_path.as_ref());
        
        // Add release flag if optimizing for performance
        if self.config.optimize_for_performance {
            cmd.arg("--release");
        }
        
        // Add library type specification
        cmd.arg("--lib");
        
        // Add verbose output if debug info is enabled
        if self.config.generate_debug_info {
            cmd.arg("--verbose");
        }
        
        // Add message format for better parsing
        cmd.arg("--message-format=json-diagnostic-rendered-ansi");
        
        Ok(cmd)
    }
    
    /// Parse compilation output to extract diagnostics
    fn parse_compilation_output(&self, stdout: &[u8], stderr: &[u8]) -> UdonSharpResult<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();
        
        // Parse stdout for JSON messages
        let stdout_str = String::from_utf8_lossy(stdout);
        for line in stdout_str.lines() {
            if let Ok(message) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(diagnostic) = self.parse_cargo_message(&message) {
                    diagnostics.push(diagnostic);
                }
            }
        }
        
        // Parse stderr for additional errors
        let stderr_str = String::from_utf8_lossy(stderr);
        if !stderr_str.trim().is_empty() {
            diagnostics.push(Diagnostic::error(stderr_str.to_string()));
        }
        
        Ok(diagnostics)
    }
    
    /// Parse a cargo JSON message into a diagnostic
    fn parse_cargo_message(&self, message: &serde_json::Value) -> Option<Diagnostic> {
        let reason = message.get("reason")?.as_str()?;
        
        match reason {
            "compiler-message" => {
                let msg = message.get("message")?;
                let level = msg.get("level")?.as_str()?;
                let text = msg.get("message")?.as_str()?.to_string();
                
                // Create base diagnostic
                let mut diagnostic = match level {
                    "error" => Diagnostic::error(text),
                    "warning" => Diagnostic::warning(text),
                    "note" => Diagnostic::info(text),
                    _ => return None,
                };
                
                // Extract file location if available
                if let Some(location) = msg.get("spans")
                    .and_then(|spans| spans.as_array())
                    .and_then(|spans| spans.first())
                    .and_then(|span| {
                        let file = span.get("file_name")?.as_str()?;
                        let line = span.get("line_start")?.as_u64()? as u32;
                        let column = span.get("column_start")?.as_u64()? as u32;
                        Some((file.to_string(), line, column))
                    })
                {
                    diagnostic = diagnostic.with_location(PathBuf::from(location.0), location.1, location.2);
                }
                
                Some(diagnostic)
            }
            _ => None,
        }
    }
    
    /// Format compilation error message with diagnostics
    fn format_compilation_error(&self, stderr: &[u8], diagnostics: &[Diagnostic]) -> String {
        let mut error_msg = String::from("Rust compilation failed:\n");
        
        // Add diagnostics
        for diagnostic in diagnostics {
            error_msg.push_str(&format!("  {}\n", diagnostic));
        }
        
        // Add raw stderr if no diagnostics were parsed
        if diagnostics.is_empty() {
            let stderr_str = String::from_utf8_lossy(stderr);
            if !stderr_str.trim().is_empty() {
                error_msg.push_str(&format!("Raw error output:\n{}", stderr_str));
            }
        }
        
        error_msg
    }
    
    /// Find the generated WASM file in the target directory
    fn find_wasm_output<P: AsRef<Path>>(&self, project_path: P) -> UdonSharpResult<PathBuf> {
        let project_path = project_path.as_ref();
        
        // Determine target directory
        let target_dir = if let Some(custom_target) = self.get_target_directory() {
            PathBuf::from(custom_target)
        } else {
            project_path.join("target")
        };
        
        // Determine profile directory
        let profile_dir = if self.config.optimize_for_performance {
            "release"
        } else {
            "debug"
        };
        
        let wasm_dir = target_dir
            .join("wasm32-unknown-unknown")
            .join(profile_dir);
        
        // Look for .wasm files
        let entries = fs::read_dir(&wasm_dir)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read target directory: {}", e)))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| UdonSharpError::compilation(format!("Failed to read directory entry: {}", e)))?;
            let path = entry.path();
            
            if path.extension().map_or(false, |ext| ext == "wasm") {
                return Ok(path);
            }
        }
        
        Err(UdonSharpError::compilation(
            "No WASM file found in target directory".to_string()
        ))
    }
    
    /// Get custom target directory if configured
    fn get_target_directory(&self) -> Option<String> {
        env::var("CARGO_TARGET_DIR").ok()
    }
    
    /// Get rustflags for UdonSharp-compatible WASM compilation
    pub fn get_rustflags(&self) -> Vec<String> {
        // Validate WASM configuration
        if let Err(error) = self.wasm_config.validate() {
            log::warn!("WASM configuration validation failed: {}", error);
        }
        
        // Get flags from WASM configuration
        self.wasm_config.get_rustc_flags()
    }
    
    /// Get the WASM target configuration
    pub fn wasm_config(&self) -> &WasmTargetConfig {
        &self.wasm_config
    }
    
    /// Get the UdonSharp configuration
    pub fn config(&self) -> &UdonSharpConfig {
        &self.config
    }
}