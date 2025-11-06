//! Data models and infrastructure for standard multi-behavior pattern
//! 
//! This module provides the core data structures and type mapping systems
//! needed to analyze and generate code for multiple UdonBehaviour structs
//! in a single WASM module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an analyzed Rust UdonBehaviour struct with all its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonBehaviourStruct {
    /// Name of the struct (will become C# class name)
    pub name: String,
    /// Fields defined in the struct
    pub fields: Vec<StructField>,
    /// Methods implemented for this struct
    pub methods: Vec<StructMethod>,
    /// Attributes applied to the struct
    pub attributes: Vec<StructAttribute>,
    /// UdonBehaviour trait implementation details
    pub trait_impl: Option<UdonBehaviourTraitImpl>,
    /// Names of other UdonBehaviour structs this depends on
    pub dependencies: Vec<String>,
}

impl UdonBehaviourStruct {
    /// Create a new UdonBehaviourStruct
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            methods: Vec::new(),
            attributes: Vec::new(),
            trait_impl: None,
            dependencies: Vec::new(),
        }
    }

    /// Add a field to this struct
    pub fn add_field(&mut self, field: StructField) {
        self.fields.push(field);
    }

    /// Add a method to this struct
    pub fn add_method(&mut self, method: StructMethod) {
        self.methods.push(method);
    }

    /// Add an attribute to this struct
    pub fn add_attribute(&mut self, attribute: StructAttribute) {
        self.attributes.push(attribute);
    }

    /// Set the UdonBehaviour trait implementation
    pub fn set_trait_impl(&mut self, trait_impl: UdonBehaviourTraitImpl) {
        self.trait_impl = Some(trait_impl);
    }

    /// Add a dependency on another UdonBehaviour struct
    pub fn add_dependency(&mut self, dependency: String) {
        if !self.dependencies.contains(&dependency) {
            self.dependencies.push(dependency);
        }
    }

    /// Check if this struct has networking capabilities
    pub fn has_networking(&self) -> bool {
        self.fields.iter().any(|f| f.has_sync_attribute()) ||
        self.attributes.iter().any(|a| matches!(a, StructAttribute::UdonSyncMode(_)))
    }

    /// Get all synchronized fields
    pub fn get_sync_fields(&self) -> Vec<&StructField> {
        self.fields.iter().filter(|f| f.has_sync_attribute()).collect()
    }

    /// Validate the struct definition
    pub fn validate(&self) -> Result<(), String> {
        // Check if struct name is valid C# identifier
        if !is_valid_csharp_identifier(&self.name) {
            return Err(format!("Invalid struct name '{}': must be a valid C# identifier", self.name));
        }

        // Check if trait implementation exists
        if self.trait_impl.is_none() {
            return Err(format!("Struct '{}' must implement UdonBehaviour trait", self.name));
        }

        // Validate all fields
        for field in &self.fields {
            field.validate()?;
        }

        // Validate all methods
        for method in &self.methods {
            method.validate()?;
        }

        Ok(())
    }
}

/// Represents a field in a UdonBehaviour struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    /// Field name
    pub name: String,
    /// Rust type of the field
    pub field_type: RustType,
    /// Visibility of the field
    pub visibility: Visibility,
    /// Attributes applied to this field
    pub attributes: Vec<FieldAttribute>,
    /// Default value if any
    pub default_value: Option<String>,
}

impl StructField {
    /// Create a new struct field
    pub fn new(name: String, field_type: RustType) -> Self {
        Self {
            name,
            field_type,
            visibility: Visibility::Private,
            attributes: Vec::new(),
            default_value: None,
        }
    }

    /// Add an attribute to this field
    pub fn add_attribute(&mut self, attribute: FieldAttribute) {
        self.attributes.push(attribute);
    }

    /// Set the visibility of this field
    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    /// Set the default value
    pub fn set_default_value(&mut self, value: String) {
        self.default_value = Some(value);
    }

    /// Check if this field has a sync attribute
    pub fn has_sync_attribute(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, FieldAttribute::UdonSync))
    }

    /// Check if this field is public
    pub fn is_public(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, FieldAttribute::UdonPublic)) ||
        matches!(self.visibility, Visibility::Public)
    }

    /// Validate the field definition
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_csharp_identifier(&self.name) {
            return Err(format!("Invalid field name '{}': must be a valid C# identifier", self.name));
        }
        Ok(())
    }
}

/// Represents a method in a UdonBehaviour struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructMethod {
    /// Method name
    pub name: String,
    /// Attributes applied to this method
    pub attributes: Vec<MethodAttribute>,
    /// Method parameters
    pub parameters: Vec<MethodParameter>,
    /// Return type
    pub return_type: RustType,
    /// Whether this is an async method
    pub is_async: bool,
}

impl StructMethod {
    /// Create a new struct method
    pub fn new(name: String, return_type: RustType) -> Self {
        Self {
            name,
            attributes: Vec::new(),
            parameters: Vec::new(),
            return_type,
            is_async: false,
        }
    }

    /// Add an attribute to this method
    pub fn add_attribute(&mut self, attribute: MethodAttribute) {
        self.attributes.push(attribute);
    }

    /// Add a parameter to this method
    pub fn add_parameter(&mut self, parameter: MethodParameter) {
        self.parameters.push(parameter);
    }

    /// Set whether this method is async
    pub fn set_async(&mut self, is_async: bool) {
        self.is_async = is_async;
    }

    /// Check if this is a Unity event method
    pub fn is_unity_event(&self) -> bool {
        matches!(self.name.as_str(), 
            "start" | "update" | "fixed_update" | "late_update" |
            "on_enable" | "on_disable" | "on_destroy" |
            "on_trigger_enter" | "on_trigger_exit" | "on_trigger_stay" |
            "on_collision_enter" | "on_collision_exit" | "on_collision_stay" |
            "on_player_joined" | "on_player_left" |
            "on_pickup" | "on_drop" | "on_pickup_use_down" | "on_pickup_use_up" |
            "on_station_entered" | "on_station_exited" |
            "on_post_deserialization"
        )
    }

    /// Check if this is a custom event handler
    pub fn is_custom_event(&self) -> bool {
        self.attributes.iter().any(|a| matches!(a, MethodAttribute::UdonEvent(_)))
    }

    /// Validate the method definition
    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_csharp_identifier(&self.name) {
            return Err(format!("Invalid method name '{}': must be a valid C# identifier", self.name));
        }
        Ok(())
    }
}

/// Method parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: RustType,
    /// Whether this parameter is mutable
    pub is_mut: bool,
}

impl MethodParameter {
    /// Create a new method parameter
    pub fn new(name: String, param_type: RustType) -> Self {
        Self {
            name,
            param_type,
            is_mut: false,
        }
    }

    /// Set whether this parameter is mutable
    pub fn set_mut(&mut self, is_mut: bool) {
        self.is_mut = is_mut;
    }
}

/// Represents Rust types that can be mapped to C#
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RustType {
    // Primitive types
    Bool,
    I8, I16, I32, I64, I128,
    U8, U16, U32, U64, U128,
    F32, F64,
    Char,
    String,
    
    // Unity types
    Vector2, Vector3, Vector4,
    Quaternion,
    Color, Color32,
    GameObject,
    Transform,
    
    // VRChat types
    VRCPlayerApi,
    
    // Container types
    Option(Box<RustType>),
    Vec(Box<RustType>),
    HashMap(Box<RustType>, Box<RustType>),
    Array(Box<RustType>, usize),
    
    // Custom types
    Custom(String),
    
    // Unit type (void)
    Unit,
}

impl RustType {
    /// Check if this type is supported by UdonSharp
    pub fn is_udonsharp_compatible(&self) -> bool {
        match self {
            RustType::Bool | RustType::I32 | RustType::F32 | RustType::String => true,
            RustType::Vector2 | RustType::Vector3 | RustType::Vector4 => true,
            RustType::Quaternion | RustType::Color | RustType::Color32 => true,
            RustType::GameObject | RustType::Transform => true,
            RustType::VRCPlayerApi => true,
            RustType::Option(inner) => inner.is_udonsharp_compatible(),
            RustType::Vec(inner) => inner.is_udonsharp_compatible(),
            RustType::Array(inner, _) => inner.is_udonsharp_compatible(),
            RustType::Unit => true,
            _ => false,
        }
    }

    /// Get suggested UdonSharp-compatible alternatives for unsupported types
    pub fn get_alternatives(&self) -> Vec<String> {
        match self {
            RustType::I8 | RustType::I16 | RustType::I64 | RustType::I128 => vec!["i32".to_string()],
            RustType::U8 | RustType::U16 | RustType::U32 | RustType::U64 | RustType::U128 => vec!["i32".to_string()],
            RustType::F64 => vec!["f32".to_string()],
            RustType::Char => vec!["String".to_string()],
            RustType::HashMap(_, _) => vec!["Vec<T>".to_string(), "Array".to_string()],
            _ => Vec::new(),
        }
    }
}

/// Field visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Visibility {
    Private,
    Public,
}

/// Attributes that can be applied to struct fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FieldAttribute {
    /// #[udon_public] - makes field public and serializable
    UdonPublic,
    /// #[udon_sync] - makes field synchronized across network
    UdonSync,
    /// #[header("text")] - adds header in Unity inspector
    Header(String),
    /// #[tooltip("text")] - adds tooltip in Unity inspector
    Tooltip(String),
}

/// Attributes that can be applied to struct methods
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MethodAttribute {
    /// #[udon_event("EventName")] - creates custom event handler
    UdonEvent(String),
}

/// Attributes that can be applied to structs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StructAttribute {
    /// #[udon_sync_mode(Manual)] - sets synchronization mode
    UdonSyncMode(UdonSyncMode),
}

/// UdonSharp synchronization modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UdonSyncMode {
    None,
    Manual,
    Continuous,
}

/// UdonBehaviour trait implementation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonBehaviourTraitImpl {
    /// Methods implemented from the trait
    pub implemented_methods: Vec<String>,
    /// Whether all required methods are implemented
    pub is_complete: bool,
}

impl UdonBehaviourTraitImpl {
    /// Create a new trait implementation
    pub fn new() -> Self {
        Self {
            implemented_methods: Vec::new(),
            is_complete: false,
        }
    }

    /// Add an implemented method
    pub fn add_method(&mut self, method_name: String) {
        if !self.implemented_methods.contains(&method_name) {
            self.implemented_methods.push(method_name);
        }
    }

    /// Check if implementation is complete
    pub fn check_completeness(&mut self) {
        // At minimum, we need start() method
        self.is_complete = self.implemented_methods.contains(&"start".to_string());
    }

    /// Get missing required methods
    pub fn get_missing_methods(&self) -> Vec<String> {
        let required = vec!["start".to_string()];
        required.into_iter()
            .filter(|method| !self.implemented_methods.contains(method))
            .collect()
    }
}

/// Maps Rust types to their C# equivalents
#[derive(Debug, Clone)]
pub struct RustToCSharpTypeMapper {
    /// Basic type mappings
    basic_mappings: HashMap<RustType, String>,
    /// Unity-specific type mappings
    unity_mappings: HashMap<RustType, String>,
    /// VRChat-specific type mappings
    vrchat_mappings: HashMap<RustType, String>,
}

impl RustToCSharpTypeMapper {
    /// Create a new type mapper with default mappings
    pub fn new() -> Self {
        let mut mapper = Self {
            basic_mappings: HashMap::new(),
            unity_mappings: HashMap::new(),
            vrchat_mappings: HashMap::new(),
        };
        
        mapper.initialize_mappings();
        mapper
    }

    /// Initialize all type mappings
    fn initialize_mappings(&mut self) {
        // Basic type mappings
        self.basic_mappings.insert(RustType::Bool, "bool".to_string());
        self.basic_mappings.insert(RustType::I8, "sbyte".to_string());
        self.basic_mappings.insert(RustType::I16, "short".to_string());
        self.basic_mappings.insert(RustType::I32, "int".to_string());
        self.basic_mappings.insert(RustType::I64, "long".to_string());
        self.basic_mappings.insert(RustType::U8, "byte".to_string());
        self.basic_mappings.insert(RustType::U16, "ushort".to_string());
        self.basic_mappings.insert(RustType::U32, "uint".to_string());
        self.basic_mappings.insert(RustType::U64, "ulong".to_string());
        self.basic_mappings.insert(RustType::F32, "float".to_string());
        self.basic_mappings.insert(RustType::F64, "double".to_string());
        self.basic_mappings.insert(RustType::Char, "char".to_string());
        self.basic_mappings.insert(RustType::String, "string".to_string());
        self.basic_mappings.insert(RustType::Unit, "void".to_string());

        // Unity type mappings
        self.unity_mappings.insert(RustType::Vector2, "Vector2".to_string());
        self.unity_mappings.insert(RustType::Vector3, "Vector3".to_string());
        self.unity_mappings.insert(RustType::Vector4, "Vector4".to_string());
        self.unity_mappings.insert(RustType::Quaternion, "Quaternion".to_string());
        self.unity_mappings.insert(RustType::Color, "Color".to_string());
        self.unity_mappings.insert(RustType::Color32, "Color32".to_string());
        self.unity_mappings.insert(RustType::GameObject, "GameObject".to_string());
        self.unity_mappings.insert(RustType::Transform, "Transform".to_string());

        // VRChat type mappings
        self.vrchat_mappings.insert(RustType::VRCPlayerApi, "VRCPlayerApi".to_string());
    }

    /// Map a Rust type to its C# equivalent
    pub fn map_type(&self, rust_type: &RustType) -> Result<String, String> {
        match rust_type {
            // Check basic mappings first
            t if self.basic_mappings.contains_key(t) => {
                Ok(self.basic_mappings[t].clone())
            },
            // Check Unity mappings
            t if self.unity_mappings.contains_key(t) => {
                Ok(self.unity_mappings[t].clone())
            },
            // Check VRChat mappings
            t if self.vrchat_mappings.contains_key(t) => {
                Ok(self.vrchat_mappings[t].clone())
            },
            // Handle container types
            RustType::Option(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(inner_type) // In C#, we'll handle nullability differently
            },
            RustType::Vec(inner) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("{}[]", inner_type))
            },
            RustType::Array(inner, size) => {
                let inner_type = self.map_type(inner)?;
                Ok(format!("{}[{}]", inner_type, size))
            },
            RustType::HashMap(key, value) => {
                let key_type = self.map_type(key)?;
                let value_type = self.map_type(value)?;
                Ok(format!("Dictionary<{}, {}>", key_type, value_type))
            },
            RustType::Custom(name) => {
                Ok(name.clone())
            },
            _ => {
                Err(format!("Unsupported type: {:?}", rust_type))
            }
        }
    }

    /// Check if a type requires special handling
    pub fn requires_special_handling(&self, rust_type: &RustType) -> bool {
        matches!(rust_type, 
            RustType::Option(_) | 
            RustType::Vec(_) | 
            RustType::HashMap(_, _) |
            RustType::Array(_, _)
        )
    }

    /// Get the default value for a C# type
    pub fn get_default_value(&self, rust_type: &RustType) -> String {
        match rust_type {
            RustType::Bool => "false".to_string(),
            RustType::I8 | RustType::I16 | RustType::I32 => "0".to_string(),
            RustType::I64 => "0L".to_string(),
            RustType::U8 | RustType::U16 | RustType::U32 => "0".to_string(),
            RustType::U64 => "0UL".to_string(),
            RustType::F32 => "0.0f".to_string(),
            RustType::F64 => "0.0".to_string(),
            RustType::Char => "'\\0'".to_string(),
            RustType::String => "\"\"".to_string(),
            RustType::Vector2 => "Vector2.zero".to_string(),
            RustType::Vector3 => "Vector3.zero".to_string(),
            RustType::Vector4 => "Vector4.zero".to_string(),
            RustType::Quaternion => "Quaternion.identity".to_string(),
            RustType::Color => "Color.white".to_string(),
            RustType::Color32 => "Color32.white".to_string(),
            RustType::GameObject => "null".to_string(),
            RustType::Transform => "null".to_string(),
            RustType::VRCPlayerApi => "null".to_string(),
            RustType::Option(_) => "null".to_string(),
            RustType::Vec(inner) => {
                let inner_type = self.map_type(inner).unwrap_or("object".to_string());
                format!("new {}[0]", inner_type)
            },
            RustType::Array(inner, size) => {
                let inner_type = self.map_type(inner).unwrap_or("object".to_string());
                format!("new {}[{}]", inner_type, size)
            },
            RustType::HashMap(key, value) => {
                let key_type = self.map_type(key).unwrap_or("object".to_string());
                let value_type = self.map_type(value).unwrap_or("object".to_string());
                format!("new Dictionary<{}, {}>()", key_type, value_type)
            },
            RustType::Custom(_) => "null".to_string(),
            _ => "null".to_string(),
        }
    }

    /// Check if a type is a Unity GameObject reference
    pub fn is_gameobject_reference(&self, rust_type: &RustType) -> bool {
        match rust_type {
            RustType::GameObject => true,
            RustType::Option(inner) => matches!(**inner, RustType::GameObject),
            _ => false,
        }
    }

    /// Check if a type is a Unity component reference
    pub fn is_unity_component(&self, rust_type: &RustType) -> bool {
        match rust_type {
            RustType::Transform => true,
            RustType::Option(inner) => matches!(**inner, RustType::Transform),
            _ => false,
        }
    }

    /// Check if a type is a VRChat-specific type
    pub fn is_vrchat_type(&self, rust_type: &RustType) -> bool {
        match rust_type {
            RustType::VRCPlayerApi => true,
            RustType::Option(inner) => matches!(**inner, RustType::VRCPlayerApi),
            _ => false,
        }
    }

    /// Get required using statements for a type
    pub fn get_required_usings(&self, rust_type: &RustType) -> Vec<String> {
        let mut usings = Vec::new();
        
        match rust_type {
            RustType::Vector2 | RustType::Vector3 | RustType::Vector4 | 
            RustType::Quaternion | RustType::Color | RustType::Color32 |
            RustType::GameObject | RustType::Transform => {
                usings.push("using UnityEngine;".to_string());
            },
            RustType::VRCPlayerApi => {
                usings.push("using VRC.SDKBase;".to_string());
            },
            RustType::HashMap(_, _) => {
                usings.push("using System.Collections.Generic;".to_string());
            },
            RustType::Option(inner) | RustType::Vec(inner) | RustType::Array(inner, _) => {
                usings.extend(self.get_required_usings(inner));
            },
            _ => {}
        }
        
        usings.sort();
        usings.dedup();
        usings
    }

    /// Convert a Rust type to a UdonSharp-compatible type with warnings
    pub fn map_type_with_warnings(&self, rust_type: &RustType) -> (Result<String, String>, Vec<String>) {
        let mut warnings = Vec::new();
        
        let result = match rust_type {
            // Types that need warnings about UdonSharp limitations
            RustType::I64 | RustType::U64 => {
                warnings.push("64-bit integers have limited support in UdonSharp. Consider using int (32-bit) instead.".to_string());
                self.map_type(rust_type)
            },
            RustType::F64 => {
                warnings.push("Double precision floats have limited support in UdonSharp. Consider using float instead.".to_string());
                self.map_type(rust_type)
            },
            RustType::HashMap(_, _) => {
                warnings.push("Dictionary types have limited support in UdonSharp. Consider using arrays instead.".to_string());
                self.map_type(rust_type)
            },
            RustType::I128 | RustType::U128 => {
                warnings.push("128-bit integers are not supported in UdonSharp. Use int instead.".to_string());
                Err("Unsupported 128-bit integer type".to_string())
            },
            _ => self.map_type(rust_type)
        };
        
        (result, warnings)
    }
}

impl Default for RustToCSharpTypeMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps Rust attributes to their C# equivalents
#[derive(Debug, Clone)]
pub struct AttributeMapper {
    /// Field attribute mappings
    field_mappings: HashMap<FieldAttribute, Vec<String>>,
    /// Class attribute mappings  
    class_mappings: HashMap<StructAttribute, Vec<String>>,
}

impl AttributeMapper {
    /// Create a new attribute mapper with default mappings
    pub fn new() -> Self {
        let mut mapper = Self {
            field_mappings: HashMap::new(),
            class_mappings: HashMap::new(),
        };
        
        mapper.initialize_mappings();
        mapper
    }

    /// Initialize all attribute mappings
    fn initialize_mappings(&mut self) {
        // Field attribute mappings
        self.field_mappings.insert(
            FieldAttribute::UdonPublic, 
            vec!["[SerializeField]".to_string()]
        );
        self.field_mappings.insert(
            FieldAttribute::UdonSync, 
            vec!["[UdonSynced]".to_string()]
        );

        // Class attribute mappings
        self.class_mappings.insert(
            StructAttribute::UdonSyncMode(UdonSyncMode::Manual),
            vec!["[UdonSyncMode(BehaviourSyncMode.Manual)]".to_string()]
        );
        self.class_mappings.insert(
            StructAttribute::UdonSyncMode(UdonSyncMode::Continuous),
            vec!["[UdonSyncMode(BehaviourSyncMode.Continuous)]".to_string()]
        );
        self.class_mappings.insert(
            StructAttribute::UdonSyncMode(UdonSyncMode::None),
            vec!["[UdonSyncMode(BehaviourSyncMode.None)]".to_string()]
        );
    }

    /// Map a field attribute to C# attributes
    pub fn map_field_attribute(&self, attribute: &FieldAttribute) -> Vec<String> {
        match attribute {
            FieldAttribute::Header(text) => {
                vec![format!("[Header(\"{}\")]", text)]
            },
            FieldAttribute::Tooltip(text) => {
                vec![format!("[Tooltip(\"{}\")]", text)]
            },
            attr if self.field_mappings.contains_key(attr) => {
                self.field_mappings[attr].clone()
            },
            _ => Vec::new(),
        }
    }

    /// Map a class attribute to C# attributes
    pub fn map_class_attribute(&self, attribute: &StructAttribute) -> Vec<String> {
        self.class_mappings.get(attribute).cloned().unwrap_or_default()
    }

    /// Generate C# field visibility modifier
    pub fn map_field_visibility(&self, field: &StructField) -> String {
        if field.is_public() {
            "public".to_string()
        } else {
            "private".to_string()
        }
    }

    /// Generate complete C# field declaration with attributes
    pub fn generate_field_declaration(&self, field: &StructField, type_mapper: &RustToCSharpTypeMapper) -> Result<String, String> {
        let mut lines = Vec::new();
        
        // Add all field attributes
        for attribute in &field.attributes {
            let attrs = self.map_field_attribute(attribute);
            lines.extend(attrs);
        }
        
        // Generate the field declaration
        let visibility = self.map_field_visibility(field);
        let csharp_type = type_mapper.map_type(&field.field_type)?;
        let field_name = to_camel_case(&field.name);
        
        let mut field_line = format!("{} {} {}", visibility, csharp_type, field_name);
        
        // Add default value if specified
        if let Some(default) = &field.default_value {
            field_line.push_str(&format!(" = {}", default));
        } else if field.is_public() {
            // For public fields, add default value to ensure proper initialization
            let default_value = type_mapper.get_default_value(&field.field_type);
            if default_value != "null" {
                field_line.push_str(&format!(" = {}", default_value));
            }
        }
        
        field_line.push(';');
        lines.push(field_line);
        
        Ok(lines.join("\n    "))
    }

    /// Generate C# class attributes
    pub fn generate_class_attributes(&self, attributes: &[StructAttribute]) -> Vec<String> {
        let mut result = Vec::new();
        
        for attribute in attributes {
            let attrs = self.map_class_attribute(attribute);
            result.extend(attrs);
        }
        
        result
    }

    /// Check if field attributes are compatible
    pub fn validate_field_attributes(&self, field: &StructField) -> Result<(), String> {
        let has_public = field.attributes.iter().any(|a| matches!(a, FieldAttribute::UdonPublic));
        let has_sync = field.attributes.iter().any(|a| matches!(a, FieldAttribute::UdonSync));
        
        // UdonSync fields should typically be public for networking to work properly
        if has_sync && !has_public && !matches!(field.visibility, Visibility::Public) {
            return Err(format!(
                "Field '{}' has #[udon_sync] but is not public. Synchronized fields should be public for proper networking.",
                field.name
            ));
        }
        
        Ok(())
    }

    /// Get required using statements for attributes
    pub fn get_required_usings_for_attributes(&self, field_attributes: &[FieldAttribute], class_attributes: &[StructAttribute]) -> Vec<String> {
        let mut usings = Vec::new();
        
        // Check if we need UdonSharp usings
        let needs_udonsharp = field_attributes.iter().any(|a| matches!(a, FieldAttribute::UdonSync | FieldAttribute::UdonPublic)) ||
                             class_attributes.iter().any(|a| matches!(a, StructAttribute::UdonSyncMode(_)));
        
        if needs_udonsharp {
            usings.push("using UdonSharp;".to_string());
            usings.push("using VRC.Udon;".to_string());
        }
        
        // Check if we need Unity usings for SerializeField
        let needs_unity = field_attributes.iter().any(|a| matches!(a, FieldAttribute::UdonPublic | FieldAttribute::Header(_) | FieldAttribute::Tooltip(_)));
        
        if needs_unity {
            usings.push("using UnityEngine;".to_string());
        }
        
        usings.sort();
        usings.dedup();
        usings
    }

    /// Generate method attributes for custom events
    pub fn generate_method_attributes(&self, method: &StructMethod) -> Vec<String> {
        let mut attributes = Vec::new();
        
        for attr in &method.attributes {
            match attr {
                MethodAttribute::UdonEvent(event_name) => {
                    // Custom event methods should be public
                    attributes.push(format!("// Custom event handler for '{}'", event_name));
                }
            }
        }
        
        attributes
    }
}

impl Default for AttributeMapper {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to check if a string is a valid C# identifier
pub fn is_valid_csharp_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // First character must be letter or underscore
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    // Remaining characters must be alphanumeric or underscore
    name.chars().skip(1).all(|c| c.is_alphanumeric() || c == '_')
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udon_behaviour_struct_creation() {
        let mut behavior = UdonBehaviourStruct::new("TestBehavior".to_string());
        
        let field = StructField::new("test_field".to_string(), RustType::I32);
        behavior.add_field(field);
        
        assert_eq!(behavior.name, "TestBehavior");
        assert_eq!(behavior.fields.len(), 1);
        assert_eq!(behavior.fields[0].name, "test_field");
    }

    #[test]
    fn test_type_mapper() {
        let mapper = RustToCSharpTypeMapper::new();
        
        assert_eq!(mapper.map_type(&RustType::I32).unwrap(), "int");
        assert_eq!(mapper.map_type(&RustType::F32).unwrap(), "float");
        assert_eq!(mapper.map_type(&RustType::String).unwrap(), "string");
        assert_eq!(mapper.map_type(&RustType::Vector3).unwrap(), "Vector3");
    }

    #[test]
    fn test_attribute_mapper() {
        let mapper = AttributeMapper::new();
        
        let public_attrs = mapper.map_field_attribute(&FieldAttribute::UdonPublic);
        assert!(public_attrs.contains(&"[SerializeField]".to_string()));
        
        let sync_attrs = mapper.map_field_attribute(&FieldAttribute::UdonSync);
        assert!(sync_attrs.contains(&"[UdonSynced]".to_string()));
    }

    #[test]
    fn test_valid_csharp_identifier() {
        assert!(is_valid_csharp_identifier("ValidName"));
        assert!(is_valid_csharp_identifier("_validName"));
        assert!(is_valid_csharp_identifier("valid123"));
        
        assert!(!is_valid_csharp_identifier("123invalid"));
        assert!(!is_valid_csharp_identifier("invalid-name"));
        assert!(!is_valid_csharp_identifier(""));
    }

    #[test]
    fn test_struct_validation() {
        let mut behavior = UdonBehaviourStruct::new("TestBehavior".to_string());
        
        // Should fail without trait implementation
        assert!(behavior.validate().is_err());
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        behavior.set_trait_impl(trait_impl);
        
        // Should pass now
        assert!(behavior.validate().is_ok());
    }

    #[test]
    fn test_complex_type_mapping() {
        let mapper = RustToCSharpTypeMapper::new();
        
        // Test Option types
        let option_int = RustType::Option(Box::new(RustType::I32));
        assert_eq!(mapper.map_type(&option_int).unwrap(), "int");
        
        // Test Vec types
        let vec_string = RustType::Vec(Box::new(RustType::String));
        assert_eq!(mapper.map_type(&vec_string).unwrap(), "string[]");
        
        // Test HashMap types
        let hashmap = RustType::HashMap(Box::new(RustType::String), Box::new(RustType::I32));
        assert_eq!(mapper.map_type(&hashmap).unwrap(), "Dictionary<string, int>");
    }

    #[test]
    fn test_attribute_validation() {
        let mapper = AttributeMapper::new();
        
        // Test sync field without public attribute
        let mut field = StructField::new("sync_field".to_string(), RustType::I32);
        field.add_attribute(FieldAttribute::UdonSync);
        
        // Should fail validation
        assert!(mapper.validate_field_attributes(&field).is_err());
        
        // Add public attribute
        field.add_attribute(FieldAttribute::UdonPublic);
        
        // Should pass now
        assert!(mapper.validate_field_attributes(&field).is_ok());
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_camel_case("test_field"), "testField");
        assert_eq!(to_camel_case("player_count"), "playerCount");
        assert_eq!(to_pascal_case("test_method"), "TestMethod");
        assert_eq!(to_pascal_case("on_player_joined"), "OnPlayerJoined");
    }

    #[test]
    fn test_field_declaration_generation() {
        let mapper = AttributeMapper::new();
        let type_mapper = RustToCSharpTypeMapper::new();
        
        let mut field = StructField::new("player_count".to_string(), RustType::I32);
        field.add_attribute(FieldAttribute::UdonPublic);
        field.add_attribute(FieldAttribute::UdonSync);
        
        let declaration = mapper.generate_field_declaration(&field, &type_mapper).unwrap();
        
        assert!(declaration.contains("[SerializeField]"));
        assert!(declaration.contains("[UdonSynced]"));
        assert!(declaration.contains("public int playerCount"));
    }
}