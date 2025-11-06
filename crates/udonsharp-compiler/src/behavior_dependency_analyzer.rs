//! Dependency analysis for UdonBehaviour structs
//! 
//! This module provides functionality to analyze dependencies between UdonBehaviour structs,
//! detect circular dependencies, and determine proper initialization order.

use crate::multi_behavior::{UdonBehaviourStruct, RustType};
use std::collections::{HashMap, HashSet, VecDeque};

/// Result type for dependency analysis operations
pub type DependencyResult<T> = Result<T, DependencyError>;

/// Errors that can occur during dependency analysis
#[derive(Debug, Clone)]
pub enum DependencyError {
    /// Circular dependency detected between behaviors
    CircularDependency { cycle: Vec<String>, description: String },
    /// Missing dependency reference
    MissingDependency { behavior: String, missing_dependency: String },
    /// Invalid dependency configuration
    InvalidDependency { behavior: String, dependency: String, reason: String },
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::CircularDependency { cycle, description } => {
                write!(f, "Circular dependency detected: {} - {}", cycle.join(" -> "), description)
            }
            DependencyError::MissingDependency { behavior, missing_dependency } => {
                write!(f, "Behavior '{}' depends on missing behavior '{}'", behavior, missing_dependency)
            }
            DependencyError::InvalidDependency { behavior, dependency, reason } => {
                write!(f, "Invalid dependency from '{}' to '{}': {}", behavior, dependency, reason)
            }
        }
    }
}

impl std::error::Error for DependencyError {}

/// Analyzer for dependencies between UdonBehaviour structs
pub struct BehaviorDependencyAnalyzer {
    /// Dependency graph between behaviors
    dependency_graph: DependencyGraph,
    /// Behaviors being analyzed
    behaviors: HashMap<String, UdonBehaviourStruct>,
    /// Analysis errors
    errors: Vec<DependencyError>,
    /// Analysis warnings
    warnings: Vec<String>,
}

impl BehaviorDependencyAnalyzer {
    /// Create a new behavior dependency analyzer
    pub fn new() -> Self {
        Self {
            dependency_graph: DependencyGraph::new(),
            behaviors: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Analyze dependencies between UdonBehaviour structs
    pub fn analyze_dependencies(&mut self, behaviors: Vec<UdonBehaviourStruct>) -> DependencyResult<DependencyAnalysisResult> {
        // Clear previous analysis
        self.dependency_graph = DependencyGraph::new();
        self.behaviors.clear();
        self.errors.clear();
        self.warnings.clear();

        // Store behaviors for analysis
        for behavior in behaviors {
            self.behaviors.insert(behavior.name.clone(), behavior);
        }

        // Build dependency graph
        self.build_dependency_graph()?;

        // Detect circular dependencies
        let circular_dependencies = self.detect_circular_dependencies()?;

        // Calculate initialization order
        let initialization_order = if circular_dependencies.is_empty() {
            Some(self.calculate_initialization_order()?)
        } else {
            None
        };

        // Generate dependency metrics
        let metrics = self.calculate_dependency_metrics();

        Ok(DependencyAnalysisResult {
            dependency_graph: self.dependency_graph.clone(),
            circular_dependencies,
            initialization_order,
            metrics,
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
        })
    }

    /// Build the dependency graph by analyzing GameObject references and dependencies
    fn build_dependency_graph(&mut self) -> DependencyResult<()> {
        // Create nodes for each behavior
        for (name, behavior) in &self.behaviors {
            let node = DependencyNode {
                name: name.clone(),
                behavior_type: self.classify_behavior_type(behavior),
                dependencies: Vec::new(),
                dependents: Vec::new(),
            };
            self.dependency_graph.nodes.insert(name.clone(), node);
        }

        // Collect dependencies first to avoid borrowing issues
        let mut dependencies_to_add = Vec::new();

        // Analyze field dependencies (GameObject references)
        for (behavior_name, behavior) in &self.behaviors {
            for field in &behavior.fields {
                if let Some(dependency) = self.extract_gameobject_dependency(&field.field_type) {
                    dependencies_to_add.push((behavior_name.clone(), dependency, DependencyType::GameObject));
                }
            }

            // Analyze explicit dependencies from behavior.dependencies
            for dependency in &behavior.dependencies {
                dependencies_to_add.push((behavior_name.clone(), dependency.clone(), DependencyType::Explicit));
            }
        }

        // Add all collected dependencies
        for (from, to, dep_type) in dependencies_to_add {
            self.add_dependency(from, to, dep_type)?;
        }

        // Build adjacency lists for efficient traversal
        self.build_adjacency_lists();

        Ok(())
    }

    /// Extract GameObject dependency from a field type
    fn extract_gameobject_dependency(&self, field_type: &RustType) -> Option<String> {
        match field_type {
            RustType::GameObject => {
                // For now, we can't determine the specific behavior from just GameObject type
                // This would require more sophisticated analysis of the code
                None
            }
            RustType::Option(inner) => {
                if matches!(**inner, RustType::GameObject) {
                    None // Same as above
                } else {
                    None
                }
            }
            RustType::Custom(type_name) => {
                // Check if this custom type is another UdonBehaviour struct
                if self.behaviors.contains_key(type_name) {
                    Some(type_name.clone())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Add a dependency between two behaviors
    fn add_dependency(&mut self, from: String, to: String, dependency_type: DependencyType) -> DependencyResult<()> {
        // Validate that the target behavior exists
        if !self.behaviors.contains_key(&to) {
            self.errors.push(DependencyError::MissingDependency {
                behavior: from.clone(),
                missing_dependency: to.clone(),
            });
            return Ok(()); // Continue analysis with other dependencies
        }

        // Don't allow self-dependencies
        if from == to {
            self.warnings.push(format!("Behavior '{}' has a self-dependency, which is unnecessary", from));
            return Ok(());
        }

        // Create dependency edge
        let edge = DependencyEdge {
            from: from.clone(),
            to: to.clone(),
            dependency_type,
            strength: self.calculate_dependency_strength(&from, &to),
        };

        self.dependency_graph.edges.push(edge);

        // Update node dependencies
        if let Some(from_node) = self.dependency_graph.nodes.get_mut(&from) {
            if !from_node.dependencies.contains(&to) {
                from_node.dependencies.push(to.clone());
            }
        }

        if let Some(to_node) = self.dependency_graph.nodes.get_mut(&to) {
            if !to_node.dependents.contains(&from) {
                to_node.dependents.push(from);
            }
        }

        Ok(())
    }

    /// Calculate the strength of a dependency
    fn calculate_dependency_strength(&self, from: &str, to: &str) -> DependencyStrength {
        let from_behavior = &self.behaviors[from];
        let to_behavior = &self.behaviors[to];

        // Explicit dependencies are always high strength
        if from_behavior.dependencies.contains(&to.to_string()) {
            return DependencyStrength::High;
        }

        // Dependencies involving networking behaviors are high strength
        if from_behavior.has_networking() || to_behavior.has_networking() {
            return DependencyStrength::High;
        }

        // Default to medium strength
        DependencyStrength::Medium
    }

    /// Classify the type of a behavior based on its characteristics
    fn classify_behavior_type(&self, behavior: &UdonBehaviourStruct) -> BehaviorType {
        let name_lower = behavior.name.to_lowercase();

        // Check for common behavior patterns
        if name_lower.contains("manager") || name_lower.contains("controller") {
            BehaviorType::Manager
        } else if name_lower.contains("ui") || name_lower.contains("interface") {
            BehaviorType::UI
        } else if behavior.has_networking() {
            BehaviorType::Network
        } else if name_lower.contains("player") {
            BehaviorType::Player
        } else if name_lower.contains("game") {
            BehaviorType::Game
        } else {
            BehaviorType::Generic
        }
    }

    /// Build adjacency lists for efficient graph traversal
    fn build_adjacency_lists(&mut self) {
        self.dependency_graph.adjacency_list.clear();
        self.dependency_graph.reverse_adjacency_list.clear();

        for edge in &self.dependency_graph.edges {
            // Forward adjacency list (behavior -> its dependencies)
            self.dependency_graph.adjacency_list
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());

            // Reverse adjacency list (behavior -> its dependents)
            self.dependency_graph.reverse_adjacency_list
                .entry(edge.to.clone())
                .or_insert_with(Vec::new)
                .push(edge.from.clone());
        }
    }

    /// Detect circular dependencies using depth-first search
    fn detect_circular_dependencies(&mut self) -> DependencyResult<Vec<CircularDependency>> {
        let mut circular_dependencies = Vec::new();
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        let mut path = Vec::new();

        for behavior_name in self.behaviors.keys() {
            if !visited.contains(behavior_name) {
                if let Some(cycle) = self.dfs_detect_cycle(
                    behavior_name,
                    &mut visited,
                    &mut recursion_stack,
                    &mut path,
                ) {
                    let circular_dependency = self.analyze_circular_dependency(cycle)?;
                    circular_dependencies.push(circular_dependency);
                }
            }
        }

        Ok(circular_dependencies)
    }

    /// Depth-first search to detect cycles
    fn dfs_detect_cycle(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(current.to_string());
        recursion_stack.insert(current.to_string());
        path.push(current.to_string());

        if let Some(dependencies) = self.dependency_graph.adjacency_list.get(current) {
            for dependency in dependencies {
                if !visited.contains(dependency) {
                    if let Some(cycle) = self.dfs_detect_cycle(dependency, visited, recursion_stack, path) {
                        return Some(cycle);
                    }
                } else if recursion_stack.contains(dependency) {
                    // Found a cycle - extract the cycle path
                    let cycle_start = path.iter().position(|x| x == dependency).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(dependency.clone()); // Close the cycle
                    return Some(cycle);
                }
            }
        }

        recursion_stack.remove(current);
        path.pop();
        None
    }

    /// Analyze a circular dependency to provide detailed information
    fn analyze_circular_dependency(&self, cycle: Vec<String>) -> DependencyResult<CircularDependency> {
        let mut involved_edges = Vec::new();
        let mut dependency_types = Vec::new();

        // Find the edges involved in the cycle
        for i in 0..cycle.len() - 1 {
            let from = &cycle[i];
            let to = &cycle[i + 1];

            if let Some(edge) = self.dependency_graph.edges.iter().find(|e| e.from == *from && e.to == *to) {
                involved_edges.push(edge.clone());
                dependency_types.push(edge.dependency_type.clone());
            }
        }

        let severity = self.calculate_cycle_severity(&dependency_types);
        let description = self.generate_cycle_description(&cycle, &dependency_types);
        let resolution_suggestions = self.generate_resolution_suggestions(&cycle, &dependency_types);

        Ok(CircularDependency {
            cycle: cycle[..cycle.len() - 1].to_vec(), // Remove the duplicate last element
            involved_edges,
            severity,
            description,
            resolution_suggestions,
        })
    }

    /// Calculate the severity of a circular dependency
    fn calculate_cycle_severity(&self, dependency_types: &[DependencyType]) -> CycleSeverity {
        if dependency_types.iter().any(|dt| matches!(dt, DependencyType::Explicit)) {
            CycleSeverity::Critical
        } else if dependency_types.iter().any(|dt| matches!(dt, DependencyType::GameObject)) {
            CycleSeverity::High
        } else {
            CycleSeverity::Medium
        }
    }

    /// Generate a description for a circular dependency
    fn generate_cycle_description(&self, cycle: &[String], dependency_types: &[DependencyType]) -> String {
        let cycle_path = cycle.join(" -> ");
        let has_explicit = dependency_types.iter().any(|dt| matches!(dt, DependencyType::Explicit));
        let has_gameobject = dependency_types.iter().any(|dt| matches!(dt, DependencyType::GameObject));

        let mut description = format!("Circular dependency detected: {} -> {}", cycle_path, cycle[0]);

        if has_explicit {
            description.push_str(" (involves explicit dependencies)");
        }
        if has_gameobject {
            description.push_str(" (involves GameObject references)");
        }

        description.push_str(". This creates an initialization deadlock where behaviors cannot be properly ordered.");
        description
    }

    /// Generate suggestions for resolving circular dependencies
    fn generate_resolution_suggestions(&self, cycle: &[String], dependency_types: &[DependencyType]) -> Vec<String> {
        let mut suggestions = Vec::new();

        suggestions.push("Consider the following approaches to resolve this circular dependency:".to_string());

        if dependency_types.iter().any(|dt| matches!(dt, DependencyType::Explicit)) {
            suggestions.push("• Remove explicit dependencies by refactoring shared functionality into a separate component".to_string());
            suggestions.push("• Use dependency injection or a service locator pattern".to_string());
        }

        if dependency_types.iter().any(|dt| matches!(dt, DependencyType::GameObject)) {
            suggestions.push("• Use Unity events or a message bus for loose coupling instead of direct GameObject references".to_string());
            suggestions.push("• Implement a mediator pattern to coordinate between behaviors".to_string());
        }

        if cycle.len() == 2 {
            suggestions.push(format!("• Consider merging '{}' and '{}' into a single behavior if they are tightly coupled", cycle[0], cycle[1]));
        } else {
            suggestions.push("• Break down complex behaviors into smaller, more focused components".to_string());
        }

        suggestions.push("• Use initialization phases to establish proper startup order".to_string());
        suggestions.push("• Move shared state to a centralized manager or SharedRuntime".to_string());

        suggestions
    }

    /// Calculate initialization order using topological sort
    fn calculate_initialization_order(&self) -> DependencyResult<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degree count for all behaviors
        for behavior_name in self.behaviors.keys() {
            in_degree.insert(behavior_name.clone(), 0);
        }

        // Calculate in-degrees
        for edge in &self.dependency_graph.edges {
            *in_degree.entry(edge.from.clone()).or_insert(0) += 1;
        }

        // Find behaviors with no dependencies (in-degree 0)
        for (behavior_name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(behavior_name.clone());
            }
        }

        // Process behaviors in topological order
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // Update in-degrees of dependents
            if let Some(dependents) = self.dependency_graph.reverse_adjacency_list.get(&current) {
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

        // Check if all behaviors were processed (no cycles)
        if result.len() != self.behaviors.len() {
            return Err(DependencyError::CircularDependency {
                cycle: Vec::new(),
                description: "Cannot determine initialization order due to circular dependencies".to_string(),
            });
        }

        Ok(result)
    }

    /// Calculate dependency metrics
    fn calculate_dependency_metrics(&self) -> DependencyMetrics {
        let total_behaviors = self.behaviors.len();
        let total_dependencies = self.dependency_graph.edges.len();

        let mut in_degrees = HashMap::new();
        let mut out_degrees = HashMap::new();

        // Initialize degree counts
        for behavior_name in self.behaviors.keys() {
            in_degrees.insert(behavior_name.clone(), 0);
            out_degrees.insert(behavior_name.clone(), 0);
        }

        // Calculate degrees
        for edge in &self.dependency_graph.edges {
            *out_degrees.entry(edge.from.clone()).or_insert(0) += 1;
            *in_degrees.entry(edge.to.clone()).or_insert(0) += 1;
        }

        let max_dependencies = in_degrees.values().max().copied().unwrap_or(0);
        let max_dependents = out_degrees.values().max().copied().unwrap_or(0);
        let avg_dependencies = if total_behaviors > 0 {
            in_degrees.values().sum::<usize>() as f64 / total_behaviors as f64
        } else {
            0.0
        };

        // Find independent behaviors (no dependencies)
        let independent_behaviors: Vec<String> = in_degrees
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        // Find leaf behaviors (no dependents)
        let leaf_behaviors: Vec<String> = out_degrees
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        DependencyMetrics {
            total_behaviors,
            total_dependencies,
            max_dependencies,
            max_dependents,
            avg_dependencies,
            independent_behaviors,
            leaf_behaviors,
        }
    }

    /// Get analysis errors
    pub fn get_errors(&self) -> &[DependencyError] {
        &self.errors
    }

    /// Get analysis warnings
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }
}

impl Default for BehaviorDependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Dependency graph for UdonBehaviour structs
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes representing behaviors
    pub nodes: HashMap<String, DependencyNode>,
    /// Edges representing dependencies
    pub edges: Vec<DependencyEdge>,
    /// Adjacency list for efficient traversal (behavior -> dependencies)
    pub adjacency_list: HashMap<String, Vec<String>>,
    /// Reverse adjacency list (behavior -> dependents)
    pub reverse_adjacency_list: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
        }
    }
}

/// Node in the dependency graph representing a behavior
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Behavior name
    pub name: String,
    /// Type of behavior
    pub behavior_type: BehaviorType,
    /// Direct dependencies of this behavior
    pub dependencies: Vec<String>,
    /// Behaviors that depend on this one
    pub dependents: Vec<String>,
}

/// Edge in the dependency graph representing a dependency
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Source behavior (the one that depends)
    pub from: String,
    /// Target behavior (the dependency)
    pub to: String,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Strength of the dependency
    pub strength: DependencyStrength,
}

/// Type of dependency between behaviors
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    /// Explicit dependency declared in the behavior
    Explicit,
    /// Dependency through GameObject reference
    GameObject,
}

/// Strength of a dependency
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DependencyStrength {
    /// Low priority dependency
    Low,
    /// Medium priority dependency
    Medium,
    /// High priority dependency
    High,
}

/// Type of behavior for classification
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorType {
    /// Manager or controller behavior
    Manager,
    /// UI-related behavior
    UI,
    /// Network-related behavior
    Network,
    /// Player-related behavior
    Player,
    /// Game logic behavior
    Game,
    /// Generic behavior
    Generic,
}

/// Information about a circular dependency
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// Behaviors involved in the cycle
    pub cycle: Vec<String>,
    /// Edges involved in the cycle
    pub involved_edges: Vec<DependencyEdge>,
    /// Severity of the circular dependency
    pub severity: CycleSeverity,
    /// Human-readable description
    pub description: String,
    /// Suggestions for resolving the circular dependency
    pub resolution_suggestions: Vec<String>,
}

/// Severity level of a circular dependency
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CycleSeverity {
    /// Medium impact circular dependency
    Medium,
    /// High impact circular dependency
    High,
    /// Critical circular dependency that prevents compilation
    Critical,
}

/// Metrics about the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyMetrics {
    /// Total number of behaviors
    pub total_behaviors: usize,
    /// Total number of dependencies
    pub total_dependencies: usize,
    /// Maximum number of dependencies for any behavior
    pub max_dependencies: usize,
    /// Maximum number of dependents for any behavior
    pub max_dependents: usize,
    /// Average number of dependencies per behavior
    pub avg_dependencies: f64,
    /// Behaviors with no dependencies
    pub independent_behaviors: Vec<String>,
    /// Behaviors with no dependents
    pub leaf_behaviors: Vec<String>,
}

/// Result of dependency analysis
#[derive(Debug, Clone)]
pub struct DependencyAnalysisResult {
    /// The dependency graph
    pub dependency_graph: DependencyGraph,
    /// Detected circular dependencies
    pub circular_dependencies: Vec<CircularDependency>,
    /// Recommended initialization order (None if circular dependencies exist)
    pub initialization_order: Option<Vec<String>>,
    /// Dependency metrics
    pub metrics: DependencyMetrics,
    /// Analysis errors
    pub errors: Vec<DependencyError>,
    /// Analysis warnings
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::{StructField, FieldAttribute};

    fn create_test_behavior(name: &str) -> UdonBehaviourStruct {
        UdonBehaviourStruct::new(name.to_string())
    }

    fn create_test_behavior_with_dependency(name: &str, dependency: &str) -> UdonBehaviourStruct {
        let mut behavior = UdonBehaviourStruct::new(name.to_string());
        behavior.add_dependency(dependency.to_string());
        behavior
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = BehaviorDependencyAnalyzer::new();
        assert_eq!(analyzer.behaviors.len(), 0);
        assert_eq!(analyzer.errors.len(), 0);
    }

    #[test]
    fn test_simple_dependency_analysis() {
        let mut analyzer = BehaviorDependencyAnalyzer::new();
        
        let behavior_a = create_test_behavior("BehaviorA");
        let behavior_b = create_test_behavior_with_dependency("BehaviorB", "BehaviorA");
        
        let result = analyzer.analyze_dependencies(vec![behavior_a, behavior_b]).unwrap();
        
        assert_eq!(result.circular_dependencies.len(), 0);
        assert!(result.initialization_order.is_some());
        
        let order = result.initialization_order.unwrap();
        assert_eq!(order, vec!["BehaviorA", "BehaviorB"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = BehaviorDependencyAnalyzer::new();
        
        let behavior_a = create_test_behavior_with_dependency("BehaviorA", "BehaviorB");
        let behavior_b = create_test_behavior_with_dependency("BehaviorB", "BehaviorA");
        
        let result = analyzer.analyze_dependencies(vec![behavior_a, behavior_b]).unwrap();
        
        assert_eq!(result.circular_dependencies.len(), 1);
        assert!(result.initialization_order.is_none());
        
        let cycle = &result.circular_dependencies[0];
        assert_eq!(cycle.cycle.len(), 2);
        assert!(cycle.cycle.contains(&"BehaviorA".to_string()));
        assert!(cycle.cycle.contains(&"BehaviorB".to_string()));
    }

    #[test]
    fn test_missing_dependency() {
        let mut analyzer = BehaviorDependencyAnalyzer::new();
        
        let behavior_a = create_test_behavior_with_dependency("BehaviorA", "NonExistentBehavior");
        
        let result = analyzer.analyze_dependencies(vec![behavior_a]).unwrap();
        
        assert_eq!(result.errors.len(), 1);
        if let DependencyError::MissingDependency { behavior, missing_dependency } = &result.errors[0] {
            assert_eq!(behavior, "BehaviorA");
            assert_eq!(missing_dependency, "NonExistentBehavior");
        } else {
            panic!("Expected MissingDependency error");
        }
    }

    #[test]
    fn test_complex_dependency_chain() {
        let mut analyzer = BehaviorDependencyAnalyzer::new();
        
        let behavior_a = create_test_behavior("BehaviorA");
        let behavior_b = create_test_behavior_with_dependency("BehaviorB", "BehaviorA");
        let behavior_c = create_test_behavior_with_dependency("BehaviorC", "BehaviorB");
        
        let result = analyzer.analyze_dependencies(vec![behavior_a, behavior_b, behavior_c]).unwrap();
        
        assert_eq!(result.circular_dependencies.len(), 0);
        assert!(result.initialization_order.is_some());
        
        let order = result.initialization_order.unwrap();
        assert_eq!(order, vec!["BehaviorA", "BehaviorB", "BehaviorC"]);
    }

    #[test]
    fn test_dependency_metrics() {
        let mut analyzer = BehaviorDependencyAnalyzer::new();
        
        let behavior_a = create_test_behavior("BehaviorA");
        let behavior_b = create_test_behavior_with_dependency("BehaviorB", "BehaviorA");
        let behavior_c = create_test_behavior("BehaviorC");
        
        let result = analyzer.analyze_dependencies(vec![behavior_a, behavior_b, behavior_c]).unwrap();
        
        assert_eq!(result.metrics.total_behaviors, 3);
        assert_eq!(result.metrics.total_dependencies, 1);
        assert_eq!(result.metrics.max_dependencies, 1);
        assert_eq!(result.metrics.independent_behaviors.len(), 2); // A and C have no dependencies
        assert_eq!(result.metrics.leaf_behaviors.len(), 2); // B and C have no dependents
    }
}