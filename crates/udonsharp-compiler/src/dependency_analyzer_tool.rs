//! Dependency analyzer tool for multi-behavior systems
//! 
//! This module provides tools for analyzing and visualizing dependencies
//! between UdonBehaviour structs in a multi-behavior system.

use crate::multi_behavior::UdonBehaviourStruct;
use std::collections::{HashMap, HashSet, VecDeque};
use serde::{Serialize, Deserialize};

/// Analyzes dependencies between behaviors in a multi-behavior system
pub struct DependencyAnalyzerTool {
    /// Dependency graph (behavior -> dependencies)
    dependency_graph: HashMap<String, Vec<String>>,
    /// Reverse dependency graph (behavior -> dependents)
    reverse_dependency_graph: HashMap<String, Vec<String>>,
    /// All behaviors in the system
    behaviors: HashMap<String, UdonBehaviourStruct>,
}

/// Result of dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysisReport {
    /// Total number of behaviors analyzed
    pub total_behaviors: usize,
    /// Total number of dependencies
    pub total_dependencies: usize,
    /// Behaviors with no dependencies (roots)
    pub root_behaviors: Vec<String>,
    /// Behaviors with no dependents (leaves)
    pub leaf_behaviors: Vec<String>,
    /// Circular dependencies detected
    pub circular_dependencies: Vec<Vec<String>>,
    /// Dependency chains (longest paths)
    pub dependency_chains: Vec<Vec<String>>,
    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    /// Initialization order recommendation
    pub recommended_initialization_order: Vec<String>,
    /// Potential issues and warnings
    pub warnings: Vec<DependencyWarning>,
}

/// Complexity metrics for the dependency system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Maximum dependency depth
    pub max_depth: usize,
    /// Average dependencies per behavior
    pub avg_dependencies_per_behavior: f32,
    /// Cyclomatic complexity (number of cycles)
    pub cyclomatic_complexity: usize,
    /// Coupling factor (0.0 to 1.0)
    pub coupling_factor: f32,
    /// Cohesion score (0.0 to 1.0)
    pub cohesion_score: f32,
}

/// Warning about potential dependency issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyWarning {
    /// Warning type
    pub warning_type: DependencyWarningType,
    /// Affected behaviors
    pub affected_behaviors: Vec<String>,
    /// Warning message
    pub message: String,
    /// Severity level
    pub severity: WarningSeverity,
    /// Suggested fixes
    pub suggestions: Vec<String>,
}

/// Types of dependency warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyWarningType {
    CircularDependency,
    DeepDependencyChain,
    HighCoupling,
    IsolatedBehavior,
    MissingDependency,
    UnusedBehavior,
}

/// Warning severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl DependencyAnalyzerTool {
    /// Create a new dependency analyzer
    pub fn new() -> Self {
        Self {
            dependency_graph: HashMap::new(),
            reverse_dependency_graph: HashMap::new(),
            behaviors: HashMap::new(),
        }
    }

    /// Add a behavior to the analysis
    pub fn add_behavior(&mut self, behavior: UdonBehaviourStruct) {
        let name = behavior.name.clone();
        let dependencies = behavior.dependencies.clone();
        
        self.behaviors.insert(name.clone(), behavior);
        self.dependency_graph.insert(name.clone(), dependencies.clone());
        
        // Update reverse dependencies
        for dep in dependencies {
            self.reverse_dependency_graph
                .entry(dep)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }
        
        // Ensure all behaviors have entries in reverse graph
        self.reverse_dependency_graph.entry(name).or_insert_with(Vec::new);
    }

    /// Perform complete dependency analysis
    pub fn analyze(&self) -> DependencyAnalysisReport {
        let total_behaviors = self.behaviors.len();
        let total_dependencies = self.dependency_graph.values()
            .map(|deps| deps.len())
            .sum();

        let root_behaviors = self.find_root_behaviors();
        let leaf_behaviors = self.find_leaf_behaviors();
        let circular_dependencies = self.detect_circular_dependencies();
        let dependency_chains = self.find_dependency_chains();
        let complexity_metrics = self.calculate_complexity_metrics();
        let recommended_initialization_order = self.calculate_initialization_order();
        let warnings = self.generate_warnings();

        DependencyAnalysisReport {
            total_behaviors,
            total_dependencies,
            root_behaviors,
            leaf_behaviors,
            circular_dependencies,
            dependency_chains,
            complexity_metrics,
            recommended_initialization_order,
            warnings,
        }
    }

    /// Find behaviors with no dependencies (root nodes)
    fn find_root_behaviors(&self) -> Vec<String> {
        self.dependency_graph
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Find behaviors with no dependents (leaf nodes)
    fn find_leaf_behaviors(&self) -> Vec<String> {
        self.reverse_dependency_graph
            .iter()
            .filter(|(_, dependents)| dependents.is_empty())
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Detect circular dependencies using DFS
    fn detect_circular_dependencies(&self) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut cycles = Vec::new();
        let mut path = Vec::new();

        for behavior in self.behaviors.keys() {
            if !visited.contains(behavior) {
                self.dfs_detect_cycles(
                    behavior,
                    &mut visited,
                    &mut rec_stack,
                    &mut cycles,
                    &mut path,
                );
            }
        }

        cycles
    }

    /// DFS helper for cycle detection
    fn dfs_detect_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
        path: &mut Vec<String>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(dependencies) = self.dependency_graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.dfs_detect_cycles(dep, visited, rec_stack, cycles, path);
                } else if rec_stack.contains(dep) {
                    // Found a cycle
                    if let Some(cycle_start) = path.iter().position(|x| x == dep) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
    }

    /// Find the longest dependency chains
    fn find_dependency_chains(&self) -> Vec<Vec<String>> {
        let mut chains = Vec::new();
        let root_behaviors = self.find_root_behaviors();

        for root in root_behaviors {
            let mut longest_chain = Vec::new();
            self.find_longest_path_from(&root, &mut Vec::new(), &mut longest_chain);
            if !longest_chain.is_empty() {
                chains.push(longest_chain);
            }
        }

        // Sort by length (longest first)
        chains.sort_by(|a, b| b.len().cmp(&a.len()));
        chains.truncate(10); // Keep top 10 longest chains
        chains
    }

    /// Find longest path from a given node
    fn find_longest_path_from(
        &self,
        node: &str,
        current_path: &mut Vec<String>,
        longest_path: &mut Vec<String>,
    ) {
        current_path.push(node.to_string());

        if current_path.len() > longest_path.len() {
            *longest_path = current_path.clone();
        }

        if let Some(dependents) = self.reverse_dependency_graph.get(node) {
            for dependent in dependents {
                if !current_path.contains(dependent) {
                    self.find_longest_path_from(dependent, current_path, longest_path);
                }
            }
        }

        current_path.pop();
    }

    /// Calculate complexity metrics
    fn calculate_complexity_metrics(&self) -> ComplexityMetrics {
        let total_behaviors = self.behaviors.len() as f32;
        let total_dependencies = self.dependency_graph.values()
            .map(|deps| deps.len())
            .sum::<usize>() as f32;

        let max_depth = self.calculate_max_depth();
        let avg_dependencies_per_behavior = if total_behaviors > 0.0 {
            total_dependencies / total_behaviors
        } else {
            0.0
        };

        let cyclomatic_complexity = self.detect_circular_dependencies().len();
        
        // Coupling factor: ratio of actual dependencies to possible dependencies
        let max_possible_dependencies = total_behaviors * (total_behaviors - 1.0);
        let coupling_factor = if max_possible_dependencies > 0.0 {
            total_dependencies / max_possible_dependencies
        } else {
            0.0
        };

        // Cohesion score: inverse of coupling (simplified metric)
        let cohesion_score = 1.0 - coupling_factor.min(1.0);

        ComplexityMetrics {
            max_depth,
            avg_dependencies_per_behavior,
            cyclomatic_complexity,
            coupling_factor,
            cohesion_score,
        }
    }

    /// Calculate maximum dependency depth
    fn calculate_max_depth(&self) -> usize {
        let mut max_depth = 0;
        let root_behaviors = self.find_root_behaviors();

        for root in root_behaviors {
            let depth = self.calculate_depth_from(&root, &mut HashSet::new());
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Calculate depth from a given node
    fn calculate_depth_from(&self, node: &str, visited: &mut HashSet<String>) -> usize {
        if visited.contains(node) {
            return 0; // Avoid infinite recursion in cycles
        }

        visited.insert(node.to_string());
        let mut max_child_depth = 0;

        if let Some(dependents) = self.reverse_dependency_graph.get(node) {
            for dependent in dependents {
                let child_depth = self.calculate_depth_from(dependent, visited);
                max_child_depth = max_child_depth.max(child_depth);
            }
        }

        visited.remove(node);
        1 + max_child_depth
    }

    /// Calculate recommended initialization order using topological sort
    fn calculate_initialization_order(&self) -> Vec<String> {
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees
        for behavior in self.behaviors.keys() {
            in_degree.insert(behavior.clone(), 0);
        }

        for dependencies in self.dependency_graph.values() {
            for dep in dependencies {
                *in_degree.entry(dep.clone()).or_insert(0) += 1;
            }
        }

        // Add nodes with no incoming edges to queue
        for (behavior, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(behavior.clone());
            }
        }

        // Process queue
        while let Some(behavior) = queue.pop_front() {
            result.push(behavior.clone());

            // Reduce in-degree of dependents
            if let Some(dependents) = self.reverse_dependency_graph.get(&behavior) {
                for dependent in dependents {
                    if let Some(degree) = in_degree.get_mut(dependent) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Generate warnings about potential issues
    fn generate_warnings(&self) -> Vec<DependencyWarning> {
        let mut warnings = Vec::new();

        // Check for circular dependencies
        let circular_deps = self.detect_circular_dependencies();
        for cycle in circular_deps {
            warnings.push(DependencyWarning {
                warning_type: DependencyWarningType::CircularDependency,
                affected_behaviors: cycle.clone(),
                message: format!("Circular dependency detected: {}", cycle.join(" -> ")),
                severity: WarningSeverity::Error,
                suggestions: vec![
                    "Consider using events instead of direct references".to_string(),
                    "Extract shared functionality to SharedRuntime".to_string(),
                    "Redesign the dependency structure".to_string(),
                ],
            });
        }

        // Check for deep dependency chains
        let chains = self.find_dependency_chains();
        for chain in chains {
            if chain.len() > 5 {
                warnings.push(DependencyWarning {
                    warning_type: DependencyWarningType::DeepDependencyChain,
                    affected_behaviors: chain.clone(),
                    message: format!("Deep dependency chain detected ({} levels): {}", 
                        chain.len(), chain.join(" -> ")),
                    severity: WarningSeverity::Warning,
                    suggestions: vec![
                        "Consider flattening the dependency structure".to_string(),
                        "Use mediator pattern for complex interactions".to_string(),
                    ],
                });
            }
        }

        // Check for high coupling
        let metrics = self.calculate_complexity_metrics();
        if metrics.coupling_factor > 0.7 {
            warnings.push(DependencyWarning {
                warning_type: DependencyWarningType::HighCoupling,
                affected_behaviors: self.behaviors.keys().cloned().collect(),
                message: format!("High coupling detected (factor: {:.2})", metrics.coupling_factor),
                severity: WarningSeverity::Warning,
                suggestions: vec![
                    "Reduce direct dependencies between behaviors".to_string(),
                    "Use event-driven communication".to_string(),
                    "Extract shared functionality".to_string(),
                ],
            });
        }

        // Check for isolated behaviors
        let isolated = self.find_isolated_behaviors();
        for behavior in isolated {
            warnings.push(DependencyWarning {
                warning_type: DependencyWarningType::IsolatedBehavior,
                affected_behaviors: vec![behavior.clone()],
                message: format!("Behavior '{}' has no dependencies or dependents", behavior),
                severity: WarningSeverity::Info,
                suggestions: vec![
                    "Consider if this behavior is needed".to_string(),
                    "Integrate with other behaviors if appropriate".to_string(),
                ],
            });
        }

        warnings
    }

    /// Find behaviors that are completely isolated
    fn find_isolated_behaviors(&self) -> Vec<String> {
        self.behaviors
            .keys()
            .filter(|behavior| {
                let has_dependencies = self.dependency_graph
                    .get(*behavior)
                    .map(|deps| !deps.is_empty())
                    .unwrap_or(false);
                
                let has_dependents = self.reverse_dependency_graph
                    .get(*behavior)
                    .map(|deps| !deps.is_empty())
                    .unwrap_or(false);
                
                !has_dependencies && !has_dependents
            })
            .cloned()
            .collect()
    }

    /// Generate a visual representation of the dependency graph
    pub fn generate_dependency_graph_dot(&self) -> String {
        let mut dot = String::from("digraph DependencyGraph {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [shape=box, style=rounded];\n\n");

        // Add nodes
        for behavior in self.behaviors.keys() {
            dot.push_str(&format!("    \"{}\" [label=\"{}\"];\n", behavior, behavior));
        }

        dot.push_str("\n");

        // Add edges
        for (behavior, dependencies) in &self.dependency_graph {
            for dep in dependencies {
                dot.push_str(&format!("    \"{}\" -> \"{}\";\n", behavior, dep));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate a detailed text report
    pub fn generate_text_report(&self) -> String {
        let analysis = self.analyze();
        let mut report = String::new();

        report.push_str("=== Multi-Behavior Dependency Analysis Report ===\n\n");

        // Summary
        report.push_str("## Summary\n");
        report.push_str(&format!("Total Behaviors: {}\n", analysis.total_behaviors));
        report.push_str(&format!("Total Dependencies: {}\n", analysis.total_dependencies));
        report.push_str(&format!("Root Behaviors: {}\n", analysis.root_behaviors.len()));
        report.push_str(&format!("Leaf Behaviors: {}\n", analysis.leaf_behaviors.len()));
        report.push_str(&format!("Circular Dependencies: {}\n", analysis.circular_dependencies.len()));
        report.push_str("\n");

        // Complexity Metrics
        report.push_str("## Complexity Metrics\n");
        report.push_str(&format!("Max Depth: {}\n", analysis.complexity_metrics.max_depth));
        report.push_str(&format!("Avg Dependencies per Behavior: {:.2}\n", 
            analysis.complexity_metrics.avg_dependencies_per_behavior));
        report.push_str(&format!("Coupling Factor: {:.2}\n", 
            analysis.complexity_metrics.coupling_factor));
        report.push_str(&format!("Cohesion Score: {:.2}\n", 
            analysis.complexity_metrics.cohesion_score));
        report.push_str("\n");

        // Initialization Order
        report.push_str("## Recommended Initialization Order\n");
        for (i, behavior) in analysis.recommended_initialization_order.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, behavior));
        }
        report.push_str("\n");

        // Warnings
        if !analysis.warnings.is_empty() {
            report.push_str("## Warnings and Issues\n");
            for warning in &analysis.warnings {
                report.push_str(&format!("### {:?}: {}\n", warning.severity, warning.message));
                report.push_str(&format!("Affected: {}\n", warning.affected_behaviors.join(", ")));
                report.push_str("Suggestions:\n");
                for suggestion in &warning.suggestions {
                    report.push_str(&format!("  - {}\n", suggestion));
                }
                report.push_str("\n");
            }
        }

        // Dependency Details
        report.push_str("## Dependency Details\n");
        for (behavior, dependencies) in &self.dependency_graph {
            report.push_str(&format!("### {}\n", behavior));
            if dependencies.is_empty() {
                report.push_str("  No dependencies\n");
            } else {
                report.push_str("  Dependencies:\n");
                for dep in dependencies {
                    report.push_str(&format!("    - {}\n", dep));
                }
            }
            
            if let Some(dependents) = self.reverse_dependency_graph.get(behavior) {
                if !dependents.is_empty() {
                    report.push_str("  Dependents:\n");
                    for dep in dependents {
                        report.push_str(&format!("    - {}\n", dep));
                    }
                }
            }
            report.push_str("\n");
        }

        report
    }
}

impl Default for DependencyAnalyzerTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::UdonBehaviourStruct;

    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzerTool::new();
        assert_eq!(analyzer.behaviors.len(), 0);
    }

    #[test]
    fn test_add_behavior() {
        let mut analyzer = DependencyAnalyzerTool::new();
        let behavior = UdonBehaviourStruct::new("TestBehavior".to_string());
        
        analyzer.add_behavior(behavior);
        assert_eq!(analyzer.behaviors.len(), 1);
        assert!(analyzer.behaviors.contains_key("TestBehavior"));
    }

    #[test]
    fn test_find_root_behaviors() {
        let mut analyzer = DependencyAnalyzerTool::new();
        
        let mut behavior1 = UdonBehaviourStruct::new("Root".to_string());
        let mut behavior2 = UdonBehaviourStruct::new("Dependent".to_string());
        behavior2.add_dependency("Root".to_string());
        
        analyzer.add_behavior(behavior1);
        analyzer.add_behavior(behavior2);
        
        let roots = analyzer.find_root_behaviors();
        assert_eq!(roots.len(), 1);
        assert!(roots.contains(&"Root".to_string()));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = DependencyAnalyzerTool::new();
        
        let mut behavior1 = UdonBehaviourStruct::new("A".to_string());
        behavior1.add_dependency("B".to_string());
        
        let mut behavior2 = UdonBehaviourStruct::new("B".to_string());
        behavior2.add_dependency("A".to_string());
        
        analyzer.add_behavior(behavior1);
        analyzer.add_behavior(behavior2);
        
        let cycles = analyzer.detect_circular_dependencies();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_initialization_order() {
        let mut analyzer = DependencyAnalyzerTool::new();
        
        let behavior1 = UdonBehaviourStruct::new("Root".to_string());
        let mut behavior2 = UdonBehaviourStruct::new("Middle".to_string());
        behavior2.add_dependency("Root".to_string());
        let mut behavior3 = UdonBehaviourStruct::new("Leaf".to_string());
        behavior3.add_dependency("Middle".to_string());
        
        analyzer.add_behavior(behavior1);
        analyzer.add_behavior(behavior2);
        analyzer.add_behavior(behavior3);
        
        let order = analyzer.calculate_initialization_order();
        assert_eq!(order.len(), 3);
        
        let root_pos = order.iter().position(|x| x == "Root").unwrap();
        let middle_pos = order.iter().position(|x| x == "Middle").unwrap();
        let leaf_pos = order.iter().position(|x| x == "Leaf").unwrap();
        
        assert!(root_pos < middle_pos);
        assert!(middle_pos < leaf_pos);
    }
}