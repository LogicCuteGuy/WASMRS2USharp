//! Code optimization system for UdonSharp compilation pipeline

use crate::metrics::{PerformanceIssue, VRChatMetrics, IssueSeverity, IssueCategory, ImpactLevel};
use crate::profiler::{OptimizationOpportunity, OptimizationCategory, OptimizationDifficulty};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Main code optimizer for UdonSharp compilation pipeline
#[derive(Debug)]
pub struct CodeOptimizer {
    optimization_passes: Vec<Box<dyn OptimizationPass>>,
    strategies: HashMap<OptimizationStrategy, StrategyConfig>,
    vrchat_constraints: VRChatConstraints,
}

/// Optimization strategy configuration
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationStrategy {
    Development,    // Fast compilation, minimal optimization
    Balanced,       // Balance between speed and optimization
    Production,     // Maximum optimization for deployment
    VRChatOptimal,  // Optimized specifically for VRChat constraints
    MemoryFocused,  // Prioritize memory usage optimization
    SpeedFocused,   // Prioritize execution speed optimization
}

/// Configuration for an optimization strategy
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    pub enabled_passes: Vec<String>,
    pub optimization_level: OptimizationLevel,
    pub target_constraints: TargetConstraints,
    pub time_budget: Option<std::time::Duration>,
}

/// Optimization level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
    Maximum,
}

/// Target platform constraints
#[derive(Debug, Clone)]
pub struct TargetConstraints {
    pub max_instruction_count: Option<u64>,
    pub max_memory_usage: Option<u64>,
    pub max_sync_variables: Option<u32>,
    pub max_file_size: Option<u64>,
    pub vrchat_compatibility: bool,
}

/// VRChat-specific constraints and limits
#[derive(Debug, Clone)]
pub struct VRChatConstraints {
    pub max_instruction_count: u64,
    pub max_memory_footprint: u64,
    pub max_sync_variables: u32,
    pub max_string_length: u32,
    pub max_array_size: u32,
    pub forbidden_apis: Vec<String>,
    pub performance_thresholds: VRChatPerformanceThresholds,
}

/// VRChat performance thresholds
#[derive(Debug, Clone)]
pub struct VRChatPerformanceThresholds {
    pub excellent_instruction_limit: u64,
    pub good_instruction_limit: u64,
    pub poor_instruction_limit: u64,
    pub excellent_memory_limit: u64,
    pub good_memory_limit: u64,
    pub poor_memory_limit: u64,
}

/// Trait for optimization passes
pub trait OptimizationPass: std::fmt::Debug + Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> OptimizationCategory;
    fn difficulty(&self) -> OptimizationDifficulty;
    fn estimated_improvement(&self) -> f64;
    
    /// Apply optimization to Rust source code
    fn optimize_rust(&self, source: &str) -> Result<String>;
    
    /// Apply optimization to generated C# code
    fn optimize_csharp(&self, source: &str) -> Result<String>;
    
    /// Apply optimization to WASM bytecode
    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>>;
    
    /// Check if this pass is applicable to the given code
    fn is_applicable(&self, source: &str, language: CodeLanguage) -> bool;
    
    /// Get prerequisites for this optimization
    fn prerequisites(&self) -> Vec<String>;
}

/// Code language for optimization
#[derive(Debug, Clone, PartialEq)]
pub enum CodeLanguage {
    Rust,
    CSharp,
    Wasm,
}

/// Optimization result with metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub original_size: usize,
    pub optimized_size: usize,
    pub size_reduction: f64, // percentage
    pub applied_passes: Vec<String>,
    pub optimization_time: std::time::Duration,
    pub estimated_performance_gain: f64,
    pub issues_fixed: Vec<String>,
    pub warnings: Vec<String>,
}

impl CodeOptimizer {
    /// Create a new code optimizer
    pub fn new() -> Self {
        let mut optimizer = Self {
            optimization_passes: Vec::new(),
            strategies: HashMap::new(),
            vrchat_constraints: VRChatConstraints::default(),
        };

        optimizer.initialize_optimization_passes();
        optimizer.initialize_strategies();
        optimizer
    }

    /// Optimize code using the specified strategy
    pub fn optimize(&self, code: &str, strategy: OptimizationStrategy) -> Result<String> {
        let start_time = Instant::now();
        
        let config = self.strategies.get(&strategy)
            .ok_or_else(|| anyhow!("Unknown optimization strategy: {:?}", strategy))?;

        let mut optimized_code = code.to_string();
        let mut applied_passes = Vec::new();

        // Apply optimization passes in order
        for pass_name in &config.enabled_passes {
            if let Some(pass) = self.optimization_passes.iter()
                .find(|p| p.name() == pass_name) {
                
                if pass.is_applicable(&optimized_code, CodeLanguage::Rust) {
                    match pass.optimize_rust(&optimized_code) {
                        Ok(result) => {
                            optimized_code = result;
                            applied_passes.push(pass_name.clone());
                        }
                        Err(e) => {
                            eprintln!("Warning: Optimization pass '{}' failed: {}", pass_name, e);
                        }
                    }
                }
            }
        }

        // Check time budget
        if let Some(budget) = config.time_budget {
            if start_time.elapsed() > budget {
                eprintln!("Warning: Optimization exceeded time budget");
            }
        }

        Ok(optimized_code)
    }

    /// Optimize C# code specifically
    pub fn optimize_csharp(&self, code: &str, strategy: OptimizationStrategy) -> Result<String> {
        let config = self.strategies.get(&strategy)
            .ok_or_else(|| anyhow!("Unknown optimization strategy: {:?}", strategy))?;

        let mut optimized_code = code.to_string();

        for pass_name in &config.enabled_passes {
            if let Some(pass) = self.optimization_passes.iter()
                .find(|p| p.name() == pass_name) {
                
                if pass.is_applicable(&optimized_code, CodeLanguage::CSharp) {
                    match pass.optimize_csharp(&optimized_code) {
                        Ok(result) => optimized_code = result,
                        Err(e) => eprintln!("Warning: C# optimization pass '{}' failed: {}", pass_name, e),
                    }
                }
            }
        }

        Ok(optimized_code)
    }

    /// Optimize WASM bytecode
    pub fn optimize_wasm(&self, wasm_bytes: &[u8], strategy: OptimizationStrategy) -> Result<Vec<u8>> {
        let config = self.strategies.get(&strategy)
            .ok_or_else(|| anyhow!("Unknown optimization strategy: {:?}", strategy))?;

        let mut optimized_wasm = wasm_bytes.to_vec();

        for pass_name in &config.enabled_passes {
            if let Some(pass) = self.optimization_passes.iter()
                .find(|p| p.name() == pass_name) {
                
                match pass.optimize_wasm(&optimized_wasm) {
                    Ok(result) => optimized_wasm = result,
                    Err(e) => eprintln!("Warning: WASM optimization pass '{}' failed: {}", pass_name, e),
                }
            }
        }

        Ok(optimized_wasm)
    }

    /// Analyze code for optimization opportunities
    pub fn analyze_optimization_opportunities(&self, code: &str, language: CodeLanguage) -> Result<Vec<OptimizationOpportunity>> {
        let mut opportunities = Vec::new();

        for pass in &self.optimization_passes {
            if pass.is_applicable(code, language.clone()) {
                opportunities.push(OptimizationOpportunity {
                    category: pass.category(),
                    description: format!("{}: {}", pass.name(), pass.description()),
                    location: None,
                    estimated_improvement: pass.estimated_improvement(),
                    difficulty: pass.difficulty(),
                    suggested_changes: pass.prerequisites(),
                });
            }
        }

        Ok(opportunities)
    }

    /// Check VRChat compatibility and constraints
    pub fn check_vrchat_compatibility(&self, metrics: &VRChatMetrics) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();

        // Check instruction count
        if metrics.estimated_instruction_count > self.vrchat_constraints.max_instruction_count {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::VRChatLimits,
                description: format!(
                    "Instruction count ({}) exceeds VRChat limit ({})",
                    metrics.estimated_instruction_count,
                    self.vrchat_constraints.max_instruction_count
                ),
                location: None,
                suggested_fix: Some("Reduce code complexity or split into multiple behaviors".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }

        // Check memory footprint
        if metrics.estimated_memory_footprint > self.vrchat_constraints.max_memory_footprint {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Memory,
                description: format!(
                    "Memory footprint ({:.1}MB) exceeds recommended limit ({:.1}MB)",
                    metrics.estimated_memory_footprint as f64 / 1_000_000.0,
                    self.vrchat_constraints.max_memory_footprint as f64 / 1_000_000.0
                ),
                location: None,
                suggested_fix: Some("Optimize data structures and reduce memory allocations".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }

        // Check sync variables
        if metrics.network_sync_variables > self.vrchat_constraints.max_sync_variables {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Network,
                description: format!(
                    "Sync variables ({}) exceed recommended limit ({})",
                    metrics.network_sync_variables,
                    self.vrchat_constraints.max_sync_variables
                ),
                location: None,
                suggested_fix: Some("Use manual sync mode or reduce synchronized data".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }

        Ok(issues)
    }

    /// Get optimization recommendations for VRChat performance
    pub fn get_vrchat_optimization_recommendations(&self, metrics: &VRChatMetrics) -> Result<Vec<OptimizationOpportunity>> {
        let mut recommendations = Vec::new();

        // Performance rank-based recommendations
        match metrics.performance_rank_estimate {
            crate::metrics::PerformanceRank::VeryPoor | crate::metrics::PerformanceRank::Poor => {
                recommendations.push(OptimizationOpportunity {
                    category: OptimizationCategory::VRChatOptimization,
                    description: "Critical performance optimization needed for VRChat compatibility".to_string(),
                    location: None,
                    estimated_improvement: 40.0,
                    difficulty: OptimizationDifficulty::Hard,
                    suggested_changes: vec![
                        "Reduce instruction count through algorithm optimization".to_string(),
                        "Minimize memory allocations and object creation".to_string(),
                        "Optimize network synchronization patterns".to_string(),
                    ],
                });
            }
            crate::metrics::PerformanceRank::Medium => {
                recommendations.push(OptimizationOpportunity {
                    category: OptimizationCategory::VRChatOptimization,
                    description: "Moderate optimization can improve VRChat performance ranking".to_string(),
                    location: None,
                    estimated_improvement: 20.0,
                    difficulty: OptimizationDifficulty::Medium,
                    suggested_changes: vec![
                        "Profile and optimize performance hotspots".to_string(),
                        "Review and optimize data structures".to_string(),
                    ],
                });
            }
            _ => {}
        }

        // Instruction count optimization
        if metrics.estimated_instruction_count > self.vrchat_constraints.performance_thresholds.good_instruction_limit {
            recommendations.push(OptimizationOpportunity {
                category: OptimizationCategory::ComputationSpeed,
                description: "Instruction count optimization for better VRChat performance".to_string(),
                location: None,
                estimated_improvement: 25.0,
                difficulty: OptimizationDifficulty::Medium,
                suggested_changes: vec![
                    "Optimize loops and recursive algorithms".to_string(),
                    "Use more efficient data structures".to_string(),
                    "Cache expensive calculations".to_string(),
                ],
            });
        }

        Ok(recommendations)
    }

    /// Initialize built-in optimization passes
    fn initialize_optimization_passes(&mut self) {
        self.optimization_passes.push(Box::new(DeadCodeEliminationPass));
        self.optimization_passes.push(Box::new(LoopOptimizationPass));
        self.optimization_passes.push(Box::new(MemoryOptimizationPass));
        self.optimization_passes.push(Box::new(UdonSharpOptimizationPass));
        self.optimization_passes.push(Box::new(VRChatCompatibilityPass));
        self.optimization_passes.push(Box::new(NetworkOptimizationPass));
        self.optimization_passes.push(Box::new(StringOptimizationPass));
        self.optimization_passes.push(Box::new(CollectionOptimizationPass));
        self.optimization_passes.push(Box::new(VRChatWorldOptimizationPass));
        self.optimization_passes.push(Box::new(InstructionCountOptimizationPass));
        self.optimization_passes.push(Box::new(PerformanceRankOptimizationPass));
    }

    /// Initialize optimization strategies
    fn initialize_strategies(&mut self) {
        // Development strategy - fast compilation
        self.strategies.insert(OptimizationStrategy::Development, StrategyConfig {
            enabled_passes: vec![
                "dead_code_elimination".to_string(),
                "basic_memory_optimization".to_string(),
            ],
            optimization_level: OptimizationLevel::Basic,
            target_constraints: TargetConstraints {
                max_instruction_count: None,
                max_memory_usage: None,
                max_sync_variables: None,
                max_file_size: None,
                vrchat_compatibility: false,
            },
            time_budget: Some(std::time::Duration::from_secs(10)),
        });

        // Production strategy - maximum optimization
        self.strategies.insert(OptimizationStrategy::Production, StrategyConfig {
            enabled_passes: vec![
                "dead_code_elimination".to_string(),
                "loop_optimization".to_string(),
                "memory_optimization".to_string(),
                "udonsharp_optimization".to_string(),
                "string_optimization".to_string(),
                "collection_optimization".to_string(),
            ],
            optimization_level: OptimizationLevel::Maximum,
            target_constraints: TargetConstraints {
                max_instruction_count: Some(1_000_000),
                max_memory_usage: Some(100_000_000),
                max_sync_variables: Some(200),
                max_file_size: Some(10_000_000),
                vrchat_compatibility: true,
            },
            time_budget: None,
        });

        // VRChat optimal strategy
        self.strategies.insert(OptimizationStrategy::VRChatOptimal, StrategyConfig {
            enabled_passes: vec![
                "dead_code_elimination".to_string(),
                "loop_optimization".to_string(),
                "memory_optimization".to_string(),
                "udonsharp_optimization".to_string(),
                "vrchat_compatibility".to_string(),
                "network_optimization".to_string(),
                "string_optimization".to_string(),
            ],
            optimization_level: OptimizationLevel::Aggressive,
            target_constraints: TargetConstraints {
                max_instruction_count: Some(500_000),
                max_memory_usage: Some(50_000_000),
                max_sync_variables: Some(100),
                max_file_size: Some(5_000_000),
                vrchat_compatibility: true,
            },
            time_budget: Some(std::time::Duration::from_secs(60)),
        });

        // Memory focused strategy
        self.strategies.insert(OptimizationStrategy::MemoryFocused, StrategyConfig {
            enabled_passes: vec![
                "memory_optimization".to_string(),
                "collection_optimization".to_string(),
                "string_optimization".to_string(),
                "dead_code_elimination".to_string(),
            ],
            optimization_level: OptimizationLevel::Aggressive,
            target_constraints: TargetConstraints {
                max_instruction_count: None,
                max_memory_usage: Some(25_000_000), // 25MB
                max_sync_variables: None,
                max_file_size: None,
                vrchat_compatibility: true,
            },
            time_budget: Some(std::time::Duration::from_secs(30)),
        });
    }
}

// Optimization pass implementations

#[derive(Debug)]
struct DeadCodeEliminationPass;

impl OptimizationPass for DeadCodeEliminationPass {
    fn name(&self) -> &str { "dead_code_elimination" }
    fn description(&self) -> &str { "Remove unused code and variables" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::CodeStructure }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Easy }
    fn estimated_improvement(&self) -> f64 { 15.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized_lines = Vec::new();
        let mut in_unused_block = false;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }
            
            // Remove unused variable declarations
            if trimmed.starts_with("let _unused") || trimmed.starts_with("let _ =") {
                continue;
            }
            
            // Remove unreachable code after return statements
            if trimmed.starts_with("return") {
                optimized_lines.push(line.to_string());
                in_unused_block = true;
                continue;
            }
            
            // Skip code in unreachable blocks until next function/block
            if in_unused_block {
                if trimmed.starts_with("fn ") || trimmed.starts_with("impl ") || trimmed == "}" {
                    in_unused_block = false;
                } else {
                    continue;
                }
            }
            
            // Remove debug prints in release builds
            if trimmed.contains("println!") || trimmed.contains("dbg!") {
                continue;
            }
            
            optimized_lines.push(line.to_string());
        }
        
        Ok(optimized_lines.join("\n"))
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized_lines = Vec::new();
        let mut used_namespaces = std::collections::HashSet::new();
        
        // First pass: collect used namespaces
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Look for type usage to determine which using statements are needed
            if trimmed.contains("GameObject") || trimmed.contains("Transform") {
                used_namespaces.insert("UnityEngine");
            }
            if trimmed.contains("VRCPlayerApi") || trimmed.contains("Networking") {
                used_namespaces.insert("VRC.SDKBase");
            }
            if trimmed.contains("UdonBehaviour") {
                used_namespaces.insert("UdonSharp");
            }
        }
        
        // Second pass: filter lines
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }
            
            // Filter unused using statements
            if trimmed.starts_with("using ") {
                let namespace = trimmed.trim_start_matches("using ").trim_end_matches(";");
                if !used_namespaces.contains(namespace) {
                    continue;
                }
            }
            
            // Remove unused variables and debug code
            if trimmed.starts_with("// unused") || 
               trimmed.contains("Debug.Log") ||
               trimmed.contains("Console.WriteLine") {
                continue;
            }
            
            optimized_lines.push(line.to_string());
        }
        
        Ok(optimized_lines.join("\n"))
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        // Use wasm-opt for dead code elimination
        // This is a placeholder - in real implementation would call wasm-opt
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, _source: &str, _language: CodeLanguage) -> bool { true }
    fn prerequisites(&self) -> Vec<String> { vec![] }
}

#[derive(Debug)]
struct LoopOptimizationPass;

impl OptimizationPass for LoopOptimizationPass {
    fn name(&self) -> &str { "loop_optimization" }
    fn description(&self) -> &str { "Optimize loop structures for better performance" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::ComputationSpeed }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Medium }
    fn estimated_improvement(&self) -> f64 { 20.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Optimize iterator chains
        optimized = optimized.replace(
            ".iter().collect::<Vec<_>>()",
            ".collect::<Vec<_>>()"
        );
        
        // Replace inefficient patterns
        optimized = optimized.replace(
            "for i in 0..vec.len() { vec[i]",
            "for item in &vec { *item"
        );
        
        // Optimize range loops
        optimized = optimized.replace(
            "for i in 0..n { if i == target",
            "if target < n { // Direct access instead of loop"
        );
        
        // Suggest using iterators instead of index-based loops
        if optimized.contains("for i in 0..") && optimized.contains("[i]") {
            optimized = format!("// Consider using iterators for better performance\n{}", optimized);
        }
        
        Ok(optimized)
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Optimize foreach loops for better UdonSharp performance
        optimized = optimized.replace(
            "foreach (var item in list)",
            "for (int i = 0; i < list.Length; i++) { var item = list[i];"
        );
        
        // Cache array length in loops
        optimized = optimized.replace(
            "for (int i = 0; i < array.Length; i++)",
            "int length = array.Length; for (int i = 0; i < length; i++)"
        );
        
        // Optimize string concatenation in loops
        if optimized.contains("for ") && optimized.contains("string") && optimized.contains("+") {
            optimized = format!("// Consider using StringBuilder for string concatenation in loops\n{}", optimized);
        }
        
        Ok(optimized)
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        // WASM loop optimizations would be handled by wasm-opt
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("for ") || source.contains("while ") || source.contains("loop") || source.contains("foreach")
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["Loop analysis and profiling".to_string()]
    }
}

#[derive(Debug)]
struct MemoryOptimizationPass;

impl OptimizationPass for MemoryOptimizationPass {
    fn name(&self) -> &str { "memory_optimization" }
    fn description(&self) -> &str { "Optimize memory usage and reduce allocations" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::MemoryUsage }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Medium }
    fn estimated_improvement(&self) -> f64 { 25.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Pre-allocate vectors with known capacity
        optimized = optimized.replace(
            "Vec::new()",
            "Vec::with_capacity(expected_size) // TODO: Set appropriate capacity"
        );
        
        // Use string literals instead of String::new() + push_str
        optimized = optimized.replace(
            "let mut s = String::new();\n    s.push_str(",
            "let s = String::from("
        );
        
        // Suggest using &str instead of String where possible
        if optimized.contains("String") && !optimized.contains("mut") {
            optimized = format!("// Consider using &str instead of String for read-only strings\n{}", optimized);
        }
        
        // Optimize clone operations
        optimized = optimized.replace(".clone()", ".as_ref() // Consider avoiding clone if possible");
        
        // Use Box for large stack allocations
        if optimized.contains("let large_array = [") {
            optimized = optimized.replace(
                "let large_array = [",
                "let large_array = Box::new(["
            );
        }
        
        Ok(optimized)
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Use object pooling for frequently created objects
        if optimized.contains("new GameObject") {
            optimized = format!("// Consider using object pooling for GameObjects\n{}", optimized);
        }
        
        // Optimize array allocations
        optimized = optimized.replace(
            "new int[]",
            "new int[capacity] // Pre-allocate with known size"
        );
        
        // Use StringBuilder for string concatenation
        if optimized.contains("string") && optimized.contains("+") {
            optimized = format!("// Use StringBuilder for multiple string concatenations\n{}", optimized);
        }
        
        // Cache component references
        if optimized.contains("GetComponent<") {
            optimized = format!("// Cache GetComponent results to avoid repeated calls\n{}", optimized);
        }
        
        // Suggest using struct instead of class for small data
        if optimized.contains("public class") && optimized.lines().count() < 10 {
            optimized = format!("// Consider using struct for small data types\n{}", optimized);
        }
        
        Ok(optimized)
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        // WASM memory optimizations would be handled by wasm-opt
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("Vec::new") || source.contains("String::new") || source.contains("new ") || 
        source.contains("clone()") || source.contains("GetComponent")
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["Memory profiling analysis".to_string()]
    }
}

#[derive(Debug)]
struct UdonSharpOptimizationPass;

impl OptimizationPass for UdonSharpOptimizationPass {
    fn name(&self) -> &str { "udonsharp_optimization" }
    fn description(&self) -> &str { "Apply UdonSharp-specific optimizations" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::UdonSharpSpecific }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Medium }
    fn estimated_improvement(&self) -> f64 { 30.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Add UdonSharp-specific attributes and optimizations
        if optimized.contains("pub struct") && !optimized.contains("#[UdonBehaviour]") {
            optimized = optimized.replace(
                "pub struct",
                "#[derive(UdonBehaviour)]\npub struct"
            );
        }
        
        // Optimize for UdonSharp field access patterns
        optimized = optimized.replace(
            "pub fn get_",
            "// UdonSharp: Use direct field access instead of getters\npub fn get_"
        );
        
        Ok(optimized)
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Cache expensive GameObject.Find calls
        optimized = optimized.replace(
            "GameObject.Find(",
            "// Cache this result: GameObject.Find("
        );
        
        // Optimize GetComponent calls
        optimized = optimized.replace(
            "GetComponent<",
            "// Cache component reference: GetComponent<"
        );
        
        // Use UdonSharp-optimized patterns
        optimized = optimized.replace(
            "transform.position",
            "transform.position // Consider caching transform reference"
        );
        
        // Optimize array access patterns for UdonSharp
        optimized = optimized.replace(
            "array[index]",
            "array[index] // Bounds check is automatic in UdonSharp"
        );
        
        // Add proper UdonSharp attributes
        if optimized.contains("public class") && !optimized.contains("UdonBehaviour") {
            optimized = optimized.replace(
                "public class",
                "[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\npublic class"
            );
        }
        
        // Optimize networking patterns
        if optimized.contains("SendCustomNetworkEvent") {
            optimized = format!("// Consider batching network events for better performance\n{}", optimized);
        }
        
        // Optimize Update method patterns
        if optimized.contains("void Update()") {
            optimized = format!("// Minimize Update() logic for better VRChat performance\n{}", optimized);
        }
        
        Ok(optimized)
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        // UdonSharp-specific WASM optimizations
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("UdonBehaviour") || source.contains("GameObject") || 
        source.contains("Transform") || source.contains("VRCPlayerApi")
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["UdonSharp compatibility analysis".to_string()]
    }
}

#[derive(Debug)]
struct VRChatCompatibilityPass;

impl OptimizationPass for VRChatCompatibilityPass {
    fn name(&self) -> &str { "vrchat_compatibility" }
    fn description(&self) -> &str { "Ensure VRChat compatibility and optimize for VRChat constraints" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::VRChatOptimization }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Hard }
    fn estimated_improvement(&self) -> f64 { 35.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, _source: &str, _language: CodeLanguage) -> bool { true }
    fn prerequisites(&self) -> Vec<String> {
        vec!["VRChat SDK compatibility check".to_string()]
    }
}

#[derive(Debug)]
struct NetworkOptimizationPass;

impl OptimizationPass for NetworkOptimizationPass {
    fn name(&self) -> &str { "network_optimization" }
    fn description(&self) -> &str { "Optimize network synchronization and reduce bandwidth usage" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::NetworkEfficiency }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Medium }
    fn estimated_improvement(&self) -> f64 { 20.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("UdonSync") || source.contains("SendCustomNetworkEvent")
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["Network usage analysis".to_string()]
    }
}

#[derive(Debug)]
struct StringOptimizationPass;

impl OptimizationPass for StringOptimizationPass {
    fn name(&self) -> &str { "string_optimization" }
    fn description(&self) -> &str { "Optimize string operations and reduce string allocations" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::MemoryUsage }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Easy }
    fn estimated_improvement(&self) -> f64 { 15.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("String") || source.contains("&str")
    }

    fn prerequisites(&self) -> Vec<String> { vec![] }
}

#[derive(Debug)]
struct CollectionOptimizationPass;

impl OptimizationPass for CollectionOptimizationPass {
    fn name(&self) -> &str { "collection_optimization" }
    fn description(&self) -> &str { "Optimize collection usage and reduce collection overhead" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::MemoryUsage }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Medium }
    fn estimated_improvement(&self) -> f64 { 18.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        Ok(source.to_string())
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("Vec<") || source.contains("HashMap") || source.contains("List<") || source.contains("Array")
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["Collection usage analysis".to_string()]
    }
}

impl Default for VRChatConstraints {
    fn default() -> Self {
        Self {
            max_instruction_count: 1_000_000,
            max_memory_footprint: 100_000_000, // 100MB
            max_sync_variables: 200,
            max_string_length: 10_000,
            max_array_size: 10_000,
            forbidden_apis: vec![
                "System.IO".to_string(),
                "System.Net".to_string(),
                "System.Threading".to_string(),
                "System.Reflection".to_string(),
            ],
            performance_thresholds: VRChatPerformanceThresholds {
                excellent_instruction_limit: 10_000,
                good_instruction_limit: 50_000,
                poor_instruction_limit: 500_000,
                excellent_memory_limit: 1_000_000,   // 1MB
                good_memory_limit: 10_000_000,       // 10MB
                poor_memory_limit: 50_000_000,       // 50MB
            },
        }
    }
}

#[derive(Debug)]
struct VRChatWorldOptimizationPass;

impl OptimizationPass for VRChatWorldOptimizationPass {
    fn name(&self) -> &str { "vrchat_world_optimization" }
    fn description(&self) -> &str { "Optimize code specifically for VRChat world performance" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::VRChatOptimization }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Hard }
    fn estimated_improvement(&self) -> f64 { 40.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Optimize for VRChat's execution model
        if optimized.contains("Update") {
            optimized = format!("// VRChat: Minimize Update() usage, prefer events\n{}", optimized);
        }
        
        // Suggest using VRChat-specific optimizations
        if optimized.contains("Vector3") {
            optimized = optimized.replace(
                "Vector3::new(",
                "Vector3::new( // VRChat: Consider using Vector3 constants"
            );
        }
        
        Ok(optimized)
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // VRChat world-specific optimizations
        
        // Optimize physics interactions
        if optimized.contains("Rigidbody") {
            optimized = format!("// VRChat: Use kinematic rigidbodies when possible for better performance\n{}", optimized);
        }
        
        // Optimize audio sources
        if optimized.contains("AudioSource") {
            optimized = format!("// VRChat: Limit concurrent audio sources, use audio occlusion\n{}", optimized);
        }
        
        // Optimize particle systems
        if optimized.contains("ParticleSystem") {
            optimized = format!("// VRChat: Limit particle count and use LOD for particles\n{}", optimized);
        }
        
        // Optimize lighting
        if optimized.contains("Light") {
            optimized = format!("// VRChat: Use baked lighting when possible, limit real-time lights\n{}", optimized);
        }
        
        // Optimize materials and shaders
        if optimized.contains("Material") || optimized.contains("Shader") {
            optimized = format!("// VRChat: Use mobile-friendly shaders, avoid expensive shader operations\n{}", optimized);
        }
        
        // Optimize colliders
        if optimized.contains("Collider") {
            optimized = format!("// VRChat: Use primitive colliders when possible, avoid mesh colliders\n{}", optimized);
        }
        
        // Optimize animation
        if optimized.contains("Animator") {
            optimized = format!("// VRChat: Optimize animator controllers, use culling modes\n{}", optimized);
        }
        
        // Optimize UI elements
        if optimized.contains("Canvas") || optimized.contains("UI") {
            optimized = format!("// VRChat: Use world space UI sparingly, optimize UI draw calls\n{}", optimized);
        }
        
        Ok(optimized)
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, language: CodeLanguage) -> bool {
        language == CodeLanguage::CSharp && (
            source.contains("VRCPlayerApi") || 
            source.contains("UdonBehaviour") ||
            source.contains("GameObject") ||
            source.contains("Transform")
        )
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["VRChat world performance analysis".to_string()]
    }
}

#[derive(Debug)]
struct InstructionCountOptimizationPass;

impl OptimizationPass for InstructionCountOptimizationPass {
    fn name(&self) -> &str { "instruction_count_optimization" }
    fn description(&self) -> &str { "Optimize code to reduce VRChat instruction count" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::VRChatOptimization }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Hard }
    fn estimated_improvement(&self) -> f64 { 35.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Reduce instruction count through algorithmic improvements
        
        // Replace expensive operations with cheaper alternatives
        optimized = optimized.replace(
            ".pow(2)",
            " * self // Multiplication is cheaper than pow(2)"
        );
        
        // Optimize mathematical operations
        optimized = optimized.replace(
            "f32::sqrt(",
            "// Consider using fast inverse square root if precision allows: f32::sqrt("
        );
        
        // Suggest loop unrolling for small, fixed loops
        if optimized.contains("for i in 0..3") || optimized.contains("for i in 0..4") {
            optimized = format!("// Consider manual loop unrolling for small fixed loops\n{}", optimized);
        }
        
        Ok(optimized)
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // VRChat instruction count optimizations
        
        // Replace expensive math operations
        optimized = optimized.replace(
            "Mathf.Pow(x, 2)",
            "x * x // Multiplication is cheaper than Pow"
        );
        
        optimized = optimized.replace(
            "Mathf.Pow(x, 3)",
            "x * x * x // Manual multiplication is cheaper"
        );
        
        // Optimize trigonometric functions
        optimized = optimized.replace(
            "Mathf.Sin(",
            "// Cache sin/cos values if used repeatedly: Mathf.Sin("
        );
        
        // Optimize vector operations
        optimized = optimized.replace(
            "Vector3.Distance(",
            "// Use sqrMagnitude if you only need comparison: Vector3.Distance("
        );
        
        optimized = optimized.replace(
            ".magnitude",
            ".sqrMagnitude // Use sqrMagnitude for comparisons to avoid sqrt"
        );
        
        // Optimize array/list operations
        optimized = optimized.replace(
            ".Contains(",
            "// Use HashSet for O(1) lookups instead of Contains: .Contains("
        );
        
        // Optimize string operations
        optimized = optimized.replace(
            "string.Format(",
            "// Use string interpolation or StringBuilder: string.Format("
        );
        
        // Optimize GameObject operations
        optimized = optimized.replace(
            "GameObject.FindWithTag(",
            "// Cache references instead of repeated Find calls: GameObject.FindWithTag("
        );
        
        Ok(optimized)
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, _language: CodeLanguage) -> bool {
        source.contains("Mathf.") || source.contains("Vector3") || 
        source.contains("for ") || source.contains("while ") ||
        source.contains("GameObject.Find") || source.contains(".magnitude")
    }

    fn prerequisites(&self) -> Vec<String> {
        vec!["Instruction count profiling".to_string()]
    }
}

#[derive(Debug)]
struct PerformanceRankOptimizationPass;

impl OptimizationPass for PerformanceRankOptimizationPass {
    fn name(&self) -> &str { "performance_rank_optimization" }
    fn description(&self) -> &str { "Optimize code to achieve better VRChat performance ranking" }
    fn category(&self) -> OptimizationCategory { OptimizationCategory::VRChatOptimization }
    fn difficulty(&self) -> OptimizationDifficulty { OptimizationDifficulty::Hard }
    fn estimated_improvement(&self) -> f64 { 45.0 }

    fn optimize_rust(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // Add performance rank optimization comments and suggestions
        if optimized.contains("Update") {
            optimized = format!(
                "// Performance Rank: Minimize Update() usage for Excellent rank\n{}",
                optimized
            );
        }
        
        Ok(optimized)
    }

    fn optimize_csharp(&self, source: &str) -> Result<String> {
        let mut optimized = source.to_string();
        
        // VRChat performance rank optimizations
        
        // Excellent rank optimizations (< 10k instructions)
        optimized = format!(
            "// VRChat Performance Rank Optimization:\n\
             // Excellent: < 10,000 instructions\n\
             // Good: < 50,000 instructions\n\
             // Medium: < 500,000 instructions\n\
             // Poor: > 500,000 instructions\n\n{}",
            optimized
        );
        
        // Optimize for excellent performance rank
        if optimized.contains("void Update()") {
            optimized = optimized.replace(
                "void Update()",
                "// PERFORMANCE: Avoid Update() for Excellent rank\nvoid Update()"
            );
        }
        
        if optimized.contains("void FixedUpdate()") {
            optimized = optimized.replace(
                "void FixedUpdate()",
                "// PERFORMANCE: Avoid FixedUpdate() for Excellent rank\nvoid FixedUpdate()"
            );
        }
        
        // Suggest event-driven alternatives
        if optimized.contains("Update()") {
            optimized = format!(
                "// PERFORMANCE TIP: Use events instead of Update() for better performance rank:\n\
                 // - OnTriggerEnter/Exit for proximity detection\n\
                 // - OnCollisionEnter/Exit for collision detection\n\
                 // - Custom events for state changes\n\n{}",
                optimized
            );
        }
        
        // Optimize networking for performance rank
        if optimized.contains("SendCustomNetworkEvent") {
            optimized = optimized.replace(
                "SendCustomNetworkEvent",
                "// PERFORMANCE: Batch network events, use Manual sync mode\nSendCustomNetworkEvent"
            );
        }
        
        // Optimize array operations
        if optimized.contains("Array.") || optimized.contains("List<") {
            optimized = format!(
                "// PERFORMANCE: Pre-allocate collections, avoid resizing\n{}",
                optimized
            );
        }
        
        // Optimize physics operations
        if optimized.contains("Physics.") {
            optimized = format!(
                "// PERFORMANCE: Cache physics queries, use layer masks\n{}",
                optimized
            );
        }
        
        Ok(optimized)
    }

    fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(wasm_bytes.to_vec())
    }

    fn is_applicable(&self, source: &str, language: CodeLanguage) -> bool {
        language == CodeLanguage::CSharp && (
            source.contains("UdonBehaviour") ||
            source.contains("Update") ||
            source.contains("VRCPlayerApi") ||
            source.contains("SendCustomNetworkEvent")
        )
    }

    fn prerequisites(&self) -> Vec<String> {
        vec![
            "VRChat performance rank analysis".to_string(),
            "Instruction count estimation".to_string(),
        ]
    }
}

impl Default for CodeOptimizer {
    fn default() -> Self {
        Self::new()
    }
}