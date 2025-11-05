//! Initialization order management for multi-behavior UdonSharp projects
//! 
//! This module provides functionality to generate initialization coordinators
//! that manage the startup sequence of multiple UdonBehaviour instances.

use crate::config::InitializationOrderSettings;
use wasm2usharp_enhanced::{BehaviorUnit, InterBehaviorCall, CallType};
use anyhow::{Result, Context};
use std::collections::{HashMap, HashSet, VecDeque};

/// Initialization coordinator generator
pub struct InitializationCoordinator {
    settings: InitializationOrderSettings,
}

impl InitializationCoordinator {
    /// Create a new initialization coordinator with the given settings
    pub fn new(settings: InitializationOrderSettings) -> Self {
        Self { settings }
    }
    
    /// Generate initialization coordinator code
    pub fn generate_coordinator(&self, behavior_units: &[BehaviorUnit], namespace: Option<&str>) -> Result<CoordinatorGenerationResult> {
        let initialization_order = self.determine_initialization_order(behavior_units)?;
        let coordinator_code = self.generate_coordinator_code(&initialization_order, behavior_units, namespace)?;
        let script_execution_order = if self.settings.use_script_execution_order {
            Some(self.generate_script_execution_order(&initialization_order)?)
        } else {
            None
        };
        
        Ok(CoordinatorGenerationResult {
            coordinator_code,
            initialization_order: initialization_order.clone(),
            script_execution_order,
            metadata: CoordinatorMetadata {
                total_behaviors: behavior_units.len(),
                has_dependencies: behavior_units.iter().any(|b| !b.inter_behavior_calls.is_empty()),
                coordinator_class_name: self.settings.coordinator_class_name.clone(),
                uses_auto_order: self.settings.auto_determine_order,
            },
        })
    }
    
    /// Determine the initialization order based on dependencies
    fn determine_initialization_order(&self, behavior_units: &[BehaviorUnit]) -> Result<Vec<String>> {
        if self.settings.auto_determine_order {
            self.auto_determine_order(behavior_units)
        } else {
            // Use manual order, but validate it
            self.validate_manual_order(&self.settings.manual_order, behavior_units)?;
            Ok(self.settings.manual_order.clone())
        }
    }
    
    /// Automatically determine initialization order using topological sort
    fn auto_determine_order(&self, behavior_units: &[BehaviorUnit]) -> Result<Vec<String>> {
        let mut dependency_graph = self.build_dependency_graph(behavior_units)?;
        let mut initialization_order = Vec::new();
        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        
        // Calculate in-degrees for all behaviors
        for behavior_unit in behavior_units {
            in_degree.insert(behavior_unit.name.clone(), 0);
        }
        
        for (behavior_name, dependencies) in &dependency_graph {
            for dependency in dependencies {
                *in_degree.entry(dependency.clone()).or_insert(0) += 1;
            }
        }
        
        // Find behaviors with no dependencies (in-degree 0)
        for (behavior_name, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(behavior_name.clone());
            }
        }
        
        // Process behaviors in topological order
        while let Some(current_behavior) = queue.pop_front() {
            initialization_order.push(current_behavior.clone());
            
            // Reduce in-degree for dependent behaviors
            if let Some(dependencies) = dependency_graph.get(&current_behavior) {
                for dependency in dependencies {
                    if let Some(degree) = in_degree.get_mut(dependency) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependency.clone());
                        }
                    }
                }
            }
        }
        
        // Check for circular dependencies
        if initialization_order.len() != behavior_units.len() {
            let remaining_behaviors: Vec<_> = behavior_units
                .iter()
                .map(|b| &b.name)
                .filter(|name| !initialization_order.contains(name))
                .collect();
            
            return Err(anyhow::anyhow!(
                "Circular dependency detected among behaviors: {:?}",
                remaining_behaviors
            ));
        }
        
        Ok(initialization_order)
    }
    
    /// Build dependency graph from behavior units
    fn build_dependency_graph(&self, behavior_units: &[BehaviorUnit]) -> Result<HashMap<String, Vec<String>>> {
        let mut dependency_graph = HashMap::new();
        
        for behavior_unit in behavior_units {
            let mut dependencies = Vec::new();
            
            // Add dependencies based on inter-behavior calls
            for call in &behavior_unit.inter_behavior_calls {
                if call.call_type == CallType::Direct {
                    // Direct calls require the target behavior to be initialized first
                    dependencies.push(call.target_behavior.clone());
                }
            }
            
            // Remove duplicates
            dependencies.sort();
            dependencies.dedup();
            
            dependency_graph.insert(behavior_unit.name.clone(), dependencies);
        }
        
        Ok(dependency_graph)
    }
    
    /// Validate manual initialization order
    fn validate_manual_order(&self, manual_order: &[String], behavior_units: &[BehaviorUnit]) -> Result<()> {
        let behavior_names: HashSet<_> = behavior_units.iter().map(|b| &b.name).collect();
        let manual_names: HashSet<_> = manual_order.iter().collect();
        
        // Check that all behaviors are included
        for behavior_unit in behavior_units {
            if !manual_names.contains(&behavior_unit.name) {
                return Err(anyhow::anyhow!(
                    "Behavior '{}' is missing from manual initialization order",
                    behavior_unit.name
                ));
            }
        }
        
        // Check that no extra behaviors are included
        for manual_name in manual_order {
            if !behavior_names.contains(manual_name) {
                return Err(anyhow::anyhow!(
                    "Unknown behavior '{}' in manual initialization order",
                    manual_name
                ));
            }
        }
        
        // Check for dependency violations
        let dependency_graph = self.build_dependency_graph(behavior_units)?;
        let position_map: HashMap<_, _> = manual_order
            .iter()
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();
        
        for (behavior_name, dependencies) in &dependency_graph {
            let behavior_position = position_map[behavior_name];
            
            for dependency in dependencies {
                let dependency_position = position_map[dependency];
                
                if dependency_position >= behavior_position {
                    return Err(anyhow::anyhow!(
                        "Dependency violation: '{}' depends on '{}' but is initialized before it",
                        behavior_name,
                        dependency
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate coordinator class code
    fn generate_coordinator_code(
        &self,
        initialization_order: &[String],
        behavior_units: &[BehaviorUnit],
        namespace: Option<&str>,
    ) -> Result<String> {
        let mut code = String::new();
        
        // Add using statements
        code.push_str("using UnityEngine;\n");
        code.push_str("using VRC.SDKBase;\n");
        code.push_str("using VRC.Udon;\n");
        code.push_str("using UdonSharp;\n");
        code.push_str("using System.Collections;\n\n");
        
        // Add namespace if specified
        let indent = if let Some(ns) = namespace {
            code.push_str(&format!("namespace {}\n{{\n", ns));
            "    "
        } else {
            ""
        };
        
        // Add class header
        code.push_str(&format!("{}/// <summary>\n", indent));
        code.push_str(&format!("{}/// Initialization coordinator for multi-behavior UdonSharp system\n", indent));
        code.push_str(&format!("{}/// Manages the startup sequence of {} behaviors\n", indent, behavior_units.len()));
        code.push_str(&format!("{}/// </summary>\n", indent));
        code.push_str(&format!("{}[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\n", indent));
        code.push_str(&format!("{}public class {} : UdonSharpBehaviour\n", indent, self.settings.coordinator_class_name));
        code.push_str(&format!("{}{{\n", indent));
        
        // Add behavior reference fields
        for behavior_name in initialization_order {
            let field_name = self.to_snake_case(behavior_name);
            let class_name = self.to_pascal_case(behavior_name);
            
            code.push_str(&format!("{}    [Header(\"{} Behavior\")]\n", indent, behavior_name));
            code.push_str(&format!("{}    [SerializeField] private {} _{}_behavior;\n", indent, class_name, field_name));
        }
        
        // Add initialization state fields
        code.push_str(&format!("\n{}    [Header(\"Initialization State\")]\n", indent));
        code.push_str(&format!("{}    [SerializeField] private bool _initializationComplete = false;\n", indent));
        code.push_str(&format!("{}    [SerializeField] private int _currentInitializationStep = 0;\n", indent));
        code.push_str(&format!("{}    [SerializeField] private float _initializationTimeout = 30.0f;\n", indent));
        code.push_str(&format!("{}    private float _initializationStartTime;\n", indent));
        
        // Add initialization status tracking
        code.push_str(&format!("\n{}    // Initialization status for each behavior\n", indent));
        for behavior_name in initialization_order {
            let field_name = self.to_snake_case(behavior_name);
            code.push_str(&format!("{}    private bool _{}_initialized = false;\n", indent, field_name));
        }
        
        // Add Start method
        code.push_str(&format!("\n{}    void Start()\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        _initializationStartTime = Time.time;\n", indent));
        code.push_str(&format!("{}        StartCoroutine(nameof(InitializeBehaviors));\n", indent));
        code.push_str(&format!("{}    }}\n", indent));
        
        // Add initialization coroutine
        code.push_str(&format!("\n{}    /// <summary>\n", indent));
        code.push_str(&format!("{}    /// Initializes behaviors in the correct order\n", indent));
        code.push_str(&format!("{}    /// </summary>\n", indent));
        code.push_str(&format!("{}    public IEnumerator InitializeBehaviors()\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        Debug.Log(\"[{}] Starting behavior initialization sequence\");\n", indent, self.settings.coordinator_class_name));
        
        // Generate initialization steps
        for (index, behavior_name) in initialization_order.iter().enumerate() {
            let field_name = self.to_snake_case(behavior_name);
            
            code.push_str(&format!("\n{}        // Step {}: Initialize {}\n", indent, index + 1, behavior_name));
            code.push_str(&format!("{}        _currentInitializationStep = {};\n", indent, index + 1));
            code.push_str(&format!("{}        Debug.Log($\"[{}] Initializing behavior: {}\");\n", indent, self.settings.coordinator_class_name, behavior_name));
            
            // Check if behavior reference exists
            code.push_str(&format!("{}        if (_{}_behavior == null)\n", indent, field_name));
            code.push_str(&format!("{}        {{\n", indent));
            code.push_str(&format!("{}            Debug.LogError($\"[{}] {} behavior reference is null!\");\n", indent, self.settings.coordinator_class_name, behavior_name));
            code.push_str(&format!("{}            yield break;\n", indent));
            code.push_str(&format!("{}        }}\n", indent));
            
            // Initialize the behavior
            code.push_str(&format!("{}        _{}_behavior.gameObject.SetActive(true);\n", indent, field_name));
            code.push_str(&format!("{}        \n", indent));
            
            // Wait for initialization to complete (simplified - in a real implementation, 
            // behaviors would signal when they're ready)
            code.push_str(&format!("{}        // Wait for behavior to initialize\n", indent));
            code.push_str(&format!("{}        yield return new WaitForSeconds(0.1f);\n", indent));
            code.push_str(&format!("{}        _{}_initialized = true;\n", indent, field_name));
            
            // Check for timeout
            code.push_str(&format!("{}        if (Time.time - _initializationStartTime > _initializationTimeout)\n", indent));
            code.push_str(&format!("{}        {{\n", indent));
            code.push_str(&format!("{}            Debug.LogError($\"[{}] Initialization timeout reached!\");\n", indent, self.settings.coordinator_class_name));
            code.push_str(&format!("{}            yield break;\n", indent));
            code.push_str(&format!("{}        }}\n", indent));
        }
        
        // Mark initialization as complete
        code.push_str(&format!("\n{}        _initializationComplete = true;\n", indent));
        code.push_str(&format!("{}        Debug.Log($\"[{}] All behaviors initialized successfully in {{Time.time - _initializationStartTime:F2}} seconds\");\n", indent, self.settings.coordinator_class_name));
        code.push_str(&format!("{}        \n", indent));
        code.push_str(&format!("{}        // Notify all behaviors that initialization is complete\n", indent));
        code.push_str(&format!("{}        SendCustomEvent(\"OnInitializationComplete\");\n", indent));
        code.push_str(&format!("{}    }}\n", indent));
        
        // Add public properties for checking initialization status
        code.push_str(&format!("\n{}    /// <summary>\n", indent));
        code.push_str(&format!("{}    /// Returns true if all behaviors have been initialized\n", indent));
        code.push_str(&format!("{}    /// </summary>\n", indent));
        code.push_str(&format!("{}    public bool IsInitializationComplete => _initializationComplete;\n", indent));
        
        code.push_str(&format!("\n{}    /// <summary>\n", indent));
        code.push_str(&format!("{}    /// Returns the current initialization step (1-based)\n", indent));
        code.push_str(&format!("{}    /// </summary>\n", indent));
        code.push_str(&format!("{}    public int CurrentInitializationStep => _currentInitializationStep;\n", indent));
        
        // Add method to check if specific behavior is initialized
        code.push_str(&format!("\n{}    /// <summary>\n", indent));
        code.push_str(&format!("{}    /// Check if a specific behavior has been initialized\n", indent));
        code.push_str(&format!("{}    /// </summary>\n", indent));
        code.push_str(&format!("{}    public bool IsBehaviorInitialized(string behaviorName)\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        switch (behaviorName)\n", indent));
        code.push_str(&format!("{}        {{\n", indent));
        
        for behavior_name in initialization_order {
            let field_name = self.to_snake_case(behavior_name);
            code.push_str(&format!("{}            case \"{}\":\n", indent, behavior_name));
            code.push_str(&format!("{}                return _{}_initialized;\n", indent, field_name));
        }
        
        code.push_str(&format!("{}            default:\n", indent));
        code.push_str(&format!("{}                Debug.LogWarning($\"[{}] Unknown behavior name: {{behaviorName}}\");\n", indent, self.settings.coordinator_class_name));
        code.push_str(&format!("{}                return false;\n", indent));
        code.push_str(&format!("{}        }}\n", indent));
        code.push_str(&format!("{}    }}\n", indent));
        
        // Add event handler for initialization complete
        code.push_str(&format!("\n{}    /// <summary>\n", indent));
        code.push_str(&format!("{}    /// Called when all behaviors have been initialized\n", indent));
        code.push_str(&format!("{}    /// </summary>\n", indent));
        code.push_str(&format!("{}    public void OnInitializationComplete()\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        // Override this method to add custom logic after initialization\n", indent));
        code.push_str(&format!("{}        Debug.Log($\"[{}] Initialization sequence completed!\");\n", indent, self.settings.coordinator_class_name));
        code.push_str(&format!("{}    }}\n", indent));
        
        // Add manual initialization trigger (for debugging)
        code.push_str(&format!("\n{}    /// <summary>\n", indent));
        code.push_str(&format!("{}    /// Manually trigger initialization (for debugging)\n", indent));
        code.push_str(&format!("{}    /// </summary>\n", indent));
        code.push_str(&format!("{}    [ContextMenu(\"Reinitialize Behaviors\")]\n", indent));
        code.push_str(&format!("{}    public void ReinitializeBehaviors()\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        if (Application.isPlaying)\n", indent));
        code.push_str(&format!("{}        {{\n", indent));
        code.push_str(&format!("{}            _initializationComplete = false;\n", indent));
        code.push_str(&format!("{}            _currentInitializationStep = 0;\n", indent));
        
        for behavior_name in initialization_order {
            let field_name = self.to_snake_case(behavior_name);
            code.push_str(&format!("{}            _{}_initialized = false;\n", indent, field_name));
        }
        
        code.push_str(&format!("{}            _initializationStartTime = Time.time;\n", indent));
        code.push_str(&format!("{}            StartCoroutine(nameof(InitializeBehaviors));\n", indent));
        code.push_str(&format!("{}        }}\n", indent));
        code.push_str(&format!("{}    }}\n", indent));
        
        // Close class
        code.push_str(&format!("{}}}\n", indent));
        
        // Close namespace if opened
        if namespace.is_some() {
            code.push_str("}\n");
        }
        
        Ok(code)
    }
    
    /// Generate Unity script execution order settings
    fn generate_script_execution_order(&self, initialization_order: &[String]) -> Result<ScriptExecutionOrder> {
        let mut execution_order = Vec::new();
        
        // Coordinator should execute first
        execution_order.push(ScriptExecutionEntry {
            script_name: self.settings.coordinator_class_name.clone(),
            execution_order: -1000,
        });
        
        // Behaviors should execute in dependency order
        for (index, behavior_name) in initialization_order.iter().enumerate() {
            execution_order.push(ScriptExecutionEntry {
                script_name: self.to_pascal_case(behavior_name),
                execution_order: (index as i32) * 100,
            });
        }
        
        Ok(ScriptExecutionOrder {
            entries: execution_order,
        })
    }
    
    /// Convert string to PascalCase
    fn to_pascal_case(&self, input: &str) -> String {
        input
            .split(|c| c == '_' || c == '-' || c == ' ')
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join("")
    }
    
    /// Convert PascalCase to snake_case
    fn to_snake_case(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        
        result
    }
}

/// Result of coordinator generation
#[derive(Debug)]
pub struct CoordinatorGenerationResult {
    /// Generated coordinator class code
    pub coordinator_code: String,
    /// Determined initialization order
    pub initialization_order: Vec<String>,
    /// Unity script execution order settings (if enabled)
    pub script_execution_order: Option<ScriptExecutionOrder>,
    /// Generation metadata
    pub metadata: CoordinatorMetadata,
}

/// Metadata about coordinator generation
#[derive(Debug)]
pub struct CoordinatorMetadata {
    /// Total number of behaviors managed
    pub total_behaviors: usize,
    /// Whether any behaviors have dependencies
    pub has_dependencies: bool,
    /// Name of the coordinator class
    pub coordinator_class_name: String,
    /// Whether automatic order determination was used
    pub uses_auto_order: bool,
}

/// Unity script execution order configuration
#[derive(Debug)]
pub struct ScriptExecutionOrder {
    /// Script execution order entries
    pub entries: Vec<ScriptExecutionEntry>,
}

/// Single script execution order entry
#[derive(Debug)]
pub struct ScriptExecutionEntry {
    /// Name of the script/class
    pub script_name: String,
    /// Execution order (lower numbers execute first)
    pub execution_order: i32,
}

impl ScriptExecutionOrder {
    /// Generate Unity editor script to set execution order
    pub fn generate_editor_script(&self, namespace: Option<&str>) -> String {
        let mut code = String::new();
        
        code.push_str("#if UNITY_EDITOR\n");
        code.push_str("using UnityEngine;\n");
        code.push_str("using UnityEditor;\n\n");
        
        let indent = if let Some(ns) = namespace {
            code.push_str(&format!("namespace {}\n{{\n", ns));
            "    "
        } else {
            ""
        };
        
        code.push_str(&format!("{}/// <summary>\n", indent));
        code.push_str(&format!("{}/// Automatically sets script execution order for multi-behavior system\n", indent));
        code.push_str(&format!("{}/// </summary>\n", indent));
        code.push_str(&format!("{}[InitializeOnLoad]\n", indent));
        code.push_str(&format!("{}public static class MultiBehaviorExecutionOrderSetup\n", indent));
        code.push_str(&format!("{}{{\n", indent));
        
        code.push_str(&format!("{}    static MultiBehaviorExecutionOrderSetup()\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        SetExecutionOrder();\n", indent));
        code.push_str(&format!("{}    }}\n", indent));
        
        code.push_str(&format!("\n{}    private static void SetExecutionOrder()\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        
        for entry in &self.entries {
            code.push_str(&format!("{}        SetScriptExecutionOrder(\"{}\", {});\n", indent, entry.script_name, entry.execution_order));
        }
        
        code.push_str(&format!("{}    }}\n", indent));
        
        code.push_str(&format!("\n{}    private static void SetScriptExecutionOrder(string scriptName, int order)\n", indent));
        code.push_str(&format!("{}    {{\n", indent));
        code.push_str(&format!("{}        var script = AssetDatabase.FindAssets($\"t:MonoScript {{scriptName}}\").FirstOrDefault();\n", indent));
        code.push_str(&format!("{}        if (script != null)\n", indent));
        code.push_str(&format!("{}        {{\n", indent));
        code.push_str(&format!("{}            var scriptPath = AssetDatabase.GUIDToAssetPath(script);\n", indent));
        code.push_str(&format!("{}            var monoScript = AssetDatabase.LoadAssetAtPath<MonoScript>(scriptPath);\n", indent));
        code.push_str(&format!("{}            if (monoScript != null)\n", indent));
        code.push_str(&format!("{}            {{\n", indent));
        code.push_str(&format!("{}                MonoImporter.SetExecutionOrder(monoScript, order);\n", indent));
        code.push_str(&format!("{}            }}\n", indent));
        code.push_str(&format!("{}        }}\n", indent));
        code.push_str(&format!("{}    }}\n", indent));
        
        code.push_str(&format!("{}}}\n", indent));
        
        if namespace.is_some() {
            code.push_str("}\n");
        }
        
        code.push_str("#endif\n");
        
        code
    }
}