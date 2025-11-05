//! Performance metrics data structures and calculations

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Comprehensive performance metrics for a compilation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub compilation: CompilationMetrics,
    pub memory: MemoryMetrics,
    pub session_duration: Duration,
    pub step_count: usize,
    pub failed_steps: usize,
}

/// Compilation-specific performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompilationMetrics {
    pub total_compilation_time: Duration,
    pub rust_compilation_time: Duration,
    pub wasm_generation_time: Duration,
    pub wasm_optimization_time: Duration,
    pub csharp_generation_time: Duration,
    pub file_io_time: Duration,
    pub binding_generation_time: Duration,
    
    // Code metrics
    pub rust_lines_of_code: u32,
    pub generated_csharp_lines: u32,
    pub wasm_size_bytes: u64,
    pub csharp_file_count: u32,
    pub binding_count: u32,
    
    // Performance indicators
    pub compilation_speed_loc_per_second: f64,
    pub wasm_compression_ratio: f64,
    pub csharp_expansion_ratio: f64,
}

/// Memory usage metrics during compilation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_usage: u64,
    pub average_usage: u64,
    pub initial_usage: u64,
    pub final_usage: u64,
    pub memory_growth: u64,
    pub gc_collections: u32,
    pub allocation_rate: f64, // bytes per second
}

/// VRChat-specific performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VRChatMetrics {
    pub estimated_instruction_count: u64,
    pub estimated_memory_footprint: u64,
    pub network_sync_variables: u32,
    pub udon_event_count: u32,
    pub performance_rank_estimate: PerformanceRank,
    pub vrchat_compatibility_score: f64,
}

/// UdonSharp performance ranking estimate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceRank {
    Excellent,
    Good,
    Medium,
    Poor,
    VeryPoor,
    Unknown,
}

impl Default for PerformanceRank {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Code quality metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    pub cyclomatic_complexity: f64,
    pub code_duplication_percentage: f64,
    pub optimization_opportunities: u32,
    pub potential_performance_issues: Vec<PerformanceIssue>,
    pub udonsharp_best_practices_score: f64,
}

/// Identified performance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceIssue {
    pub severity: IssueSeverity,
    pub category: IssueCategory,
    pub description: String,
    pub location: Option<CodeLocation>,
    pub suggested_fix: Option<String>,
    pub estimated_impact: ImpactLevel,
}

/// Severity level of a performance issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Category of performance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Memory,
    CPU,
    Network,
    UdonSharpSpecific,
    VRChatLimits,
    CodeStructure,
}

/// Location in source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub function: Option<String>,
}

/// Estimated impact level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
    Negligible,
}

impl CompilationMetrics {
    /// Calculate compilation speed in lines of code per second
    pub fn calculate_compilation_speed(&mut self) {
        if self.total_compilation_time.as_secs_f64() > 0.0 {
            self.compilation_speed_loc_per_second = 
                self.rust_lines_of_code as f64 / self.total_compilation_time.as_secs_f64();
        }
    }

    /// Calculate WASM compression ratio
    pub fn calculate_wasm_compression_ratio(&mut self, original_size: u64) {
        if original_size > 0 {
            self.wasm_compression_ratio = self.wasm_size_bytes as f64 / original_size as f64;
        }
    }

    /// Calculate C# code expansion ratio
    pub fn calculate_csharp_expansion_ratio(&mut self) {
        if self.rust_lines_of_code > 0 {
            self.csharp_expansion_ratio = 
                self.generated_csharp_lines as f64 / self.rust_lines_of_code as f64;
        }
    }

    /// Get compilation efficiency score (0.0 to 1.0)
    pub fn get_efficiency_score(&self) -> f64 {
        let speed_score = (self.compilation_speed_loc_per_second / 1000.0).min(1.0);
        let size_score = (1.0 - self.wasm_compression_ratio).max(0.0);
        let expansion_score = (1.0 / self.csharp_expansion_ratio.max(1.0)).min(1.0);
        
        (speed_score + size_score + expansion_score) / 3.0
    }
}

impl MemoryMetrics {
    /// Calculate memory efficiency score (0.0 to 1.0)
    pub fn get_efficiency_score(&self) -> f64 {
        if self.peak_usage == 0 {
            return 1.0;
        }

        let growth_ratio = self.memory_growth as f64 / self.peak_usage as f64;
        let efficiency = (1.0 - growth_ratio.min(1.0)).max(0.0);
        
        // Penalize high allocation rates
        let allocation_penalty = (self.allocation_rate / 1_000_000.0).min(0.5); // 1MB/s threshold
        
        (efficiency - allocation_penalty).max(0.0)
    }

    /// Get memory usage trend
    pub fn get_usage_trend(&self) -> MemoryTrend {
        if self.memory_growth == 0 {
            MemoryTrend::Stable
        } else if self.memory_growth < self.initial_usage / 10 {
            MemoryTrend::SlightIncrease
        } else if self.memory_growth < self.initial_usage / 2 {
            MemoryTrend::ModerateIncrease
        } else {
            MemoryTrend::HighIncrease
        }
    }
}

/// Memory usage trend classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryTrend {
    Stable,
    SlightIncrease,
    ModerateIncrease,
    HighIncrease,
    Decreasing,
}

impl VRChatMetrics {
    /// Estimate VRChat performance rank based on metrics
    pub fn estimate_performance_rank(&mut self) {
        let instruction_score = match self.estimated_instruction_count {
            0..=10_000 => 5,
            10_001..=50_000 => 4,
            50_001..=100_000 => 3,
            100_001..=500_000 => 2,
            _ => 1,
        };

        let memory_score = match self.estimated_memory_footprint {
            0..=1_000_000 => 5,      // < 1MB
            1_000_001..=10_000_000 => 4,   // < 10MB
            10_000_001..=50_000_000 => 3,  // < 50MB
            50_000_001..=100_000_000 => 2, // < 100MB
            _ => 1,
        };

        let sync_score = match self.network_sync_variables {
            0..=10 => 5,
            11..=50 => 4,
            51..=100 => 3,
            101..=200 => 2,
            _ => 1,
        };

        let total_score = (instruction_score + memory_score + sync_score) as f64 / 3.0;

        self.performance_rank_estimate = match total_score {
            4.5..=5.0 => PerformanceRank::Excellent,
            3.5..=4.4 => PerformanceRank::Good,
            2.5..=3.4 => PerformanceRank::Medium,
            1.5..=2.4 => PerformanceRank::Poor,
            _ => PerformanceRank::VeryPoor,
        };

        self.vrchat_compatibility_score = total_score / 5.0;
    }

    /// Check if metrics exceed VRChat limits
    pub fn check_vrchat_limits(&self) -> Vec<PerformanceIssue> {
        let mut issues = Vec::new();

        // Check instruction count limits (approximate)
        if self.estimated_instruction_count > 1_000_000 {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::VRChatLimits,
                description: "Estimated instruction count exceeds VRChat limits".to_string(),
                location: None,
                suggested_fix: Some("Reduce code complexity or split into multiple behaviors".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }

        // Check memory limits
        if self.estimated_memory_footprint > 100_000_000 { // 100MB
            issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::VRChatLimits,
                description: "Estimated memory usage is very high for VRChat".to_string(),
                location: None,
                suggested_fix: Some("Optimize data structures and reduce memory allocations".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }

        // Check sync variable limits
        if self.network_sync_variables > 200 {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Network,
                description: "High number of network sync variables may impact performance".to_string(),
                location: None,
                suggested_fix: Some("Reduce sync variables or use manual sync mode".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }

        issues
    }
}

impl PerformanceMetrics {
    /// Get overall performance score (0.0 to 1.0)
    pub fn get_overall_score(&self) -> f64 {
        let compilation_score = self.compilation.get_efficiency_score();
        let memory_score = self.memory.get_efficiency_score();
        let success_rate = 1.0 - (self.failed_steps as f64 / self.step_count.max(1) as f64);
        
        (compilation_score + memory_score + success_rate) / 3.0
    }

    /// Get performance grade
    pub fn get_performance_grade(&self) -> PerformanceGrade {
        let score = self.get_overall_score();
        match score {
            0.9..=1.0 => PerformanceGrade::A,
            0.8..=0.89 => PerformanceGrade::B,
            0.7..=0.79 => PerformanceGrade::C,
            0.6..=0.69 => PerformanceGrade::D,
            _ => PerformanceGrade::F,
        }
    }
}

/// Performance grade classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PerformanceGrade {
    A, // Excellent
    B, // Good
    C, // Average
    D, // Below Average
    F, // Poor
}