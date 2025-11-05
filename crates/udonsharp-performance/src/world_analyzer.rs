//! VRChat world performance analysis tools

use crate::metrics::{VRChatMetrics, PerformanceIssue, IssueSeverity, IssueCategory, ImpactLevel, PerformanceRank};
use crate::profiler::OptimizationDifficulty;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// VRChat world performance analyzer
#[derive(Debug)]
pub struct VRChatWorldAnalyzer {
    performance_thresholds: VRChatPerformanceThresholds,
    optimization_rules: Vec<WorldOptimizationRule>,
}

/// VRChat performance thresholds for different ranks
#[derive(Debug, Clone)]
pub struct VRChatPerformanceThresholds {
    pub excellent_instruction_limit: u64,
    pub good_instruction_limit: u64,
    pub medium_instruction_limit: u64,
    pub poor_instruction_limit: u64,
    pub excellent_memory_limit: u64,
    pub good_memory_limit: u64,
    pub medium_memory_limit: u64,
    pub poor_memory_limit: u64,
    pub max_sync_variables: u32,
    pub max_audio_sources: u32,
    pub max_particle_systems: u32,
    pub max_lights: u32,
}

/// World optimization rule
#[derive(Debug, Clone)]
pub struct WorldOptimizationRule {
    pub name: String,
    pub category: WorldOptimizationCategory,
    pub condition: WorldCondition,
    pub recommendation: String,
    pub estimated_improvement: f64,
    pub difficulty: OptimizationDifficulty,
}

/// Category of world optimization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorldOptimizationCategory {
    InstructionCount,
    MemoryUsage,
    NetworkOptimization,
    PhysicsOptimization,
    RenderingOptimization,
    AudioOptimization,
    UIOptimization,
    GeneralPerformance,
}

/// Condition for world optimization rules
#[derive(Debug, Clone)]
pub enum WorldCondition {
    InstructionCountExceeds(u64),
    MemoryUsageExceeds(u64),
    SyncVariablesExceed(u32),
    AudioSourcesExceed(u32),
    ParticleSystemsExceed(u32),
    LightsExceed(u32),
    PerformanceRankBelow(PerformanceRank),
    Custom(fn(&VRChatWorldMetrics) -> bool),
}

/// Comprehensive VRChat world metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VRChatWorldMetrics {
    pub base_metrics: VRChatMetrics,
    pub world_complexity: WorldComplexityMetrics,
    pub resource_usage: WorldResourceUsage,
    pub optimization_opportunities: Vec<WorldOptimizationOpportunity>,
}

/// World complexity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldComplexityMetrics {
    pub total_gameobjects: u32,
    pub total_components: u32,
    pub total_scripts: u32,
    pub udonsharp_behaviors: u32,
    pub mesh_renderers: u32,
    pub colliders: u32,
    pub rigidbodies: u32,
    pub joints: u32,
    pub particle_systems: u32,
    pub audio_sources: u32,
    pub lights: u32,
    pub cameras: u32,
    pub ui_elements: u32,
    pub animation_controllers: u32,
}

/// World resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldResourceUsage {
    pub texture_memory: u64,
    pub mesh_memory: u64,
    pub audio_memory: u64,
    pub animation_memory: u64,
    pub script_memory: u64,
    pub total_vertices: u32,
    pub total_triangles: u32,
    pub draw_calls: u32,
    pub material_count: u32,
    pub shader_count: u32,
}

/// World-specific optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldOptimizationOpportunity {
    pub category: WorldOptimizationCategory,
    pub title: String,
    pub description: String,
    pub impact: ImpactLevel,
    pub difficulty: OptimizationDifficulty,
    pub estimated_improvement: f64,
    pub implementation_steps: Vec<String>,
    pub affected_objects: Vec<String>,
}

impl VRChatWorldAnalyzer {
    /// Create a new VRChat world analyzer
    pub fn new() -> Self {
        let mut analyzer = Self {
            performance_thresholds: VRChatPerformanceThresholds::default(),
            optimization_rules: Vec::new(),
        };
        
        analyzer.initialize_optimization_rules();
        analyzer
    }

    /// Analyze VRChat world performance
    pub fn analyze_world_performance(&self, metrics: &VRChatWorldMetrics) -> Result<WorldPerformanceAnalysis> {
        let mut issues = Vec::new();
        let mut opportunities = Vec::new();
        
        // Analyze instruction count
        issues.extend(self.analyze_instruction_count(&metrics.base_metrics)?);
        
        // Analyze memory usage
        issues.extend(self.analyze_memory_usage(&metrics.base_metrics, &metrics.resource_usage)?);
        
        // Analyze world complexity
        issues.extend(self.analyze_world_complexity(&metrics.world_complexity)?);
        
        // Analyze resource usage
        issues.extend(self.analyze_resource_usage(&metrics.resource_usage)?);
        
        // Generate optimization opportunities
        opportunities.extend(self.generate_optimization_opportunities(metrics)?);
        
        // Calculate overall performance score
        let performance_score = self.calculate_performance_score(metrics);
        let performance_rank = self.determine_performance_rank(&metrics.base_metrics);
        
        Ok(WorldPerformanceAnalysis {
            performance_score,
            performance_rank,
            issues,
            optimization_opportunities: opportunities,
            vrchat_compatibility: self.check_vrchat_compatibility(metrics)?,
            recommendations: self.generate_recommendations(metrics)?,
        })
    }

    /// Analyze instruction count specifically
    fn analyze_instruction_count(&self, metrics: &VRChatMetrics) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();
        
        if metrics.estimated_instruction_count > self.performance_thresholds.poor_instruction_limit {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::VRChatLimits,
                description: format!(
                    "Instruction count ({}) exceeds VRChat limits and will cause world rejection",
                    metrics.estimated_instruction_count
                ),
                location: None,
                suggested_fix: Some("Reduce code complexity, optimize algorithms, or split into multiple behaviors".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        } else if metrics.estimated_instruction_count > self.performance_thresholds.medium_instruction_limit {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::UdonSharpSpecific,
                description: format!(
                    "Instruction count ({}) will result in Poor performance rank",
                    metrics.estimated_instruction_count
                ),
                location: None,
                suggested_fix: Some("Optimize code to achieve better performance rank".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        } else if metrics.estimated_instruction_count > self.performance_thresholds.good_instruction_limit {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::UdonSharpSpecific,
                description: format!(
                    "Instruction count ({}) will result in Medium performance rank",
                    metrics.estimated_instruction_count
                ),
                location: None,
                suggested_fix: Some("Further optimization can achieve Good or Excellent rank".to_string()),
                estimated_impact: ImpactLevel::Low,
            });
        }
        
        Ok(issues)
    }

    /// Analyze memory usage
    fn analyze_memory_usage(&self, metrics: &VRChatMetrics, resource_usage: &WorldResourceUsage) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();
        
        // Check overall memory footprint
        if metrics.estimated_memory_footprint > self.performance_thresholds.poor_memory_limit {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::Memory,
                description: format!(
                    "Memory footprint ({:.1}MB) is very high and may cause performance issues",
                    metrics.estimated_memory_footprint as f64 / 1_000_000.0
                ),
                location: None,
                suggested_fix: Some("Optimize textures, meshes, and reduce memory allocations".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }
        
        // Check texture memory specifically
        if resource_usage.texture_memory > 100_000_000 { // 100MB
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Memory,
                description: format!(
                    "Texture memory usage ({:.1}MB) is high",
                    resource_usage.texture_memory as f64 / 1_000_000.0
                ),
                location: None,
                suggested_fix: Some("Compress textures, reduce resolution, or use texture atlasing".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        // Check mesh memory
        if resource_usage.mesh_memory > 50_000_000 { // 50MB
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Memory,
                description: format!(
                    "Mesh memory usage ({:.1}MB) is high",
                    resource_usage.mesh_memory as f64 / 1_000_000.0
                ),
                location: None,
                suggested_fix: Some("Optimize mesh complexity, use LOD, or implement mesh compression".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        Ok(issues)
    }

    /// Analyze world complexity
    fn analyze_world_complexity(&self, complexity: &WorldComplexityMetrics) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();
        
        // Check particle systems
        if complexity.particle_systems > self.performance_thresholds.max_particle_systems {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::CPU,
                description: format!(
                    "High number of particle systems ({}) may impact performance",
                    complexity.particle_systems
                ),
                location: None,
                suggested_fix: Some("Reduce particle count, use LOD, or implement object pooling".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        // Check audio sources
        if complexity.audio_sources > self.performance_thresholds.max_audio_sources {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::CPU,
                description: format!(
                    "High number of audio sources ({}) may impact performance",
                    complexity.audio_sources
                ),
                location: None,
                suggested_fix: Some("Use audio occlusion, limit concurrent sources, or implement audio pooling".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        // Check lights
        if complexity.lights > self.performance_thresholds.max_lights {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::CPU,
                description: format!(
                    "High number of lights ({}) will significantly impact performance",
                    complexity.lights
                ),
                location: None,
                suggested_fix: Some("Use baked lighting, reduce real-time lights, or implement light culling".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }
        
        Ok(issues)
    }

    /// Analyze resource usage
    fn analyze_resource_usage(&self, resource_usage: &WorldResourceUsage) -> Result<Vec<PerformanceIssue>> {
        let mut issues = Vec::new();
        
        // Check triangle count
        if resource_usage.total_triangles > 1_000_000 {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::CPU,
                description: format!(
                    "High triangle count ({}) will impact rendering performance",
                    resource_usage.total_triangles
                ),
                location: None,
                suggested_fix: Some("Implement LOD system, optimize meshes, or use occlusion culling".to_string()),
                estimated_impact: ImpactLevel::High,
            });
        }
        
        // Check draw calls
        if resource_usage.draw_calls > 500 {
            issues.push(PerformanceIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::CPU,
                description: format!(
                    "High draw call count ({}) may impact rendering performance",
                    resource_usage.draw_calls
                ),
                location: None,
                suggested_fix: Some("Use texture atlasing, mesh combining, or GPU instancing".to_string()),
                estimated_impact: ImpactLevel::Medium,
            });
        }
        
        Ok(issues)
    }

    /// Generate optimization opportunities
    fn generate_optimization_opportunities(&self, metrics: &VRChatWorldMetrics) -> Result<Vec<WorldOptimizationOpportunity>> {
        let mut opportunities = Vec::new();
        
        // Instruction count optimization
        if metrics.base_metrics.estimated_instruction_count > self.performance_thresholds.excellent_instruction_limit {
            opportunities.push(WorldOptimizationOpportunity {
                category: WorldOptimizationCategory::InstructionCount,
                title: "Optimize Code for Better Performance Rank".to_string(),
                description: "Reduce instruction count to achieve better VRChat performance ranking".to_string(),
                impact: ImpactLevel::High,
                difficulty: OptimizationDifficulty::Medium,
                estimated_improvement: 30.0,
                implementation_steps: vec![
                    "Profile code to identify instruction-heavy operations".to_string(),
                    "Replace Update() methods with event-driven alternatives".to_string(),
                    "Optimize mathematical operations and loops".to_string(),
                    "Cache expensive calculations and component references".to_string(),
                ],
                affected_objects: vec!["UdonSharp scripts".to_string()],
            });
        }
        
        // Memory optimization
        if metrics.resource_usage.texture_memory > 50_000_000 {
            opportunities.push(WorldOptimizationOpportunity {
                category: WorldOptimizationCategory::MemoryUsage,
                title: "Optimize Texture Memory Usage".to_string(),
                description: "Reduce texture memory footprint for better performance".to_string(),
                impact: ImpactLevel::Medium,
                difficulty: OptimizationDifficulty::Easy,
                estimated_improvement: 20.0,
                implementation_steps: vec![
                    "Compress textures using appropriate formats".to_string(),
                    "Reduce texture resolution where possible".to_string(),
                    "Use texture atlasing to combine multiple textures".to_string(),
                    "Remove unused textures from the project".to_string(),
                ],
                affected_objects: vec!["Textures and materials".to_string()],
            });
        }
        
        // Rendering optimization
        if metrics.resource_usage.draw_calls > 200 {
            opportunities.push(WorldOptimizationOpportunity {
                category: WorldOptimizationCategory::RenderingOptimization,
                title: "Reduce Draw Calls".to_string(),
                description: "Optimize rendering performance by reducing draw calls".to_string(),
                impact: ImpactLevel::Medium,
                difficulty: OptimizationDifficulty::Medium,
                estimated_improvement: 25.0,
                implementation_steps: vec![
                    "Combine meshes with the same material".to_string(),
                    "Use texture atlasing for multiple materials".to_string(),
                    "Implement GPU instancing for repeated objects".to_string(),
                    "Use occlusion culling to hide non-visible objects".to_string(),
                ],
                affected_objects: vec!["Meshes and materials".to_string()],
            });
        }
        
        Ok(opportunities)
    }

    /// Calculate overall performance score
    fn calculate_performance_score(&self, metrics: &VRChatWorldMetrics) -> f64 {
        let mut score: f64 = 1.0;
        
        // Instruction count factor (40% weight)
        let instruction_factor = if metrics.base_metrics.estimated_instruction_count <= self.performance_thresholds.excellent_instruction_limit {
            1.0
        } else if metrics.base_metrics.estimated_instruction_count <= self.performance_thresholds.good_instruction_limit {
            0.8
        } else if metrics.base_metrics.estimated_instruction_count <= self.performance_thresholds.medium_instruction_limit {
            0.6
        } else {
            0.3
        };
        
        // Memory factor (30% weight)
        let memory_factor = if metrics.base_metrics.estimated_memory_footprint <= self.performance_thresholds.excellent_memory_limit {
            1.0
        } else if metrics.base_metrics.estimated_memory_footprint <= self.performance_thresholds.good_memory_limit {
            0.8
        } else if metrics.base_metrics.estimated_memory_footprint <= self.performance_thresholds.medium_memory_limit {
            0.6
        } else {
            0.4
        };
        
        // Complexity factor (20% weight)
        let complexity_factor = {
            let mut factor = 1.0;
            if metrics.world_complexity.lights > self.performance_thresholds.max_lights {
                factor *= 0.7;
            }
            if metrics.world_complexity.particle_systems > self.performance_thresholds.max_particle_systems {
                factor *= 0.8;
            }
            if metrics.world_complexity.audio_sources > self.performance_thresholds.max_audio_sources {
                factor *= 0.9;
            }
            factor
        };
        
        // Resource factor (10% weight)
        let resource_factor = if metrics.resource_usage.total_triangles > 1_000_000 {
            0.6
        } else if metrics.resource_usage.total_triangles > 500_000 {
            0.8
        } else {
            1.0
        };
        
        score = instruction_factor * 0.4 + memory_factor * 0.3 + complexity_factor * 0.2 + resource_factor * 0.1;
        score.max(0.0).min(1.0)
    }

    /// Determine VRChat performance rank
    fn determine_performance_rank(&self, metrics: &VRChatMetrics) -> PerformanceRank {
        if metrics.estimated_instruction_count <= self.performance_thresholds.excellent_instruction_limit {
            PerformanceRank::Excellent
        } else if metrics.estimated_instruction_count <= self.performance_thresholds.good_instruction_limit {
            PerformanceRank::Good
        } else if metrics.estimated_instruction_count <= self.performance_thresholds.medium_instruction_limit {
            PerformanceRank::Medium
        } else if metrics.estimated_instruction_count <= self.performance_thresholds.poor_instruction_limit {
            PerformanceRank::Poor
        } else {
            PerformanceRank::VeryPoor
        }
    }

    /// Check VRChat compatibility
    fn check_vrchat_compatibility(&self, metrics: &VRChatWorldMetrics) -> Result<VRChatCompatibilityReport> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Check hard limits
        if metrics.base_metrics.estimated_instruction_count > 2_000_000 {
            issues.push("Instruction count exceeds VRChat hard limit".to_string());
        }
        
        if metrics.base_metrics.network_sync_variables > 200 {
            issues.push("Sync variable count exceeds VRChat limit".to_string());
        }
        
        // Check soft limits
        if metrics.world_complexity.lights > 20 {
            warnings.push("High light count may cause performance issues".to_string());
        }
        
        if metrics.resource_usage.total_triangles > 2_000_000 {
            warnings.push("Very high triangle count may cause rendering issues".to_string());
        }
        
        let is_compatible = issues.is_empty();
        let compatibility_score = if is_compatible {
            1.0 - (warnings.len() as f64 * 0.1)
        } else {
            0.5 - (issues.len() as f64 * 0.1)
        }.max(0.0);
        
        Ok(VRChatCompatibilityReport {
            is_compatible,
            compatibility_score,
            issues,
            warnings,
        })
    }

    /// Generate recommendations
    fn generate_recommendations(&self, metrics: &VRChatWorldMetrics) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();
        
        let performance_rank = self.determine_performance_rank(&metrics.base_metrics);
        
        match performance_rank {
            PerformanceRank::VeryPoor | PerformanceRank::Poor => {
                recommendations.push("Critical optimization needed - world may be rejected by VRChat".to_string());
                recommendations.push("Focus on reducing instruction count through algorithm optimization".to_string());
                recommendations.push("Consider splitting complex behaviors into multiple scripts".to_string());
            }
            PerformanceRank::Medium => {
                recommendations.push("Good foundation, but optimization can improve user experience".to_string());
                recommendations.push("Profile and optimize performance hotspots".to_string());
                recommendations.push("Consider implementing LOD systems for complex objects".to_string());
            }
            PerformanceRank::Good => {
                recommendations.push("Well optimized world with good performance".to_string());
                recommendations.push("Minor optimizations can achieve Excellent rank".to_string());
            }
            PerformanceRank::Excellent => {
                recommendations.push("Excellent performance optimization achieved".to_string());
                recommendations.push("Focus on maintaining performance as features are added".to_string());
            }
            PerformanceRank::Unknown => {
                recommendations.push("Performance analysis incomplete - gather more metrics".to_string());
                recommendations.push("Run comprehensive performance profiling to determine optimization needs".to_string());
            }
        }
        
        Ok(recommendations)
    }

    /// Initialize optimization rules
    fn initialize_optimization_rules(&mut self) {
        // Add various optimization rules
        self.optimization_rules.push(WorldOptimizationRule {
            name: "reduce_instruction_count".to_string(),
            category: WorldOptimizationCategory::InstructionCount,
            condition: WorldCondition::InstructionCountExceeds(50_000),
            recommendation: "Optimize code to reduce instruction count".to_string(),
            estimated_improvement: 30.0,
            difficulty: OptimizationDifficulty::Medium,
        });
        
        // Add more rules as needed...
    }
}

/// World performance analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldPerformanceAnalysis {
    pub performance_score: f64,
    pub performance_rank: PerformanceRank,
    pub issues: Vec<PerformanceIssue>,
    pub optimization_opportunities: Vec<WorldOptimizationOpportunity>,
    pub vrchat_compatibility: VRChatCompatibilityReport,
    pub recommendations: Vec<String>,
}

/// VRChat compatibility report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VRChatCompatibilityReport {
    pub is_compatible: bool,
    pub compatibility_score: f64,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

impl Default for VRChatPerformanceThresholds {
    fn default() -> Self {
        Self {
            excellent_instruction_limit: 10_000,
            good_instruction_limit: 50_000,
            medium_instruction_limit: 500_000,
            poor_instruction_limit: 1_000_000,
            excellent_memory_limit: 1_000_000,   // 1MB
            good_memory_limit: 10_000_000,       // 10MB
            medium_memory_limit: 50_000_000,     // 50MB
            poor_memory_limit: 100_000_000,      // 100MB
            max_sync_variables: 100,
            max_audio_sources: 20,
            max_particle_systems: 10,
            max_lights: 8,
        }
    }
}

impl Default for VRChatWorldAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}