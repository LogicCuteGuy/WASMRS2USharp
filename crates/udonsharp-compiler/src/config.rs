//! Configuration structures for the UdonSharp compilation pipeline
//! 
//! This module provides configuration options for controlling the Rust to UdonSharp
//! compilation process.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Conflicting settings: {0}")]
    ConflictingSettings(String),
}

/// Configuration for UdonSharp compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonSharpConfig {
    /// Optional namespace for generated UdonSharp classes
    pub namespace: Option<String>,
    
    /// Synchronization mode for UdonSharp networking
    pub sync_mode: UdonSyncMode,
    
    /// Whether to generate debug information
    pub generate_debug_info: bool,
    
    /// Whether to optimize for performance
    pub optimize_for_performance: bool,
    
    /// Target UdonSharp version
    pub target_udonsharp_version: String,
    
    /// Directories to scan for .asmdef files
    pub asmdef_scan_directories: Vec<PathBuf>,
    
    /// Path to custom binding rules file
    pub custom_binding_rules: Option<PathBuf>,
    
    /// Output directory for generated files
    pub output_directory: Option<String>,
    
    /// Number of parallel jobs for compilation
    pub parallel_jobs: Option<usize>,
    
    /// Check only mode (don't generate output)
    pub check_only: bool,
    
    /// Test mode
    pub test_mode: bool,
    
    /// Test filter pattern
    pub test_filter: Option<String>,
    
    /// Capture test output
    pub capture_test_output: bool,
    
    /// Multi-behavior compilation settings
    pub multi_behavior: MultiBehaviorSettings,
}

impl Default for UdonSharpConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            sync_mode: UdonSyncMode::None,
            generate_debug_info: false,
            optimize_for_performance: true,
            target_udonsharp_version: "1.0".to_string(),
            asmdef_scan_directories: Vec::new(),
            custom_binding_rules: None,
            output_directory: None,
            parallel_jobs: None,
            check_only: false,
            test_mode: false,
            test_filter: None,
            capture_test_output: true,
            multi_behavior: MultiBehaviorSettings::default(),
        }
    }
}

impl UdonSharpConfig {
    /// Create a configuration for a Unity project
    pub fn with_unity_project<P: Into<PathBuf>>(unity_project_path: P) -> Self {
        let project_path = unity_project_path.into();
        let mut config = Self::default();
        
        // Automatically discover common Unity and VRChat SDK paths
        config.asmdef_scan_directories = vec![
            project_path.join("Library/PackageCache"),
            project_path.join("Assets"),
            project_path.join("Packages"),
        ];
        
        config
    }
    
    /// Add a custom library path for scanning
    pub fn add_custom_library_path<P: Into<PathBuf>>(&mut self, path: P) {
        self.asmdef_scan_directories.push(path.into());
    }
    
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| ConfigError::IoError(format!("Failed to read config file {:?}: {}", path.as_ref(), e)))?;
        
        Self::from_str(&content)
    }
    
    /// Parse configuration from a TOML string
    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let config: Self = toml::from_str(content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse TOML: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate multi-behavior settings
        self.multi_behavior.validate()?;
        
        // Validate target UdonSharp version
        if !self.is_valid_udonsharp_version(&self.target_udonsharp_version) {
            return Err(ConfigError::InvalidValue(format!(
                "Invalid UdonSharp version: {}. Expected format: x.y or x.y.z",
                self.target_udonsharp_version
            )));
        }
        
        // Validate parallel jobs
        if let Some(jobs) = self.parallel_jobs {
            if jobs == 0 {
                return Err(ConfigError::InvalidValue(
                    "parallel_jobs must be greater than 0".to_string()
                ));
            }
            if jobs > 64 {
                return Err(ConfigError::InvalidValue(
                    "parallel_jobs should not exceed 64 for optimal performance".to_string()
                ));
            }
        }
        
        // Validate output directory
        if let Some(output_dir) = &self.output_directory {
            if output_dir.is_empty() {
                return Err(ConfigError::InvalidValue(
                    "output_directory cannot be empty".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if a UdonSharp version string is valid
    fn is_valid_udonsharp_version(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return false;
        }
        
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
}

/// UdonSharp synchronization modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UdonSyncMode {
    None,
    Manual,
    Continuous,
}

/// Multi-behavior compilation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiBehaviorSettings {
    /// Enable multi-behavior compilation
    pub enabled: bool,
    
    /// Generate SharedRuntime class for shared functions
    pub generate_shared_runtime: bool,
    
    /// Naming convention for behavior classes
    pub naming_convention: BehaviorNamingConvention,
    
    /// Minimum number of behaviors required to enable multi-behavior mode
    pub min_behaviors_threshold: usize,
    
    /// Whether to generate Unity prefab files
    pub generate_prefabs: bool,
    
    /// Prefab generation settings
    pub prefab_settings: PrefabGenerationSettings,
    
    /// Initialization order management
    pub initialization_order: InitializationOrderSettings,
}

impl Default for MultiBehaviorSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            generate_shared_runtime: true,
            naming_convention: BehaviorNamingConvention::PascalCase,
            min_behaviors_threshold: 2,
            generate_prefabs: true,
            prefab_settings: PrefabGenerationSettings::default(),
            initialization_order: InitializationOrderSettings::default(),
        }
    }
}

impl MultiBehaviorSettings {
    /// Validate multi-behavior settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate minimum behaviors threshold
        if self.min_behaviors_threshold == 0 {
            return Err(ConfigError::InvalidValue(
                "min_behaviors_threshold must be greater than 0".to_string()
            ));
        }
        
        if self.min_behaviors_threshold > 100 {
            return Err(ConfigError::InvalidValue(
                "min_behaviors_threshold should not exceed 100 for practical use".to_string()
            ));
        }
        
        // Validate naming convention
        self.naming_convention.validate()?;
        
        // Validate prefab settings
        self.prefab_settings.validate()?;
        
        // Validate initialization order settings
        self.initialization_order.validate()?;
        
        Ok(())
    }
}

/// Naming convention for behavior classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BehaviorNamingConvention {
    /// PascalCase: PlayerManager
    PascalCase,
    /// PascalCaseWithSuffix: PlayerManagerBehaviour
    PascalCaseWithSuffix,
    /// Custom format string with {name} placeholder
    Custom(String),
}

impl BehaviorNamingConvention {
    /// Validate the naming convention
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self {
            BehaviorNamingConvention::Custom(format) => {
                if format.is_empty() {
                    return Err(ConfigError::InvalidValue(
                        "Custom naming convention format cannot be empty".to_string()
                    ));
                }
                
                if !format.contains("{name}") {
                    return Err(ConfigError::InvalidValue(
                        "Custom naming convention must contain {name} placeholder".to_string()
                    ));
                }
                
                // Check for valid C# identifier characters
                let test_name = format.replace("{name}", "TestName");
                if !Self::is_valid_csharp_identifier(&test_name) {
                    return Err(ConfigError::InvalidValue(
                        "Custom naming convention must produce valid C# identifiers".to_string()
                    ));
                }
            }
            _ => {} // PascalCase and PascalCaseWithSuffix are always valid
        }
        
        Ok(())
    }
    
    /// Check if a string is a valid C# identifier
    fn is_valid_csharp_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        
        let mut chars = name.chars();
        let first = chars.next().unwrap();
        
        // First character must be letter or underscore
        if !first.is_alphabetic() && first != '_' {
            return false;
        }
        
        // Remaining characters must be alphanumeric or underscore
        chars.all(|c| c.is_alphanumeric() || c == '_')
    }
}

/// Settings for Unity prefab generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabGenerationSettings {
    /// Generate individual prefabs for each behavior
    pub generate_individual_prefabs: bool,
    
    /// Generate a master prefab containing all behaviors
    pub generate_master_prefab: bool,
    
    /// Automatically set up component references
    pub auto_setup_references: bool,
    
    /// Include example scene setup
    pub include_example_scene: bool,
    
    /// Prefab output directory
    pub output_directory: Option<String>,
}

impl Default for PrefabGenerationSettings {
    fn default() -> Self {
        Self {
            generate_individual_prefabs: true,
            generate_master_prefab: true,
            auto_setup_references: true,
            include_example_scene: false,
            output_directory: None,
        }
    }
}

impl PrefabGenerationSettings {
    /// Validate prefab generation settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        // At least one prefab type must be enabled
        if !self.generate_individual_prefabs && !self.generate_master_prefab {
            return Err(ConfigError::InvalidValue(
                "At least one of generate_individual_prefabs or generate_master_prefab must be true".to_string()
            ));
        }
        
        // Validate output directory
        if let Some(output_dir) = &self.output_directory {
            if output_dir.is_empty() {
                return Err(ConfigError::InvalidValue(
                    "output_directory cannot be empty".to_string()
                ));
            }
            
            // Check for invalid path characters
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            if output_dir.chars().any(|c| invalid_chars.contains(&c)) {
                return Err(ConfigError::InvalidValue(
                    "output_directory contains invalid path characters".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

/// Settings for initialization order management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationOrderSettings {
    /// Automatically determine initialization order based on dependencies
    pub auto_determine_order: bool,
    
    /// Manual initialization order (behavior names in order)
    pub manual_order: Vec<String>,
    
    /// Generate initialization coordinator
    pub generate_coordinator: bool,
    
    /// Coordinator class name
    pub coordinator_class_name: String,
    
    /// Use Unity's script execution order
    pub use_script_execution_order: bool,
}

impl Default for InitializationOrderSettings {
    fn default() -> Self {
        Self {
            auto_determine_order: true,
            manual_order: Vec::new(),
            generate_coordinator: true,
            coordinator_class_name: "BehaviorCoordinator".to_string(),
            use_script_execution_order: true,
        }
    }
}

impl InitializationOrderSettings {
    /// Validate initialization order settings
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate coordinator class name
        if self.coordinator_class_name.is_empty() {
            return Err(ConfigError::InvalidValue(
                "coordinator_class_name cannot be empty".to_string()
            ));
        }
        
        if !BehaviorNamingConvention::is_valid_csharp_identifier(&self.coordinator_class_name) {
            return Err(ConfigError::InvalidValue(
                "coordinator_class_name must be a valid C# identifier".to_string()
            ));
        }
        
        // Validate manual order if specified
        if !self.auto_determine_order && self.manual_order.is_empty() {
            return Err(ConfigError::InvalidValue(
                "manual_order cannot be empty when auto_determine_order is false".to_string()
            ));
        }
        
        // Check for duplicate behavior names in manual order
        let mut seen_names = std::collections::HashSet::new();
        for behavior_name in &self.manual_order {
            if behavior_name.is_empty() {
                return Err(ConfigError::InvalidValue(
                    "Behavior names in manual_order cannot be empty".to_string()
                ));
            }
            
            if !seen_names.insert(behavior_name) {
                return Err(ConfigError::InvalidValue(
                    format!("Duplicate behavior name in manual_order: {}", behavior_name)
                ));
            }
        }
        
        // Conflicting settings check
        if !self.auto_determine_order && !self.manual_order.is_empty() && !self.generate_coordinator {
            return Err(ConfigError::ConflictingSettings(
                "Manual initialization order requires coordinator generation to be enabled".to_string()
            ));
        }
        
        Ok(())
    }
}

/// WASM target configuration for UdonSharp compatibility
/// 
/// This configuration ensures that generated WASM is compatible with
/// the UdonSharp runtime environment and VRChat's constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmTargetConfig {
    /// Enable bulk memory operations (required for efficient memory management)
    pub bulk_memory: bool,
    
    /// Enable sign extension operations (improves performance for integer operations)
    pub sign_extension: bool,
    
    /// Enable mutable globals (required for static variables)
    pub mutable_globals: bool,
    
    /// Disable threading support (not supported in UdonSharp)
    pub disable_threads: bool,
    
    /// Disable SIMD instructions (not supported in UdonSharp)
    pub disable_simd: bool,
    
    /// Disable atomic operations (not supported in UdonSharp)
    pub disable_atomics: bool,
    
    /// Disable exception handling (not supported in UdonSharp)
    pub disable_exceptions: bool,
    
    /// Disable tail calls (may cause issues in UdonSharp)
    pub disable_tail_calls: bool,
    
    /// Disable multi-value returns (not supported in UdonSharp)
    pub disable_multi_value: bool,
    
    /// Enable reference types (needed for object references)
    pub reference_types: bool,
    
    /// Maximum memory pages (64KB each, UdonSharp has memory limits)
    pub max_memory_pages: Option<u32>,
    
    /// Stack size limit in bytes
    pub stack_size_limit: Option<u32>,
    
    /// Enable debug information in WASM
    pub debug_info: bool,
    
    /// Optimization level for WASM generation
    pub optimization_level: WasmOptimizationLevel,
}

/// WASM optimization levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmOptimizationLevel {
    /// No optimization (fastest compilation)
    None,
    /// Size optimization (smallest output)
    Size,
    /// Speed optimization (fastest execution)
    Speed,
    /// Balanced optimization
    Balanced,
}

impl Default for WasmTargetConfig {
    fn default() -> Self {
        Self {
            // Enable UdonSharp-compatible features
            bulk_memory: true,
            sign_extension: true,
            mutable_globals: true,
            reference_types: true,
            
            // Disable unsupported features
            disable_threads: true,
            disable_simd: true,
            disable_atomics: true,
            disable_exceptions: true,
            disable_tail_calls: true,
            disable_multi_value: true,
            
            // Set reasonable limits for VRChat
            max_memory_pages: Some(256), // 16MB limit
            stack_size_limit: Some(1024 * 1024), // 1MB stack
            
            // Default to size optimization for VRChat constraints
            optimization_level: WasmOptimizationLevel::Size,
            debug_info: false,
        }
    }
}

impl WasmTargetConfig {
    /// Create a configuration optimized for development (with debug info)
    pub fn development() -> Self {
        Self {
            debug_info: true,
            optimization_level: WasmOptimizationLevel::None,
            ..Default::default()
        }
    }
    
    /// Create a configuration optimized for production (maximum optimization)
    pub fn production() -> Self {
        Self {
            debug_info: false,
            optimization_level: WasmOptimizationLevel::Size,
            max_memory_pages: Some(128), // Stricter memory limit for production
            stack_size_limit: Some(512 * 1024), // Smaller stack for production
            ..Default::default()
        }
    }
    
    /// Create a configuration for testing (balanced settings)
    pub fn testing() -> Self {
        Self {
            debug_info: true,
            optimization_level: WasmOptimizationLevel::Balanced,
            max_memory_pages: Some(512), // More memory for testing
            ..Default::default()
        }
    }
    
    /// Get the target features as rustc flags
    pub fn get_target_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        
        if self.bulk_memory {
            features.push("+bulk-memory".to_string());
        } else {
            features.push("-bulk-memory".to_string());
        }
        
        if self.sign_extension {
            features.push("+sign-ext".to_string());
        } else {
            features.push("-sign-ext".to_string());
        }
        
        if self.mutable_globals {
            features.push("+mutable-globals".to_string());
        } else {
            features.push("-mutable-globals".to_string());
        }
        
        if self.reference_types {
            features.push("+reference-types".to_string());
        } else {
            features.push("-reference-types".to_string());
        }
        
        if self.disable_threads || self.disable_atomics {
            features.push("-atomics".to_string());
        }
        
        if self.disable_simd {
            features.push("-simd128".to_string());
        }
        
        if self.disable_exceptions {
            features.push("-exception-handling".to_string());
        }
        
        if self.disable_tail_calls {
            features.push("-tail-call".to_string());
        }
        
        if self.disable_multi_value {
            features.push("-multivalue".to_string());
        }
        
        features
    }
    
    /// Get rustc flags for this configuration
    pub fn get_rustc_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();
        
        // Add target features
        let features = self.get_target_features();
        if !features.is_empty() {
            flags.push("-C".to_string());
            flags.push(format!("target-feature={}", features.join(",")));
        }
        
        // Add optimization flags
        match self.optimization_level {
            WasmOptimizationLevel::None => {
                flags.push("-C".to_string());
                flags.push("opt-level=0".to_string());
            }
            WasmOptimizationLevel::Size => {
                flags.push("-C".to_string());
                flags.push("opt-level=s".to_string());
            }
            WasmOptimizationLevel::Speed => {
                flags.push("-C".to_string());
                flags.push("opt-level=3".to_string());
            }
            WasmOptimizationLevel::Balanced => {
                flags.push("-C".to_string());
                flags.push("opt-level=2".to_string());
            }
        }
        
        // Add debug info flags
        if self.debug_info {
            flags.push("-C".to_string());
            flags.push("debuginfo=2".to_string());
        } else {
            flags.push("-C".to_string());
            flags.push("debuginfo=0".to_string());
        }
        
        // Add panic handling
        flags.push("-C".to_string());
        flags.push("panic=abort".to_string());
        
        // Add LTO for size optimization
        if matches!(self.optimization_level, WasmOptimizationLevel::Size | WasmOptimizationLevel::Speed) {
            flags.push("-C".to_string());
            flags.push("lto=fat".to_string());
            
            flags.push("-C".to_string());
            flags.push("codegen-units=1".to_string());
        }
        
        // Add stack size limit if specified
        if let Some(stack_size) = self.stack_size_limit {
            flags.push("-C".to_string());
            flags.push(format!("link-arg=-z"));
            flags.push("-C".to_string());
            flags.push(format!("link-arg=stack-size={}", stack_size));
        }
        
        flags
    }
    
    /// Validate the configuration for UdonSharp compatibility
    pub fn validate(&self) -> Result<(), String> {
        // Check that required features are enabled
        if !self.bulk_memory {
            return Err("Bulk memory must be enabled for UdonSharp compatibility".to_string());
        }
        
        if !self.mutable_globals {
            return Err("Mutable globals must be enabled for UdonSharp compatibility".to_string());
        }
        
        // Check that unsupported features are disabled
        if !self.disable_threads {
            return Err("Threads must be disabled for UdonSharp compatibility".to_string());
        }
        
        if !self.disable_simd {
            return Err("SIMD must be disabled for UdonSharp compatibility".to_string());
        }
        
        if !self.disable_atomics {
            return Err("Atomics must be disabled for UdonSharp compatibility".to_string());
        }
        
        // Check memory limits
        if let Some(max_pages) = self.max_memory_pages {
            if max_pages > 1024 {
                return Err("Maximum memory pages should not exceed 1024 (64MB) for VRChat compatibility".to_string());
            }
        }
        
        if let Some(stack_size) = self.stack_size_limit {
            if stack_size > 2 * 1024 * 1024 {
                return Err("Stack size should not exceed 2MB for VRChat compatibility".to_string());
            }
        }
        
        Ok(())
    }
}