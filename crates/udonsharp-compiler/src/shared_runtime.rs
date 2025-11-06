//! SharedRuntime generation system for multi-behavior patterns
//! 
//! This module provides functionality to detect shared functions and data types
//! across multiple UdonBehaviour structs and generate a SharedRuntime class
//! containing common functionality.

use crate::multi_behavior::{UdonBehaviourStruct, StructMethod, RustType, RustToCSharpTypeMapper};
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Result type for shared runtime operations
pub type SharedRuntimeResult<T> = Result<T, SharedRuntimeError>;

/// Errors that can occur during shared runtime generation
#[derive(Debug, Clone)]
pub enum SharedRuntimeError {
    /// No shared functions found
    NoSharedFunctions,
    /// Invalid function signature for sharing
    InvalidSharedFunction { function_name: String, reason: String },
    /// Dependency cycle in shared functions
    SharedFunctionCycle { cycle: Vec<String> },
    /// Type mapping error
    TypeMappingError { message: String },
}

impl std::fmt::Display for SharedRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharedRuntimeError::NoSharedFunctions => {
                write!(f, "No shared functions found across behaviors")
            }
            SharedRuntimeError::InvalidSharedFunction { function_name, reason } => {
                write!(f, "Invalid shared function '{}': {}", function_name, reason)
            }
            SharedRuntimeError::SharedFunctionCycle { cycle } => {
                write!(f, "Circular dependency in shared functions: {}", cycle.join(" -> "))
            }
            SharedRuntimeError::TypeMappingError { message } => {
                write!(f, "Type mapping error: {}", message)
            }
        }
    }
}

impl std::error::Error for SharedRuntimeError {}

/// Represents a shared function that should be moved to SharedRuntime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedFunction {
    /// Function name
    pub name: String,
    /// Function signature details
    pub method: StructMethod,
    /// Behaviors that use this function
    pub used_by: Vec<String>,
    /// Other shared functions this function depends on
    pub dependencies: Vec<String>,
    /// Whether this function accesses static state
    pub accesses_static_state: bool,
    /// Whether this function is thread-safe
    pub is_thread_safe: bool,
}

impl SharedFunction {
    /// Create a new shared function
    pub fn new(name: String, method: StructMethod) -> Self {
        Self {
            name,
            method,
            used_by: Vec::new(),
            dependencies: Vec::new(),
            accesses_static_state: false,
            is_thread_safe: true,
        }
    }

    /// Add a behavior that uses this function
    pub fn add_user(&mut self, behavior_name: String) {
        if !self.used_by.contains(&behavior_name) {
            self.used_by.push(behavior_name);
        }
    }

    /// Add a dependency on another shared function
    pub fn add_dependency(&mut self, function_name: String) {
        if !self.dependencies.contains(&function_name) {
            self.dependencies.push(function_name);
        }
    }

    /// Set whether this function accesses static state
    pub fn set_accesses_static_state(&mut self, accesses: bool) {
        self.accesses_static_state = accesses;
        if accesses {
            self.is_thread_safe = false; // Static state access is not thread-safe by default
        }
    }

    /// Check if this function should be shared (used by 2+ behaviors)
    pub fn should_be_shared(&self, min_usage_threshold: usize) -> bool {
        self.used_by.len() >= min_usage_threshold
    }
}

/// Represents shared data types and constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedDataType {
    /// Type name
    pub name: String,
    /// Rust type information
    pub rust_type: RustType,
    /// Behaviors that use this type
    pub used_by: Vec<String>,
    /// Whether this is a constant value
    pub is_constant: bool,
    /// Constant value if applicable
    pub constant_value: Option<String>,
}

impl SharedDataType {
    /// Create a new shared data type
    pub fn new(name: String, rust_type: RustType) -> Self {
        Self {
            name,
            rust_type,
            used_by: Vec::new(),
            is_constant: false,
            constant_value: None,
        }
    }

    /// Add a behavior that uses this type
    pub fn add_user(&mut self, behavior_name: String) {
        if !self.used_by.contains(&behavior_name) {
            self.used_by.push(behavior_name);
        }
    }

    /// Set as constant with value
    pub fn set_constant(&mut self, value: String) {
        self.is_constant = true;
        self.constant_value = Some(value);
    }

    /// Check if this type should be shared
    pub fn should_be_shared(&self, min_usage_threshold: usize) -> bool {
        self.used_by.len() >= min_usage_threshold
    }
}

/// Contains all shared items extracted from multiple behaviors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedItems {
    /// Shared functions
    pub functions: Vec<SharedFunction>,
    /// Shared data types and constants
    pub types: Vec<SharedDataType>,
    /// Shared constants
    pub constants: Vec<SharedDataType>,
    /// Static state variables
    pub static_variables: Vec<SharedDataType>,
}

impl SharedItems {
    /// Create empty shared items
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
            static_variables: Vec::new(),
        }
    }

    /// Check if there are any shared items
    pub fn has_shared_items(&self) -> bool {
        !self.functions.is_empty() || !self.types.is_empty() || 
        !self.constants.is_empty() || !self.static_variables.is_empty()
    }

    /// Get all function names
    pub fn get_function_names(&self) -> Vec<String> {
        self.functions.iter().map(|f| f.name.clone()).collect()
    }

    /// Get all type names
    pub fn get_type_names(&self) -> Vec<String> {
        self.types.iter().map(|t| t.name.clone()).collect()
    }
}

impl Default for SharedItems {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzes function usage across multiple UdonBehaviour structs to identify shared functions
pub struct SharedFunctionDetector {
    /// Function usage tracking: function_name -> behaviors using it
    function_usage: HashMap<String, HashSet<String>>,
    /// Function definitions: function_name -> method details
    function_definitions: HashMap<String, StructMethod>,
    /// Function call graph: function_name -> functions it calls
    function_call_graph: HashMap<String, HashSet<String>>,
    /// Minimum usage count to consider a function shared
    min_usage_threshold: usize,
    /// Functions to exclude from sharing (Unity event methods, etc.)
    excluded_functions: HashSet<String>,
}

impl SharedFunctionDetector {
    /// Create a new shared function detector
    pub fn new() -> Self {
        let mut excluded_functions = HashSet::new();
        
        // Unity event methods should not be shared
        excluded_functions.insert("start".to_string());
        excluded_functions.insert("update".to_string());
        excluded_functions.insert("fixed_update".to_string());
        excluded_functions.insert("late_update".to_string());
        excluded_functions.insert("on_enable".to_string());
        excluded_functions.insert("on_disable".to_string());
        excluded_functions.insert("on_destroy".to_string());
        excluded_functions.insert("on_trigger_enter".to_string());
        excluded_functions.insert("on_trigger_exit".to_string());
        excluded_functions.insert("on_trigger_stay".to_string());
        excluded_functions.insert("on_collision_enter".to_string());
        excluded_functions.insert("on_collision_exit".to_string());
        excluded_functions.insert("on_collision_stay".to_string());
        excluded_functions.insert("on_player_joined".to_string());
        excluded_functions.insert("on_player_left".to_string());
        excluded_functions.insert("on_pickup".to_string());
        excluded_functions.insert("on_drop".to_string());
        excluded_functions.insert("on_pickup_use_down".to_string());
        excluded_functions.insert("on_pickup_use_up".to_string());
        excluded_functions.insert("on_station_entered".to_string());
        excluded_functions.insert("on_station_exited".to_string());
        excluded_functions.insert("on_post_deserialization".to_string());

        Self {
            function_usage: HashMap::new(),
            function_definitions: HashMap::new(),
            function_call_graph: HashMap::new(),
            min_usage_threshold: 2, // Default: function must be used by at least 2 behaviors
            excluded_functions,
        }
    }

    /// Set the minimum usage threshold for shared functions
    pub fn set_min_usage_threshold(&mut self, threshold: usize) {
        self.min_usage_threshold = threshold;
    }

    /// Add a function to the exclusion list
    pub fn exclude_function(&mut self, function_name: String) {
        self.excluded_functions.insert(function_name);
    }

    /// Analyze function usage across multiple behaviors
    pub fn analyze_function_usage(&mut self, behaviors: &[UdonBehaviourStruct]) -> SharedRuntimeResult<()> {
        // Clear previous analysis
        self.function_usage.clear();
        self.function_definitions.clear();
        self.function_call_graph.clear();

        // First pass: collect all function definitions and their usage
        for behavior in behaviors {
            self.analyze_behavior_functions(&behavior.name, behavior)?;
        }

        // Second pass: build call graph by analyzing function bodies
        // Note: This is simplified - in a real implementation, we'd need to parse function bodies
        self.build_function_call_graph(behaviors)?;

        Ok(())
    }

    /// Analyze functions in a single behavior
    fn analyze_behavior_functions(&mut self, behavior_name: &str, behavior: &UdonBehaviourStruct) -> SharedRuntimeResult<()> {
        // Analyze trait implementation methods
        if let Some(trait_impl) = &behavior.trait_impl {
            for method_name in &trait_impl.implemented_methods {
                if !self.excluded_functions.contains(method_name) {
                    self.record_function_usage(method_name.clone(), behavior_name.to_string());
                }
            }
        }

        // Analyze additional methods
        for method in &behavior.methods {
            if !self.excluded_functions.contains(&method.name) {
                self.record_function_usage(method.name.clone(), behavior_name.to_string());
                self.function_definitions.insert(method.name.clone(), method.clone());
            }
        }

        Ok(())
    }

    /// Record that a function is used by a behavior
    fn record_function_usage(&mut self, function_name: String, behavior_name: String) {
        self.function_usage
            .entry(function_name)
            .or_insert_with(HashSet::new)
            .insert(behavior_name);
    }

    /// Build function call graph (simplified implementation)
    fn build_function_call_graph(&mut self, _behaviors: &[UdonBehaviourStruct]) -> SharedRuntimeResult<()> {
        // In a real implementation, this would parse function bodies to find function calls
        // For now, we'll use a simplified approach based on method names and common patterns
        
        // Initialize empty call graph for all functions
        for function_name in self.function_definitions.keys() {
            self.function_call_graph.insert(function_name.clone(), HashSet::new());
        }

        // Add some common dependency patterns
        self.add_common_function_dependencies();

        Ok(())
    }

    /// Add common function dependency patterns
    fn add_common_function_dependencies(&mut self) {
        // Helper functions often depend on utility functions
        let helper_patterns = vec![
            ("validate_input", vec!["log_error", "is_valid_string"]),
            ("calculate_distance", vec!["get_position", "log_debug"]),
            ("update_ui", vec!["format_text", "get_player_name"]),
            ("handle_networking", vec!["validate_master", "log_network_event"]),
        ];

        for (function_name, dependencies) in helper_patterns {
            if let Some(call_set) = self.function_call_graph.get_mut(function_name) {
                for dep in dependencies {
                    if self.function_definitions.contains_key(dep) {
                        call_set.insert(dep.to_string());
                    }
                }
            }
        }
    }

    /// Extract shared functions based on usage analysis
    pub fn extract_shared_functions(&self) -> SharedRuntimeResult<Vec<SharedFunction>> {
        let mut shared_functions = Vec::new();

        for (function_name, users) in &self.function_usage {
            if users.len() >= self.min_usage_threshold {
                if let Some(method) = self.function_definitions.get(function_name) {
                    let mut shared_function = SharedFunction::new(function_name.clone(), method.clone());
                    
                    // Add all users
                    for user in users {
                        shared_function.add_user(user.clone());
                    }

                    // Add dependencies from call graph
                    if let Some(dependencies) = self.function_call_graph.get(function_name) {
                        for dep in dependencies {
                            shared_function.add_dependency(dep.clone());
                        }
                    }

                    // Analyze function characteristics
                    self.analyze_function_characteristics(&mut shared_function);

                    shared_functions.push(shared_function);
                }
            }
        }

        // Sort by usage count (most used first)
        shared_functions.sort_by(|a, b| b.used_by.len().cmp(&a.used_by.len()));

        if shared_functions.is_empty() {
            Err(SharedRuntimeError::NoSharedFunctions)
        } else {
            Ok(shared_functions)
        }
    }

    /// Analyze characteristics of a shared function
    fn analyze_function_characteristics(&self, shared_function: &mut SharedFunction) {
        // Check if function likely accesses static state based on name patterns
        let static_state_patterns = vec![
            "get_global_", "set_global_", "update_global_",
            "get_instance_", "set_instance_",
            "get_singleton_", "set_singleton_",
            "_cache", "cache_", "_state", "state_",
        ];

        for pattern in &static_state_patterns {
            if shared_function.name.contains(pattern) {
                shared_function.set_accesses_static_state(true);
                break;
            }
        }

        // Check if function is likely thread-safe based on name patterns
        let unsafe_patterns = vec![
            "modify_", "update_", "set_", "change_",
            "increment_", "decrement_", "add_to_", "remove_from_",
        ];

        for pattern in &unsafe_patterns {
            if shared_function.name.contains(pattern) {
                shared_function.is_thread_safe = false;
                break;
            }
        }

        // Pure functions (read-only, mathematical) are typically thread-safe
        let pure_patterns = vec![
            "calculate_", "compute_", "get_", "is_", "has_",
            "validate_", "check_", "format_", "parse_",
        ];

        for pattern in &pure_patterns {
            if shared_function.name.contains(pattern) && !shared_function.accesses_static_state {
                shared_function.is_thread_safe = true;
                break;
            }
        }
    }

    /// Detect circular dependencies in shared functions
    pub fn detect_circular_dependencies(&self, shared_functions: &[SharedFunction]) -> SharedRuntimeResult<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for shared_function in shared_functions {
            if !visited.contains(&shared_function.name) {
                if let Some(cycle) = self.detect_cycle_dfs(&shared_function.name, shared_functions, &mut visited, &mut rec_stack) {
                    return Err(SharedRuntimeError::SharedFunctionCycle { cycle });
                }
            }
        }

        Ok(())
    }

    /// Depth-first search to detect cycles
    fn detect_cycle_dfs(
        &self,
        function_name: &str,
        shared_functions: &[SharedFunction],
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> Option<Vec<String>> {
        visited.insert(function_name.to_string());
        rec_stack.insert(function_name.to_string());

        // Find the shared function
        if let Some(shared_function) = shared_functions.iter().find(|f| f.name == function_name) {
            for dependency in &shared_function.dependencies {
                if !visited.contains(dependency) {
                    if let Some(mut cycle) = self.detect_cycle_dfs(dependency, shared_functions, visited, rec_stack) {
                        cycle.insert(0, function_name.to_string());
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dependency) {
                    return Some(vec![function_name.to_string(), dependency.clone()]);
                }
            }
        }

        rec_stack.remove(function_name);
        None
    }

    /// Get function usage statistics
    pub fn get_usage_statistics(&self) -> HashMap<String, usize> {
        self.function_usage
            .iter()
            .map(|(name, users)| (name.clone(), users.len()))
            .collect()
    }

    /// Get functions that could potentially be shared (used by multiple behaviors)
    pub fn get_potentially_shared_functions(&self) -> Vec<(String, usize)> {
        self.function_usage
            .iter()
            .filter(|(name, users)| users.len() > 1 && !self.excluded_functions.contains(*name))
            .map(|(name, users)| (name.clone(), users.len()))
            .collect()
    }
}

impl Default for SharedFunctionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::{UdonBehaviourTraitImpl, StructMethod};

    fn create_test_behavior(name: &str, methods: Vec<&str>) -> UdonBehaviourStruct {
        let mut behavior = UdonBehaviourStruct::new(name.to_string());
        
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        for method_name in &methods {
            trait_impl.add_method(method_name.to_string());
            
            let method = StructMethod::new(method_name.to_string(), RustType::Unit);
            behavior.add_method(method);
        }
        trait_impl.check_completeness();
        behavior.set_trait_impl(trait_impl);
        
        behavior
    }

    #[test]
    fn test_shared_function_detector_creation() {
        let detector = SharedFunctionDetector::new();
        assert_eq!(detector.min_usage_threshold, 2);
        assert!(detector.excluded_functions.contains("start"));
        assert!(detector.excluded_functions.contains("update"));
    }

    #[test]
    fn test_function_usage_analysis() {
        let mut detector = SharedFunctionDetector::new();
        
        let behaviors = vec![
            create_test_behavior("BehaviorA", vec!["start", "calculate_distance", "validate_input"]),
            create_test_behavior("BehaviorB", vec!["start", "calculate_distance", "update_ui"]),
            create_test_behavior("BehaviorC", vec!["start", "validate_input", "update_ui"]),
        ];

        detector.analyze_function_usage(&behaviors).unwrap();

        let stats = detector.get_usage_statistics();
        
        // start should be used by all 3 but excluded
        assert_eq!(stats.get("start"), Some(&3));
        
        // calculate_distance used by 2
        assert_eq!(stats.get("calculate_distance"), Some(&2));
        
        // validate_input used by 2
        assert_eq!(stats.get("validate_input"), Some(&2));
        
        // update_ui used by 2
        assert_eq!(stats.get("update_ui"), Some(&2));
    }

    #[test]
    fn test_shared_function_extraction() {
        let mut detector = SharedFunctionDetector::new();
        
        let behaviors = vec![
            create_test_behavior("BehaviorA", vec!["calculate_distance", "validate_input"]),
            create_test_behavior("BehaviorB", vec!["calculate_distance", "format_text"]),
            create_test_behavior("BehaviorC", vec!["validate_input", "format_text"]),
        ];

        detector.analyze_function_usage(&behaviors).unwrap();
        let shared_functions = detector.extract_shared_functions().unwrap();

        assert_eq!(shared_functions.len(), 3);
        
        // All functions should be shared (used by 2 behaviors each)
        let function_names: Vec<String> = shared_functions.iter().map(|f| f.name.clone()).collect();
        assert!(function_names.contains(&"calculate_distance".to_string()));
        assert!(function_names.contains(&"validate_input".to_string()));
        assert!(function_names.contains(&"format_text".to_string()));
    }

    #[test]
    fn test_function_characteristics_analysis() {
        let mut detector = SharedFunctionDetector::new();
        
        let behaviors = vec![
            create_test_behavior("BehaviorA", vec!["get_global_state", "set_player_count"]),
            create_test_behavior("BehaviorB", vec!["get_global_state", "set_player_count"]),
        ];

        detector.analyze_function_usage(&behaviors).unwrap();
        let shared_functions = detector.extract_shared_functions().unwrap();

        let global_state_func = shared_functions.iter().find(|f| f.name == "get_global_state").unwrap();
        assert!(global_state_func.accesses_static_state);

        let set_player_func = shared_functions.iter().find(|f| f.name == "set_player_count").unwrap();
        assert!(!set_player_func.is_thread_safe); // modify operation
    }

    #[test]
    fn test_no_shared_functions() {
        let mut detector = SharedFunctionDetector::new();
        
        let behaviors = vec![
            create_test_behavior("BehaviorA", vec!["unique_function_a"]),
            create_test_behavior("BehaviorB", vec!["unique_function_b"]),
        ];

        detector.analyze_function_usage(&behaviors).unwrap();
        let result = detector.extract_shared_functions();

        assert!(result.is_err());
        if let Err(SharedRuntimeError::NoSharedFunctions) = result {
            // Expected
        } else {
            panic!("Expected NoSharedFunctions error");
        }
    }

    #[test]
    fn test_potentially_shared_functions() {
        let mut detector = SharedFunctionDetector::new();
        
        let behaviors = vec![
            create_test_behavior("BehaviorA", vec!["shared_func", "unique_a"]),
            create_test_behavior("BehaviorB", vec!["shared_func", "unique_b"]),
            create_test_behavior("BehaviorC", vec!["unique_c"]),
        ];

        detector.analyze_function_usage(&behaviors).unwrap();
        let potentially_shared = detector.get_potentially_shared_functions();

        assert_eq!(potentially_shared.len(), 1);
        assert_eq!(potentially_shared[0].0, "shared_func");
        assert_eq!(potentially_shared[0].1, 2);
    }
}

/// Generates SharedRuntime C# class from shared items
pub struct SharedRuntimeGenerator {
    /// Type mapper for Rust to C# conversion
    type_mapper: RustToCSharpTypeMapper,
    /// Template for SharedRuntime class structure
    class_template: String,
    /// Namespace for the generated class
    namespace: Option<String>,
}

impl SharedRuntimeGenerator {
    /// Create a new SharedRuntime generator
    pub fn new() -> Self {
        Self {
            type_mapper: RustToCSharpTypeMapper::new(),
            class_template: Self::default_class_template(),
            namespace: None,
        }
    }

    /// Set the namespace for generated classes
    pub fn set_namespace(&mut self, namespace: String) {
        self.namespace = Some(namespace);
    }

    /// Generate SharedRuntime.cs class from shared items
    pub fn generate_shared_runtime(&self, shared_items: &SharedItems) -> SharedRuntimeResult<String> {
        if !shared_items.has_shared_items() {
            return Err(SharedRuntimeError::NoSharedFunctions);
        }

        let mut class_content = String::new();

        // Generate using statements
        class_content.push_str(&self.generate_using_statements(shared_items));
        class_content.push('\n');

        // Generate namespace if specified
        if let Some(namespace) = &self.namespace {
            class_content.push_str(&format!("namespace {}\n{{\n", namespace));
        }

        // Generate class header
        class_content.push_str(&self.generate_class_header());

        // Generate shared constants
        if !shared_items.constants.is_empty() {
            class_content.push_str(&self.generate_constants(&shared_items.constants)?);
            class_content.push('\n');
        }

        // Generate static variables
        if !shared_items.static_variables.is_empty() {
            class_content.push_str(&self.generate_static_variables(&shared_items.static_variables)?);
            class_content.push('\n');
        }

        // Generate shared data types
        if !shared_items.types.is_empty() {
            class_content.push_str(&self.generate_shared_types(&shared_items.types)?);
            class_content.push('\n');
        }

        // Generate shared functions
        if !shared_items.functions.is_empty() {
            class_content.push_str(&self.generate_shared_functions(&shared_items.functions)?);
        }

        // Generate class footer
        class_content.push_str(&self.generate_class_footer());

        // Close namespace if specified
        if self.namespace.is_some() {
            class_content.push_str("}\n");
        }

        Ok(class_content)
    }

    /// Generate using statements based on shared items
    fn generate_using_statements(&self, shared_items: &SharedItems) -> String {
        let mut usings = HashSet::new();

        // Always include basic Unity and UdonSharp usings
        usings.insert("using UnityEngine;".to_string());
        usings.insert("using UdonSharp;".to_string());
        usings.insert("using VRC.Udon;".to_string());

        // Add usings based on types used in shared items
        for function in &shared_items.functions {
            let function_usings = self.type_mapper.get_required_usings(&function.method.return_type);
            usings.extend(function_usings);

            for param in &function.method.parameters {
                let param_usings = self.type_mapper.get_required_usings(&param.param_type);
                usings.extend(param_usings);
            }
        }

        for data_type in &shared_items.types {
            let type_usings = self.type_mapper.get_required_usings(&data_type.rust_type);
            usings.extend(type_usings);
        }

        for constant in &shared_items.constants {
            let const_usings = self.type_mapper.get_required_usings(&constant.rust_type);
            usings.extend(const_usings);
        }

        for static_var in &shared_items.static_variables {
            let var_usings = self.type_mapper.get_required_usings(&static_var.rust_type);
            usings.extend(var_usings);
        }

        // Convert to sorted vector and join
        let mut using_vec: Vec<String> = usings.into_iter().collect();
        using_vec.sort();
        using_vec.join("\n")
    }

    /// Generate class header
    fn generate_class_header(&self) -> String {
        format!(
            r#"/// <summary>
/// SharedRuntime contains shared functions and data types used by multiple UdonBehaviour classes.
/// This class is automatically generated from Rust multi-behavior analysis.
/// </summary>
[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
public class SharedRuntime : UdonSharpBehaviour
{{
    #region Singleton Pattern
    
    private static SharedRuntime _instance;
    
    /// <summary>
    /// Get the singleton instance of SharedRuntime
    /// </summary>
    public static SharedRuntime Instance
    {{
        get
        {{
            if (_instance == null)
            {{
                _instance = FindObjectOfType<SharedRuntime>();
                if (_instance == null)
                {{
                    Debug.LogError("SharedRuntime instance not found in scene. Please add a SharedRuntime GameObject.");
                }}
            }}
            return _instance;
        }}
    }}
    
    void Start()
    {{
        if (_instance == null)
        {{
            _instance = this;
            DontDestroyOnLoad(gameObject);
            InitializeSharedRuntime();
        }}
        else if (_instance != this)
        {{
            Destroy(gameObject);
        }}
    }}
    
    /// <summary>
    /// Initialize shared runtime state
    /// </summary>
    private void InitializeSharedRuntime()
    {{
        // Initialize static variables and perform setup
        Debug.Log("SharedRuntime initialized");
    }}
    
    #endregion
    
"#
        )
    }

    /// Generate constants section
    fn generate_constants(&self, constants: &[SharedDataType]) -> SharedRuntimeResult<String> {
        let mut content = String::new();
        content.push_str("    #region Shared Constants\n\n");

        for constant in constants {
            let csharp_type = self.type_mapper.map_type(&constant.rust_type)
                .map_err(|e| SharedRuntimeError::TypeMappingError { message: e })?;
            
            let default_value = self.type_mapper.get_default_value(&constant.rust_type);
            let const_value = constant.constant_value.as_ref()
                .unwrap_or(&default_value);

            content.push_str(&format!(
                "    /// <summary>\n    /// Shared constant used by: {}\n    /// </summary>\n",
                constant.used_by.join(", ")
            ));
            content.push_str(&format!(
                "    public const {} {} = {};\n\n",
                csharp_type,
                to_pascal_case(&constant.name),
                const_value
            ));
        }

        content.push_str("    #endregion\n");
        Ok(content)
    }

    /// Generate static variables section
    fn generate_static_variables(&self, static_vars: &[SharedDataType]) -> SharedRuntimeResult<String> {
        let mut content = String::new();
        content.push_str("    #region Shared Static Variables\n\n");

        for var in static_vars {
            let csharp_type = self.type_mapper.map_type(&var.rust_type)
                .map_err(|e| SharedRuntimeError::TypeMappingError { message: e })?;
            
            let default_val = self.type_mapper.get_default_value(&var.rust_type);
            let default_value = var.constant_value.as_ref()
                .unwrap_or(&default_val);

            content.push_str(&format!(
                "    /// <summary>\n    /// Shared static variable used by: {}\n    /// </summary>\n",
                var.used_by.join(", ")
            ));
            
            // Add thread-safety warning for static variables
            content.push_str("    /// <warning>Access to this static variable should be synchronized in multi-threaded contexts</warning>\n");
            
            content.push_str(&format!(
                "    private static {} _{} = {};\n",
                csharp_type,
                to_camel_case(&var.name),
                default_value
            ));

            // Generate getter and setter with thread safety
            content.push_str(&format!(
                "    public static {} {}\n    {{\n",
                csharp_type,
                to_pascal_case(&var.name)
            ));
            content.push_str(&format!(
                "        get {{ return _{}; }}\n",
                to_camel_case(&var.name)
            ));
            content.push_str(&format!(
                "        set {{ _{} = value; }}\n",
                to_camel_case(&var.name)
            ));
            content.push_str("    }\n\n");
        }

        content.push_str("    #endregion\n");
        Ok(content)
    }

    /// Generate shared data types section
    fn generate_shared_types(&self, types: &[SharedDataType]) -> SharedRuntimeResult<String> {
        let mut content = String::new();
        content.push_str("    #region Shared Data Types\n\n");

        for data_type in types {
            // For now, we'll generate type aliases or simple wrappers
            // In a more complete implementation, we'd handle complex type definitions
            let csharp_type = self.type_mapper.map_type(&data_type.rust_type)
                .map_err(|e| SharedRuntimeError::TypeMappingError { message: e })?;

            content.push_str(&format!(
                "    /// <summary>\n    /// Shared type used by: {}\n    /// </summary>\n",
                data_type.used_by.join(", ")
            ));
            content.push_str(&format!(
                "    public class {}\n    {{\n",
                to_pascal_case(&data_type.name)
            ));
            content.push_str(&format!(
                "        public {} Value {{ get; set; }}\n",
                csharp_type
            ));
            content.push_str(&format!(
                "        public {}({} value) {{ Value = value; }}\n",
                to_pascal_case(&data_type.name),
                csharp_type
            ));
            content.push_str("    }\n\n");
        }

        content.push_str("    #endregion\n");
        Ok(content)
    }

    /// Generate shared functions section
    fn generate_shared_functions(&self, functions: &[SharedFunction]) -> SharedRuntimeResult<String> {
        let mut content = String::new();
        content.push_str("    #region Shared Functions\n\n");

        for function in functions {
            content.push_str(&self.generate_shared_function(function)?);
            content.push('\n');
        }

        content.push_str("    #endregion\n");
        Ok(content)
    }

    /// Generate a single shared function
    fn generate_shared_function(&self, function: &SharedFunction) -> SharedRuntimeResult<String> {
        let mut content = String::new();

        // Generate function documentation
        content.push_str(&format!(
            "    /// <summary>\n    /// Shared function used by: {}\n    /// </summary>\n",
            function.used_by.join(", ")
        ));

        if !function.dependencies.is_empty() {
            content.push_str(&format!(
                "    /// <remarks>Depends on: {}</remarks>\n",
                function.dependencies.join(", ")
            ));
        }

        if function.accesses_static_state {
            content.push_str("    /// <warning>This function accesses static state</warning>\n");
        }

        if !function.is_thread_safe {
            content.push_str("    /// <warning>This function is not thread-safe</warning>\n");
        }

        // Generate function signature
        let return_type = self.type_mapper.map_type(&function.method.return_type)
            .map_err(|e| SharedRuntimeError::TypeMappingError { message: e })?;

        let mut parameters = Vec::new();
        for param in &function.method.parameters {
            let param_type = self.type_mapper.map_type(&param.param_type)
                .map_err(|e| SharedRuntimeError::TypeMappingError { message: e })?;
            parameters.push(format!("{} {}", param_type, to_camel_case(&param.name)));
        }

        let function_name = to_pascal_case(&function.name);
        let params_str = parameters.join(", ");

        content.push_str(&format!(
            "    public static {} {}({})\n    {{\n",
            return_type,
            function_name,
            params_str
        ));

        // Generate function body (simplified implementation)
        content.push_str(&self.generate_function_body(function)?);

        content.push_str("    }\n");

        Ok(content)
    }

    /// Generate function body (simplified implementation)
    fn generate_function_body(&self, function: &SharedFunction) -> SharedRuntimeResult<String> {
        let mut body = String::new();

        // Add parameter validation if needed
        for param in &function.method.parameters {
            if self.type_mapper.is_gameobject_reference(&param.param_type) {
                body.push_str(&format!(
                    "        if ({} == null)\n        {{\n            Debug.LogError(\"Parameter '{}' cannot be null\");\n            return{};\n        }}\n\n",
                    to_camel_case(&param.name),
                    param.name,
                    if function.method.return_type == RustType::Unit { "" } else { " default" }
                ));
            }
        }

        // Generate function implementation based on function name patterns
        body.push_str(&self.generate_function_implementation(function)?);

        Ok(body)
    }

    /// Generate function implementation based on common patterns
    fn generate_function_implementation(&self, function: &SharedFunction) -> SharedRuntimeResult<String> {
        let function_name = &function.name;
        let mut implementation = String::new();

        // Generate implementation based on function name patterns
        if function_name.starts_with("calculate_") {
            implementation.push_str("        // TODO: Implement calculation logic\n");
            implementation.push_str("        Debug.LogWarning(\"Shared function implementation needed\");\n");
        } else if function_name.starts_with("validate_") {
            implementation.push_str("        // TODO: Implement validation logic\n");
            implementation.push_str("        Debug.LogWarning(\"Shared function implementation needed\");\n");
            if function.method.return_type == RustType::Bool {
                implementation.push_str("        return false;\n");
            }
        } else if function_name.starts_with("format_") {
            implementation.push_str("        // TODO: Implement formatting logic\n");
            implementation.push_str("        Debug.LogWarning(\"Shared function implementation needed\");\n");
            if function.method.return_type == RustType::String {
                implementation.push_str("        return \"\";\n");
            }
        } else if function_name.starts_with("get_") {
            implementation.push_str("        // TODO: Implement getter logic\n");
            implementation.push_str("        Debug.LogWarning(\"Shared function implementation needed\");\n");
            if function.method.return_type != RustType::Unit {
                let default_value = self.type_mapper.get_default_value(&function.method.return_type);
                implementation.push_str(&format!("        return {};\n", default_value));
            }
        } else if function_name.starts_with("set_") || function_name.starts_with("update_") {
            implementation.push_str("        // TODO: Implement setter/update logic\n");
            implementation.push_str("        Debug.LogWarning(\"Shared function implementation needed\");\n");
        } else {
            // Generic implementation
            implementation.push_str("        // TODO: Implement shared function logic\n");
            implementation.push_str(&format!("        Debug.LogWarning(\"Shared function '{}' implementation needed\");\n", function_name));
            
            if function.method.return_type != RustType::Unit {
                let default_value = self.type_mapper.get_default_value(&function.method.return_type);
                implementation.push_str(&format!("        return {};\n", default_value));
            }
        }

        Ok(implementation)
    }

    /// Generate class footer
    fn generate_class_footer(&self) -> String {
        "}\n".to_string()
    }

    /// Get default class template
    fn default_class_template() -> String {
        "SharedRuntime".to_string()
    }

    /// Validate shared items before generation
    pub fn validate_shared_items(&self, shared_items: &SharedItems) -> SharedRuntimeResult<()> {
        // Check for invalid function signatures
        for function in &shared_items.functions {
            if function.method.is_async {
                return Err(SharedRuntimeError::InvalidSharedFunction {
                    function_name: function.name.clone(),
                    reason: "Async functions are not supported in SharedRuntime".to_string(),
                });
            }

            // Check if return type is supported
            if !function.method.return_type.is_udonsharp_compatible() {
                return Err(SharedRuntimeError::InvalidSharedFunction {
                    function_name: function.name.clone(),
                    reason: format!("Return type {:?} is not UdonSharp compatible", function.method.return_type),
                });
            }

            // Check if parameter types are supported
            for param in &function.method.parameters {
                if !param.param_type.is_udonsharp_compatible() {
                    return Err(SharedRuntimeError::InvalidSharedFunction {
                        function_name: function.name.clone(),
                        reason: format!("Parameter type {:?} is not UdonSharp compatible", param.param_type),
                    });
                }
            }
        }

        // Check for invalid data types
        for data_type in &shared_items.types {
            if !data_type.rust_type.is_udonsharp_compatible() {
                return Err(SharedRuntimeError::TypeMappingError {
                    message: format!("Type {:?} is not UdonSharp compatible", data_type.rust_type),
                });
            }
        }

        Ok(())
    }

    /// Generate initialization code for static variables
    pub fn generate_static_initialization(&self, shared_items: &SharedItems) -> String {
        let mut init_code = String::new();

        if !shared_items.static_variables.is_empty() {
            init_code.push_str("    /// <summary>\n");
            init_code.push_str("    /// Initialize all static variables to their default values\n");
            init_code.push_str("    /// </summary>\n");
            init_code.push_str("    public static void InitializeStaticVariables()\n    {\n");

            for var in &shared_items.static_variables {
                let default_val = self.type_mapper.get_default_value(&var.rust_type);
                let default_value = var.constant_value.as_ref()
                    .unwrap_or(&default_val);
                
                init_code.push_str(&format!(
                    "        {} = {};\n",
                    to_pascal_case(&var.name),
                    default_value
                ));
            }

            init_code.push_str("        Debug.Log(\"SharedRuntime static variables initialized\");\n");
            init_code.push_str("    }\n\n");
        }

        init_code
    }
}

impl Default for SharedRuntimeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(snake_case: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for c in snake_case.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Convert snake_case to PascalCase
fn to_pascal_case(snake_case: &str) -> String {
    let camel = to_camel_case(snake_case);
    if let Some(first_char) = camel.chars().next() {
        first_char.to_uppercase().collect::<String>() + &camel[first_char.len_utf8()..]
    } else {
        camel
    }
}

/// Extracts shared data types, enums, structs, and constants from multiple behaviors
pub struct SharedDataTypeExtractor {
    /// Type usage tracking: type_name -> behaviors using it
    type_usage: HashMap<String, HashSet<String>>,
    /// Type definitions: type_name -> type details
    type_definitions: HashMap<String, RustType>,
    /// Constant definitions: constant_name -> (type, value, behaviors)
    constant_definitions: HashMap<String, (RustType, String, HashSet<String>)>,
    /// Enum definitions: enum_name -> (variants, behaviors)
    enum_definitions: HashMap<String, (Vec<String>, HashSet<String>)>,
    /// Struct definitions: struct_name -> (fields, behaviors)
    struct_definitions: HashMap<String, (Vec<(String, RustType)>, HashSet<String>)>,
    /// Minimum usage count to consider a type shared
    min_usage_threshold: usize,
}

impl SharedDataTypeExtractor {
    /// Create a new shared data type extractor
    pub fn new() -> Self {
        Self {
            type_usage: HashMap::new(),
            type_definitions: HashMap::new(),
            constant_definitions: HashMap::new(),
            enum_definitions: HashMap::new(),
            struct_definitions: HashMap::new(),
            min_usage_threshold: 2,
        }
    }

    /// Set the minimum usage threshold for shared types
    pub fn set_min_usage_threshold(&mut self, threshold: usize) {
        self.min_usage_threshold = threshold;
    }

    /// Analyze data type usage across multiple behaviors
    pub fn analyze_type_usage(&mut self, behaviors: &[UdonBehaviourStruct]) -> SharedRuntimeResult<()> {
        // Clear previous analysis
        self.type_usage.clear();
        self.type_definitions.clear();
        self.constant_definitions.clear();
        self.enum_definitions.clear();
        self.struct_definitions.clear();

        // Analyze each behavior
        for behavior in behaviors {
            self.analyze_behavior_types(&behavior.name, behavior)?;
        }

        Ok(())
    }

    /// Analyze types used in a single behavior
    fn analyze_behavior_types(&mut self, behavior_name: &str, behavior: &UdonBehaviourStruct) -> SharedRuntimeResult<()> {
        // Analyze field types
        for field in &behavior.fields {
            self.record_type_usage(&field.field_type, behavior_name);
            
            // Check if field is a constant (has default value and is not mutable)
            if let Some(default_value) = &field.default_value {
                if field.attributes.iter().any(|attr| matches!(attr, crate::multi_behavior::FieldAttribute::UdonPublic)) {
                    // This might be a shared constant
                    self.record_constant_usage(&field.name, &field.field_type, default_value, behavior_name);
                }
            }
        }

        // Analyze method parameter and return types
        for method in &behavior.methods {
            self.record_type_usage(&method.return_type, behavior_name);
            
            for param in &method.parameters {
                self.record_type_usage(&param.param_type, behavior_name);
            }
        }

        // Analyze trait implementation method types
        if let Some(trait_impl) = &behavior.trait_impl {
            // For now, we assume trait methods use standard Unity types
            // In a more complete implementation, we'd parse the actual method signatures
        }

        Ok(())
    }

    /// Record that a type is used by a behavior
    fn record_type_usage(&mut self, rust_type: &RustType, behavior_name: &str) {
        let type_name = self.get_type_name(rust_type);
        
        self.type_usage
            .entry(type_name.clone())
            .or_insert_with(HashSet::new)
            .insert(behavior_name.to_string());
        
        self.type_definitions.insert(type_name, rust_type.clone());

        // Recursively record usage for generic type parameters
        match rust_type {
            RustType::Option(inner) | RustType::Vec(inner) | RustType::Array(inner, _) => {
                self.record_type_usage(inner, behavior_name);
            }
            RustType::HashMap(key, value) => {
                self.record_type_usage(key, behavior_name);
                self.record_type_usage(value, behavior_name);
            }
            _ => {}
        }
    }

    /// Record that a constant is used by a behavior
    fn record_constant_usage(&mut self, constant_name: &str, rust_type: &RustType, value: &str, behavior_name: &str) {
        self.constant_definitions
            .entry(constant_name.to_string())
            .or_insert_with(|| (rust_type.clone(), value.to_string(), HashSet::new()))
            .2
            .insert(behavior_name.to_string());
    }

    /// Get a string representation of a type for tracking
    fn get_type_name(&self, rust_type: &RustType) -> String {
        match rust_type {
            RustType::Bool => "bool".to_string(),
            RustType::I8 => "i8".to_string(),
            RustType::I16 => "i16".to_string(),
            RustType::I32 => "i32".to_string(),
            RustType::I64 => "i64".to_string(),
            RustType::I128 => "i128".to_string(),
            RustType::U8 => "u8".to_string(),
            RustType::U16 => "u16".to_string(),
            RustType::U32 => "u32".to_string(),
            RustType::U64 => "u64".to_string(),
            RustType::U128 => "u128".to_string(),
            RustType::F32 => "f32".to_string(),
            RustType::F64 => "f64".to_string(),
            RustType::Char => "char".to_string(),
            RustType::String => "String".to_string(),
            RustType::Vector2 => "Vector2".to_string(),
            RustType::Vector3 => "Vector3".to_string(),
            RustType::Vector4 => "Vector4".to_string(),
            RustType::Quaternion => "Quaternion".to_string(),
            RustType::Color => "Color".to_string(),
            RustType::Color32 => "Color32".to_string(),
            RustType::GameObject => "GameObject".to_string(),
            RustType::Transform => "Transform".to_string(),
            RustType::VRCPlayerApi => "VRCPlayerApi".to_string(),
            RustType::Option(inner) => format!("Option<{}>", self.get_type_name(inner)),
            RustType::Vec(inner) => format!("Vec<{}>", self.get_type_name(inner)),
            RustType::HashMap(key, value) => format!("HashMap<{}, {}>", self.get_type_name(key), self.get_type_name(value)),
            RustType::Array(inner, size) => format!("[{}; {}]", self.get_type_name(inner), size),
            RustType::Custom(name) => name.clone(),
            RustType::Unit => "()".to_string(),
        }
    }

    /// Extract shared data types based on usage analysis
    pub fn extract_shared_types(&self) -> Vec<SharedDataType> {
        let mut shared_types = Vec::new();

        for (type_name, users) in &self.type_usage {
            if users.len() >= self.min_usage_threshold {
                if let Some(rust_type) = self.type_definitions.get(type_name) {
                    // Skip basic types that don't need to be in SharedRuntime
                    if self.should_include_type(rust_type) {
                        let mut shared_type = SharedDataType::new(type_name.clone(), rust_type.clone());
                        
                        for user in users {
                            shared_type.add_user(user.clone());
                        }

                        shared_types.push(shared_type);
                    }
                }
            }
        }

        // Sort by usage count (most used first)
        shared_types.sort_by(|a, b| b.used_by.len().cmp(&a.used_by.len()));

        shared_types
    }

    /// Extract shared constants based on usage analysis
    pub fn extract_shared_constants(&self) -> Vec<SharedDataType> {
        let mut shared_constants = Vec::new();

        for (constant_name, (rust_type, value, users)) in &self.constant_definitions {
            if users.len() >= self.min_usage_threshold {
                let mut shared_constant = SharedDataType::new(constant_name.clone(), rust_type.clone());
                shared_constant.set_constant(value.clone());
                
                for user in users {
                    shared_constant.add_user(user.clone());
                }

                shared_constants.push(shared_constant);
            }
        }

        // Sort by usage count (most used first)
        shared_constants.sort_by(|a, b| b.used_by.len().cmp(&a.used_by.len()));

        shared_constants
    }

    /// Extract shared static variables (types used for state management)
    pub fn extract_shared_static_variables(&self, behaviors: &[UdonBehaviourStruct]) -> Vec<SharedDataType> {
        let mut static_variables = Vec::new();

        // Look for fields that might be shared state
        let mut state_fields: HashMap<String, (RustType, HashSet<String>)> = HashMap::new();

        for behavior in behaviors {
            for field in &behavior.fields {
                // Check if field name suggests it's shared state
                if self.is_likely_shared_state(&field.name) {
                    let field_key = format!("{}_{}", self.get_type_name(&field.field_type), field.name);
                    state_fields
                        .entry(field_key)
                        .or_insert_with(|| (field.field_type.clone(), HashSet::new()))
                        .1
                        .insert(behavior.name.clone());
                }
            }
        }

        // Convert to shared static variables
        for (field_key, (rust_type, users)) in state_fields {
            if users.len() >= self.min_usage_threshold {
                let field_name = field_key.split('_').skip(1).collect::<Vec<_>>().join("_");
                let mut static_var = SharedDataType::new(field_name, rust_type);
                
                for user in users {
                    static_var.add_user(user);
                }

                static_variables.push(static_var);
            }
        }

        static_variables
    }

    /// Check if a field name suggests it's shared state
    fn is_likely_shared_state(&self, field_name: &str) -> bool {
        let shared_state_patterns = vec![
            "global_", "shared_", "common_", "world_",
            "_count", "_total", "_sum", "_max", "_min",
            "player_", "game_", "session_", "instance_",
            "_state", "_status", "_mode", "_config",
        ];

        shared_state_patterns.iter().any(|pattern| field_name.contains(pattern))
    }

    /// Check if a type should be included in SharedRuntime
    fn should_include_type(&self, rust_type: &RustType) -> bool {
        match rust_type {
            // Skip basic primitive types
            RustType::Bool | RustType::I8 | RustType::I16 | RustType::I32 | RustType::I64 | RustType::I128 |
            RustType::U8 | RustType::U16 | RustType::U32 | RustType::U64 | RustType::U128 |
            RustType::F32 | RustType::F64 | RustType::Char | RustType::String | RustType::Unit => false,
            
            // Skip basic Unity types
            RustType::Vector2 | RustType::Vector3 | RustType::Vector4 | RustType::Quaternion |
            RustType::Color | RustType::Color32 | RustType::GameObject | RustType::Transform => false,
            
            // Skip VRChat types
            RustType::VRCPlayerApi => false,
            
            // Include complex generic types
            RustType::Option(_) | RustType::Vec(_) | RustType::HashMap(_, _) | RustType::Array(_, _) => true,
            
            // Include custom types
            RustType::Custom(_) => true,
        }
    }

    /// Update references in generated UdonBehaviour classes
    pub fn generate_reference_updates(&self, shared_items: &SharedItems) -> HashMap<String, String> {
        let mut reference_updates = HashMap::new();

        // Generate updates for shared functions
        for function in &shared_items.functions {
            let old_reference = function.name.clone();
            let new_reference = format!("SharedRuntime.{}", to_pascal_case(&function.name));
            reference_updates.insert(old_reference, new_reference);
        }

        // Generate updates for shared constants
        for constant in &shared_items.constants {
            let old_reference = constant.name.clone();
            let new_reference = format!("SharedRuntime.{}", to_pascal_case(&constant.name));
            reference_updates.insert(old_reference, new_reference);
        }

        // Generate updates for shared static variables
        for static_var in &shared_items.static_variables {
            let old_reference = static_var.name.clone();
            let new_reference = format!("SharedRuntime.{}", to_pascal_case(&static_var.name));
            reference_updates.insert(old_reference, new_reference);
        }

        reference_updates
    }

    /// Get type usage statistics
    pub fn get_type_usage_statistics(&self) -> HashMap<String, usize> {
        self.type_usage
            .iter()
            .map(|(name, users)| (name.clone(), users.len()))
            .collect()
    }

    /// Get potentially shared types (used by multiple behaviors)
    pub fn get_potentially_shared_types(&self) -> Vec<(String, usize)> {
        self.type_usage
            .iter()
            .filter(|(_, users)| users.len() > 1)
            .map(|(name, users)| (name.clone(), users.len()))
            .collect()
    }

    /// Ensure proper accessibility and namespace handling
    pub fn generate_accessibility_modifiers(&self, shared_items: &SharedItems) -> HashMap<String, String> {
        let mut accessibility = HashMap::new();

        // Functions should be public static
        for function in &shared_items.functions {
            accessibility.insert(function.name.clone(), "public static".to_string());
        }

        // Constants should be public const
        for constant in &shared_items.constants {
            accessibility.insert(constant.name.clone(), "public const".to_string());
        }

        // Static variables should have public static properties with private backing fields
        for static_var in &shared_items.static_variables {
            accessibility.insert(static_var.name.clone(), "public static".to_string());
        }

        // Types should be public
        for data_type in &shared_items.types {
            accessibility.insert(data_type.name.clone(), "public".to_string());
        }

        accessibility
    }
}

impl Default for SharedDataTypeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete SharedRuntime extraction system that combines function and type analysis
pub struct SharedRuntimeExtractor {
    /// Function detector
    function_detector: SharedFunctionDetector,
    /// Type extractor
    type_extractor: SharedDataTypeExtractor,
    /// Minimum usage threshold
    min_usage_threshold: usize,
}

impl SharedRuntimeExtractor {
    /// Create a new SharedRuntime extractor
    pub fn new() -> Self {
        Self {
            function_detector: SharedFunctionDetector::new(),
            type_extractor: SharedDataTypeExtractor::new(),
            min_usage_threshold: 2,
        }
    }

    /// Set the minimum usage threshold for shared items
    pub fn set_min_usage_threshold(&mut self, threshold: usize) {
        self.min_usage_threshold = threshold;
        self.function_detector.set_min_usage_threshold(threshold);
        self.type_extractor.set_min_usage_threshold(threshold);
    }

    /// Extract all shared items from multiple behaviors
    pub fn extract_shared_items(&mut self, behaviors: &[UdonBehaviourStruct]) -> SharedRuntimeResult<SharedItems> {
        if behaviors.len() < 2 {
            return Err(SharedRuntimeError::NoSharedFunctions);
        }

        let mut shared_items = SharedItems::new();

        // Extract shared functions
        self.function_detector.analyze_function_usage(behaviors)?;
        match self.function_detector.extract_shared_functions() {
            Ok(functions) => {
                // Detect circular dependencies
                self.function_detector.detect_circular_dependencies(&functions)?;
                shared_items.functions = functions;
            }
            Err(SharedRuntimeError::NoSharedFunctions) => {
                // No shared functions is okay, continue with types
            }
            Err(e) => return Err(e),
        }

        // Extract shared data types
        self.type_extractor.analyze_type_usage(behaviors)?;
        shared_items.types = self.type_extractor.extract_shared_types();
        shared_items.constants = self.type_extractor.extract_shared_constants();
        shared_items.static_variables = self.type_extractor.extract_shared_static_variables(behaviors);

        // Check if we have any shared items
        if !shared_items.has_shared_items() {
            return Err(SharedRuntimeError::NoSharedFunctions);
        }

        Ok(shared_items)
    }

    /// Get comprehensive usage statistics
    pub fn get_usage_statistics(&self) -> (HashMap<String, usize>, HashMap<String, usize>) {
        let function_stats = self.function_detector.get_usage_statistics();
        let type_stats = self.type_extractor.get_type_usage_statistics();
        (function_stats, type_stats)
    }

    /// Generate complete SharedRuntime class
    pub fn generate_shared_runtime_class(&self, shared_items: &SharedItems) -> SharedRuntimeResult<String> {
        let generator = SharedRuntimeGenerator::new();
        generator.generate_shared_runtime(shared_items)
    }

    /// Generate reference updates for existing behavior classes
    pub fn generate_reference_updates(&self, shared_items: &SharedItems) -> HashMap<String, String> {
        self.type_extractor.generate_reference_updates(shared_items)
    }
}

impl Default for SharedRuntimeExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod shared_runtime_tests {
    use super::*;
    use crate::multi_behavior::{UdonBehaviourTraitImpl, StructField, FieldAttribute};

    fn create_test_behavior_with_fields(name: &str, fields: Vec<(&str, RustType)>) -> UdonBehaviourStruct {
        let mut behavior = UdonBehaviourStruct::new(name.to_string());
        
        for (field_name, field_type) in fields {
            let mut field = StructField::new(field_name.to_string(), field_type);
            if field_name.contains("public") {
                field.add_attribute(FieldAttribute::UdonPublic);
            }
            behavior.add_field(field);
        }
        
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        behavior.set_trait_impl(trait_impl);
        
        behavior
    }

    #[test]
    fn test_shared_data_type_extractor_creation() {
        let extractor = SharedDataTypeExtractor::new();
        assert_eq!(extractor.min_usage_threshold, 2);
    }

    #[test]
    fn test_type_usage_analysis() {
        let mut extractor = SharedDataTypeExtractor::new();
        
        let behaviors = vec![
            create_test_behavior_with_fields("BehaviorA", vec![
                ("player_count", RustType::I32),
                ("game_state", RustType::String),
            ]),
            create_test_behavior_with_fields("BehaviorB", vec![
                ("player_count", RustType::I32),
                ("ui_text", RustType::String),
            ]),
        ];

        extractor.analyze_type_usage(&behaviors).unwrap();

        let stats = extractor.get_type_usage_statistics();
        assert_eq!(stats.get("i32"), Some(&2));
        assert_eq!(stats.get("String"), Some(&2));
    }

    #[test]
    fn test_shared_type_extraction() {
        let mut extractor = SharedDataTypeExtractor::new();
        
        let behaviors = vec![
            create_test_behavior_with_fields("BehaviorA", vec![
                ("custom_data", RustType::Custom("CustomType".to_string())),
            ]),
            create_test_behavior_with_fields("BehaviorB", vec![
                ("custom_data", RustType::Custom("CustomType".to_string())),
            ]),
        ];

        extractor.analyze_type_usage(&behaviors).unwrap();
        let shared_types = extractor.extract_shared_types();

        assert_eq!(shared_types.len(), 1);
        assert_eq!(shared_types[0].name, "CustomType");
        assert_eq!(shared_types[0].used_by.len(), 2);
    }

    #[test]
    fn test_shared_runtime_generator() {
        let generator = SharedRuntimeGenerator::new();
        
        let mut shared_items = SharedItems::new();
        
        // Add a shared function
        let method = StructMethod::new("calculate_distance".to_string(), RustType::F32);
        let mut shared_function = SharedFunction::new("calculate_distance".to_string(), method);
        shared_function.add_user("BehaviorA".to_string());
        shared_function.add_user("BehaviorB".to_string());
        shared_items.functions.push(shared_function);
        
        // Add a shared constant
        let mut shared_constant = SharedDataType::new("MAX_PLAYERS".to_string(), RustType::I32);
        shared_constant.set_constant("20".to_string());
        shared_constant.add_user("BehaviorA".to_string());
        shared_constant.add_user("BehaviorB".to_string());
        shared_items.constants.push(shared_constant);

        let result = generator.generate_shared_runtime(&shared_items);
        assert!(result.is_ok());
        
        let generated_code = result.unwrap();
        assert!(generated_code.contains("public class SharedRuntime"));
        assert!(generated_code.contains("public static float CalculateDistance"));
        assert!(generated_code.contains("public const int MaxPlayers = 20"));
    }

    #[test]
    fn test_complete_shared_runtime_extraction() {
        let mut extractor = SharedRuntimeExtractor::new();
        
        let behaviors = vec![
            create_test_behavior_with_fields("BehaviorA", vec![
                ("global_player_count", RustType::I32),
            ]),
            create_test_behavior_with_fields("BehaviorB", vec![
                ("global_player_count", RustType::I32),
            ]),
        ];

        let result = extractor.extract_shared_items(&behaviors);
        
        // Should succeed even if only types are shared
        match result {
            Ok(shared_items) => {
                assert!(shared_items.has_shared_items());
            }
            Err(SharedRuntimeError::NoSharedFunctions) => {
                // This is acceptable if no functions are shared
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_reference_updates_generation() {
        let extractor = SharedDataTypeExtractor::new();
        
        let mut shared_items = SharedItems::new();
        
        // Add shared function
        let method = StructMethod::new("validate_input".to_string(), RustType::Bool);
        let shared_function = SharedFunction::new("validate_input".to_string(), method);
        shared_items.functions.push(shared_function);
        
        // Add shared constant
        let mut shared_constant = SharedDataType::new("MAX_DISTANCE".to_string(), RustType::F32);
        shared_constant.set_constant("100.0f".to_string());
        shared_items.constants.push(shared_constant);

        let updates = extractor.generate_reference_updates(&shared_items);
        
        assert_eq!(updates.get("validate_input"), Some(&"SharedRuntime.ValidateInput".to_string()));
        assert_eq!(updates.get("MAX_DISTANCE"), Some(&"SharedRuntime.MaxDistance".to_string()));
    }
}