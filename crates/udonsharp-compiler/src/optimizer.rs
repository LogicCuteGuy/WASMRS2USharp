//! WASM optimization pipeline for UdonSharp compatibility
//! 
//! This module provides WASM optimization functionality specifically
//! tailored for UdonSharp constraints and performance characteristics.

use udonsharp_core::{UdonSharpResult, UdonSharpError};
use crate::config::WasmTargetConfig;
use std::process::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

/// WASM optimizer wrapper for wasm-opt integration
pub struct WasmOptimizer {
    optimization_level: OptimizationLevel,
    wasm_config: WasmTargetConfig,
    wasm_opt_path: Option<PathBuf>,
    enable_custom_passes: bool,
}

impl WasmOptimizer {
    /// Create a new WASM optimizer with default settings
    pub fn new(level: OptimizationLevel) -> Self {
        Self {
            optimization_level: level,
            wasm_config: WasmTargetConfig::default(),
            wasm_opt_path: None,
            enable_custom_passes: true,
        }
    }
    
    /// Create a new WASM optimizer with custom configuration
    pub fn with_config(level: OptimizationLevel, config: WasmTargetConfig) -> Self {
        Self {
            optimization_level: level,
            wasm_config: config,
            wasm_opt_path: None,
            enable_custom_passes: true,
        }
    }
    
    /// Set custom path to wasm-opt binary
    pub fn with_wasm_opt_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.wasm_opt_path = Some(path.into());
        self
    }
    
    /// Enable or disable custom UdonSharp optimization passes
    pub fn with_custom_passes(mut self, enable: bool) -> Self {
        self.enable_custom_passes = enable;
        self
    }
    
    /// Optimize WASM bytecode for UdonSharp compatibility
    pub fn optimize(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::info!("Starting WASM optimization with level: {:?}", self.optimization_level);
        
        // Validate WASM configuration
        self.wasm_config.validate()
            .map_err(|e| UdonSharpError::compilation(format!("Invalid WASM config: {}", e)))?;
        
        // Check if wasm-opt is available
        if !self.is_wasm_opt_available() {
            log::warn!("wasm-opt not found, skipping optimization");
            return Ok(wasm_bytes.to_vec());
        }
        
        // Validate input WASM
        self.validate_wasm_input(wasm_bytes)?;
        
        // Create temporary files for input and output
        let input_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        let output_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        
        // Write input WASM to temporary file
        fs::write(input_file.path(), wasm_bytes)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to write WASM input: {}", e)))?;
        
        // Run wasm-opt optimization
        let optimized_bytes = self.run_wasm_opt(input_file.path(), output_file.path())?;
        
        // Validate output WASM
        self.validate_wasm_output(&optimized_bytes)?;
        
        log::info!("WASM optimization completed successfully");
        log::info!("Original size: {} bytes, Optimized size: {} bytes", 
                  wasm_bytes.len(), optimized_bytes.len());
        
        Ok(optimized_bytes)
    }
    
    /// Run wasm-opt with UdonSharp-specific optimization passes
    fn run_wasm_opt(&self, input_path: &Path, output_path: &Path) -> UdonSharpResult<Vec<u8>> {
        let wasm_opt_cmd = self.get_wasm_opt_command();
        let mut cmd = Command::new(&wasm_opt_cmd);
        
        // Add optimization flags
        let flags = self.get_optimization_flags();
        for flag in flags {
            cmd.arg(flag);
        }
        
        // Add UdonSharp-specific passes if enabled
        if self.enable_custom_passes {
            let custom_passes = self.get_udonsharp_passes();
            for pass in custom_passes {
                cmd.arg(pass);
            }
        }
        
        // Add input and output files
        cmd.arg(input_path);
        cmd.arg("-o");
        cmd.arg(output_path);
        
        // Execute wasm-opt
        log::debug!("Running wasm-opt command: {:?}", cmd);
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to run wasm-opt: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UdonSharpError::compilation(
                format!("wasm-opt failed: {}", stderr)
            ));
        }
        
        // Read optimized WASM
        fs::read(output_path)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read optimized WASM: {}", e)))
    }
    
    /// Get the wasm-opt command path
    fn get_wasm_opt_command(&self) -> String {
        if let Some(ref path) = self.wasm_opt_path {
            path.to_string_lossy().to_string()
        } else {
            "wasm-opt".to_string()
        }
    }
    
    /// Check if wasm-opt tool is available
    fn is_wasm_opt_available(&self) -> bool {
        let cmd = self.get_wasm_opt_command();
        Command::new(&cmd)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Get optimization flags for wasm-opt based on optimization level
    fn get_optimization_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();
        
        match self.optimization_level {
            OptimizationLevel::None => {
                flags.push("-O0".to_string());
            }
            OptimizationLevel::Size => {
                flags.push("-Os".to_string());
                flags.push("--converge".to_string());
            }
            OptimizationLevel::Speed => {
                flags.push("-O3".to_string());
                flags.push("--converge".to_string());
            }
            OptimizationLevel::UdonSharp => {
                // Balanced optimization for UdonSharp constraints
                flags.push("-Os".to_string());
                flags.push("--converge".to_string());
            }
        }
        
        // Add feature flags based on WASM config
        if self.wasm_config.bulk_memory {
            flags.push("--enable-bulk-memory".to_string());
        }
        
        if self.wasm_config.sign_extension {
            flags.push("--enable-sign-ext".to_string());
        }
        
        if self.wasm_config.mutable_globals {
            flags.push("--enable-mutable-globals".to_string());
        }
        
        if self.wasm_config.reference_types {
            flags.push("--enable-reference-types".to_string());
        }
        
        // Disable unsupported features
        if self.wasm_config.disable_simd {
            flags.push("--disable-simd".to_string());
        }
        
        if self.wasm_config.disable_threads {
            flags.push("--disable-threads".to_string());
        }
        
        flags
    }
    
    /// Get UdonSharp-specific optimization passes
    fn get_udonsharp_passes(&self) -> Vec<String> {
        let mut passes = Vec::new();
        
        // Dead code elimination - remove unused functions and globals
        passes.push("--dce".to_string());
        
        // Remove unused imports and exports
        passes.push("--remove-unused-module-elements".to_string());
        
        // Optimize function calls
        passes.push("--optimize-calls".to_string());
        
        // Merge similar functions
        passes.push("--merge-similar-functions".to_string());
        
        // Optimize memory access patterns
        passes.push("--optimize-added-constants".to_string());
        passes.push("--optimize-added-constants-propagate".to_string());
        
        // Simplify control flow
        passes.push("--simplify-locals".to_string());
        passes.push("--reorder-locals".to_string());
        
        // Remove debug information if not needed
        if !self.wasm_config.debug_info {
            passes.push("--strip-debug".to_string());
            passes.push("--strip-dwarf".to_string());
        }
        
        // UdonSharp-specific optimizations
        match self.optimization_level {
            OptimizationLevel::UdonSharp => {
                // Focus on size reduction for VRChat constraints
                passes.push("--vacuum".to_string());
                passes.push("--merge-blocks".to_string());
                passes.push("--remove-unused-brs".to_string());
                passes.push("--remove-unused-names".to_string());
                
                // Optimize for VRChat runtime characteristics
                passes.push("--precompute".to_string());
                passes.push("--precompute-propagate".to_string());
            }
            OptimizationLevel::Size => {
                // Maximum size reduction
                passes.push("--vacuum".to_string());
                passes.push("--merge-blocks".to_string());
                passes.push("--remove-unused-brs".to_string());
                passes.push("--remove-unused-names".to_string());
                passes.push("--minify-imports-and-exports".to_string());
            }
            OptimizationLevel::Speed => {
                // Focus on execution speed
                passes.push("--precompute".to_string());
                passes.push("--precompute-propagate".to_string());
                passes.push("--optimize-instructions".to_string());
            }
            OptimizationLevel::None => {
                // Minimal optimization
            }
        }
        
        passes
    }
    
    /// Validate input WASM bytecode
    fn validate_wasm_input(&self, wasm_bytes: &[u8]) -> UdonSharpResult<()> {
        // Check WASM magic number
        if wasm_bytes.len() < 8 {
            return Err(UdonSharpError::wasm(
                "Invalid WASM: file too small"
            ));
        }
        
        let magic = &wasm_bytes[0..4];
        if magic != b"\x00asm" {
            return Err(UdonSharpError::wasm(
                "Invalid WASM: missing magic number"
            ));
        }
        
        // Check WASM version
        let version = u32::from_le_bytes([
            wasm_bytes[4], wasm_bytes[5], wasm_bytes[6], wasm_bytes[7]
        ]);
        if version != 1 {
            return Err(UdonSharpError::wasm(
                format!("Unsupported WASM version: {}", version)
            ));
        }
        
        // Check size limits
        if let Some(max_size) = self.get_max_wasm_size() {
            if wasm_bytes.len() > max_size {
                return Err(UdonSharpError::wasm(
                    format!("WASM file too large: {} bytes (max: {} bytes)", 
                           wasm_bytes.len(), max_size)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate output WASM bytecode
    fn validate_wasm_output(&self, wasm_bytes: &[u8]) -> UdonSharpResult<()> {
        // Perform the same basic validation as input
        self.validate_wasm_input(wasm_bytes)?;
        
        // Additional output-specific validation could be added here
        // For example, checking that required exports are present
        
        Ok(())
    }
    
    /// Get maximum allowed WASM size based on VRChat constraints
    fn get_max_wasm_size(&self) -> Option<usize> {
        // VRChat has practical limits on world size
        // This is a conservative estimate for WASM module size
        Some(10 * 1024 * 1024) // 10MB limit
    }
    
    /// Get optimization statistics
    pub fn get_optimization_stats(&self, original_size: usize, optimized_size: usize) -> OptimizationStats {
        let size_reduction = if original_size > 0 {
            ((original_size - optimized_size) as f64 / original_size as f64) * 100.0
        } else {
            0.0
        };
        
        OptimizationStats {
            original_size,
            optimized_size,
            size_reduction_percent: size_reduction,
            optimization_level: self.optimization_level.clone(),
        }
    }
}

/// WASM optimization levels
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationLevel {
    /// No optimization (fastest compilation)
    None,
    /// Size optimization (smallest output)
    Size,
    /// Speed optimization (fastest execution)
    Speed,
    /// UdonSharp-specific optimization (balanced for VRChat)
    UdonSharp,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        Self::UdonSharp
    }
}

/// Statistics about WASM optimization
#[derive(Debug, Clone)]
pub struct OptimizationStats {
    pub original_size: usize,
    pub optimized_size: usize,
    pub size_reduction_percent: f64,
    pub optimization_level: OptimizationLevel,
}

impl OptimizationStats {
    /// Check if optimization was effective
    pub fn is_effective(&self) -> bool {
        self.size_reduction_percent > 1.0 // At least 1% reduction
    }
    
    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Optimization ({:?}): {} → {} bytes ({:.1}% reduction)",
            self.optimization_level,
            self.original_size,
            self.optimized_size,
            self.size_reduction_percent
        )
    }
}

/// UdonSharp-specific optimization passes and strategies
pub struct UdonSharpOptimizer {
    base_optimizer: WasmOptimizer,
    enable_dead_code_elimination: bool,
    enable_vrc_runtime_optimizations: bool,
    enable_memory_optimizations: bool,
    enable_networking_optimizations: bool,
}

impl UdonSharpOptimizer {
    /// Create a new UdonSharp-specific optimizer
    pub fn new() -> Self {
        Self {
            base_optimizer: WasmOptimizer::new(OptimizationLevel::UdonSharp),
            enable_dead_code_elimination: true,
            enable_vrc_runtime_optimizations: true,
            enable_memory_optimizations: true,
            enable_networking_optimizations: true,
        }
    }
    
    /// Create optimizer with custom configuration
    pub fn with_config(config: UdonSharpOptimizerConfig) -> Self {
        let wasm_config = WasmTargetConfig {
            optimization_level: config.wasm_optimization_level,
            max_memory_pages: config.max_memory_pages,
            stack_size_limit: config.stack_size_limit,
            debug_info: config.preserve_debug_info,
            ..WasmTargetConfig::default()
        };
        
        Self {
            base_optimizer: WasmOptimizer::with_config(OptimizationLevel::UdonSharp, wasm_config),
            enable_dead_code_elimination: config.enable_dead_code_elimination,
            enable_vrc_runtime_optimizations: config.enable_vrc_runtime_optimizations,
            enable_memory_optimizations: config.enable_memory_optimizations,
            enable_networking_optimizations: config.enable_networking_optimizations,
        }
    }
    
    /// Optimize WASM with UdonSharp-specific passes
    pub fn optimize(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::info!("Starting UdonSharp-specific WASM optimization");
        
        // Step 1: Apply base WASM optimization
        let mut optimized_wasm = self.base_optimizer.optimize(wasm_bytes)?;
        
        // Step 2: Apply UdonSharp-specific optimizations
        if self.enable_dead_code_elimination {
            optimized_wasm = self.eliminate_unused_udonsharp_features(&optimized_wasm)?;
        }
        
        if self.enable_vrc_runtime_optimizations {
            optimized_wasm = self.optimize_for_vrc_runtime(&optimized_wasm)?;
        }
        
        if self.enable_memory_optimizations {
            optimized_wasm = self.optimize_memory_usage(&optimized_wasm)?;
        }
        
        if self.enable_networking_optimizations {
            optimized_wasm = self.optimize_networking_code(&optimized_wasm)?;
        }
        
        // Step 3: Final validation and cleanup
        optimized_wasm = self.final_cleanup_pass(&optimized_wasm)?;
        
        log::info!("UdonSharp-specific optimization completed");
        Ok(optimized_wasm)
    }
    
    /// Eliminate unused UdonSharp features and dead code
    fn eliminate_unused_udonsharp_features(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::debug!("Eliminating unused UdonSharp features");
        
        // Create temporary files for the optimization pass
        let input_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        let output_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        
        // Write input WASM
        fs::write(input_file.path(), wasm_bytes)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to write WASM: {}", e)))?;
        
        // Run wasm-opt with dead code elimination passes
        let mut cmd = Command::new("wasm-opt");
        cmd.arg("--dce")                                    // Dead code elimination
           .arg("--remove-unused-module-elements")          // Remove unused imports/exports
           .arg("--remove-unused-brs")                      // Remove unused branches
           .arg("--remove-unused-names")                    // Remove unused names
           .arg("--remove-unused-nonfunction-module-elements") // Remove unused globals/tables
           .arg("--vacuum")                                 // Clean up after other passes
           .arg(input_file.path())
           .arg("-o")
           .arg(output_file.path());
        
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to run wasm-opt: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UdonSharpError::compilation(
                format!("Dead code elimination failed: {}", stderr)
            ));
        }
        
        // Read optimized WASM
        fs::read(output_file.path())
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read optimized WASM: {}", e)))
    }
    
    /// Optimize for VRChat runtime performance characteristics
    fn optimize_for_vrc_runtime(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::debug!("Optimizing for VRChat runtime characteristics");
        
        let input_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        let output_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        
        fs::write(input_file.path(), wasm_bytes)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to write WASM: {}", e)))?;
        
        // VRChat runtime optimizations
        let mut cmd = Command::new("wasm-opt");
        cmd.arg("--optimize-calls")                         // Optimize function calls
           .arg("--optimize-instructions")                  // Optimize instruction sequences
           .arg("--optimize-added-constants")               // Optimize constant operations
           .arg("--optimize-added-constants-propagate")     // Propagate constant optimizations
           .arg("--precompute")                            // Precompute constant expressions
           .arg("--precompute-propagate")                  // Propagate precomputed values
           .arg("--simplify-locals")                       // Simplify local variable usage
           .arg("--reorder-locals")                        // Reorder locals for better performance
           .arg("--merge-similar-functions")               // Merge similar functions
           .arg("--merge-blocks")                          // Merge basic blocks
           .arg(input_file.path())
           .arg("-o")
           .arg(output_file.path());
        
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to run wasm-opt: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UdonSharpError::compilation(
                format!("VRChat runtime optimization failed: {}", stderr)
            ));
        }
        
        fs::read(output_file.path())
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read optimized WASM: {}", e)))
    }
    
    /// Optimize memory usage for VRChat constraints
    fn optimize_memory_usage(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::debug!("Optimizing memory usage for VRChat constraints");
        
        let input_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        let output_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        
        fs::write(input_file.path(), wasm_bytes)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to write WASM: {}", e)))?;
        
        // Memory optimization passes
        let mut cmd = Command::new("wasm-opt");
        cmd.arg("--memory-packing")                         // Pack memory more efficiently
           .arg("--optimize-stack-ir")                      // Optimize stack operations
           .arg("--local-cse")                             // Common subexpression elimination
           .arg("--global-refining")                       // Refine global variable types
           .arg("--duplicate-function-elimination")         // Remove duplicate functions
           .arg("--duplicate-import-elimination")           // Remove duplicate imports
           .arg("--flatten")                               // Flatten nested structures
           .arg(input_file.path())
           .arg("-o")
           .arg(output_file.path());
        
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to run wasm-opt: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UdonSharpError::compilation(
                format!("Memory optimization failed: {}", stderr)
            ));
        }
        
        fs::read(output_file.path())
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read optimized WASM: {}", e)))
    }
    
    /// Optimize networking-related code for UdonSharp
    fn optimize_networking_code(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::debug!("Optimizing networking code for UdonSharp");
        
        // For networking optimizations, we focus on reducing the overhead
        // of network synchronization and event handling
        let input_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        let output_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        
        fs::write(input_file.path(), wasm_bytes)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to write WASM: {}", e)))?;
        
        // Networking-specific optimizations
        let mut cmd = Command::new("wasm-opt");
        cmd.arg("--inlining-optimizing")                    // Inline small functions (reduces call overhead)
           .arg("--optimize-level=2")                       // Moderate optimization for networking code
           .arg("--fast-math")                             // Optimize math operations
           .arg("--zero-filled-memory")                    // Optimize zero-filled memory
           .arg(input_file.path())
           .arg("-o")
           .arg(output_file.path());
        
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to run wasm-opt: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UdonSharpError::compilation(
                format!("Networking optimization failed: {}", stderr)
            ));
        }
        
        fs::read(output_file.path())
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read optimized WASM: {}", e)))
    }
    
    /// Final cleanup pass to ensure optimal output
    fn final_cleanup_pass(&self, wasm_bytes: &[u8]) -> UdonSharpResult<Vec<u8>> {
        log::debug!("Running final cleanup pass");
        
        let input_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        let output_file = NamedTempFile::new()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to create temp file: {}", e)))?;
        
        fs::write(input_file.path(), wasm_bytes)
            .map_err(|e| UdonSharpError::compilation(format!("Failed to write WASM: {}", e)))?;
        
        // Final cleanup and validation
        let mut cmd = Command::new("wasm-opt");
        cmd.arg("--post-emscripten")                        // Clean up Emscripten artifacts
           .arg("--vacuum")                                 // Final vacuum pass
           .arg("--converge")                               // Run passes until convergence
           .arg("--closed-world")                           // Assume closed world for better optimization
           .arg(input_file.path())
           .arg("-o")
           .arg(output_file.path());
        
        let output = cmd.output()
            .map_err(|e| UdonSharpError::compilation(format!("Failed to run wasm-opt: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(UdonSharpError::compilation(
                format!("Final cleanup failed: {}", stderr)
            ));
        }
        
        fs::read(output_file.path())
            .map_err(|e| UdonSharpError::compilation(format!("Failed to read optimized WASM: {}", e)))
    }
    
    /// Get optimization statistics comparing before and after
    pub fn get_detailed_stats(&self, original_wasm: &[u8], optimized_wasm: &[u8]) -> UdonSharpOptimizationStats {
        UdonSharpOptimizationStats {
            base_stats: self.base_optimizer.get_optimization_stats(original_wasm.len(), optimized_wasm.len()),
            dead_code_eliminated: self.enable_dead_code_elimination,
            vrc_runtime_optimized: self.enable_vrc_runtime_optimizations,
            memory_optimized: self.enable_memory_optimizations,
            networking_optimized: self.enable_networking_optimizations,
        }
    }
}

/// Configuration for UdonSharp-specific optimizer
#[derive(Debug, Clone)]
pub struct UdonSharpOptimizerConfig {
    pub wasm_optimization_level: crate::config::WasmOptimizationLevel,
    pub enable_dead_code_elimination: bool,
    pub enable_vrc_runtime_optimizations: bool,
    pub enable_memory_optimizations: bool,
    pub enable_networking_optimizations: bool,
    pub max_memory_pages: Option<u32>,
    pub stack_size_limit: Option<u32>,
    pub preserve_debug_info: bool,
}

impl Default for UdonSharpOptimizerConfig {
    fn default() -> Self {
        Self {
            wasm_optimization_level: crate::config::WasmOptimizationLevel::Size,
            enable_dead_code_elimination: true,
            enable_vrc_runtime_optimizations: true,
            enable_memory_optimizations: true,
            enable_networking_optimizations: true,
            max_memory_pages: Some(256), // 16MB for VRChat
            stack_size_limit: Some(1024 * 1024), // 1MB stack
            preserve_debug_info: false,
        }
    }
}

impl UdonSharpOptimizerConfig {
    /// Create configuration optimized for development
    pub fn development() -> Self {
        Self {
            wasm_optimization_level: crate::config::WasmOptimizationLevel::None,
            preserve_debug_info: true,
            enable_dead_code_elimination: false, // Keep all code for debugging
            ..Default::default()
        }
    }
    
    /// Create configuration optimized for production
    pub fn production() -> Self {
        Self {
            wasm_optimization_level: crate::config::WasmOptimizationLevel::Size,
            preserve_debug_info: false,
            max_memory_pages: Some(128), // Stricter limit for production
            stack_size_limit: Some(512 * 1024), // Smaller stack for production
            ..Default::default()
        }
    }
    
    /// Create configuration for testing
    pub fn testing() -> Self {
        Self {
            wasm_optimization_level: crate::config::WasmOptimizationLevel::Balanced,
            preserve_debug_info: true,
            enable_networking_optimizations: false, // Disable for isolated testing
            ..Default::default()
        }
    }
}

/// Detailed optimization statistics for UdonSharp
#[derive(Debug, Clone)]
pub struct UdonSharpOptimizationStats {
    pub base_stats: OptimizationStats,
    pub dead_code_eliminated: bool,
    pub vrc_runtime_optimized: bool,
    pub memory_optimized: bool,
    pub networking_optimized: bool,
}

impl UdonSharpOptimizationStats {
    /// Get a detailed summary of optimizations applied
    pub fn detailed_summary(&self) -> String {
        let mut summary = self.base_stats.summary();
        summary.push_str("\nOptimizations applied:");
        
        if self.dead_code_eliminated {
            summary.push_str("\n  ✓ Dead code elimination");
        }
        if self.vrc_runtime_optimized {
            summary.push_str("\n  ✓ VRChat runtime optimization");
        }
        if self.memory_optimized {
            summary.push_str("\n  ✓ Memory usage optimization");
        }
        if self.networking_optimized {
            summary.push_str("\n  ✓ Networking code optimization");
        }
        
        summary
    }
    
    /// Check if all optimizations were applied
    pub fn all_optimizations_applied(&self) -> bool {
        self.dead_code_eliminated && 
        self.vrc_runtime_optimized && 
        self.memory_optimized && 
        self.networking_optimized
    }
}