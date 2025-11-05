//! Multi-behavior specific performance metrics and analysis

use crate::metrics::{PerformanceIssue, IssueSeverity, IssueCategory, ImpactLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Performance metrics specific to multi-behavior compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiBehaviorMetrics {
    /// Total number of behaviors generated
    pub behavior_count: usize,
    
    /// Number of shared functions moved to SharedRuntime
    pub shared_functions_count: usize,
    
    /// Inter-behavior communication calls
    pub inter_behavior_calls: usize,
    
    /// Code sharing efficiency (0.0 to 1.0)
    pub code_sharing_efficiency: f64,
    
    /// Dependency analysis metrics
    pub dependency_metrics: DependencyMetrics,
    
    /// Generated file metrics
    pub file_generation_metrics: FileGenerationMetrics,
    
    /// Behavior-specific metrics
    pub behavior_metrics: HashMap<String, BehaviorMetrics>,
    
    /// Optimization metrics
    pub optimization_metrics: OptimizationMetrics,
    
    /// Performance issues found
    pub performance_issues: Vec<PerformanceIssue>,
}

/// Dependency analysis performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetrics {
    /// Time spent analyzing dependencies
    pub analysis_time: Duration,
    
    /// Number of dependency edges analyzed
    pub dependency_edges: usize,
    
    /// Circular dependencies detected
    pub circular_dependencies: usize,
    
    /// Dependency depth (maximum chain length)
    pub max_dependency_depth: usize,
    
    /// Average dependency depth
    pub avg_dependency_depth: f64,
    
    /// Dependency complexity score (0.0 to 1.0)
    pub complexity_score: f64,
}

/// File generation performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileGenerationMetrics {
    /// Time spent generating behavior files
    pub behavior_generation_time: Duration,
    
    /// Time spent generating SharedRuntime
    pub shared_runtime_generation_time: Duration,
    
    /// Time spent generating prefabs
    pub prefab_generation_time: Duration,
    
    /// Time spent generating coordinator
    pub coordinator_generation_time: Duration,
    
    /// Total generated file size in bytes
    pub total_file_size: u64,
    
    /// Number of generated files
    pub file_count: usize,
    
    /// Average file size
    pub avg_file_size: u64,
    
    /// Code generation efficiency (lines per second)
    pub generation_efficiency: f64,
}

/// Metrics for individual behaviors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorMetrics {
    /// Behavior name
    pub name: String,
    
    /// Number of functions in this behavior
    pub function_count: usize,
    
    /// Lines of generated C# code
    pub generated_lines: usize,
    
    /// Number of Unity events handled
    pub unity_events: usize,
    
    /// Number of custom methods
    pub custom_methods: usize,
    
    /// Dependencies on other behaviors
    pub dependencies: Vec<String>,
    
    /// Functions called from other behaviors
    pub incoming_calls: usize,
    
    /// Functions calling other behaviors
    pub outgoing_calls: usize,
    
    /// Estimated memory footprint
    pub estimated_memory: u64,
    
    /// Complexity score (0.0 to 1.0)
    pub complexity_score: f64,
}

/// Code optimization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {
    /// Code duplication eliminated (percentage)
    pub duplication_eliminated: f64,
    
    /// Functions moved to SharedRuntime
    pub functions_shared: usize,
    
    /// Estimated memory savings from sharing
    pub memory_savings_bytes: u64,
    
    /// Estimated performance improvement
    pub performance_improvement: f64,
    
    /// Optimization opportunities identified
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    
    /// Code sharing recommendations
    pub sharing_recommendations: Vec<SharingRecommendation>,
}

/// Identified optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    
    /// Description of the opportunity
    pub description: String,
    
    /// Estimated improvement percentage
    pub estimated_improvement: f64,
    
    /// Implementation difficulty
    pub difficulty: OptimizationDifficulty,
    
    /// Affected behaviors
    pub affected_behaviors: Vec<String>,
    
    /// Implementation steps
    pub implementation_steps: Vec<String>,
}

/// Types of optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    CodeSharing,
    DependencyReduction,
    MemoryOptimization,
    CommunicationOptimization,
    InitializationOptimization,
    NetworkOptimization,
}

/// Optimization implementation difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationDifficulty {
    Easy,
    Medium,
    Hard,
    VeryHard,
}

/// Code sharing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingRecommendation {
    /// Functions that could be shared
    pub functions: Vec<String>,
    
    /// Behaviors that would benefit
    pub benefiting_behaviors: Vec<String>,
    
    /// Estimated memory savings
    pub memory_savings: u64,
    
    /// Estimated performance improvement
    pub performance_improvement: f64,
    
    /// Implementation complexity
    pub complexity: SharingComplexity,
}

/// Code sharing complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharingComplexity {
    Simple,      // Direct function move
    Moderate,    // Requires parameter changes
    Complex,     // Requires significant refactoring
    VeryComplex, // May not be worth it
}

impl MultiBehaviorMetrics {
    /// Create new multi-behavior metrics
    pub fn new() -> Self {
        Self {
            behavior_count: 0,
            shared_functions_count: 0,
            inter_behavior_calls: 0,
            code_sharing_efficiency: 0.0,
            dependency_metrics: DependencyMetrics::default(),
            file_generation_metrics: FileGenerationMetrics::default(),
            behavior_metrics: HashMap::new(),
            optimization_metrics: OptimizationMetrics::default(),
            performance_issues: Vec::new(),
        }
    }
    
    /// Calculate code sharing efficiency
    pub fn calculate_sharing_efficiency(&mut self) {
        if self.behavior_count == 0 {
            self.code_sharing_efficiency = 0.0;
            return;
        }
        
        let total_functions: usize = self.behavior_metrics.values()
            .map(|b| b.function_count)
            .sum();
        
        if total_functions == 0 {
            self.code_sharing_efficiency = 0.0;
            return;
        }
        
        // Efficiency is the ratio of shared functions to total functions
        self.code_sharing_efficiency = self.shared_functions_count as f64 / total_functions as f64;
    }
    
    /// Get overall multi-behavior performance score
    pub fn get_performance_score(&self) -> f64 {
        let sharing_score = self.code_sharing_efficiency;
        let dependency_score = 1.0 - self.dependency_metrics.complexity_score;
        let optimization_score = self.optimization_metrics.performance_improvement / 100.0;
        let issue_penalty = (self.performance_issues.len() as f64 * 0.1).min(0.5);
        
        ((sharing_score + dependency_score + optimization_score) / 3.0 - issue_penalty).max(0.0)
    }
    
    /// Analyze multi-behavior performance issues
    pub fn analyze_performance_issues(&mut self) {
        self.performance_issues.clear();
        
        // Check for excessive behavior count
        if self.behavior_count > 20 {
            self.performance_issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::CodeStructure,
                description: format!("High number of behaviors ({}). Consider consolidating related functionality.", self.behavior_count),
                location: None,
                suggested_fix: Some("Group related behaviors or use composition patterns".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        // Check for low code sharing efficiency
        if self.code_sharing_efficiency < 0.2 && self.behavior_count > 2 {
            self.performance_issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Memory,
                description: format!("Low code sharing efficiency ({:.1}%). Significant code duplication detected.", self.code_sharing_efficiency * 100.0),
                location: None,
                suggested_fix: Some("Move common functions to SharedRuntime or refactor duplicate code".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }
        
        // Check for circular dependencies
        if self.dependency_metrics.circular_dependencies > 0 {
            self.performance_issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::CodeStructure,
                description: format!("Circular dependencies detected ({}). This can cause initialization issues.", self.dependency_metrics.circular_dependencies),
                location: None,
                suggested_fix: Some("Refactor dependencies to eliminate cycles or use dependency injection".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }
        
        // Check for excessive inter-behavior communication
        if self.inter_behavior_calls > self.behavior_count * 10 {
            self.performance_issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::CPU,
                description: format!("High inter-behavior communication ({} calls). May impact performance.", self.inter_behavior_calls),
                location: None,
                suggested_fix: Some("Reduce cross-behavior calls or batch communications".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        // Check individual behavior complexity
        for (name, metrics) in &self.behavior_metrics {
            if metrics.complexity_score > 0.8 {
                self.performance_issues.push(PerformanceIssue {
                    severity: IssueSeverity::Medium,
                    category: IssueCategory::CodeStructure,
                    description: format!("Behavior '{}' has high complexity ({:.1}). Consider splitting.", name, metrics.complexity_score * 100.0),
                    location: None,
                    suggested_fix: Some("Split complex behavior into smaller, focused behaviors".to_string()),
                    estimated_impact: ImpactLevel::Medium,
                });
            }
        }
    }
    
    /// Generate optimization recommendations
    pub fn generate_optimization_recommendations(&self) -> Vec<OptimizationOpportunity> {
        let mut opportunities = Vec::new();
        
        // Code sharing opportunities
        if self.code_sharing_efficiency < 0.5 {
            opportunities.push(OptimizationOpportunity {
                optimization_type: OptimizationType::CodeSharing,
                description: "Increase code sharing by moving common functions to SharedRuntime".to_string(),
                estimated_improvement: (0.5 - self.code_sharing_efficiency) * 100.0,
                difficulty: OptimizationDifficulty::Medium,
                affected_behaviors: self.behavior_metrics.keys().cloned().collect(),
                implementation_steps: vec![
                    "Identify duplicate or similar functions across behaviors".to_string(),
                    "Extract common functionality to shared functions".to_string(),
                    "Update behavior code to use shared functions".to_string(),
                    "Test inter-behavior communication".to_string(),
                ],
            });
        }
        
        // Dependency optimization
        if self.dependency_metrics.complexity_score > 0.6 {
            opportunities.push(OptimizationOpportunity {
                optimization_type: OptimizationType::DependencyReduction,
                description: "Simplify dependency structure to improve initialization and maintainability".to_string(),
                estimated_improvement: self.dependency_metrics.complexity_score * 30.0,
                difficulty: OptimizationDifficulty::Hard,
                affected_behaviors: self.behavior_metrics.keys().cloned().collect(),
                implementation_steps: vec![
                    "Analyze dependency graph for simplification opportunities".to_string(),
                    "Refactor tightly coupled behaviors".to_string(),
                    "Introduce dependency injection where appropriate".to_string(),
                    "Update initialization order".to_string(),
                ],
            });
        }
        
        // Communication optimization
        if self.inter_behavior_calls > self.behavior_count * 5 {
            opportunities.push(OptimizationOpportunity {
                optimization_type: OptimizationType::CommunicationOptimization,
                description: "Optimize inter-behavior communication to reduce overhead".to_string(),
                estimated_improvement: 15.0,
                difficulty: OptimizationDifficulty::Medium,
                affected_behaviors: self.behavior_metrics.keys().cloned().collect(),
                implementation_steps: vec![
                    "Identify frequently called cross-behavior methods".to_string(),
                    "Batch multiple calls where possible".to_string(),
                    "Use direct references instead of SendCustomEvent where safe".to_string(),
                    "Cache results of expensive cross-behavior calls".to_string(),
                ],
            });
        }
        
        opportunities
    }
}

impl Default for DependencyMetrics {
    fn default() -> Self {
        Self {
            analysis_time: Duration::default(),
            dependency_edges: 0,
            circular_dependencies: 0,
            max_dependency_depth: 0,
            avg_dependency_depth: 0.0,
            complexity_score: 0.0,
        }
    }
}

impl Default for FileGenerationMetrics {
    fn default() -> Self {
        Self {
            behavior_generation_time: Duration::default(),
            shared_runtime_generation_time: Duration::default(),
            prefab_generation_time: Duration::default(),
            coordinator_generation_time: Duration::default(),
            total_file_size: 0,
            file_count: 0,
            avg_file_size: 0,
            generation_efficiency: 0.0,
        }
    }
}

impl Default for OptimizationMetrics {
    fn default() -> Self {
        Self {
            duplication_eliminated: 0.0,
            functions_shared: 0,
            memory_savings_bytes: 0,
            performance_improvement: 0.0,
            optimization_opportunities: Vec::new(),
            sharing_recommendations: Vec::new(),
        }
    }
}

impl BehaviorMetrics {
    /// Calculate complexity score for this behavior
    pub fn calculate_complexity_score(&mut self) {
        let function_complexity = (self.function_count as f64 / 20.0).min(1.0);
        let dependency_complexity = (self.dependencies.len() as f64 / 10.0).min(1.0);
        let communication_complexity = ((self.incoming_calls + self.outgoing_calls) as f64 / 20.0).min(1.0);
        
        self.complexity_score = (function_complexity + dependency_complexity + communication_complexity) / 3.0;
    }
}

/// Multi-behavior performance analyzer
pub struct MultiBehaviorAnalyzer {
    metrics: MultiBehaviorMetrics,
}

impl MultiBehaviorAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self {
            metrics: MultiBehaviorMetrics::new(),
        }
    }
    
    /// Analyze multi-behavior compilation results
    pub fn analyze_compilation_result(
        &mut self,
        behavior_count: usize,
        shared_functions: &[String],
        inter_behavior_calls: usize,
        dependency_analysis_time: Duration,
        generation_times: &FileGenerationMetrics,
    ) -> &MultiBehaviorMetrics {
        self.metrics.behavior_count = behavior_count;
        self.metrics.shared_functions_count = shared_functions.len();
        self.metrics.inter_behavior_calls = inter_behavior_calls;
        self.metrics.dependency_metrics.analysis_time = dependency_analysis_time;
        self.metrics.file_generation_metrics = generation_times.clone();
        
        self.metrics.calculate_sharing_efficiency();
        self.metrics.analyze_performance_issues();
        
        &self.metrics
    }
    
    /// Add behavior-specific metrics
    pub fn add_behavior_metrics(&mut self, name: String, metrics: BehaviorMetrics) {
        self.metrics.behavior_metrics.insert(name, metrics);
    }
    
    /// Get current metrics
    pub fn get_metrics(&self) -> &MultiBehaviorMetrics {
        &self.metrics
    }
    
    /// Generate comprehensive report
    pub fn generate_report(&self) -> MultiBehaviorReport {
        MultiBehaviorReport {
            metrics: self.metrics.clone(),
            recommendations: self.metrics.generate_optimization_recommendations(),
            summary: self.generate_summary(),
        }
    }
    
    /// Generate performance summary
    fn generate_summary(&self) -> MultiBehaviorSummary {
        MultiBehaviorSummary {
            total_behaviors: self.metrics.behavior_count,
            shared_functions: self.metrics.shared_functions_count,
            sharing_efficiency: self.metrics.code_sharing_efficiency,
            performance_score: self.metrics.get_performance_score(),
            critical_issues: self.metrics.performance_issues.iter()
                .filter(|i| i.severity == IssueSeverity::Critical)
                .count(),
            optimization_opportunities: self.metrics.generate_optimization_recommendations().len(),
        }
    }
}

/// Multi-behavior performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiBehaviorReport {
    pub metrics: MultiBehaviorMetrics,
    pub recommendations: Vec<OptimizationOpportunity>,
    pub summary: MultiBehaviorSummary,
}

/// Multi-behavior performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiBehaviorSummary {
    pub total_behaviors: usize,
    pub shared_functions: usize,
    pub sharing_efficiency: f64,
    pub performance_score: f64,
    pub critical_issues: usize,
    pub optimization_opportunities: usize,
}

impl Default for MultiBehaviorAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}