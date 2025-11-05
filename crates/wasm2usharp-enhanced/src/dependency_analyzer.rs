//! Dependency analysis for multi-UdonBehaviour support
//! 
//! This module provides functionality to analyze dependencies between UdonBehaviour units,
//! detect circular dependencies, and optimize code sharing strategies.

use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use crate::analyzer::{BehaviorUnit, CallGraph, CallGraphNode, CrossBehaviorDependency, CircularDependency};

/// Analyzer for dependencies between UdonBehaviour units
pub struct DependencyAnalyzer {
    /// Dependency graph structure
    dependency_graph: DependencyGraph,
    /// Behavior units being analyzed
    behavior_units: Vec<BehaviorUnit>,
    /// Function call graph
    call_graph: Option<CallGraph>,
}

impl DependencyAnalyzer {
    /// Create a new dependency analyzer
    pub fn new() -> Self {
        Self {
            dependency_graph: DependencyGraph::new(),
            behavior_units: Vec::new(),
            call_graph: None,
        }
    }

    /// Initialize the analyzer with behavior units and call graph
    pub fn initialize(&mut self, behavior_units: Vec<BehaviorUnit>, call_graph: Option<CallGraph>) {
        self.behavior_units = behavior_units;
        self.call_graph = call_graph;
        self.build_dependency_graph().unwrap_or_else(|e| {
            log::error!("Failed to build dependency graph: {}", e);
        });
    }

    /// Build the dependency graph from behavior units
    fn build_dependency_graph(&mut self) -> Result<()> {
        // Clear existing graph
        self.dependency_graph = DependencyGraph::new();

        // Create nodes for each behavior
        for behavior in &self.behavior_units {
            let node = BehaviorNode {
                name: behavior.name.clone(),
                entry_function: behavior.entry_function.clone(),
                local_functions: behavior.local_functions.clone(),
                unity_events: behavior.unity_events.clone(),
                is_entry_point: true,
            };
            self.dependency_graph.nodes.insert(behavior.name.clone(), node);
        }

        // Create edges based on inter-behavior calls
        for behavior in &self.behavior_units {
            for inter_call in &behavior.inter_behavior_calls {
                let edge = DependencyEdge {
                    source: inter_call.source_behavior.clone(),
                    target: inter_call.target_behavior.clone(),
                    function_name: inter_call.function_name.clone(),
                    dependency_type: match inter_call.call_type {
                        crate::analyzer::CallType::Direct => DependencyType::Direct,
                        crate::analyzer::CallType::Event => DependencyType::Event,
                        crate::analyzer::CallType::Network => DependencyType::Network,
                    },
                    strength: self.calculate_dependency_strength(&inter_call.function_name),
                };

                self.dependency_graph.edges.push(edge);
            }
        }

        // Build adjacency lists for efficient traversal
        self.build_adjacency_lists();

        Ok(())
    }

    /// Build adjacency lists for efficient graph traversal
    fn build_adjacency_lists(&mut self) {
        self.dependency_graph.adjacency_list.clear();
        self.dependency_graph.reverse_adjacency_list.clear();

        for edge in &self.dependency_graph.edges {
            // Forward adjacency list
            self.dependency_graph.adjacency_list
                .entry(edge.source.clone())
                .or_insert_with(Vec::new)
                .push(edge.target.clone());

            // Reverse adjacency list
            self.dependency_graph.reverse_adjacency_list
                .entry(edge.target.clone())
                .or_insert_with(Vec::new)
                .push(edge.source.clone());
        }
    }

    /// Calculate the strength of a dependency based on function usage patterns
    fn calculate_dependency_strength(&self, function_name: &str) -> DependencyStrength {
        // Analyze function name patterns to determine dependency strength
        if function_name.contains("init") || function_name.contains("start") || function_name.contains("awake") {
            DependencyStrength::Critical
        } else if function_name.contains("update") || function_name.contains("fixed_update") {
            DependencyStrength::High
        } else if function_name.contains("event") || function_name.contains("trigger") {
            DependencyStrength::Medium
        } else {
            DependencyStrength::Low
        }
    }

    /// Get the dependency graph
    pub fn get_dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Perform topological sort to determine initialization order
    pub fn get_initialization_order(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Initialize in-degree count for all nodes
        for behavior_name in self.dependency_graph.nodes.keys() {
            in_degree.insert(behavior_name.clone(), 0);
        }

        // Calculate in-degrees (for initialization order, we want dependencies first)
        // So if B depends on A (B -> A), A should be initialized first
        // This means we count incoming edges to the source (dependents)
        for edge in &self.dependency_graph.edges {
            *in_degree.entry(edge.source.clone()).or_insert(0) += 1;
        }

        // Find nodes with no incoming edges
        for (behavior_name, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(behavior_name.clone());
            }
        }

        // Process nodes in topological order
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // Update in-degrees of dependents (reverse direction for initialization)
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

        // Check if all nodes were processed (no cycles)
        if result.len() != self.dependency_graph.nodes.len() {
            return Err(anyhow::anyhow!("Circular dependency detected - cannot determine initialization order"));
        }

        Ok(result)
    }

    /// Find all dependencies of a behavior (direct and transitive)
    pub fn get_all_dependencies(&self, behavior_name: &str) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        let mut visited = HashSet::new();
        self.collect_dependencies_recursive(behavior_name, &mut dependencies, &mut visited);
        dependencies
    }

    /// Recursively collect dependencies using depth-first search
    fn collect_dependencies_recursive(
        &self,
        behavior_name: &str,
        dependencies: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(behavior_name) {
            return;
        }
        visited.insert(behavior_name.to_string());

        if let Some(neighbors) = self.dependency_graph.adjacency_list.get(behavior_name) {
            for neighbor in neighbors {
                dependencies.insert(neighbor.clone());
                self.collect_dependencies_recursive(neighbor, dependencies, visited);
            }
        }
    }

    /// Find all dependents of a behavior (behaviors that depend on this one)
    pub fn get_all_dependents(&self, behavior_name: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        let mut visited = HashSet::new();
        self.collect_dependents_recursive(behavior_name, &mut dependents, &mut visited);
        dependents
    }

    /// Recursively collect dependents using depth-first search
    fn collect_dependents_recursive(
        &self,
        behavior_name: &str,
        dependents: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) {
        if visited.contains(behavior_name) {
            return;
        }
        visited.insert(behavior_name.to_string());

        if let Some(neighbors) = self.dependency_graph.reverse_adjacency_list.get(behavior_name) {
            for neighbor in neighbors {
                dependents.insert(neighbor.clone());
                self.collect_dependents_recursive(neighbor, dependents, visited);
            }
        }
    }

    /// Get direct dependencies of a behavior
    pub fn get_direct_dependencies(&self, behavior_name: &str) -> Vec<String> {
        self.dependency_graph.adjacency_list
            .get(behavior_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Get direct dependents of a behavior
    pub fn get_direct_dependents(&self, behavior_name: &str) -> Vec<String> {
        self.dependency_graph.reverse_adjacency_list
            .get(behavior_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if there's a dependency path between two behaviors
    pub fn has_dependency_path(&self, from: &str, to: &str) -> bool {
        let mut visited = HashSet::new();
        self.has_path_recursive(from, to, &mut visited)
    }

    /// Recursively check for dependency path
    fn has_path_recursive(&self, current: &str, target: &str, visited: &mut HashSet<String>) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(current) {
            return false;
        }
        visited.insert(current.to_string());

        if let Some(neighbors) = self.dependency_graph.adjacency_list.get(current) {
            for neighbor in neighbors {
                if self.has_path_recursive(neighbor, target, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Get dependency edges between two behaviors
    pub fn get_dependency_edges(&self, from: &str, to: &str) -> Vec<&DependencyEdge> {
        self.dependency_graph.edges
            .iter()
            .filter(|edge| edge.source == from && edge.target == to)
            .collect()
    }

    /// Get all dependency edges for a behavior
    pub fn get_behavior_edges(&self, behavior_name: &str) -> Vec<&DependencyEdge> {
        self.dependency_graph.edges
            .iter()
            .filter(|edge| edge.source == behavior_name || edge.target == behavior_name)
            .collect()
    }

    /// Detect circular dependencies using Tarjan's strongly connected components algorithm
    pub fn detect_circular_dependencies(&self) -> Result<Vec<CircularDependencyInfo>> {
        let sccs = self.find_strongly_connected_components();
        let mut circular_dependencies = Vec::new();

        for scc in sccs {
            if scc.len() > 1 {
                // This is a strongly connected component with multiple behaviors
                let cycle_info = self.analyze_circular_dependency(&scc)?;
                circular_dependencies.push(cycle_info);
            }
        }

        Ok(circular_dependencies)
    }

    /// Find strongly connected components using Tarjan's algorithm
    fn find_strongly_connected_components(&self) -> Vec<Vec<String>> {
        let mut tarjan = TarjanSCC::new(&self.dependency_graph);
        tarjan.find_sccs()
    }

    /// Analyze a circular dependency to provide detailed information
    fn analyze_circular_dependency(&self, behaviors: &[String]) -> Result<CircularDependencyInfo> {
        let mut cycle_path = Vec::new();
        let mut involved_functions = HashMap::new();
        let mut dependency_types = Vec::new();

        // Find a cycle path through the behaviors
        if let Some(path) = self.find_cycle_path(behaviors) {
            cycle_path = path.clone();
            
            // Collect information about each step in the cycle
            for i in 0..path.len() {
                let current = &path[i];
                let next = &path[(i + 1) % path.len()];
                
                // Find the dependency edge between current and next
                let edges = self.get_dependency_edges(current, next);
                if let Some(edge) = edges.first() {
                    involved_functions.insert(
                        format!("{} -> {}", current, next),
                        edge.function_name.clone()
                    );
                    dependency_types.push(edge.dependency_type.clone());
                }
            }
        }

        let severity = self.calculate_cycle_severity(&dependency_types);
        let suggestions = self.generate_cycle_resolution_suggestions(behaviors, &dependency_types);

        Ok(CircularDependencyInfo {
            behaviors: behaviors.to_vec(),
            cycle_path: cycle_path.clone(),
            involved_functions,
            dependency_types,
            severity,
            error_message: self.generate_cycle_error_message(behaviors, &cycle_path),
            resolution_suggestions: suggestions,
        })
    }

    /// Find a cycle path through the given behaviors
    fn find_cycle_path(&self, behaviors: &[String]) -> Option<Vec<String>> {
        // Use DFS to find a cycle path
        for start_behavior in behaviors {
            if let Some(path) = self.find_cycle_from_behavior(start_behavior, behaviors) {
                return Some(path);
            }
        }
        None
    }

    /// Find a cycle starting from a specific behavior
    fn find_cycle_from_behavior(&self, start: &str, allowed_behaviors: &[String]) -> Option<Vec<String>> {
        let mut path = Vec::new();
        let mut visited = HashSet::new();
        
        if self.dfs_find_cycle(start, start, allowed_behaviors, &mut path, &mut visited) {
            return Some(path);
        }
        
        None
    }

    /// Depth-first search to find a cycle
    fn dfs_find_cycle(
        &self,
        current: &str,
        target: &str,
        allowed_behaviors: &[String],
        path: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        path.push(current.to_string());
        
        if path.len() > 1 && current == target {
            return true; // Found cycle back to target
        }
        
        if visited.contains(current) {
            path.pop();
            return false; // Already visited, no cycle here
        }
        
        visited.insert(current.to_string());
        
        if let Some(neighbors) = self.dependency_graph.adjacency_list.get(current) {
            for neighbor in neighbors {
                if allowed_behaviors.contains(neighbor) {
                    if self.dfs_find_cycle(neighbor, target, allowed_behaviors, path, visited) {
                        return true;
                    }
                }
            }
        }
        
        visited.remove(current);
        path.pop();
        false
    }

    /// Calculate the severity of a circular dependency
    fn calculate_cycle_severity(&self, dependency_types: &[DependencyType]) -> CycleSeverity {
        let has_critical = dependency_types.iter().any(|dt| matches!(dt, DependencyType::Network));
        let has_high = dependency_types.iter().any(|dt| matches!(dt, DependencyType::Event));
        
        if has_critical {
            CycleSeverity::Critical
        } else if has_high {
            CycleSeverity::High
        } else {
            // Any circular dependency should be at least High severity
            // Direct dependencies in cycles are still problematic for initialization
            CycleSeverity::High
        }
    }

    /// Generate error message for circular dependency
    fn generate_cycle_error_message(&self, behaviors: &[String], cycle_path: &[String]) -> String {
        if cycle_path.is_empty() {
            return format!(
                "Circular dependency detected between behaviors: {}. \
                These behaviors depend on each other, creating an initialization deadlock.",
                behaviors.join(", ")
            );
        }

        let cycle_description = cycle_path.join(" -> ");
        format!(
            "Circular dependency detected: {} -> {}. \
            This creates an initialization deadlock where behaviors cannot be properly ordered. \
            The cycle involves {} behaviors and must be resolved before compilation can proceed.",
            cycle_description,
            cycle_path[0],
            behaviors.len()
        )
    }

    /// Generate suggestions for resolving circular dependencies
    fn generate_cycle_resolution_suggestions(
        &self,
        behaviors: &[String],
        dependency_types: &[DependencyType],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // General suggestions
        suggestions.push(
            "Consider refactoring the code to remove circular dependencies by:".to_string()
        );

        // Specific suggestions based on dependency types
        if dependency_types.iter().any(|dt| matches!(dt, DependencyType::Direct)) {
            suggestions.push(
                "• Moving shared functionality to a separate SharedRuntime class".to_string()
            );
            suggestions.push(
                "• Using dependency injection to break direct coupling".to_string()
            );
        }

        if dependency_types.iter().any(|dt| matches!(dt, DependencyType::Event)) {
            suggestions.push(
                "• Using Unity events or a mediator pattern for loose coupling".to_string()
            );
            suggestions.push(
                "• Implementing a message bus system for inter-behavior communication".to_string()
            );
        }

        if dependency_types.iter().any(|dt| matches!(dt, DependencyType::Network)) {
            suggestions.push(
                "• Reviewing network synchronization requirements".to_string()
            );
            suggestions.push(
                "• Using a centralized network manager to coordinate behaviors".to_string()
            );
        }

        // Behavior-specific suggestions
        if behaviors.len() == 2 {
            suggestions.push(format!(
                "• Consider merging {} and {} into a single behavior if they are tightly coupled",
                behaviors[0], behaviors[1]
            ));
        } else {
            suggestions.push(
                "• Break down complex behaviors into smaller, more focused components".to_string()
            );
        }

        suggestions.push(
            "• Use initialization phases to establish proper startup order".to_string()
        );

        suggestions
    }

    /// Check if the dependency graph has any circular dependencies
    pub fn has_circular_dependencies(&self) -> bool {
        let sccs = self.find_strongly_connected_components();
        sccs.iter().any(|scc| scc.len() > 1)
    }

    /// Validate the dependency graph and return any issues found
    pub fn validate_dependencies(&self) -> Result<DependencyValidationResult> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check for circular dependencies
        let circular_deps = self.detect_circular_dependencies()?;
        if !circular_deps.is_empty() {
            for cycle in &circular_deps {
                issues.push(DependencyIssue {
                    issue_type: DependencyIssueType::CircularDependency,
                    severity: match cycle.severity {
                        CycleSeverity::Critical => IssueSeverity::Error,
                        CycleSeverity::High => IssueSeverity::Error,
                        CycleSeverity::Medium => IssueSeverity::Warning,
                    },
                    message: cycle.error_message.clone(),
                    affected_behaviors: cycle.behaviors.clone(),
                    suggestions: cycle.resolution_suggestions.clone(),
                });
            }
        }

        // Check for missing dependencies
        self.check_missing_dependencies(&mut issues, &mut warnings)?;

        // Check for excessive dependencies
        self.check_excessive_dependencies(&mut warnings)?;

        // Check for isolated behaviors
        self.check_isolated_behaviors(&mut warnings)?;

        let is_valid = issues.iter().all(|issue| issue.severity != IssueSeverity::Error);

        Ok(DependencyValidationResult {
            is_valid,
            issues,
            warnings,
            circular_dependencies: circular_deps,
        })
    }

    /// Check for missing dependencies
    fn check_missing_dependencies(
        &self,
        issues: &mut Vec<DependencyIssue>,
        _warnings: &mut Vec<DependencyIssue>,
    ) -> Result<()> {
        // This would require more sophisticated analysis of function calls
        // For now, we'll implement a basic check
        for behavior in &self.behavior_units {
            for inter_call in &behavior.inter_behavior_calls {
                if !self.dependency_graph.nodes.contains_key(&inter_call.target_behavior) {
                    issues.push(DependencyIssue {
                        issue_type: DependencyIssueType::MissingDependency,
                        severity: IssueSeverity::Error,
                        message: format!(
                            "Behavior '{}' calls function '{}' from undefined behavior '{}'",
                            behavior.name, inter_call.function_name, inter_call.target_behavior
                        ),
                        affected_behaviors: vec![behavior.name.clone(), inter_call.target_behavior.clone()],
                        suggestions: vec![
                            format!("Ensure behavior '{}' is properly defined", inter_call.target_behavior),
                            "Check for typos in behavior names".to_string(),
                            "Verify that all required behaviors are included in the compilation".to_string(),
                        ],
                    });
                }
            }
        }
        Ok(())
    }

    /// Check for excessive dependencies
    fn check_excessive_dependencies(&self, warnings: &mut Vec<DependencyIssue>) -> Result<()> {
        const MAX_DEPENDENCIES_WARNING: usize = 5;
        
        for (behavior_name, _node) in &self.dependency_graph.nodes {
            let dependency_count = self.get_direct_dependencies(behavior_name).len();
            
            if dependency_count > MAX_DEPENDENCIES_WARNING {
                warnings.push(DependencyIssue {
                    issue_type: DependencyIssueType::ExcessiveDependencies,
                    severity: IssueSeverity::Warning,
                    message: format!(
                        "Behavior '{}' has {} dependencies, which may indicate tight coupling",
                        behavior_name, dependency_count
                    ),
                    affected_behaviors: vec![behavior_name.clone()],
                    suggestions: vec![
                        "Consider breaking down this behavior into smaller components".to_string(),
                        "Move shared functionality to a common base class or utility".to_string(),
                        "Use dependency injection to reduce direct coupling".to_string(),
                    ],
                });
            }
        }
        Ok(())
    }

    /// Check for isolated behaviors
    fn check_isolated_behaviors(&self, warnings: &mut Vec<DependencyIssue>) -> Result<()> {
        for (behavior_name, _node) in &self.dependency_graph.nodes {
            let dependencies = self.get_direct_dependencies(behavior_name);
            let dependents = self.get_direct_dependents(behavior_name);
            
            if dependencies.is_empty() && dependents.is_empty() {
                warnings.push(DependencyIssue {
                    issue_type: DependencyIssueType::IsolatedBehavior,
                    severity: IssueSeverity::Info,
                    message: format!(
                        "Behavior '{}' has no dependencies or dependents - it may be unused",
                        behavior_name
                    ),
                    affected_behaviors: vec![behavior_name.clone()],
                    suggestions: vec![
                        "Verify that this behavior is actually needed".to_string(),
                        "Consider removing unused behaviors to reduce complexity".to_string(),
                        "Check if this behavior should be connected to others".to_string(),
                    ],
                });
            }
        }
        Ok(())
    }

    /// Identify functions that should be moved to SharedRuntime
    pub fn identify_shared_functions(&self) -> Result<SharedFunctionAnalysis> {
        let mut shared_functions = HashMap::new();
        let mut function_usage = HashMap::new();
        let mut sharing_opportunities = Vec::new();

        // Analyze function usage across behaviors
        self.analyze_function_usage(&mut function_usage)?;

        // Identify functions used by multiple behaviors
        for (function_name, usage_info) in &function_usage {
            if usage_info.used_by_behaviors.len() > 1 {
                let sharing_benefit = self.calculate_sharing_benefit(usage_info);
                let sharing_strategy = self.determine_sharing_strategy(function_name, usage_info);

                shared_functions.insert(function_name.clone(), SharedFunctionInfo {
                    function_name: function_name.clone(),
                    used_by_behaviors: usage_info.used_by_behaviors.clone(),
                    call_frequency: usage_info.total_calls,
                    sharing_benefit,
                    sharing_strategy: sharing_strategy.clone(),
                    estimated_size_reduction: self.estimate_size_reduction(usage_info),
                });

                sharing_opportunities.push(SharingOpportunity {
                    function_name: function_name.clone(),
                    strategy: sharing_strategy,
                    benefit_score: sharing_benefit,
                    affected_behaviors: usage_info.used_by_behaviors.clone(),
                });
            }
        }

        // Sort sharing opportunities by benefit
        sharing_opportunities.sort_by(|a, b| b.benefit_score.partial_cmp(&a.benefit_score).unwrap_or(std::cmp::Ordering::Equal));

        // Identify utility functions that should always be shared
        let utility_functions = self.identify_utility_functions(&function_usage)?;

        // Calculate overall sharing metrics
        let total_functions = function_usage.len();
        let shareable_functions = shared_functions.len();
        let sharing_ratio = if total_functions > 0 {
            shareable_functions as f64 / total_functions as f64
        } else {
            0.0
        };

        let estimated_total_size_reduction = sharing_opportunities.iter()
            .map(|op| shared_functions.get(&op.function_name)
                .map(|info| info.estimated_size_reduction)
                .unwrap_or(0))
            .sum();

        Ok(SharedFunctionAnalysis {
            shared_functions,
            sharing_opportunities,
            utility_functions,
            total_functions,
            shareable_functions,
            sharing_ratio,
            estimated_total_size_reduction,
        })
    }

    /// Analyze how functions are used across behaviors
    fn analyze_function_usage(&self, function_usage: &mut HashMap<String, FunctionUsageInfo>) -> Result<()> {
        // Initialize usage info for all functions
        for behavior in &self.behavior_units {
            for function_name in &behavior.local_functions {
                function_usage.entry(function_name.clone()).or_insert_with(|| FunctionUsageInfo {
                    function_name: function_name.clone(),
                    used_by_behaviors: HashSet::new(),
                    total_calls: 0,
                    call_sites: Vec::new(),
                    is_entry_point: function_name == &behavior.entry_function,
                    function_type: self.classify_function_type(function_name),
                });
            }
        }

        // Analyze inter-behavior calls to track usage
        for behavior in &self.behavior_units {
            for inter_call in &behavior.inter_behavior_calls {
                if let Some(usage_info) = function_usage.get_mut(&inter_call.function_name) {
                    usage_info.used_by_behaviors.insert(behavior.name.clone());
                    usage_info.total_calls += 1;
                    usage_info.call_sites.push(CallSiteInfo {
                        calling_behavior: behavior.name.clone(),
                        called_function: inter_call.function_name.clone(),
                        call_type: inter_call.call_type.clone(),
                    });
                }
            }
        }

        // Mark functions that are used within their own behavior
        for behavior in &self.behavior_units {
            for function_name in &behavior.local_functions {
                if let Some(usage_info) = function_usage.get_mut(function_name) {
                    usage_info.used_by_behaviors.insert(behavior.name.clone());
                }
            }
        }

        Ok(())
    }

    /// Classify the type of a function based on its name and usage patterns
    fn classify_function_type(&self, function_name: &str) -> FunctionType {
        let name_lower = function_name.to_lowercase();

        // Unity event functions
        if name_lower.contains("start") || name_lower.contains("awake") || 
           name_lower.contains("update") || name_lower.contains("fixed_update") ||
           name_lower.contains("on_enable") || name_lower.contains("on_disable") {
            return FunctionType::UnityEvent;
        }

        // Utility functions
        if name_lower.contains("calculate") || name_lower.contains("compute") ||
           name_lower.contains("convert") || name_lower.contains("format") ||
           name_lower.contains("parse") || name_lower.contains("validate") ||
           name_lower.contains("helper") || name_lower.contains("util") {
            return FunctionType::Utility;
        }

        // Data access functions
        if name_lower.contains("get") || name_lower.contains("set") ||
           name_lower.contains("load") || name_lower.contains("save") ||
           name_lower.contains("read") || name_lower.contains("write") {
            return FunctionType::DataAccess;
        }

        // Event handlers
        if name_lower.starts_with("on_") || name_lower.contains("handle") ||
           name_lower.contains("event") || name_lower.contains("trigger") {
            return FunctionType::EventHandler;
        }

        // Business logic (default)
        FunctionType::BusinessLogic
    }

    /// Calculate the benefit of sharing a function
    fn calculate_sharing_benefit(&self, usage_info: &FunctionUsageInfo) -> f64 {
        let behavior_count = usage_info.used_by_behaviors.len() as f64;
        let call_frequency = usage_info.total_calls as f64;
        
        // Base benefit from code deduplication
        let deduplication_benefit = (behavior_count - 1.0) * 10.0;
        
        // Additional benefit from call frequency
        let frequency_benefit = call_frequency * 2.0;
        
        // Type-based multiplier
        let type_multiplier = match usage_info.function_type {
            FunctionType::Utility => 2.0,        // High benefit for utilities
            FunctionType::DataAccess => 1.5,     // Medium-high benefit
            FunctionType::BusinessLogic => 1.0,  // Standard benefit
            FunctionType::EventHandler => 0.8,   // Lower benefit (often behavior-specific)
            FunctionType::UnityEvent => 0.3,     // Very low benefit (usually not shareable)
        };

        // Penalty for entry points (should not be shared)
        let entry_point_penalty = if usage_info.is_entry_point { -50.0 } else { 0.0 };

        (deduplication_benefit + frequency_benefit) * type_multiplier + entry_point_penalty
    }

    /// Determine the best sharing strategy for a function
    fn determine_sharing_strategy(&self, _function_name: &str, usage_info: &FunctionUsageInfo) -> SharingStrategy {
        // Entry points should not be shared
        if usage_info.is_entry_point {
            return SharingStrategy::NoSharing;
        }

        // Unity events are typically not shareable
        if matches!(usage_info.function_type, FunctionType::UnityEvent) {
            return SharingStrategy::NoSharing;
        }

        // High-benefit functions should be moved to SharedRuntime
        let benefit = self.calculate_sharing_benefit(usage_info);
        if benefit > 20.0 {
            return SharingStrategy::MoveToSharedRuntime;
        }

        // Medium-benefit functions can use static methods
        if benefit > 10.0 {
            return SharingStrategy::StaticMethod;
        }

        // Low-benefit functions might use interfaces
        if benefit > 5.0 && usage_info.used_by_behaviors.len() > 2 {
            return SharingStrategy::InterfaceMethod;
        }

        // Very low benefit - keep duplicated
        SharingStrategy::NoSharing
    }

    /// Estimate the size reduction from sharing a function
    fn estimate_size_reduction(&self, usage_info: &FunctionUsageInfo) -> usize {
        // Rough estimate based on function type and usage
        let base_size = match usage_info.function_type {
            FunctionType::Utility => 50,         // Small utility functions
            FunctionType::DataAccess => 100,     // Medium data access functions
            FunctionType::BusinessLogic => 200,  // Larger business logic functions
            FunctionType::EventHandler => 75,    // Medium event handlers
            FunctionType::UnityEvent => 150,     // Unity event methods
        };

        // Size reduction = base_size * (behaviors_using_it - 1)
        // Minus overhead for shared runtime access
        let behaviors_using = usage_info.used_by_behaviors.len();
        if behaviors_using > 1 {
            base_size * (behaviors_using - 1) - 20 // 20 bytes overhead for shared access
        } else {
            0
        }
    }

    /// Identify utility functions that should always be shared
    fn identify_utility_functions(&self, function_usage: &HashMap<String, FunctionUsageInfo>) -> Result<Vec<String>> {
        let mut utility_functions = Vec::new();

        for (function_name, usage_info) in function_usage {
            // Always share utility functions used by multiple behaviors
            if matches!(usage_info.function_type, FunctionType::Utility) && 
               usage_info.used_by_behaviors.len() > 1 {
                utility_functions.push(function_name.clone());
            }

            // Always share common data access functions
            if matches!(usage_info.function_type, FunctionType::DataAccess) && 
               usage_info.used_by_behaviors.len() > 2 {
                utility_functions.push(function_name.clone());
            }
        }

        utility_functions.sort();
        Ok(utility_functions)
    }

    /// Generate optimization recommendations for code sharing
    pub fn generate_sharing_recommendations(&self) -> Result<Vec<SharingRecommendation>> {
        let analysis = self.identify_shared_functions()?;
        let mut recommendations = Vec::new();

        // Recommend high-benefit sharing opportunities
        for opportunity in &analysis.sharing_opportunities {
            if opportunity.benefit_score > 15.0 {
                recommendations.push(SharingRecommendation {
                    recommendation_type: RecommendationType::HighPrioritySharing,
                    function_name: opportunity.function_name.clone(),
                    strategy: opportunity.strategy.clone(),
                    benefit_score: opportunity.benefit_score,
                    description: format!(
                        "Move '{}' to SharedRuntime - used by {} behaviors with high benefit score ({:.1})",
                        opportunity.function_name,
                        opportunity.affected_behaviors.len(),
                        opportunity.benefit_score
                    ),
                    affected_behaviors: opportunity.affected_behaviors.iter().cloned().collect(),
                });
            }
        }

        // Recommend utility function consolidation
        if !analysis.utility_functions.is_empty() {
            recommendations.push(SharingRecommendation {
                recommendation_type: RecommendationType::UtilityConsolidation,
                function_name: "multiple".to_string(),
                strategy: SharingStrategy::MoveToSharedRuntime,
                benefit_score: analysis.utility_functions.len() as f64 * 5.0,
                description: format!(
                    "Consolidate {} utility functions into SharedRuntime for better code organization",
                    analysis.utility_functions.len()
                ),
                affected_behaviors: analysis.shared_functions.values()
                    .flat_map(|info| info.used_by_behaviors.iter())
                    .cloned()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect(),
            });
        }

        // Recommend interface extraction for complex sharing scenarios
        let complex_sharing_functions: Vec<_> = analysis.sharing_opportunities.iter()
            .filter(|op| op.affected_behaviors.len() > 3 && op.benefit_score > 10.0)
            .collect();

        if !complex_sharing_functions.is_empty() {
            recommendations.push(SharingRecommendation {
                recommendation_type: RecommendationType::InterfaceExtraction,
                function_name: "multiple".to_string(),
                strategy: SharingStrategy::InterfaceMethod,
                benefit_score: complex_sharing_functions.len() as f64 * 8.0,
                description: format!(
                    "Consider extracting interfaces for {} functions used across many behaviors",
                    complex_sharing_functions.len()
                ),
                affected_behaviors: complex_sharing_functions.iter()
                    .flat_map(|op| op.affected_behaviors.iter())
                    .cloned()
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect(),
            });
        }

        // Sort recommendations by benefit score
        recommendations.sort_by(|a, b| b.benefit_score.partial_cmp(&a.benefit_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(recommendations)
    }

    /// Calculate dependency metrics for analysis
    pub fn calculate_dependency_metrics(&self) -> DependencyMetrics {
        let total_behaviors = self.dependency_graph.nodes.len();
        let total_dependencies = self.dependency_graph.edges.len();

        let mut in_degrees = HashMap::new();
        let mut out_degrees = HashMap::new();

        // Initialize degree counts
        for behavior_name in self.dependency_graph.nodes.keys() {
            in_degrees.insert(behavior_name.clone(), 0);
            out_degrees.insert(behavior_name.clone(), 0);
        }

        // Calculate degrees
        for edge in &self.dependency_graph.edges {
            *out_degrees.entry(edge.source.clone()).or_insert(0) += 1;
            *in_degrees.entry(edge.target.clone()).or_insert(0) += 1;
        }

        let max_in_degree = in_degrees.values().max().copied().unwrap_or(0);
        let max_out_degree = out_degrees.values().max().copied().unwrap_or(0);
        let avg_in_degree = if total_behaviors > 0 {
            in_degrees.values().sum::<usize>() as f64 / total_behaviors as f64
        } else {
            0.0
        };
        let avg_out_degree = if total_behaviors > 0 {
            out_degrees.values().sum::<usize>() as f64 / total_behaviors as f64
        } else {
            0.0
        };

        // Find behaviors with no dependencies (independent)
        let independent_behaviors: Vec<String> = in_degrees
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        // Find behaviors with no dependents (leaf nodes)
        let leaf_behaviors: Vec<String> = out_degrees
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();

        DependencyMetrics {
            total_behaviors,
            total_dependencies,
            max_in_degree,
            max_out_degree,
            avg_in_degree,
            avg_out_degree,
            independent_behaviors,
            leaf_behaviors,
        }
    }
}

/// Dependency graph data structure
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes representing behaviors
    pub nodes: HashMap<String, BehaviorNode>,
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
pub struct BehaviorNode {
    /// Behavior name
    pub name: String,
    /// Entry function name
    pub entry_function: String,
    /// Functions belonging to this behavior
    pub local_functions: HashSet<String>,
    /// Unity events handled by this behavior
    pub unity_events: Vec<String>,
    /// Whether this is an entry point behavior
    pub is_entry_point: bool,
}

/// Edge in the dependency graph representing a dependency
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Source behavior (dependent)
    pub source: String,
    /// Target behavior (dependency)
    pub target: String,
    /// Function being called
    pub function_name: String,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Strength of the dependency
    pub strength: DependencyStrength,
}

/// Type of dependency between behaviors
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    /// Direct method call
    Direct,
    /// Event-based dependency
    Event,
    /// Network-based dependency
    Network,
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
    /// Critical dependency (initialization, etc.)
    Critical,
}

/// Metrics about the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyMetrics {
    /// Total number of behaviors
    pub total_behaviors: usize,
    /// Total number of dependencies
    pub total_dependencies: usize,
    /// Maximum in-degree (most dependencies)
    pub max_in_degree: usize,
    /// Maximum out-degree (most dependents)
    pub max_out_degree: usize,
    /// Average in-degree
    pub avg_in_degree: f64,
    /// Average out-degree
    pub avg_out_degree: f64,
    /// Behaviors with no dependencies
    pub independent_behaviors: Vec<String>,
    /// Behaviors with no dependents
    pub leaf_behaviors: Vec<String>,
}

/// Information about a circular dependency
#[derive(Debug, Clone)]
pub struct CircularDependencyInfo {
    /// Behaviors involved in the circular dependency
    pub behaviors: Vec<String>,
    /// Path showing the cycle (if found)
    pub cycle_path: Vec<String>,
    /// Functions involved in each step of the cycle
    pub involved_functions: HashMap<String, String>,
    /// Types of dependencies in the cycle
    pub dependency_types: Vec<DependencyType>,
    /// Severity of the circular dependency
    pub severity: CycleSeverity,
    /// Human-readable error message
    pub error_message: String,
    /// Suggestions for resolving the circular dependency
    pub resolution_suggestions: Vec<String>,
}

/// Severity level of a circular dependency
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CycleSeverity {
    /// Low impact circular dependency
    Medium,
    /// High impact circular dependency
    High,
    /// Critical circular dependency that prevents compilation
    Critical,
}

/// Result of dependency validation
#[derive(Debug, Clone)]
pub struct DependencyValidationResult {
    /// Whether the dependency graph is valid
    pub is_valid: bool,
    /// Critical issues that prevent compilation
    pub issues: Vec<DependencyIssue>,
    /// Non-critical warnings
    pub warnings: Vec<DependencyIssue>,
    /// Detected circular dependencies
    pub circular_dependencies: Vec<CircularDependencyInfo>,
}

/// A dependency issue found during validation
#[derive(Debug, Clone)]
pub struct DependencyIssue {
    /// Type of issue
    pub issue_type: DependencyIssueType,
    /// Severity level
    pub severity: IssueSeverity,
    /// Human-readable message
    pub message: String,
    /// Behaviors affected by this issue
    pub affected_behaviors: Vec<String>,
    /// Suggestions for resolving the issue
    pub suggestions: Vec<String>,
}

/// Type of dependency issue
#[derive(Debug, Clone, PartialEq)]
pub enum DependencyIssueType {
    /// Circular dependency detected
    CircularDependency,
    /// Missing dependency
    MissingDependency,
    /// Too many dependencies (tight coupling)
    ExcessiveDependencies,
    /// Behavior with no connections
    IsolatedBehavior,
}

/// Severity level of an issue
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error that prevents compilation
    Error,
}

/// Analysis result for shared function identification
#[derive(Debug, Clone)]
pub struct SharedFunctionAnalysis {
    /// Functions that should be shared
    pub shared_functions: HashMap<String, SharedFunctionInfo>,
    /// Opportunities for code sharing
    pub sharing_opportunities: Vec<SharingOpportunity>,
    /// Utility functions that should always be shared
    pub utility_functions: Vec<String>,
    /// Total number of functions analyzed
    pub total_functions: usize,
    /// Number of functions that can be shared
    pub shareable_functions: usize,
    /// Ratio of shareable to total functions
    pub sharing_ratio: f64,
    /// Estimated total size reduction from sharing
    pub estimated_total_size_reduction: usize,
}

/// Information about a function that should be shared
#[derive(Debug, Clone)]
pub struct SharedFunctionInfo {
    /// Function name
    pub function_name: String,
    /// Behaviors that use this function
    pub used_by_behaviors: HashSet<String>,
    /// How often this function is called
    pub call_frequency: usize,
    /// Benefit score for sharing this function
    pub sharing_benefit: f64,
    /// Recommended sharing strategy
    pub sharing_strategy: SharingStrategy,
    /// Estimated size reduction from sharing
    pub estimated_size_reduction: usize,
}

/// An opportunity for code sharing
#[derive(Debug, Clone)]
pub struct SharingOpportunity {
    /// Function name
    pub function_name: String,
    /// Recommended sharing strategy
    pub strategy: SharingStrategy,
    /// Benefit score for this opportunity
    pub benefit_score: f64,
    /// Behaviors that would be affected
    pub affected_behaviors: HashSet<String>,
}

/// Strategy for sharing a function
#[derive(Debug, Clone, PartialEq)]
pub enum SharingStrategy {
    /// Don't share this function
    NoSharing,
    /// Move to SharedRuntime class
    MoveToSharedRuntime,
    /// Create as static method
    StaticMethod,
    /// Use interface method
    InterfaceMethod,
}

/// Information about how a function is used
#[derive(Debug, Clone)]
pub struct FunctionUsageInfo {
    /// Function name
    pub function_name: String,
    /// Behaviors that use this function
    pub used_by_behaviors: HashSet<String>,
    /// Total number of calls to this function
    pub total_calls: usize,
    /// Call sites where this function is used
    pub call_sites: Vec<CallSiteInfo>,
    /// Whether this is an entry point function
    pub is_entry_point: bool,
    /// Type of function
    pub function_type: FunctionType,
}

/// Information about a call site
#[derive(Debug, Clone)]
pub struct CallSiteInfo {
    /// Behavior making the call
    pub calling_behavior: String,
    /// Function being called
    pub called_function: String,
    /// Type of call
    pub call_type: crate::analyzer::CallType,
}

/// Type of function for sharing analysis
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    /// Unity event function (Start, Update, etc.)
    UnityEvent,
    /// Utility function (calculate, format, etc.)
    Utility,
    /// Data access function (get, set, load, save)
    DataAccess,
    /// Event handler function
    EventHandler,
    /// Business logic function
    BusinessLogic,
}

/// Recommendation for code sharing optimization
#[derive(Debug, Clone)]
pub struct SharingRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,
    /// Function name (or "multiple" for group recommendations)
    pub function_name: String,
    /// Recommended sharing strategy
    pub strategy: SharingStrategy,
    /// Benefit score for this recommendation
    pub benefit_score: f64,
    /// Human-readable description
    pub description: String,
    /// Behaviors affected by this recommendation
    pub affected_behaviors: Vec<String>,
}

/// Type of sharing recommendation
#[derive(Debug, Clone, PartialEq)]
pub enum RecommendationType {
    /// High priority sharing opportunity
    HighPrioritySharing,
    /// Consolidate utility functions
    UtilityConsolidation,
    /// Extract interfaces for complex sharing
    InterfaceExtraction,
}

/// Tarjan's strongly connected components algorithm implementation
struct TarjanSCC<'a> {
    graph: &'a DependencyGraph,
    index: usize,
    stack: Vec<String>,
    indices: HashMap<String, usize>,
    lowlinks: HashMap<String, usize>,
    on_stack: HashSet<String>,
    sccs: Vec<Vec<String>>,
}

impl<'a> TarjanSCC<'a> {
    fn new(graph: &'a DependencyGraph) -> Self {
        Self {
            graph,
            index: 0,
            stack: Vec::new(),
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            sccs: Vec::new(),
        }
    }

    fn find_sccs(mut self) -> Vec<Vec<String>> {
        // Run algorithm on all nodes
        for behavior_name in self.graph.nodes.keys() {
            if !self.indices.contains_key(behavior_name) {
                self.strongconnect(behavior_name.clone());
            }
        }
        self.sccs
    }

    fn strongconnect(&mut self, v: String) {
        // Set the depth index for v to the smallest unused index
        self.indices.insert(v.clone(), self.index);
        self.lowlinks.insert(v.clone(), self.index);
        self.index += 1;
        self.stack.push(v.clone());
        self.on_stack.insert(v.clone());

        // Consider successors of v
        if let Some(successors) = self.graph.adjacency_list.get(&v) {
            for w in successors {
                if !self.indices.contains_key(w) {
                    // Successor w has not yet been visited; recurse on it
                    self.strongconnect(w.clone());
                    let v_lowlink = self.lowlinks[&v];
                    let w_lowlink = self.lowlinks[w];
                    self.lowlinks.insert(v.clone(), v_lowlink.min(w_lowlink));
                } else if self.on_stack.contains(w) {
                    // Successor w is in stack and hence in the current SCC
                    let v_lowlink = self.lowlinks[&v];
                    let w_index = self.indices[w];
                    self.lowlinks.insert(v.clone(), v_lowlink.min(w_index));
                }
            }
        }

        // If v is a root node, pop the stack and create an SCC
        if self.lowlinks[&v] == self.indices[&v] {
            let mut scc = Vec::new();
            loop {
                let w = self.stack.pop().unwrap();
                self.on_stack.remove(&w);
                scc.push(w.clone());
                if w == v {
                    break;
                }
            }
            self.sccs.push(scc);
        }
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{BehaviorUnit, InterBehaviorCall};
    use udonsharp_core::attributes::UdonBehaviourMarker;

    fn create_test_behavior_unit(name: &str, entry_function: &str) -> BehaviorUnit {
        BehaviorUnit {
            name: name.to_string(),
            entry_function: entry_function.to_string(),
            entry_function_index: 0,
            unity_events: vec!["Start".to_string()],
            local_functions: [entry_function.to_string()].into_iter().collect(),
            shared_dependencies: HashSet::new(),
            inter_behavior_calls: Vec::new(),
            attribute_config: UdonBehaviourMarker {
                name: Some(name.to_string()),
                events: vec!["Start".to_string()],
                dependencies: Vec::new(),
                auto_sync: false,
            },
        }
    }

    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        assert_eq!(analyzer.dependency_graph.nodes.len(), 0);
        assert_eq!(analyzer.dependency_graph.edges.len(), 0);
    }

    #[test]
    fn test_initialization_order_simple() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        
        // B depends on A (B calls A)
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "some_function".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b], None);
        
        let order = analyzer.get_initialization_order().unwrap();
        // B depends on A, so A should be initialized first
        assert_eq!(order, vec!["BehaviorA", "BehaviorB"]);
    }

    #[test]
    fn test_dependency_detection() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "some_function".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b], None);
        
        let deps = analyzer.get_direct_dependencies("BehaviorB");
        assert_eq!(deps, vec!["BehaviorA"]);
        
        let deps = analyzer.get_direct_dependencies("BehaviorA");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_dependency_path_detection() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        let mut behavior_c = create_test_behavior_unit("BehaviorC", "behavior_c_start");
        
        // C -> B -> A
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "func_a".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });
        
        behavior_c.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorC".to_string(),
            target_behavior: "BehaviorB".to_string(),
            function_name: "func_b".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b, behavior_c], None);
        
        assert!(analyzer.has_dependency_path("BehaviorC", "BehaviorA"));
        assert!(analyzer.has_dependency_path("BehaviorB", "BehaviorA"));
        assert!(!analyzer.has_dependency_path("BehaviorA", "BehaviorC"));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        
        // Create circular dependency: A -> B -> A
        behavior_a.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorA".to_string(),
            target_behavior: "BehaviorB".to_string(),
            function_name: "func_b".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });
        
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "func_a".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b], None);
        
        assert!(analyzer.has_circular_dependencies());
        
        let circular_deps = analyzer.detect_circular_dependencies().unwrap();
        assert_eq!(circular_deps.len(), 1);
        assert_eq!(circular_deps[0].behaviors.len(), 2);
        assert!(circular_deps[0].behaviors.contains(&"BehaviorA".to_string()));
        assert!(circular_deps[0].behaviors.contains(&"BehaviorB".to_string()));
    }

    #[test]
    fn test_no_circular_dependency() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        let mut behavior_c = create_test_behavior_unit("BehaviorC", "behavior_c_start");
        
        // Linear dependency: C -> B -> A (no cycle)
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "func_a".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });
        
        behavior_c.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorC".to_string(),
            target_behavior: "BehaviorB".to_string(),
            function_name: "func_b".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b, behavior_c], None);
        
        assert!(!analyzer.has_circular_dependencies());
        
        let circular_deps = analyzer.detect_circular_dependencies().unwrap();
        assert_eq!(circular_deps.len(), 0);
    }

    #[test]
    fn test_dependency_validation() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        
        // Create circular dependency
        behavior_a.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorA".to_string(),
            target_behavior: "BehaviorB".to_string(),
            function_name: "func_b".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });
        
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "func_a".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b], None);
        
        let validation_result = analyzer.validate_dependencies().unwrap();
        assert!(!validation_result.is_valid);
        assert!(!validation_result.issues.is_empty());
        assert_eq!(validation_result.circular_dependencies.len(), 1);
    }

    #[test]
    fn test_shared_function_identification() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        let mut behavior_c = create_test_behavior_unit("BehaviorC", "behavior_c_start");
        
        // Add a utility function to all behaviors
        behavior_a.local_functions.insert("calculate_distance".to_string());
        behavior_b.local_functions.insert("calculate_distance".to_string());
        behavior_c.local_functions.insert("calculate_distance".to_string());
        
        // Add inter-behavior calls to the utility function
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "calculate_distance".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });
        
        behavior_c.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorC".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "calculate_distance".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b, behavior_c], None);
        
        let analysis = analyzer.identify_shared_functions().unwrap();
        assert!(analysis.shared_functions.contains_key("calculate_distance"));
        assert!(analysis.sharing_ratio > 0.0);
        assert!(!analysis.sharing_opportunities.is_empty());
    }

    #[test]
    fn test_function_type_classification() {
        let analyzer = DependencyAnalyzer::new();
        
        assert_eq!(analyzer.classify_function_type("player_start"), FunctionType::UnityEvent);
        assert_eq!(analyzer.classify_function_type("calculate_distance"), FunctionType::Utility);
        assert_eq!(analyzer.classify_function_type("get_player_data"), FunctionType::DataAccess);
        assert_eq!(analyzer.classify_function_type("on_trigger_enter"), FunctionType::EventHandler);
        assert_eq!(analyzer.classify_function_type("process_game_logic"), FunctionType::BusinessLogic);
    }

    #[test]
    fn test_sharing_recommendations() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let mut behavior_a = create_test_behavior_unit("BehaviorA", "behavior_a_start");
        let mut behavior_b = create_test_behavior_unit("BehaviorB", "behavior_b_start");
        
        // Add utility functions that should be shared
        behavior_a.local_functions.insert("format_message".to_string());
        behavior_b.local_functions.insert("format_message".to_string());
        
        behavior_b.inter_behavior_calls.push(InterBehaviorCall {
            source_behavior: "BehaviorB".to_string(),
            target_behavior: "BehaviorA".to_string(),
            function_name: "format_message".to_string(),
            call_type: crate::analyzer::CallType::Direct,
        });

        analyzer.initialize(vec![behavior_a, behavior_b], None);
        
        let recommendations = analyzer.generate_sharing_recommendations().unwrap();
        assert!(!recommendations.is_empty());
        
        // Should recommend sharing the utility function
        let has_utility_recommendation = recommendations.iter()
            .any(|rec| rec.function_name == "format_message" || 
                      rec.recommendation_type == RecommendationType::UtilityConsolidation);
        assert!(has_utility_recommendation);
    }
}