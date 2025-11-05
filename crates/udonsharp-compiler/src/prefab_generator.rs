//! Unity prefab generation for multi-behavior UdonSharp projects
//! 
//! This module provides functionality to generate Unity prefab files with
//! appropriate component configurations and dependencies for multi-behavior
//! UdonSharp projects.

use crate::config::{PrefabGenerationSettings, InitializationOrderSettings};
use wasm2usharp_enhanced::{BehaviorUnit, InterBehaviorCall, CallType};
use anyhow::{Result, Context};
use std::collections::{HashMap, HashSet};
use serde_json::{json, Value};
use uuid::Uuid;

/// Unity prefab generator for multi-behavior projects
pub struct UnityPrefabGenerator {
    settings: PrefabGenerationSettings,
    initialization_settings: InitializationOrderSettings,
}

impl UnityPrefabGenerator {
    /// Create a new prefab generator with the given settings
    pub fn new(settings: PrefabGenerationSettings, initialization_settings: InitializationOrderSettings) -> Self {
        Self {
            settings,
            initialization_settings,
        }
    }
    
    /// Generate all prefab files for the given behavior units
    pub fn generate_prefabs(&self, behavior_units: &[BehaviorUnit], namespace: Option<&str>) -> Result<PrefabGenerationResult> {
        let mut individual_prefabs = HashMap::new();
        let mut master_prefab = None;
        let mut example_scene = None;
        
        // Generate individual prefabs for each behavior
        if self.settings.generate_individual_prefabs {
            for behavior_unit in behavior_units {
                let prefab_name = format!("{}.prefab", behavior_unit.name);
                let prefab_content = self.generate_individual_prefab(behavior_unit, namespace)?;
                individual_prefabs.insert(prefab_name, prefab_content);
            }
        }
        
        // Generate master prefab containing all behaviors
        if self.settings.generate_master_prefab {
            let master_prefab_content = self.generate_master_prefab(behavior_units, namespace)?;
            master_prefab = Some(master_prefab_content);
        }
        
        // Generate example scene if requested
        if self.settings.include_example_scene {
            let scene_content = self.generate_example_scene(behavior_units, namespace)?;
            example_scene = Some(scene_content);
        }
        
        let total_prefabs = individual_prefabs.len() + if master_prefab.is_some() { 1 } else { 0 };
        let has_dependencies = behavior_units.iter().any(|b| !b.inter_behavior_calls.is_empty());
        
        Ok(PrefabGenerationResult {
            individual_prefabs,
            master_prefab,
            example_scene,
            metadata: PrefabGenerationMetadata {
                total_prefabs,
                total_behaviors: behavior_units.len(),
                has_dependencies,
                generated_at: chrono::Utc::now(),
            },
        })
    }
    
    /// Generate an individual prefab for a single behavior
    fn generate_individual_prefab(&self, behavior_unit: &BehaviorUnit, namespace: Option<&str>) -> Result<String> {
        let class_name = self.generate_class_name(&behavior_unit.name, namespace);
        let prefab_data = self.create_prefab_structure(&class_name, &[behavior_unit.clone()])?;
        
        // Convert to YAML format (Unity's prefab format)
        self.serialize_prefab_to_yaml(&prefab_data)
    }
    
    /// Generate a master prefab containing all behaviors
    fn generate_master_prefab(&self, behavior_units: &[BehaviorUnit], namespace: Option<&str>) -> Result<String> {
        let prefab_data = self.create_master_prefab_structure(behavior_units, namespace)?;
        self.serialize_prefab_to_yaml(&prefab_data)
    }
    
    /// Generate an example scene demonstrating the behaviors
    fn generate_example_scene(&self, behavior_units: &[BehaviorUnit], namespace: Option<&str>) -> Result<String> {
        let scene_data = self.create_example_scene_structure(behavior_units, namespace)?;
        self.serialize_scene_to_yaml(&scene_data)
    }
    
    /// Create the prefab structure for a single behavior
    fn create_prefab_structure(&self, class_name: &str, behavior_units: &[BehaviorUnit]) -> Result<Value> {
        let mut components = Vec::new();
        let mut game_objects = Vec::new();
        
        // Create the main GameObject
        let main_object_id = self.generate_file_id();
        let transform_id = self.generate_file_id();
        
        // Add Transform component
        components.push(json!({
            "component": {
                "fileID": transform_id
            }
        }));
        
        // Add UdonBehaviour components for each behavior unit
        for behavior_unit in behavior_units {
            let behavior_class_name = self.generate_class_name(&behavior_unit.name, None);
            let udon_behaviour_id = self.generate_file_id();
            
            components.push(json!({
                "component": {
                    "fileID": udon_behaviour_id
                }
            }));
            
            // Create the UdonBehaviour component data
            let udon_behaviour_data = self.create_udon_behaviour_component(
                udon_behaviour_id,
                &behavior_class_name,
                behavior_unit
            )?;
            
            game_objects.push(udon_behaviour_data);
        }
        
        // Create the main GameObject data
        let main_object_data = json!({
            "GameObject": {
                "m_ObjectHideFlags": 0,
                "m_CorrespondingSourceObject": {
                    "fileID": 0
                },
                "m_PrefabInstance": {
                    "fileID": 0
                },
                "m_PrefabAsset": {
                    "fileID": 0
                },
                "serializedVersion": 6,
                "m_Component": components,
                "m_Layer": 0,
                "m_Name": class_name,
                "m_TagString": "Untagged",
                "m_Icon": {
                    "fileID": 0
                },
                "m_NavMeshLayer": 0,
                "m_StaticEditorFlags": 0,
                "m_IsActive": 1
            }
        });
        
        game_objects.insert(0, json!({
            main_object_id.to_string(): main_object_data
        }));
        
        // Create Transform component data
        let transform_data = json!({
            "Transform": {
                "m_ObjectHideFlags": 0,
                "m_CorrespondingSourceObject": {
                    "fileID": 0
                },
                "m_PrefabInstance": {
                    "fileID": 0
                },
                "m_PrefabAsset": {
                    "fileID": 0
                },
                "m_GameObject": {
                    "fileID": main_object_id
                },
                "m_LocalRotation": {
                    "x": 0,
                    "y": 0,
                    "z": 0,
                    "w": 1
                },
                "m_LocalPosition": {
                    "x": 0,
                    "y": 0,
                    "z": 0
                },
                "m_LocalScale": {
                    "x": 1,
                    "y": 1,
                    "z": 1
                },
                "m_Children": [],
                "m_Father": {
                    "fileID": 0
                },
                "m_RootOrder": 0,
                "m_LocalEulerAnglesHint": {
                    "x": 0,
                    "y": 0,
                    "z": 0
                }
            }
        });
        
        game_objects.push(json!({
            transform_id.to_string(): transform_data
        }));
        
        // Create the complete prefab structure
        Ok(json!({
            "%YAML 1.1": null,
            "%TAG !u! tag:unity3d.com,2011:": null,
            "objects": game_objects
        }))
    }
    
    /// Create a master prefab structure containing all behaviors
    fn create_master_prefab_structure(&self, behavior_units: &[BehaviorUnit], namespace: Option<&str>) -> Result<Value> {
        let mut components = Vec::new();
        let mut game_objects = Vec::new();
        let mut child_objects = Vec::new();
        
        // Create the main GameObject
        let main_object_id = self.generate_file_id();
        let transform_id = self.generate_file_id();
        
        // Add Transform component to main object
        components.push(json!({
            "component": {
                "fileID": transform_id
            }
        }));
        
        // Add coordinator component if enabled
        if self.initialization_settings.generate_coordinator {
            let coordinator_id = self.generate_file_id();
            components.push(json!({
                "component": {
                    "fileID": coordinator_id
                }
            }));
            
            let coordinator_data = self.create_coordinator_component(coordinator_id, behavior_units)?;
            game_objects.push(coordinator_data);
        }
        
        // Create child GameObjects for each behavior
        for behavior_unit in behavior_units {
            let child_object_id = self.generate_file_id();
            let child_transform_id = self.generate_file_id();
            let udon_behaviour_id = self.generate_file_id();
            
            child_objects.push(json!({
                "fileID": child_object_id
            }));
            
            // Create child GameObject
            let child_object_data = json!({
                "GameObject": {
                    "m_ObjectHideFlags": 0,
                    "m_CorrespondingSourceObject": {
                        "fileID": 0
                    },
                    "m_PrefabInstance": {
                        "fileID": 0
                    },
                    "m_PrefabAsset": {
                        "fileID": 0
                    },
                    "serializedVersion": 6,
                    "m_Component": [
                        {
                            "component": {
                                "fileID": child_transform_id
                            }
                        },
                        {
                            "component": {
                                "fileID": udon_behaviour_id
                            }
                        }
                    ],
                    "m_Layer": 0,
                    "m_Name": behavior_unit.name,
                    "m_TagString": "Untagged",
                    "m_Icon": {
                        "fileID": 0
                    },
                    "m_NavMeshLayer": 0,
                    "m_StaticEditorFlags": 0,
                    "m_IsActive": 1
                }
            });
            
            game_objects.push(json!({
                child_object_id.to_string(): child_object_data
            }));
            
            // Create child Transform
            let child_transform_data = json!({
                "Transform": {
                    "m_ObjectHideFlags": 0,
                    "m_CorrespondingSourceObject": {
                        "fileID": 0
                    },
                    "m_PrefabInstance": {
                        "fileID": 0
                    },
                    "m_PrefabAsset": {
                        "fileID": 0
                    },
                    "m_GameObject": {
                        "fileID": child_object_id
                    },
                    "m_LocalRotation": {
                        "x": 0,
                        "y": 0,
                        "z": 0,
                        "w": 1
                    },
                    "m_LocalPosition": {
                        "x": 0,
                        "y": 0,
                        "z": 0
                    },
                    "m_LocalScale": {
                        "x": 1,
                        "y": 1,
                        "z": 1
                    },
                    "m_Children": [],
                    "m_Father": {
                        "fileID": transform_id
                    },
                    "m_RootOrder": child_objects.len() - 1,
                    "m_LocalEulerAnglesHint": {
                        "x": 0,
                        "y": 0,
                        "z": 0
                    }
                }
            });
            
            game_objects.push(json!({
                child_transform_id.to_string(): child_transform_data
            }));
            
            // Create UdonBehaviour component
            let behavior_class_name = self.generate_class_name(&behavior_unit.name, namespace);
            let udon_behaviour_data = self.create_udon_behaviour_component(
                udon_behaviour_id,
                &behavior_class_name,
                behavior_unit
            )?;
            
            game_objects.push(udon_behaviour_data);
        }
        
        // Create the main GameObject data
        let main_object_data = json!({
            "GameObject": {
                "m_ObjectHideFlags": 0,
                "m_CorrespondingSourceObject": {
                    "fileID": 0
                },
                "m_PrefabInstance": {
                    "fileID": 0
                },
                "m_PrefabAsset": {
                    "fileID": 0
                },
                "serializedVersion": 6,
                "m_Component": components,
                "m_Layer": 0,
                "m_Name": "MultiBehaviorSystem",
                "m_TagString": "Untagged",
                "m_Icon": {
                    "fileID": 0
                },
                "m_NavMeshLayer": 0,
                "m_StaticEditorFlags": 0,
                "m_IsActive": 1
            }
        });
        
        game_objects.insert(0, json!({
            main_object_id.to_string(): main_object_data
        }));
        
        // Create main Transform component data
        let transform_data = json!({
            "Transform": {
                "m_ObjectHideFlags": 0,
                "m_CorrespondingSourceObject": {
                    "fileID": 0
                },
                "m_PrefabInstance": {
                    "fileID": 0
                },
                "m_PrefabAsset": {
                    "fileID": 0
                },
                "m_GameObject": {
                    "fileID": main_object_id
                },
                "m_LocalRotation": {
                    "x": 0,
                    "y": 0,
                    "z": 0,
                    "w": 1
                },
                "m_LocalPosition": {
                    "x": 0,
                    "y": 0,
                    "z": 0
                },
                "m_LocalScale": {
                    "x": 1,
                    "y": 1,
                    "z": 1
                },
                "m_Children": child_objects,
                "m_Father": {
                    "fileID": 0
                },
                "m_RootOrder": 0,
                "m_LocalEulerAnglesHint": {
                    "x": 0,
                    "y": 0,
                    "z": 0
                }
            }
        });
        
        game_objects.push(json!({
            transform_id.to_string(): transform_data
        }));
        
        // Create the complete prefab structure
        Ok(json!({
            "%YAML 1.1": null,
            "%TAG !u! tag:unity3d.com,2011:": null,
            "objects": game_objects
        }))
    }
    
    /// Create UdonBehaviour component data
    fn create_udon_behaviour_component(&self, component_id: u64, class_name: &str, behavior_unit: &BehaviorUnit) -> Result<Value> {
        let mut serialized_fields = Vec::new();
        
        // Add reference fields for inter-behavior communication
        if self.settings.auto_setup_references {
            for call in &behavior_unit.inter_behavior_calls {
                if call.call_type == CallType::Direct {
                    let field_name = format!("_{}_reference", self.to_snake_case(&call.target_behavior));
                    serialized_fields.push(json!({
                        "fieldName": field_name,
                        "fieldType": "UnityEngine.GameObject",
                        "value": {
                            "fileID": 0
                        }
                    }));
                }
            }
        }
        
        Ok(json!({
            component_id.to_string(): {
                "MonoBehaviour": {
                    "m_ObjectHideFlags": 0,
                    "m_CorrespondingSourceObject": {
                        "fileID": 0
                    },
                    "m_PrefabInstance": {
                        "fileID": 0
                    },
                    "m_PrefabAsset": {
                        "fileID": 0
                    },
                    "m_GameObject": {
                        "fileID": 0  // Will be set by the parent
                    },
                    "m_Enabled": 1,
                    "m_EditorHideFlags": 0,
                    "m_Script": {
                        "fileID": 11500000,
                        "guid": "45115577ef41a5b4ca741ed302693907",  // UdonSharp GUID
                        "type": 3
                    },
                    "m_Name": "",
                    "m_EditorClassIdentifier": "",
                    "serializedUdonProgramAsset": {
                        "fileID": 0  // Will reference the compiled UdonSharp program
                    },
                    "udonSharpBackingUdonBehaviour": {
                        "fileID": 0
                    },
                    "serializedFields": serialized_fields
                }
            }
        }))
    }
    
    /// Create coordinator component for initialization order management
    fn create_coordinator_component(&self, component_id: u64, behavior_units: &[BehaviorUnit]) -> Result<Value> {
        let initialization_order = if self.initialization_settings.auto_determine_order {
            self.determine_initialization_order(behavior_units)?
        } else {
            self.initialization_settings.manual_order.clone()
        };
        
        let mut behavior_references = Vec::new();
        for behavior_name in &initialization_order {
            behavior_references.push(json!({
                "fieldName": format!("_{}_behavior", self.to_snake_case(behavior_name)),
                "fieldType": "UnityEngine.GameObject",
                "value": {
                    "fileID": 0
                }
            }));
        }
        
        Ok(json!({
            component_id.to_string(): {
                "MonoBehaviour": {
                    "m_ObjectHideFlags": 0,
                    "m_CorrespondingSourceObject": {
                        "fileID": 0
                    },
                    "m_PrefabInstance": {
                        "fileID": 0
                    },
                    "m_PrefabAsset": {
                        "fileID": 0
                    },
                    "m_GameObject": {
                        "fileID": 0
                    },
                    "m_Enabled": 1,
                    "m_EditorHideFlags": 0,
                    "m_Script": {
                        "fileID": 11500000,
                        "guid": "45115577ef41a5b4ca741ed302693907",
                        "type": 3
                    },
                    "m_Name": "",
                    "m_EditorClassIdentifier": "",
                    "serializedUdonProgramAsset": {
                        "fileID": 0
                    },
                    "udonSharpBackingUdonBehaviour": {
                        "fileID": 0
                    },
                    "serializedFields": behavior_references
                }
            }
        }))
    }
    
    /// Create example scene structure
    fn create_example_scene_structure(&self, behavior_units: &[BehaviorUnit], namespace: Option<&str>) -> Result<Value> {
        // Create a basic scene with the master prefab instantiated
        Ok(json!({
            "%YAML 1.1": null,
            "%TAG !u! tag:unity3d.com,2011:": null,
            "Scene": {
                "m_ObjectHideFlags": 0,
                "serializedVersion": 2,
                "m_Modification": {
                    "m_TransformParent": {
                        "fileID": 0
                    },
                    "m_Modifications": [],
                    "m_RemovedComponents": []
                },
                "m_ParentPrefab": {
                    "fileID": 0
                },
                "m_RootGameObjects": [
                    {
                        "fileID": self.generate_file_id()
                    }
                ],
                "m_SourcePrefab": {
                    "fileID": 0
                }
            }
        }))
    }
    
    /// Determine initialization order based on dependencies
    fn determine_initialization_order(&self, behavior_units: &[BehaviorUnit]) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        
        // Create dependency map
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for behavior_unit in behavior_units {
            let mut deps = Vec::new();
            for call in &behavior_unit.inter_behavior_calls {
                if call.call_type == CallType::Direct {
                    deps.push(call.target_behavior.clone());
                }
            }
            dependencies.insert(behavior_unit.name.clone(), deps);
        }
        
        // Topological sort
        for behavior_unit in behavior_units {
            if !visited.contains(&behavior_unit.name) {
                self.visit_behavior(&behavior_unit.name, &dependencies, &mut visited, &mut visiting, &mut order)?;
            }
        }
        
        Ok(order)
    }
    
    /// Visit a behavior in topological sort
    fn visit_behavior(
        &self,
        behavior_name: &str,
        dependencies: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<()> {
        if visiting.contains(behavior_name) {
            return Err(anyhow::anyhow!("Circular dependency detected involving behavior: {}", behavior_name));
        }
        
        if visited.contains(behavior_name) {
            return Ok(());
        }
        
        visiting.insert(behavior_name.to_string());
        
        if let Some(deps) = dependencies.get(behavior_name) {
            for dep in deps {
                self.visit_behavior(dep, dependencies, visited, visiting, order)?;
            }
        }
        
        visiting.remove(behavior_name);
        visited.insert(behavior_name.to_string());
        order.push(behavior_name.to_string());
        
        Ok(())
    }
    
    /// Generate a class name with optional namespace
    fn generate_class_name(&self, behavior_name: &str, namespace: Option<&str>) -> String {
        let class_name = self.to_pascal_case(behavior_name);
        if let Some(ns) = namespace {
            format!("{}.{}", ns, class_name)
        } else {
            class_name
        }
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
    
    /// Generate a unique file ID for Unity objects
    fn generate_file_id(&self) -> u64 {
        // Generate a pseudo-random file ID based on current time and random data
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).hash(&mut hasher);
        Uuid::new_v4().hash(&mut hasher);
        hasher.finish()
    }
    
    /// Serialize prefab data to YAML format
    fn serialize_prefab_to_yaml(&self, data: &Value) -> Result<String> {
        // Convert JSON to YAML
        // This is a simplified implementation - in a real scenario, you'd want
        // to use a proper YAML serializer that matches Unity's format exactly
        serde_yaml::to_string(data)
            .context("Failed to serialize prefab to YAML")
    }
    
    /// Serialize scene data to YAML format
    fn serialize_scene_to_yaml(&self, data: &Value) -> Result<String> {
        serde_yaml::to_string(data)
            .context("Failed to serialize scene to YAML")
    }
}

/// Result of prefab generation
#[derive(Debug)]
pub struct PrefabGenerationResult {
    /// Individual prefab files (filename -> content)
    pub individual_prefabs: HashMap<String, String>,
    /// Master prefab content (if generated)
    pub master_prefab: Option<String>,
    /// Example scene content (if generated)
    pub example_scene: Option<String>,
    /// Generation metadata
    pub metadata: PrefabGenerationMetadata,
}

/// Metadata about prefab generation
#[derive(Debug)]
pub struct PrefabGenerationMetadata {
    /// Total number of prefabs generated
    pub total_prefabs: usize,
    /// Total number of behaviors
    pub total_behaviors: usize,
    /// Whether any behaviors have dependencies
    pub has_dependencies: bool,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}