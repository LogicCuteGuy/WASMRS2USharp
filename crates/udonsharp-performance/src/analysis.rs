//! Performance analysis and optimization recommendation system

use crate::metrics::{PerformanceMetrics, CompilationMetrics, MemoryMetrics, VRChatMetrics, IssueSeverity};
use crate::profiler::CodeAnalysis;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Performance analyzer that provides optimization recommendations
#[derive(Debug)]
pub struct PerformanceAnalyzer {
    thresholds: PerformanceThresholds,
    recommendation_rules: Vec<RecommendationRule>,
}

/// Optimization recommendation with actionable advice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub category: RecommendationCategory,
    pub priority: RecommendationPriority,
    pub title: String,
    pub description: String,
    pub rationale: String,
    pub implementation_steps: Vec<String>,
    pub estimated_improvement: f64, // percentage
    pub estimated_effort: EffortLevel,
    pub prerequisites: Vec<String>,
    pub related_metrics: Vec<String>,
}

/// Category of optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationCategory {
    CompilationSpeed,
    MemoryUsage,
    CodeQuality,
    UdonSharpOptimization,
    VRChatPerformance,
    BuildProcess,
    Architecture,
}

/// Priority level of recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
    Optional,
}

/// Effort level required to implement recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EffortLevel {
    Minimal,    // < 1 hour
    Low,        // 1-4 hours
    Medium,     // 1-2 days
    High,       // 1 week
    Extensive,  // > 1 week
}

/// Performance thresholds for triggering recommendations
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    pub compilation_time_warning: f64,      // seconds
    pub compilation_time_critical: f64,     // seconds
    pub memory_usage_warning: u64,          // bytes
    pub memory_usage_critical: u64,         // bytes
    pub memory_growth_warning: f64,         // percentage
    pub code_complexity_warning: f64,
    pub code_complexity_critical: f64,
    pub compilation_speed_warning: f64,     // LOC per second
}

/// Rule for generating recommendations based on metrics
#[derive(Debug, Clone)]
pub struct RecommendationRule {
    pub name: String,
    pub condition: RuleCondition,
    pub recommendation_template: OptimizationRecommendation,
}

/// Condition that triggers a recommendation rule
#[derive(Debug, Clone)]
pub enum RuleCondition {
    CompilationTimeExceeds(f64),
    MemoryUsageExceeds(u64),
    MemoryGrowthExceeds(f64),
    ComplexityExceeds(f64),
    CompilationSpeedBelow(f64),
    FailureRateExceeds(f64),
    Custom(fn(&PerformanceMetrics) -> bool),
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer with default thresholds
    pub fn new() -> Self {
        let thresholds = PerformanceThresholds {
            compilation_time_warning: 30.0,    // 30 seconds
            compilation_time_critical: 120.0,  // 2 minutes
            memory_usage_warning: 500 * 1024 * 1024,      // 500MB
            memory_usage_critical: 1024 * 1024 * 1024,    // 1GB
            memory_growth_warning: 50.0,        // 50% growth
            code_complexity_warning: 10.0,
            code_complexity_critical: 20.0,
            compilation_speed_warning: 100.0,   // 100 LOC/sec
        };

        let mut analyzer = Self {
            thresholds,
            recommendation_rules: Vec::new(),
        };

        analyzer.initialize_default_rules();
        analyzer
    }

    /// Analyze performance metrics and generate recommendations
    pub fn analyze_metrics(&self, metrics: &PerformanceMetrics) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Apply all recommendation rules
        for rule in &self.recommendation_rules {
            if self.evaluate_rule_condition(&rule.condition, metrics) {
                let mut recommendation = rule.recommendation_template.clone();
                self.customize_recommendation(&mut recommendation, metrics);
                recommendations.push(recommendation);
            }
        }

        // Add specific analysis-based recommendations
        recommendations.extend(self.analyze_compilation_performance(&metrics.compilation)?);
        recommendations.extend(self.analyze_memory_performance(&metrics.memory)?);
        recommendations.extend(self.analyze_overall_performance(metrics)?);

        // Sort by priority and estimated improvement
        recommendations.sort_by(|a, b| {
            a.priority.cmp(&b.priority)
                .then_with(|| b.estimated_improvement.partial_cmp(&a.estimated_improvement).unwrap_or(std::cmp::Ordering::Equal))
        });

        Ok(recommendations)
    }

    /// Analyze code quality and generate recommendations
    pub fn analyze_code_quality(&self, analyses: &[CodeAnalysis]) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Analyze complexity across all files
        let avg_complexity = analyses.iter()
            .map(|a| a.complexity_score)
            .sum::<f64>() / analyses.len().max(1) as f64;

        if avg_complexity > self.thresholds.code_complexity_critical {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::CodeQuality,
                priority: RecommendationPriority::High,
                title: "Reduce Code Complexity".to_string(),
                description: format!("Average code complexity ({:.1}) exceeds recommended threshold", avg_complexity),
                rationale: "High complexity makes code harder to maintain and can impact compilation performance".to_string(),
                implementation_steps: vec![
                    "Break down complex functions into smaller, focused functions".to_string(),
                    "Reduce nested conditionals using early returns".to_string(),
                    "Consider using pattern matching instead of complex if-else chains".to_string(),
                    "Extract common logic into helper functions".to_string(),
                ],
                estimated_improvement: 15.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Code review and refactoring plan".to_string()],
                related_metrics: vec!["cyclomatic_complexity".to_string()],
            });
        }

        // Analyze UdonSharp compatibility
        let avg_compatibility = analyses.iter()
            .map(|a| a.udonsharp_compatibility.compatibility_score)
            .sum::<f64>() / analyses.len().max(1) as f64;

        if avg_compatibility < 0.8 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::UdonSharpOptimization,
                priority: RecommendationPriority::High,
                title: "Improve UdonSharp Compatibility".to_string(),
                description: format!("UdonSharp compatibility score ({:.1}%) is below recommended threshold", avg_compatibility * 100.0),
                rationale: "Low compatibility may cause runtime issues or prevent compilation".to_string(),
                implementation_steps: vec![
                    "Review and fix unsupported language features".to_string(),
                    "Use UdonSharp-compatible alternatives for problematic code".to_string(),
                    "Add proper UdonSharp attributes where needed".to_string(),
                ],
                estimated_improvement: 25.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["UdonSharp documentation review".to_string()],
                related_metrics: vec!["udonsharp_compatibility_score".to_string()],
            });
        }

        // Analyze performance issues
        let critical_issues = analyses.iter()
            .flat_map(|a| &a.performance_issues)
            .filter(|issue| issue.severity == IssueSeverity::Critical)
            .count();

        if critical_issues > 0 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::VRChatPerformance,
                priority: RecommendationPriority::Critical,
                title: "Fix Critical Performance Issues".to_string(),
                description: format!("Found {} critical performance issues that need immediate attention", critical_issues),
                rationale: "Critical issues can cause world rejection or poor user experience".to_string(),
                implementation_steps: vec![
                    "Review all critical performance issues in the analysis report".to_string(),
                    "Prioritize fixes based on impact and difficulty".to_string(),
                    "Test performance improvements after each fix".to_string(),
                ],
                estimated_improvement: 40.0,
                estimated_effort: EffortLevel::High,
                prerequisites: vec!["Detailed performance issue analysis".to_string()],
                related_metrics: vec!["critical_performance_issues".to_string()],
            });
        }

        Ok(recommendations)
    }

    /// Generate VRChat-specific optimization recommendations
    pub fn analyze_vrchat_performance(&self, vrchat_metrics: &VRChatMetrics) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Check instruction count
        if vrchat_metrics.estimated_instruction_count > 500_000 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::VRChatPerformance,
                priority: if vrchat_metrics.estimated_instruction_count > 1_000_000 {
                    RecommendationPriority::Critical
                } else {
                    RecommendationPriority::High
                },
                title: "Optimize Instruction Count".to_string(),
                description: format!("Estimated instruction count ({}) may exceed VRChat limits", vrchat_metrics.estimated_instruction_count),
                rationale: "High instruction counts can cause world rejection or poor performance".to_string(),
                implementation_steps: vec![
                    "Profile code to identify instruction-heavy operations".to_string(),
                    "Optimize loops and recursive functions".to_string(),
                    "Consider splitting complex behaviors into multiple scripts".to_string(),
                    "Use more efficient algorithms where possible".to_string(),
                ],
                estimated_improvement: 30.0,
                estimated_effort: EffortLevel::High,
                prerequisites: vec!["VRChat performance profiling".to_string()],
                related_metrics: vec!["estimated_instruction_count".to_string()],
            });
        }

        // Check memory footprint
        if vrchat_metrics.estimated_memory_footprint > 50_000_000 { // 50MB
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::MemoryUsage,
                priority: RecommendationPriority::Medium,
                title: "Reduce Memory Footprint".to_string(),
                description: format!("Estimated memory usage ({:.1}MB) is high for VRChat", vrchat_metrics.estimated_memory_footprint as f64 / 1_000_000.0),
                rationale: "High memory usage can impact world performance and user experience".to_string(),
                implementation_steps: vec![
                    "Audit data structures for memory efficiency".to_string(),
                    "Remove unused variables and collections".to_string(),
                    "Use object pooling for frequently created/destroyed objects".to_string(),
                    "Optimize texture and mesh data if applicable".to_string(),
                ],
                estimated_improvement: 20.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Memory profiling analysis".to_string()],
                related_metrics: vec!["estimated_memory_footprint".to_string()],
            });
        }

        // Check sync variables
        if vrchat_metrics.network_sync_variables > 100 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::VRChatPerformance,
                priority: RecommendationPriority::Medium,
                title: "Optimize Network Sync Variables".to_string(),
                description: format!("High number of sync variables ({}) may impact network performance", vrchat_metrics.network_sync_variables),
                rationale: "Too many sync variables can cause network congestion and poor multiplayer experience".to_string(),
                implementation_steps: vec![
                    "Review which variables actually need network synchronization".to_string(),
                    "Combine related variables into structs or use manual sync mode".to_string(),
                    "Use events instead of continuous sync where appropriate".to_string(),
                    "Implement custom serialization for complex data".to_string(),
                ],
                estimated_improvement: 15.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Network architecture review".to_string()],
                related_metrics: vec!["network_sync_variables".to_string()],
            });
        }

        Ok(recommendations)
    }

    /// Initialize default recommendation rules
    fn initialize_default_rules(&mut self) {
        // Compilation time rules
        self.recommendation_rules.push(RecommendationRule {
            name: "slow_compilation".to_string(),
            condition: RuleCondition::CompilationTimeExceeds(self.thresholds.compilation_time_warning),
            recommendation_template: OptimizationRecommendation {
                category: RecommendationCategory::CompilationSpeed,
                priority: RecommendationPriority::Medium,
                title: "Improve Compilation Speed".to_string(),
                description: "Compilation time is slower than expected".to_string(),
                rationale: "Slow compilation impacts development productivity".to_string(),
                implementation_steps: vec![
                    "Enable incremental compilation".to_string(),
                    "Reduce dependencies and imports".to_string(),
                    "Use parallel compilation if available".to_string(),
                    "Consider splitting large files".to_string(),
                ],
                estimated_improvement: 25.0,
                estimated_effort: EffortLevel::Low,
                prerequisites: vec!["Build system analysis".to_string()],
                related_metrics: vec!["total_compilation_time".to_string()],
            },
        });

        // Memory usage rules
        self.recommendation_rules.push(RecommendationRule {
            name: "high_memory_usage".to_string(),
            condition: RuleCondition::MemoryUsageExceeds(self.thresholds.memory_usage_warning),
            recommendation_template: OptimizationRecommendation {
                category: RecommendationCategory::MemoryUsage,
                priority: RecommendationPriority::High,
                title: "Reduce Memory Usage".to_string(),
                description: "Memory usage during compilation is high".to_string(),
                rationale: "High memory usage can slow compilation and cause system instability".to_string(),
                implementation_steps: vec![
                    "Profile memory usage to identify hotspots".to_string(),
                    "Optimize data structures and algorithms".to_string(),
                    "Implement memory pooling where appropriate".to_string(),
                    "Reduce temporary allocations".to_string(),
                ],
                estimated_improvement: 20.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Memory profiling tools".to_string()],
                related_metrics: vec!["peak_memory_usage".to_string()],
            },
        });

        // Compilation speed rules
        self.recommendation_rules.push(RecommendationRule {
            name: "low_compilation_speed".to_string(),
            condition: RuleCondition::CompilationSpeedBelow(self.thresholds.compilation_speed_warning),
            recommendation_template: OptimizationRecommendation {
                category: RecommendationCategory::CompilationSpeed,
                priority: RecommendationPriority::Medium,
                title: "Optimize Compilation Pipeline".to_string(),
                description: "Compilation speed is below optimal levels".to_string(),
                rationale: "Slow compilation reduces development efficiency".to_string(),
                implementation_steps: vec![
                    "Analyze compilation bottlenecks".to_string(),
                    "Optimize build configuration".to_string(),
                    "Use faster compilation targets where possible".to_string(),
                    "Implement build caching".to_string(),
                ],
                estimated_improvement: 30.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Compilation profiling".to_string()],
                related_metrics: vec!["compilation_speed_loc_per_second".to_string()],
            },
        });
    }

    /// Evaluate if a rule condition is met
    fn evaluate_rule_condition(&self, condition: &RuleCondition, metrics: &PerformanceMetrics) -> bool {
        match condition {
            RuleCondition::CompilationTimeExceeds(threshold) => {
                metrics.compilation.total_compilation_time.as_secs_f64() > *threshold
            }
            RuleCondition::MemoryUsageExceeds(threshold) => {
                metrics.memory.peak_usage > *threshold
            }
            RuleCondition::MemoryGrowthExceeds(threshold) => {
                let growth_percentage = if metrics.memory.initial_usage > 0 {
                    (metrics.memory.memory_growth as f64 / metrics.memory.initial_usage as f64) * 100.0
                } else {
                    0.0
                };
                growth_percentage > *threshold
            }
            RuleCondition::ComplexityExceeds(_threshold) => {
                // Would need code analysis data
                false
            }
            RuleCondition::CompilationSpeedBelow(threshold) => {
                metrics.compilation.compilation_speed_loc_per_second < *threshold
            }
            RuleCondition::FailureRateExceeds(threshold) => {
                let failure_rate = if metrics.step_count > 0 {
                    metrics.failed_steps as f64 / metrics.step_count as f64
                } else {
                    0.0
                };
                failure_rate > *threshold
            }
            RuleCondition::Custom(func) => func(metrics),
        }
    }

    /// Customize recommendation based on specific metrics
    fn customize_recommendation(&self, recommendation: &mut OptimizationRecommendation, metrics: &PerformanceMetrics) {
        // Add specific metric values to description
        match recommendation.category {
            RecommendationCategory::CompilationSpeed => {
                recommendation.description = format!(
                    "{} (Current: {:.1}s, Speed: {:.1} LOC/s)",
                    recommendation.description,
                    metrics.compilation.total_compilation_time.as_secs_f64(),
                    metrics.compilation.compilation_speed_loc_per_second
                );
            }
            RecommendationCategory::MemoryUsage => {
                recommendation.description = format!(
                    "{} (Peak: {:.1}MB, Growth: {:.1}MB)",
                    recommendation.description,
                    metrics.memory.peak_usage as f64 / 1_000_000.0,
                    metrics.memory.memory_growth as f64 / 1_000_000.0
                );
            }
            _ => {}
        }
    }

    /// Analyze compilation performance specifically
    fn analyze_compilation_performance(&self, compilation: &CompilationMetrics) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Check for slow WASM optimization
        if compilation.wasm_optimization_time.as_secs_f64() > compilation.total_compilation_time.as_secs_f64() * 0.5 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::CompilationSpeed,
                priority: RecommendationPriority::Medium,
                title: "Optimize WASM Optimization Step".to_string(),
                description: "WASM optimization is taking a large portion of compilation time".to_string(),
                rationale: "Excessive optimization time may indicate inefficient optimization settings".to_string(),
                implementation_steps: vec![
                    "Review WASM optimization level settings".to_string(),
                    "Consider using faster optimization modes for development".to_string(),
                    "Profile WASM optimization passes".to_string(),
                ],
                estimated_improvement: 20.0,
                estimated_effort: EffortLevel::Low,
                prerequisites: vec!["WASM optimization configuration review".to_string()],
                related_metrics: vec!["wasm_optimization_time".to_string()],
            });
        }

        // Check for inefficient C# generation
        if compilation.csharp_expansion_ratio > 5.0 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::CodeQuality,
                priority: RecommendationPriority::Low,
                title: "Optimize C# Code Generation".to_string(),
                description: format!("C# expansion ratio ({:.1}x) is high", compilation.csharp_expansion_ratio),
                rationale: "High expansion ratios may indicate inefficient code generation".to_string(),
                implementation_steps: vec![
                    "Review C# code generation templates".to_string(),
                    "Optimize generated code patterns".to_string(),
                    "Remove unnecessary generated code".to_string(),
                ],
                estimated_improvement: 10.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Code generation analysis".to_string()],
                related_metrics: vec!["csharp_expansion_ratio".to_string()],
            });
        }

        Ok(recommendations)
    }

    /// Analyze memory performance specifically
    fn analyze_memory_performance(&self, memory: &MemoryMetrics) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        // Check for high allocation rate
        if memory.allocation_rate > 10_000_000.0 { // 10MB/s
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::MemoryUsage,
                priority: RecommendationPriority::High,
                title: "Reduce Memory Allocation Rate".to_string(),
                description: format!("High allocation rate ({:.1}MB/s) detected", memory.allocation_rate / 1_000_000.0),
                rationale: "High allocation rates can cause GC pressure and performance issues".to_string(),
                implementation_steps: vec![
                    "Profile allocation hotspots".to_string(),
                    "Implement object pooling".to_string(),
                    "Reduce temporary object creation".to_string(),
                    "Use stack allocation where possible".to_string(),
                ],
                estimated_improvement: 25.0,
                estimated_effort: EffortLevel::Medium,
                prerequisites: vec!["Memory allocation profiling".to_string()],
                related_metrics: vec!["allocation_rate".to_string()],
            });
        }

        Ok(recommendations)
    }

    /// Analyze overall performance
    fn analyze_overall_performance(&self, metrics: &PerformanceMetrics) -> Result<Vec<OptimizationRecommendation>> {
        let mut recommendations = Vec::new();

        let overall_score = metrics.get_overall_score();
        
        if overall_score < 0.6 {
            recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::Architecture,
                priority: RecommendationPriority::High,
                title: "Comprehensive Performance Review".to_string(),
                description: format!("Overall performance score ({:.1}%) indicates significant optimization opportunities", overall_score * 100.0),
                rationale: "Low overall performance affects development productivity and user experience".to_string(),
                implementation_steps: vec![
                    "Conduct comprehensive performance audit".to_string(),
                    "Prioritize high-impact optimizations".to_string(),
                    "Implement performance monitoring".to_string(),
                    "Establish performance benchmarks".to_string(),
                ],
                estimated_improvement: 50.0,
                estimated_effort: EffortLevel::Extensive,
                prerequisites: vec!["Performance audit plan".to_string()],
                related_metrics: vec!["overall_performance_score".to_string()],
            });
        }

        Ok(recommendations)
    }
}

impl Default for PerformanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}