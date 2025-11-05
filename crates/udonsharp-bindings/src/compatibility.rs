//! UdonSharp compatibility checking for assemblies and types
//! 
//! This module provides functionality to check whether assemblies, types, and methods
//! are compatible with UdonSharp constraints.

use crate::asmdef::{AsmdefFile, TypeInfo, MethodInfo, PropertyInfo, FieldInfo, ParameterInfo};
use std::collections::{HashSet, HashMap};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Checker for UdonSharp compatibility
pub struct UdonSharpCompatibilityChecker {
    allowed_namespaces: HashSet<String>,
    forbidden_features: HashSet<String>,
    allowed_types: HashSet<String>,
    forbidden_types: HashSet<String>,
    custom_rules: Option<CustomCompatibilityRules>,
}

/// Custom compatibility rules that can be loaded from configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CustomCompatibilityRules {
    pub additional_allowed_namespaces: Vec<String>,
    pub additional_forbidden_features: Vec<String>,
    pub type_overrides: HashMap<String, bool>, // type_name -> is_compatible
    pub method_overrides: HashMap<String, bool>, // method_signature -> is_compatible
}

/// Compatibility check result with detailed information
#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub is_compatible: bool,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}

impl UdonSharpCompatibilityChecker {
    /// Create a new compatibility checker with default rules
    pub fn new() -> Self {
        let mut checker = Self {
            allowed_namespaces: HashSet::new(),
            forbidden_features: HashSet::new(),
            allowed_types: HashSet::new(),
            forbidden_types: HashSet::new(),
            custom_rules: None,
        };
        
        checker.initialize_compatibility_rules();
        checker
    }
    
    /// Create a compatibility checker with custom rules
    pub fn with_custom_rules(custom_rules: CustomCompatibilityRules) -> Self {
        let mut checker = Self::new();
        checker.apply_custom_rules(custom_rules);
        checker
    }
    
    /// Load custom rules from a JSON file
    pub fn load_custom_rules_from_file(path: &str) -> Result<CustomCompatibilityRules> {
        let content = std::fs::read_to_string(path)?;
        let rules: CustomCompatibilityRules = serde_json::from_str(&content)?;
        Ok(rules)
    }
    
    /// Apply custom compatibility rules
    pub fn apply_custom_rules(&mut self, rules: CustomCompatibilityRules) {
        // Add additional allowed namespaces
        for namespace in &rules.additional_allowed_namespaces {
            self.allowed_namespaces.insert(namespace.clone());
        }
        
        // Add additional forbidden features
        for feature in &rules.additional_forbidden_features {
            self.forbidden_features.insert(feature.clone());
        }
        
        self.custom_rules = Some(rules);
    }
    
    /// Initialize the default compatibility rules
    fn initialize_compatibility_rules(&mut self) {
        // Add UdonSharp-compatible namespaces
        self.allowed_namespaces.insert("UnityEngine".to_string());
        self.allowed_namespaces.insert("VRC.SDKBase".to_string());
        self.allowed_namespaces.insert("VRC.SDK3".to_string());
        self.allowed_namespaces.insert("VRC.Udon".to_string());
        self.allowed_namespaces.insert("System".to_string());
        
        // Add forbidden features that UdonSharp doesn't support
        self.forbidden_features.insert("System.Threading".to_string());
        self.forbidden_features.insert("System.IO".to_string());
        self.forbidden_features.insert("System.Net".to_string());
        self.forbidden_features.insert("System.Reflection".to_string());
        self.forbidden_features.insert("System.Runtime.Serialization".to_string());
        self.forbidden_features.insert("System.Security".to_string());
        
        // Add known compatible types
        let compatible_types = [
            "System.Boolean", "System.Byte", "System.SByte", "System.Int16", "System.UInt16",
            "System.Int32", "System.UInt32", "System.Int64", "System.UInt64", "System.Single",
            "System.Double", "System.Char", "System.String", "System.Object",
            "UnityEngine.Vector2", "UnityEngine.Vector3", "UnityEngine.Vector4",
            "UnityEngine.Quaternion", "UnityEngine.Color", "UnityEngine.Color32",
            "UnityEngine.GameObject", "UnityEngine.Transform", "UnityEngine.Component",
            "UnityEngine.MonoBehaviour", "UnityEngine.Rigidbody", "UnityEngine.Collider",
            "UnityEngine.Renderer", "UnityEngine.Material", "UnityEngine.Texture",
            "UnityEngine.AudioSource", "UnityEngine.Light", "UnityEngine.Camera",
            "VRC.SDKBase.VRCPlayerApi", "VRC.Udon.UdonBehaviour",
        ];
        
        for type_name in compatible_types.iter() {
            self.allowed_types.insert(type_name.to_string());
        }
        
        // Add known incompatible types
        let incompatible_types = [
            "System.Threading.Thread", "System.IO.File", "System.Net.WebClient",
            "System.Reflection.Assembly", "UnityEditor.EditorGUILayout",
        ];
        
        for type_name in incompatible_types.iter() {
            self.forbidden_types.insert(type_name.to_string());
        }
    }
    
    /// Check if an assembly is compatible with UdonSharp
    pub fn is_compatible(&self, asmdef: &AsmdefFile) -> bool {
        self.check_assembly_compatibility(asmdef).is_compatible
    }
    
    /// Check assembly compatibility with detailed results
    pub fn check_assembly_compatibility(&self, asmdef: &AsmdefFile) -> CompatibilityResult {
        let mut result = CompatibilityResult {
            is_compatible: true,
            reasons: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Skip editor-only and test assemblies
        if asmdef.name.contains("Editor") || asmdef.name.contains("Test") {
            result.is_compatible = false;
            result.reasons.push("Assembly is editor-only or test assembly".to_string());
            return result;
        }
        
        // Check if assembly references are compatible
        for reference in &asmdef.references {
            if self.forbidden_features.iter().any(|forbidden| reference.contains(forbidden)) {
                result.is_compatible = false;
                result.reasons.push(format!("Assembly references forbidden feature: {}", reference));
            }
        }
        
        // Check platform constraints
        if !asmdef.include_platforms.is_empty() {
            let has_runtime_platform = asmdef.include_platforms.iter()
                .any(|platform| platform == "Any" || platform.contains("Standalone") || platform.contains("Android"));
            
            if !has_runtime_platform {
                result.warnings.push("Assembly may not be available at runtime".to_string());
            }
        }
        
        // Check for unsafe code
        if asmdef.allow_unsafe_code {
            result.warnings.push("Assembly allows unsafe code, which may not be supported".to_string());
        }
        
        result
    }
    
    /// Check if a type is compatible with UdonSharp
    pub fn is_type_compatible(&self, type_info: &TypeInfo) -> bool {
        self.check_type_compatibility(type_info).is_compatible
    }
    
    /// Check type compatibility with detailed results
    pub fn check_type_compatibility(&self, type_info: &TypeInfo) -> CompatibilityResult {
        let mut result = CompatibilityResult {
            is_compatible: true,
            reasons: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Check custom overrides first
        if let Some(rules) = &self.custom_rules {
            if let Some(&is_compatible) = rules.type_overrides.get(&type_info.full_name) {
                result.is_compatible = is_compatible;
                if !is_compatible {
                    result.reasons.push("Type explicitly forbidden by custom rules".to_string());
                }
                return result;
            }
        }
        
        // Check if type is explicitly forbidden
        if self.forbidden_types.contains(&type_info.full_name) {
            result.is_compatible = false;
            result.reasons.push("Type is explicitly forbidden".to_string());
            return result;
        }
        
        // Check if type is explicitly allowed
        if self.allowed_types.contains(&type_info.full_name) {
            return result; // Compatible
        }
        
        // Check namespace compatibility
        if let Some(namespace) = &type_info.namespace {
            let namespace_allowed = self.allowed_namespaces.iter()
                .any(|allowed| namespace.starts_with(allowed));
            
            if !namespace_allowed {
                result.is_compatible = false;
                result.reasons.push(format!("Namespace '{}' is not allowed", namespace));
                return result;
            }
        }
        
        // Check for unsupported type features
        if type_info.is_generic && !type_info.generic_constraints.is_empty() {
            result.warnings.push("Generic type with constraints may have limited support".to_string());
        }
        
        // Check if type is abstract (may need special handling)
        if type_info.is_abstract {
            result.warnings.push("Abstract type may require special handling".to_string());
        }
        
        result
    }
    
    /// Check if a method is compatible with UdonSharp
    pub fn is_method_compatible(&self, method: &MethodInfo) -> bool {
        self.check_method_compatibility(method).is_compatible
    }
    
    /// Check method compatibility with detailed results
    pub fn check_method_compatibility(&self, method: &MethodInfo) -> CompatibilityResult {
        let mut result = CompatibilityResult {
            is_compatible: true,
            reasons: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Check custom overrides
        if let Some(rules) = &self.custom_rules {
            let method_signature = format!("{}.{}", method.declaring_type, method.name);
            if let Some(&is_compatible) = rules.method_overrides.get(&method_signature) {
                result.is_compatible = is_compatible;
                if !is_compatible {
                    result.reasons.push("Method explicitly forbidden by custom rules".to_string());
                }
                return result;
            }
        }
        
        // Check for unsupported method features
        if method.is_generic {
            result.is_compatible = false;
            result.reasons.push("Generic methods are not supported".to_string());
        }
        
        if method.has_ref_parameters {
            result.is_compatible = false;
            result.reasons.push("Methods with ref parameters are not supported".to_string());
        }
        
        if method.has_out_parameters {
            result.is_compatible = false;
            result.reasons.push("Methods with out parameters are not supported".to_string());
        }
        
        // Check parameter types
        for param in &method.parameters {
            if !self.is_parameter_type_compatible(&param.parameter_type) {
                result.is_compatible = false;
                result.reasons.push(format!("Parameter type '{}' is not compatible", param.parameter_type));
            }
        }
        
        // Check return type
        if !self.is_parameter_type_compatible(&method.return_type) {
            result.is_compatible = false;
            result.reasons.push(format!("Return type '{}' is not compatible", method.return_type));
        }
        
        result
    }
    
    /// Check if a property is compatible with UdonSharp
    pub fn is_property_compatible(&self, property: &PropertyInfo) -> bool {
        self.is_parameter_type_compatible(&property.property_type)
    }
    
    /// Check if a field is compatible with UdonSharp
    pub fn is_field_compatible(&self, field: &FieldInfo) -> bool {
        self.is_parameter_type_compatible(&field.field_type)
    }
    
    /// Check if a parameter type is compatible
    fn is_parameter_type_compatible(&self, type_name: &str) -> bool {
        // Check if type is explicitly allowed
        if self.allowed_types.contains(type_name) {
            return true;
        }
        
        // Check if type is explicitly forbidden
        if self.forbidden_types.contains(type_name) {
            return false;
        }
        
        // Check basic type patterns
        let compatible_patterns = [
            "System.Boolean", "System.Byte", "System.SByte", "System.Int16", "System.UInt16",
            "System.Int32", "System.UInt32", "System.Int64", "System.UInt64", "System.Single",
            "System.Double", "System.Char", "System.String", "System.Object",
            "UnityEngine.", "VRC.SDKBase.", "VRC.SDK3.", "VRC.Udon.",
        ];
        
        compatible_patterns.iter().any(|&pattern| type_name.starts_with(pattern))
    }
    
    /// Get all allowed namespaces
    pub fn get_allowed_namespaces(&self) -> &HashSet<String> {
        &self.allowed_namespaces
    }
    
    /// Get all forbidden features
    pub fn get_forbidden_features(&self) -> &HashSet<String> {
        &self.forbidden_features
    }
}

impl Default for UdonSharpCompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl CompatibilityResult {
    /// Create a compatible result
    pub fn compatible() -> Self {
        Self {
            is_compatible: true,
            reasons: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Create an incompatible result with a reason
    pub fn incompatible(reason: String) -> Self {
        Self {
            is_compatible: false,
            reasons: vec![reason],
            warnings: Vec::new(),
        }
    }
    
    /// Add a warning to the result
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}