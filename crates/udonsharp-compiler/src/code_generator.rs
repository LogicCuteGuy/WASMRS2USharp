//! Code generation engine for creating UdonSharp C# classes from Rust structs
//! 
//! This module provides the core functionality to generate UdonSharp-compatible
//! C# classes from analyzed UdonBehaviour structs with proper inheritance,
//! field attributes, and Unity integration.

use crate::multi_behavior::{
    UdonBehaviourStruct, StructField, StructMethod, StructAttribute,
    FieldAttribute, MethodAttribute, RustType,
    UdonBehaviourTraitImpl, RustToCSharpTypeMapper, AttributeMapper,
    is_valid_csharp_identifier
};
use std::collections::{HashMap, HashSet};

/// Result type for code generation operations
pub type GenerationResult<T> = Result<T, GenerationError>;

/// Errors that can occur during code generation
#[derive(Debug, Clone)]
pub enum GenerationError {
    /// Invalid class name
    InvalidClassName { name: String, reason: String },
    /// Type mapping failed
    TypeMappingError { rust_type: String, reason: String },
    /// Template rendering failed
    TemplateError { message: String },
    /// Missing required data
    MissingData { item: String, reason: String },
    /// Invalid method signature
    InvalidMethodSignature { method_name: String, reason: String },
    /// Attribute validation failed
    AttributeValidationError { attribute: String, reason: String },
}

impl std::fmt::Display for GenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenerationError::InvalidClassName { name, reason } => {
                write!(f, "Invalid class name '{}': {}", name, reason)
            }
            GenerationError::TypeMappingError { rust_type, reason } => {
                write!(f, "Type mapping error for '{}': {}", rust_type, reason)
            }
            GenerationError::TemplateError { message } => {
                write!(f, "Template error: {}", message)
            }
            GenerationError::MissingData { item, reason } => {
                write!(f, "Missing data for '{}': {}", item, reason)
            }
            GenerationError::InvalidMethodSignature { method_name, reason } => {
                write!(f, "Invalid method signature for '{}': {}", method_name, reason)
            }
            GenerationError::AttributeValidationError { attribute, reason } => {
                write!(f, "Attribute validation error for '{}': {}", attribute, reason)
            }
        }
    }
}

impl std::error::Error for GenerationError {}

/// Represents a generated C# class
#[derive(Debug, Clone)]
pub struct GeneratedClass {
    /// Class name
    pub class_name: String,
    /// Namespace (optional)
    pub namespace: Option<String>,
    /// Using statements
    pub using_statements: Vec<String>,
    /// Class attributes
    pub class_attributes: Vec<String>,
    /// Generated fields
    pub fields: Vec<GeneratedField>,
    /// Generated methods
    pub methods: Vec<GeneratedMethod>,
    /// Custom event handlers
    pub custom_events: Vec<CustomEventHandler>,
    /// Complete C# source code
    pub source_code: String,
}

/// Represents a generated C# field
#[derive(Debug, Clone)]
pub struct GeneratedField {
    /// Field name in C#
    pub name: String,
    /// C# type
    pub field_type: String,
    /// Visibility modifier
    pub visibility: String,
    /// Field attributes
    pub attributes: Vec<String>,
    /// Default value
    pub default_value: Option<String>,
    /// Complete field declaration
    pub declaration: String,
}

/// Represents a generated C# method
#[derive(Debug, Clone)]
pub struct GeneratedMethod {
    /// Method name in C#
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Method parameters
    pub parameters: Vec<GeneratedParameter>,
    /// Method attributes
    pub attributes: Vec<String>,
    /// Method body
    pub body: String,
    /// Complete method declaration
    pub declaration: String,
}

/// Represents a generated method parameter
#[derive(Debug, Clone)]
pub struct GeneratedParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
}

/// Represents a custom event handler
#[derive(Debug, Clone)]
pub struct CustomEventHandler {
    /// Event name
    pub event_name: String,
    /// Handler method name
    pub method_name: String,
    /// Parameters
    pub parameters: Vec<GeneratedParameter>,
    /// Method body
    pub body: String,
    /// Complete method declaration
    pub declaration: String,
}

/// Main code generator for UdonSharp classes
pub struct CodeGenerator {
    /// Type mapper for Rust to C# conversion
    type_mapper: RustToCSharpTypeMapper,
    /// Attribute mapper for field and class attributes
    attribute_mapper: AttributeMapper,
    /// Template cache for performance
    template_cache: HashMap<String, String>,
    /// Generated classes cache
    generated_classes: HashMap<String, GeneratedClass>,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new() -> Self {
        Self {
            type_mapper: RustToCSharpTypeMapper::new(),
            attribute_mapper: AttributeMapper::new(),
            template_cache: HashMap::new(),
            generated_classes: HashMap::new(),
        }
    }

    /// Generate a complete UdonSharp C# class from a UdonBehaviour struct
    pub fn generate_behavior_class(&mut self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<GeneratedClass> {
        // Validate input
        self.validate_struct(udon_struct)?;

        // Generate class components
        let class_name = self.generate_class_name(&udon_struct.name)?;
        let using_statements = self.generate_using_statements(udon_struct)?;
        let class_attributes = self.generate_class_attributes(&udon_struct.attributes)?;
        let fields = self.generate_fields(&udon_struct.fields)?;
        let mut methods = self.generate_unity_event_methods(udon_struct)?;
        let custom_events = self.generate_custom_event_handlers(udon_struct)?;

        // Generate network synchronization methods if needed
        if udon_struct.has_networking() {
            let sync_setters = self.generate_sync_field_setters(udon_struct)?;
            methods.extend(sync_setters);

            let sync_helpers = self.generate_request_serialization_helpers(udon_struct)?;
            methods.extend(sync_helpers);

            // Add sync field change notification methods
            let sync_notifications = self.generate_sync_field_change_notifications(udon_struct)?;
            methods.extend(sync_notifications);

            // Add private field for serialization optimization
            let optimization_field = GeneratedField {
                name: "_pendingSerialization".to_string(),
                field_type: "bool".to_string(),
                visibility: "private".to_string(),
                attributes: Vec::new(),
                default_value: Some("false".to_string()),
                declaration: "    private bool _pendingSerialization = false;".to_string(),
            };
            // Note: We would need to modify the fields vector, but it's immutable here
            // This field will be added via the declaration in the optimization method
        }

        // Generate complete source code
        let source_code = self.generate_complete_class_source(
            &class_name,
            &using_statements,
            &class_attributes,
            &fields,
            &methods,
            &custom_events,
        )?;

        let generated_class = GeneratedClass {
            class_name: class_name.clone(),
            namespace: None, // UdonSharp classes typically don't use namespaces
            using_statements,
            class_attributes,
            fields,
            methods,
            custom_events,
            source_code,
        };

        // Cache the generated class
        self.generated_classes.insert(class_name, generated_class.clone());

        Ok(generated_class)
    }

    /// Validate the input struct
    fn validate_struct(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<()> {
        // Check if struct name is valid
        if !is_valid_csharp_identifier(&udon_struct.name) {
            return Err(GenerationError::InvalidClassName {
                name: udon_struct.name.clone(),
                reason: "must be a valid C# identifier".to_string(),
            });
        }

        // Check if trait implementation exists
        if udon_struct.trait_impl.is_none() {
            return Err(GenerationError::MissingData {
                item: "UdonBehaviour trait implementation".to_string(),
                reason: format!("struct '{}' must implement UdonBehaviour trait", udon_struct.name),
            });
        }

        Ok(())
    }

    /// Generate C# class name from Rust struct name
    pub fn generate_class_name(&self, struct_name: &str) -> GenerationResult<String> {
        // Ensure PascalCase for C# class names
        let class_name = to_pascal_case(struct_name);
        
        if !is_valid_csharp_identifier(&class_name) {
            return Err(GenerationError::InvalidClassName {
                name: class_name,
                reason: "generated class name is not a valid C# identifier".to_string(),
            });
        }

        Ok(class_name)
    }

    /// Generate using statements for the class
    fn generate_using_statements(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<String>> {
        let mut usings = HashSet::new();

        // Always include basic UdonSharp usings
        usings.insert("using UnityEngine;".to_string());
        usings.insert("using UdonSharp;".to_string());

        // Add usings based on field types
        for field in &udon_struct.fields {
            let field_usings = self.type_mapper.get_required_usings(&field.field_type);
            usings.extend(field_usings);
        }

        // Add usings based on attributes
        let attribute_usings = self.attribute_mapper.get_required_usings_for_attributes(
            &udon_struct.fields.iter().flat_map(|f| &f.attributes).cloned().collect::<Vec<_>>(),
            &udon_struct.attributes,
        );
        usings.extend(attribute_usings);

        // Check if we need VRC usings for networking
        if udon_struct.has_networking() {
            usings.insert("using VRC.Udon;".to_string());
            usings.insert("using VRC.SDKBase;".to_string());
        }

        // Check if we need VRC usings for custom events
        let has_custom_events = udon_struct.methods.iter().any(|m| m.is_custom_event());
        if has_custom_events {
            usings.insert("using VRC.Udon;".to_string());
            usings.insert("using VRC.SDKBase;".to_string());
        }

        // Check if we need additional Unity usings based on methods
        if let Some(trait_impl) = &udon_struct.trait_impl {
            for method_name in &trait_impl.implemented_methods {
                match method_name.as_str() {
                    "on_trigger_enter" | "on_trigger_exit" | "on_trigger_stay" |
                    "on_collision_enter" | "on_collision_exit" | "on_collision_stay" => {
                        usings.insert("using UnityEngine;".to_string()); // Already included but ensure it's there
                    }
                    "on_player_joined" | "on_player_left" => {
                        usings.insert("using VRC.SDKBase;".to_string());
                    }
                    _ => {}
                }
            }
        }

        // Convert to sorted vector
        let mut using_vec: Vec<String> = usings.into_iter().collect();
        using_vec.sort();

        Ok(using_vec)
    }

    /// Generate class attributes
    fn generate_class_attributes(&self, attributes: &[StructAttribute]) -> GenerationResult<Vec<String>> {
        let mut class_attrs = self.attribute_mapper.generate_class_attributes(attributes);
        
        // Always add UdonBehaviourSyncMode if not specified
        let has_sync_mode = attributes.iter().any(|a| matches!(a, StructAttribute::UdonSyncMode(_)));
        if !has_sync_mode {
            class_attrs.push("[UdonBehaviourSyncMode(BehaviourSyncMode.None)]".to_string());
        }

        Ok(class_attrs)
    }

    /// Generate all fields for the class
    fn generate_fields(&self, fields: &[StructField]) -> GenerationResult<Vec<GeneratedField>> {
        let mut generated_fields = Vec::new();

        for field in fields {
            let generated_field = self.generate_single_field(field)?;
            generated_fields.push(generated_field);
        }

        Ok(generated_fields)
    }

    /// Generate a single field
    pub fn generate_single_field(&self, field: &StructField) -> GenerationResult<GeneratedField> {
        // Validate field attributes
        self.attribute_mapper.validate_field_attributes(field)
            .map_err(|reason| GenerationError::AttributeValidationError {
                attribute: "field attributes".to_string(),
                reason,
            })?;

        // Map field type
        let csharp_type = self.type_mapper.map_type(&field.field_type)
            .map_err(|reason| GenerationError::TypeMappingError {
                rust_type: format!("{:?}", field.field_type),
                reason,
            })?;

        // Generate field name (convert to camelCase)
        let field_name = to_camel_case(&field.name);

        // Generate field attributes
        let mut attributes = Vec::new();
        for attr in &field.attributes {
            let attr_strings = self.attribute_mapper.map_field_attribute(attr);
            attributes.extend(attr_strings);
        }

        // Generate visibility
        let visibility = self.attribute_mapper.map_field_visibility(field);

        // Generate default value
        let default_value = field.default_value.clone()
            .or_else(|| {
                if field.is_public() {
                    Some(self.type_mapper.get_default_value(&field.field_type))
                } else {
                    None
                }
            });

        // Generate complete field declaration
        let declaration = self.generate_field_declaration(
            &attributes,
            &visibility,
            &csharp_type,
            &field_name,
            &default_value,
        );

        Ok(GeneratedField {
            name: field_name,
            field_type: csharp_type,
            visibility,
            attributes,
            default_value,
            declaration,
        })
    }

    /// Generate field declaration string
    fn generate_field_declaration(
        &self,
        attributes: &[String],
        visibility: &str,
        field_type: &str,
        field_name: &str,
        default_value: &Option<String>,
    ) -> String {
        let mut lines = Vec::new();

        // Add attributes
        for attr in attributes {
            lines.push(format!("    {}", attr));
        }

        // Generate field line
        let mut field_line = format!("    {} {} {}", visibility, field_type, field_name);
        
        if let Some(default) = default_value {
            field_line.push_str(&format!(" = {}", default));
        }
        
        field_line.push(';');
        lines.push(field_line);

        lines.join("\n")
    }

    /// Generate Unity event methods from trait implementation
    fn generate_unity_event_methods(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<GeneratedMethod>> {
        let mut methods = Vec::new();

        if let Some(trait_impl) = &udon_struct.trait_impl {
            for method_name in &trait_impl.implemented_methods {
                if let Some(unity_method) = self.generate_unity_event_method(method_name, udon_struct)? {
                    methods.push(unity_method);
                }
            }
        }

        Ok(methods)
    }

    /// Generate a single Unity event method
    fn generate_unity_event_method(&self, method_name: &str, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Option<GeneratedMethod>> {
        let csharp_method_name = self.map_unity_method_name(method_name);
        
        if let Some(csharp_name) = csharp_method_name {
            // Get method parameters and return type from the struct's trait implementation
            let (parameters, return_type) = self.get_unity_method_signature(method_name, udon_struct)?;
            
            let method_body = self.generate_unity_method_body(method_name, udon_struct);
            let declaration = self.generate_unity_method_declaration_with_params(&csharp_name, &parameters, &return_type, &method_body);

            Ok(Some(GeneratedMethod {
                name: csharp_name.clone(),
                return_type,
                parameters,
                attributes: Vec::new(),
                body: method_body,
                declaration,
            }))
        } else {
            Ok(None)
        }
    }

    /// Map Rust trait method names to Unity C# method names
    pub fn map_unity_method_name(&self, rust_method: &str) -> Option<String> {
        match rust_method {
            "start" => Some("Start".to_string()),
            "update" => Some("Update".to_string()),
            "fixed_update" => Some("FixedUpdate".to_string()),
            "late_update" => Some("LateUpdate".to_string()),
            "on_enable" => Some("OnEnable".to_string()),
            "on_disable" => Some("OnDisable".to_string()),
            "on_destroy" => Some("OnDestroy".to_string()),
            "on_trigger_enter" => Some("OnTriggerEnter".to_string()),
            "on_trigger_exit" => Some("OnTriggerExit".to_string()),
            "on_trigger_stay" => Some("OnTriggerStay".to_string()),
            "on_collision_enter" => Some("OnCollisionEnter".to_string()),
            "on_collision_exit" => Some("OnCollisionExit".to_string()),
            "on_collision_stay" => Some("OnCollisionStay".to_string()),
            "on_player_joined" => Some("OnPlayerJoined".to_string()),
            "on_player_left" => Some("OnPlayerLeft".to_string()),
            "on_pickup" => Some("OnPickup".to_string()),
            "on_drop" => Some("OnDrop".to_string()),
            "on_pickup_use_down" => Some("OnPickupUseDown".to_string()),
            "on_pickup_use_up" => Some("OnPickupUseUp".to_string()),
            "on_station_entered" => Some("OnStationEntered".to_string()),
            "on_station_exited" => Some("OnStationExited".to_string()),
            "on_post_deserialization" => Some("OnDeserialization".to_string()),
            _ => None, // Not a Unity event method
        }
    }

    /// Generate method body for Unity event methods
    fn generate_unity_method_body(&self, method_name: &str, udon_struct: &UdonBehaviourStruct) -> String {
        match method_name {
            "start" => {
                let mut body = vec![
                    "        // Initialize behavior".to_string(),
                ];

                // Add GameObject reference initialization if needed
                for field in &udon_struct.fields {
                    if self.type_mapper.is_gameobject_reference(&field.field_type) {
                        let field_name = to_camel_case(&field.name);
                        body.push(format!("        // Initialize {} GameObject reference", field_name));
                        body.push(format!("        if ({} == null)", field_name));
                        body.push(format!("        {{"));
                        body.push(format!("            {} = GameObject.Find(\"{}\");", field_name, field_name));
                        body.push(format!("        }}"));
                    }
                }

                // Add synchronized field initialization
                let sync_fields = udon_struct.get_sync_fields();
                if !sync_fields.is_empty() {
                    body.push("".to_string());
                    body.push("        // Initialize synchronized fields".to_string());
                    for field in sync_fields {
                        let field_name = to_camel_case(&field.name);
                        let default_value = self.type_mapper.get_default_value(&field.field_type);
                        if default_value != "null" {
                            body.push(format!("        if (Networking.IsMaster && {} == {})", field_name, default_value));
                            body.push(format!("        {{"));
                            body.push(format!("            // Initialize {} with default value", field_name));
                            body.push(format!("        }}"));
                        }
                    }
                }

                body.join("\n")
            }
            "update" => {
                "        // Update behavior every frame\n        // Add your update logic here".to_string()
            }
            "fixed_update" => {
                "        // Fixed update for physics calculations\n        // Add your physics update logic here".to_string()
            }
            "late_update" => {
                "        // Late update after all Update calls\n        // Add your late update logic here".to_string()
            }
            "on_enable" => {
                "        // Called when the behavior becomes enabled\n        // Add your enable logic here".to_string()
            }
            "on_disable" => {
                "        // Called when the behavior becomes disabled\n        // Add your disable logic here".to_string()
            }
            "on_destroy" => {
                "        // Called when the behavior is destroyed\n        // Add your cleanup logic here".to_string()
            }
            "on_post_deserialization" => {
                self.generate_on_deserialization_body(udon_struct)
            }
            "on_player_joined" => {
                "        // Handle player joined event\n        // Use the 'player' parameter to access VRCPlayerApi\n        // Add your player joined logic here".to_string()
            }
            "on_player_left" => {
                "        // Handle player left event\n        // Use the 'player' parameter to access VRCPlayerApi\n        // Add your player left logic here".to_string()
            }
            "on_trigger_enter" => {
                "        // Handle trigger enter event\n        // Use the 'other' parameter to access the Collider\n        // Add your trigger enter logic here".to_string()
            }
            "on_trigger_exit" => {
                "        // Handle trigger exit event\n        // Use the 'other' parameter to access the Collider\n        // Add your trigger exit logic here".to_string()
            }
            "on_trigger_stay" => {
                "        // Handle trigger stay event\n        // Use the 'other' parameter to access the Collider\n        // Add your trigger stay logic here".to_string()
            }
            "on_collision_enter" => {
                "        // Handle collision enter event\n        // Use the 'collision' parameter to access the Collision\n        // Add your collision enter logic here".to_string()
            }
            "on_collision_exit" => {
                "        // Handle collision exit event\n        // Use the 'collision' parameter to access the Collision\n        // Add your collision exit logic here".to_string()
            }
            "on_collision_stay" => {
                "        // Handle collision stay event\n        // Use the 'collision' parameter to access the Collision\n        // Add your collision stay logic here".to_string()
            }
            "on_pickup" => {
                "        // Handle pickup event\n        // Add your pickup logic here".to_string()
            }
            "on_drop" => {
                "        // Handle drop event\n        // Add your drop logic here".to_string()
            }
            "on_pickup_use_down" => {
                "        // Handle pickup use down event\n        // Add your pickup use down logic here".to_string()
            }
            "on_pickup_use_up" => {
                "        // Handle pickup use up event\n        // Add your pickup use up logic here".to_string()
            }
            "on_station_entered" => {
                "        // Handle station entered event\n        // Add your station entered logic here".to_string()
            }
            "on_station_exited" => {
                "        // Handle station exited event\n        // Add your station exited logic here".to_string()
            }
            _ => {
                format!("        // {} implementation\n        // Add your custom logic here", method_name)
            }
        }
    }

    /// Get Unity method signature (parameters and return type)
    fn get_unity_method_signature(&self, method_name: &str, udon_struct: &UdonBehaviourStruct) -> GenerationResult<(Vec<GeneratedParameter>, String)> {
        // Find the method in the struct's methods
        if let Some(struct_method) = udon_struct.methods.iter().find(|m| m.name == method_name) {
            let mut parameters = Vec::new();
            
            // Convert method parameters
            for param in &struct_method.parameters {
                let param_type = self.type_mapper.map_type(&param.param_type)
                    .map_err(|reason| GenerationError::TypeMappingError {
                        rust_type: format!("{:?}", param.param_type),
                        reason,
                    })?;
                
                parameters.push(GeneratedParameter {
                    name: to_camel_case(&param.name),
                    param_type,
                });
            }
            
            // Convert return type
            let return_type = if struct_method.return_type == RustType::Unit {
                "void".to_string()
            } else {
                self.type_mapper.map_type(&struct_method.return_type)
                    .map_err(|reason| GenerationError::TypeMappingError {
                        rust_type: format!("{:?}", struct_method.return_type),
                        reason,
                    })?
            };
            
            Ok((parameters, return_type))
        } else {
            // Default signature for Unity event methods
            let parameters = match method_name {
                "on_player_joined" | "on_player_left" => {
                    vec![GeneratedParameter {
                        name: "player".to_string(),
                        param_type: "VRCPlayerApi".to_string(),
                    }]
                }
                "on_trigger_enter" | "on_trigger_exit" | "on_trigger_stay" => {
                    vec![GeneratedParameter {
                        name: "other".to_string(),
                        param_type: "Collider".to_string(),
                    }]
                }
                "on_collision_enter" | "on_collision_exit" | "on_collision_stay" => {
                    vec![GeneratedParameter {
                        name: "collision".to_string(),
                        param_type: "Collision".to_string(),
                    }]
                }
                _ => Vec::new(),
            };
            
            Ok((parameters, "void".to_string()))
        }
    }

    /// Generate Unity method declaration with parameters
    fn generate_unity_method_declaration_with_params(&self, method_name: &str, parameters: &[GeneratedParameter], return_type: &str, body: &str) -> String {
        let param_list = parameters.iter()
            .map(|p| format!("{} {}", p.param_type, p.name))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "    public override {} {}({})\n    {{\n{}\n    }}",
            return_type,
            method_name,
            param_list,
            body
        )
    }

    /// Generate Unity method declaration (legacy method for backward compatibility)
    fn generate_unity_method_declaration(&self, method_name: &str, body: &str) -> String {
        self.generate_unity_method_declaration_with_params(method_name, &[], "void", body)
    }

    /// Generate custom event handlers
    fn generate_custom_event_handlers(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<CustomEventHandler>> {
        let mut handlers = Vec::new();

        for method in &udon_struct.methods {
            if method.is_custom_event() {
                let handler = self.generate_custom_event_handler(method)?;
                handlers.push(handler);
            }
        }

        // Generate helper methods for sending custom events to other behaviors
        if !handlers.is_empty() {
            let helper_methods = self.generate_custom_event_helper_methods(udon_struct)?;
            handlers.extend(helper_methods);
        }

        Ok(handlers)
    }

    /// Generate helper methods for sending custom events
    fn generate_custom_event_helper_methods(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<CustomEventHandler>> {
        let mut helpers = Vec::new();

        // Generate a helper method for sending events to other behaviors
        let send_event_body = self.generate_send_custom_event_helper_body();
        let send_event_declaration = format!(
            "    /// <summary>\n    /// Send a custom event to another UdonBehaviour\n    /// </summary>\n    /// <param name=\"target\">Target GameObject with UdonBehaviour</param>\n    /// <param name=\"eventName\">Name of the custom event</param>\n    public void SendCustomEventToTarget(GameObject target, string eventName)\n    {{\n{}\n    }}",
            send_event_body
        );

        helpers.push(CustomEventHandler {
            event_name: "SendCustomEventToTarget".to_string(),
            method_name: "SendCustomEventToTarget".to_string(),
            parameters: vec![
                GeneratedParameter {
                    name: "target".to_string(),
                    param_type: "GameObject".to_string(),
                },
                GeneratedParameter {
                    name: "eventName".to_string(),
                    param_type: "string".to_string(),
                },
            ],
            body: send_event_body,
            declaration: send_event_declaration,
        });

        Ok(helpers)
    }

    /// Generate body for SendCustomEvent helper method
    fn generate_send_custom_event_helper_body(&self) -> String {
        vec![
            "        // Validate target GameObject".to_string(),
            "        if (target == null)".to_string(),
            "        {".to_string(),
            "            Debug.LogError(\"SendCustomEventToTarget: target GameObject is null\");".to_string(),
            "            return;".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Validate event name".to_string(),
            "        if (string.IsNullOrEmpty(eventName))".to_string(),
            "        {".to_string(),
            "            Debug.LogError(\"SendCustomEventToTarget: eventName is null or empty\");".to_string(),
            "            return;".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Get UdonBehaviour component from target".to_string(),
            "        var targetBehaviour = target.GetComponent<UdonBehaviour>();".to_string(),
            "        if (targetBehaviour == null)".to_string(),
            "        {".to_string(),
            "            Debug.LogError($\"SendCustomEventToTarget: No UdonBehaviour found on {target.name}\");".to_string(),
            "            return;".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Send the custom event".to_string(),
            "        targetBehaviour.SendCustomEvent(eventName);".to_string(),
        ].join("\n")
    }

    /// Generate a single custom event handler
    fn generate_custom_event_handler(&self, method: &StructMethod) -> GenerationResult<CustomEventHandler> {
        // Extract event name from attributes
        let event_name = method.attributes.iter()
            .find_map(|attr| match attr {
                MethodAttribute::UdonEvent(name) => Some(name.clone()),
            })
            .ok_or_else(|| GenerationError::MissingData {
                item: "event name".to_string(),
                reason: "custom event method must have #[udon_event] attribute".to_string(),
            })?;

        let method_name = to_pascal_case(&method.name);
        
        // Generate parameters
        let mut parameters = Vec::new();
        for param in &method.parameters {
            let param_type = self.type_mapper.map_type(&param.param_type)
                .map_err(|reason| GenerationError::TypeMappingError {
                    rust_type: format!("{:?}", param.param_type),
                    reason,
                })?;
            
            parameters.push(GeneratedParameter {
                name: to_camel_case(&param.name),
                param_type,
            });
        }

        // Generate method body
        let body = self.generate_custom_event_body(&event_name, &parameters);

        // Generate complete declaration with UdonSharp compatibility
        let param_list = parameters.iter()
            .map(|p| format!("{} {}", p.param_type, p.name))
            .collect::<Vec<_>>()
            .join(", ");

        // Add method attributes for UdonSharp compatibility
        let mut method_lines = vec![
            format!("    [System.Serializable]"),
            format!("    public void {}({})", method_name, param_list),
            format!("    {{"),
        ];

        // Add body lines with proper indentation
        for line in body.lines() {
            method_lines.push(line.to_string());
        }

        method_lines.push("    }".to_string());

        let declaration = method_lines.join("\n");

        Ok(CustomEventHandler {
            event_name,
            method_name,
            parameters,
            body,
            declaration,
        })
    }

    /// Generate custom event method body
    fn generate_custom_event_body(&self, event_name: &str, parameters: &[GeneratedParameter]) -> String {
        let mut body = vec![
            format!("        // Custom event handler for '{}'", event_name),
        ];

        // Add parameter validation if there are parameters
        if !parameters.is_empty() {
            body.push("".to_string());
            body.push("        // Parameter validation".to_string());
            for param in parameters {
                match param.param_type.as_str() {
                    "GameObject" => {
                        body.push(format!("        if ({} == null)", param.name));
                        body.push("        {".to_string());
                        body.push(format!("            Debug.LogWarning(\"Custom event '{}': {} parameter is null\");", event_name, param.name));
                        body.push("            return;".to_string());
                        body.push("        }".to_string());
                    }
                    "VRCPlayerApi" => {
                        body.push(format!("        if (!Utilities.IsValid({}))", param.name));
                        body.push("        {".to_string());
                        body.push(format!("            Debug.LogWarning(\"Custom event '{}': {} parameter is not a valid player\");", event_name, param.name));
                        body.push("            return;".to_string());
                        body.push("        }".to_string());
                    }
                    "string" => {
                        body.push(format!("        if (string.IsNullOrEmpty({}))", param.name));
                        body.push("        {".to_string());
                        body.push(format!("            Debug.LogWarning(\"Custom event '{}': {} parameter is null or empty\");", event_name, param.name));
                        body.push("            return;".to_string());
                        body.push("        }".to_string());
                    }
                    _ => {
                        // For other types, add a comment about parameter availability
                        body.push(format!("        // {} {} is available for use", param.param_type, param.name));
                    }
                }
            }
        }

        body.push("".to_string());
        body.push("        // Add your custom event logic here".to_string());
        
        // Add example usage based on parameters
        if !parameters.is_empty() {
            body.push("".to_string());
            body.push("        // Example usage:".to_string());
            for param in parameters {
                match param.param_type.as_str() {
                    "GameObject" => {
                        body.push(format!("        // var component = {}.GetComponent<SomeComponent>();", param.name));
                    }
                    "VRCPlayerApi" => {
                        body.push(format!("        // string playerName = {}.displayName;", param.name));
                    }
                    "string" => {
                        body.push(format!("        // Debug.Log(\"Received message: \" + {});", param.name));
                    }
                    "int" => {
                        body.push(format!("        // Debug.Log(\"Received value: \" + {});", param.name));
                    }
                    "float" => {
                        body.push(format!("        // Debug.Log(\"Received value: \" + {});", param.name));
                    }
                    "bool" => {
                        body.push(format!("        // if ({}) {{ /* do something */ }}", param.name));
                    }
                    _ => {
                        body.push(format!("        // Process {} of type {}", param.name, param.param_type));
                    }
                }
            }
        }

        body.join("\n")
    }

    /// Generate synchronized field setter methods
    fn generate_sync_field_setters(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<GeneratedMethod>> {
        let mut setters = Vec::new();
        let sync_fields = udon_struct.get_sync_fields();

        for field in sync_fields {
            let field_name = to_camel_case(&field.name);
            let csharp_type = self.type_mapper.map_type(&field.field_type)
                .map_err(|reason| GenerationError::TypeMappingError {
                    rust_type: format!("{:?}", field.field_type),
                    reason,
                })?;

            let setter_name = format!("Set{}", to_pascal_case(&field.name));
            let parameter = GeneratedParameter {
                name: "value".to_string(),
                param_type: csharp_type.clone(),
            };

            let body = self.generate_sync_setter_body(&field_name, &csharp_type);
            let declaration = format!(
                "    /// <summary>\n    /// Set {} with network synchronization\n    /// </summary>\n    /// <param name=\"value\">New value to set</param>\n    public void {}({} value)\n    {{\n{}\n    }}",
                field_name, setter_name, csharp_type, body
            );

            setters.push(GeneratedMethod {
                name: setter_name,
                return_type: "void".to_string(),
                parameters: vec![parameter],
                attributes: Vec::new(),
                body,
                declaration,
            });
        }

        Ok(setters)
    }

    /// Generate body for synchronized field setter
    fn generate_sync_setter_body(&self, field_name: &str, field_type: &str) -> String {
        vec![
            "        // Only master client can modify synchronized fields".to_string(),
            "        if (!Networking.IsMaster)".to_string(),
            "        {".to_string(),
            format!("            Debug.LogWarning(\"Only master client can modify synchronized field: {}\");", field_name),
            "            return;".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Check if value actually changed to avoid unnecessary network traffic".to_string(),
            format!("        if ({}.Equals(value))", field_name),
            "        {".to_string(),
            "            return; // No change, skip serialization".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Update the field value".to_string(),
            format!("        {} = value;", field_name),
            "".to_string(),
            "        // Request network serialization with optimization".to_string(),
            "        RequestSerialization();".to_string(),
            "".to_string(),
            "        // Optional: Log the change for debugging".to_string(),
            format!("        Debug.Log($\"Synchronized field '{}' updated to: {{value}}\");", field_name),
        ].join("\n")
    }

    /// Generate RequestSerialization helper methods
    fn generate_request_serialization_helpers(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<GeneratedMethod>> {
        let mut helpers = Vec::new();

        if udon_struct.has_networking() {
            // Generate a helper method for safe RequestSerialization calls
            let body = vec![
                "        // Ensure only master client can request serialization".to_string(),
                "        if (Networking.IsMaster)".to_string(),
                "        {".to_string(),
                "            RequestSerialization();".to_string(),
                "        }".to_string(),
                "        else".to_string(),
                "        {".to_string(),
                "            Debug.LogWarning(\"Non-master client attempted to request serialization\");".to_string(),
                "        }".to_string(),
            ].join("\n");

            let declaration = format!(
                "    /// <summary>\n    /// Safely request network serialization with master client validation\n    /// </summary>\n    public void SafeRequestSerialization()\n    {{\n{}\n    }}",
                body
            );

            helpers.push(GeneratedMethod {
                name: "SafeRequestSerialization".to_string(),
                return_type: "void".to_string(),
                parameters: Vec::new(),
                attributes: Vec::new(),
                body,
                declaration,
            });

            // Generate batch update method for multiple sync fields
            let sync_fields = udon_struct.get_sync_fields();
            if sync_fields.len() > 1 {
                let batch_body = self.generate_batch_update_body(&sync_fields);
                let batch_declaration = format!(
                    "    /// <summary>\n    /// Update multiple synchronized fields in a single network operation\n    /// </summary>\n    public void BatchUpdateSyncFields()\n    {{\n{}\n    }}",
                    batch_body
                );

                helpers.push(GeneratedMethod {
                    name: "BatchUpdateSyncFields".to_string(),
                    return_type: "void".to_string(),
                    parameters: Vec::new(),
                    attributes: Vec::new(),
                    body: batch_body,
                    declaration: batch_declaration,
                });
            }

            // Generate network optimization helper
            let optimization_body = self.generate_network_optimization_body();
            let optimization_declaration = format!(
                "    /// <summary>\n    /// Optimize network synchronization by batching multiple field updates\n    /// </summary>\n    private bool _pendingSerialization = false;\n    \n    public void OptimizedRequestSerialization()\n    {{\n{}\n    }}",
                optimization_body
            );

            helpers.push(GeneratedMethod {
                name: "OptimizedRequestSerialization".to_string(),
                return_type: "void".to_string(),
                parameters: Vec::new(),
                attributes: Vec::new(),
                body: optimization_body,
                declaration: optimization_declaration,
            });

            // Add the serialization flag reset helper
            helpers.push(self.generate_serialization_flag_reset());
        }

        Ok(helpers)
    }

    /// Generate body for batch update method
    fn generate_batch_update_body(&self, sync_fields: &[&StructField]) -> String {
        let mut body = vec![
            "        // Only master client can modify synchronized fields".to_string(),
            "        if (!Networking.IsMaster)".to_string(),
            "        {".to_string(),
            "            Debug.LogWarning(\"Only master client can batch update synchronized fields\");".to_string(),
            "            return;".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Update all synchronized fields here".to_string(),
        ];

        for field in sync_fields {
            let field_name = to_camel_case(&field.name);
            body.push(format!("        // Update {} as needed", field_name));
        }

        body.push("".to_string());
        body.push("        // Request single network serialization for all changes".to_string());
        body.push("        RequestSerialization();".to_string());

        body.join("\n")
    }

    /// Generate body for network optimization method
    fn generate_network_optimization_body(&self) -> String {
        vec![
            "        // Only master client can request serialization".to_string(),
            "        if (!Networking.IsMaster)".to_string(),
            "        {".to_string(),
            "            Debug.LogWarning(\"Non-master client attempted to request serialization\");".to_string(),
            "            return;".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        // Prevent duplicate serialization requests in the same frame".to_string(),
            "        if (_pendingSerialization)".to_string(),
            "        {".to_string(),
            "            return; // Already pending serialization this frame".to_string(),
            "        }".to_string(),
            "".to_string(),
            "        _pendingSerialization = true;".to_string(),
            "        RequestSerialization();".to_string(),
            "".to_string(),
            "        // Reset flag in next frame".to_string(),
            "        SendCustomEventDelayedFrames(\"_ResetSerializationFlag\", 1);".to_string(),
        ].join("\n")
    }

    /// Generate helper method to reset serialization flag
    fn generate_serialization_flag_reset(&self) -> GeneratedMethod {
        let body = vec![
            "        _pendingSerialization = false;".to_string(),
        ].join("\n");

        let declaration = format!(
            "    /// <summary>\n    /// Internal method to reset serialization optimization flag\n    /// </summary>\n    public void _ResetSerializationFlag()\n    {{\n{}\n    }}",
            body
        );

        GeneratedMethod {
            name: "_ResetSerializationFlag".to_string(),
            return_type: "void".to_string(),
            parameters: Vec::new(),
            attributes: Vec::new(),
            body,
            declaration,
        }
    }

    /// Generate OnDeserialization method body
    fn generate_on_deserialization_body(&self, udon_struct: &UdonBehaviourStruct) -> String {
        let mut body = vec![
            "        // Handle incoming network data deserialization".to_string(),
        ];

        let sync_fields = udon_struct.get_sync_fields();
        if !sync_fields.is_empty() {
            body.push("".to_string());
            body.push("        // Process synchronized field updates".to_string());
            
            for field in &sync_fields {
                let field_name = to_camel_case(&field.name);
                let field_type = self.type_mapper.map_type(&field.field_type).unwrap_or("object".to_string());
                
                body.push(format!("        // Handle {} ({}) synchronization", field_name, field_type));
                body.push(format!("        OnSyncField{}Changed();", to_pascal_case(&field.name)));
            }

            body.push("".to_string());
            body.push("        // Notify other behaviors about data changes".to_string());
            body.push("        NotifyNetworkDataUpdated();".to_string());
            
            body.push("".to_string());
            body.push("        // Update UI elements if needed".to_string());
            body.push("        UpdateUIFromSyncData();".to_string());
        } else {
            body.push("".to_string());
            body.push("        // No synchronized fields to process".to_string());
            body.push("        Debug.Log(\"OnDeserialization called but no sync fields defined\");".to_string());
        }

        body.join("\n")
    }

    /// Generate sync field change notification methods
    fn generate_sync_field_change_notifications(&self, udon_struct: &UdonBehaviourStruct) -> GenerationResult<Vec<GeneratedMethod>> {
        let mut notifications = Vec::new();
        let sync_fields = udon_struct.get_sync_fields();

        for field in &sync_fields {
            let field_name = to_camel_case(&field.name);
            let method_name = format!("OnSyncField{}Changed", to_pascal_case(&field.name));
            
            let body = vec![
                format!("        // Handle {} field synchronization", field_name),
                format!("        Debug.Log($\"Synchronized field '{}' updated to: {{{}}}\");", field_name, field_name),
                "".to_string(),
                "        // Add custom logic here to respond to field changes".to_string(),
                "        // Example: Update UI, trigger animations, notify other systems".to_string(),
            ].join("\n");

            let declaration = format!(
                "    /// <summary>\n    /// Called when synchronized field {} is updated from network\n    /// </summary>\n    private void {}()\n    {{\n{}\n    }}",
                field_name, method_name, body
            );

            notifications.push(GeneratedMethod {
                name: method_name,
                return_type: "void".to_string(),
                parameters: Vec::new(),
                attributes: Vec::new(),
                body,
                declaration,
            });
        }

        // Add general notification methods
        if !sync_fields.is_empty() {
            let notify_body = vec![
                "        // Send custom events to other behaviors about network updates".to_string(),
                "        // Example: Find other behaviors and notify them".to_string(),
                "        // GameObject[] otherBehaviors = GameObject.FindGameObjectsWithTag(\"NetworkListener\");".to_string(),
                "        // foreach (GameObject obj in otherBehaviors)".to_string(),
                "        // {".to_string(),
                "        //     obj.SendCustomEvent(\"OnNetworkDataChanged\");".to_string(),
                "        // }".to_string(),
            ].join("\n");

            let notify_declaration = format!(
                "    /// <summary>\n    /// Notify other behaviors about network data updates\n    /// </summary>\n    private void NotifyNetworkDataUpdated()\n    {{\n{}\n    }}",
                notify_body
            );

            notifications.push(GeneratedMethod {
                name: "NotifyNetworkDataUpdated".to_string(),
                return_type: "void".to_string(),
                parameters: Vec::new(),
                attributes: Vec::new(),
                body: notify_body,
                declaration: notify_declaration,
            });

            let ui_body = vec![
                "        // Update UI elements based on synchronized data".to_string(),
                "        // Example: Update text displays, sliders, etc.".to_string(),
                "        // if (scoreText != null) scoreText.text = score.ToString();".to_string(),
                "        // if (healthBar != null) healthBar.value = health / maxHealth;".to_string(),
            ].join("\n");

            let ui_declaration = format!(
                "    /// <summary>\n    /// Update UI elements from synchronized data\n    /// </summary>\n    private void UpdateUIFromSyncData()\n    {{\n{}\n    }}",
                ui_body
            );

            notifications.push(GeneratedMethod {
                name: "UpdateUIFromSyncData".to_string(),
                return_type: "void".to_string(),
                parameters: Vec::new(),
                attributes: Vec::new(),
                body: ui_body,
                declaration: ui_declaration,
            });
        }

        Ok(notifications)
    }

    /// Generate complete C# class source code
    fn generate_complete_class_source(
        &self,
        class_name: &str,
        using_statements: &[String],
        class_attributes: &[String],
        fields: &[GeneratedField],
        methods: &[GeneratedMethod],
        custom_events: &[CustomEventHandler],
    ) -> GenerationResult<String> {
        let mut source = Vec::new();

        // Add using statements
        for using in using_statements {
            source.push(using.clone());
        }
        source.push("".to_string()); // Empty line after usings

        // Add class attributes
        for attr in class_attributes {
            source.push(attr.clone());
        }

        // Add class declaration
        source.push(format!("public class {} : UdonSharpBehaviour", class_name));
        source.push("{".to_string());

        // Add fields
        if !fields.is_empty() {
            source.push("    // Fields".to_string());
            for field in fields {
                source.push(field.declaration.clone());
                source.push("".to_string()); // Empty line after each field
            }
        }

        // Separate methods by type
        let (unity_methods, remaining_methods): (Vec<&GeneratedMethod>, Vec<&GeneratedMethod>) = methods.iter()
            .partition(|m| self.is_unity_event_method(&m.name));
        
        let (sync_methods, notification_methods): (Vec<&GeneratedMethod>, Vec<&GeneratedMethod>) = remaining_methods.iter()
            .partition(|m| self.is_sync_method(&m.name));

        // Add Unity event methods
        if !unity_methods.is_empty() {
            source.push("    // Unity Event Methods".to_string());
            for method in unity_methods {
                source.push(method.declaration.clone());
                source.push("".to_string()); // Empty line after each method
            }
        }

        // Add network synchronization methods
        if !sync_methods.is_empty() {
            source.push("    // Network Synchronization Methods".to_string());
            for method in sync_methods {
                source.push(method.declaration.clone());
                source.push("".to_string()); // Empty line after each method
            }
        }

        // Add notification and helper methods
        if !notification_methods.is_empty() {
            source.push("    // Network Event Handlers and Helpers".to_string());
            for method in notification_methods {
                source.push(method.declaration.clone());
                source.push("".to_string()); // Empty line after each method
            }
        }

        // Add custom event handlers
        if !custom_events.is_empty() {
            source.push("    // Custom Event Handlers".to_string());
            for handler in custom_events {
                source.push(handler.declaration.clone());
                source.push("".to_string()); // Empty line after each handler
            }
        }

        source.push("}".to_string());

        Ok(source.join("\n"))
    }

    /// Check if a method name is a Unity event method
    fn is_unity_event_method(&self, method_name: &str) -> bool {
        matches!(method_name, 
            "Start" | "Update" | "FixedUpdate" | "LateUpdate" |
            "OnEnable" | "OnDisable" | "OnDestroy" |
            "OnTriggerEnter" | "OnTriggerExit" | "OnTriggerStay" |
            "OnCollisionEnter" | "OnCollisionExit" | "OnCollisionStay" |
            "OnPlayerJoined" | "OnPlayerLeft" |
            "OnPickup" | "OnDrop" | "OnPickupUseDown" | "OnPickupUseUp" |
            "OnStationEntered" | "OnStationExited" |
            "OnDeserialization"
        )
    }

    /// Check if a method name is a synchronization-related method
    fn is_sync_method(&self, method_name: &str) -> bool {
        method_name.starts_with("Set") && method_name.len() > 3 ||
        method_name == "SafeRequestSerialization" ||
        method_name == "BatchUpdateSyncFields" ||
        method_name == "OptimizedRequestSerialization" ||
        method_name == "_ResetSerializationFlag"
    }

    /// Get generated class by name
    pub fn get_generated_class(&self, class_name: &str) -> Option<&GeneratedClass> {
        self.generated_classes.get(class_name)
    }

    /// Get all generated classes
    pub fn get_all_generated_classes(&self) -> &HashMap<String, GeneratedClass> {
        &self.generated_classes
    }

    /// Clear generated classes cache
    pub fn clear_cache(&mut self) {
        self.generated_classes.clear();
        self.template_cache.clear();
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert snake_case to camelCase
pub fn to_camel_case(snake_case: &str) -> String {
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
pub fn to_pascal_case(snake_case: &str) -> String {
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
    use crate::multi_behavior::*;

    #[test]
    fn test_code_generator_creation() {
        let generator = CodeGenerator::new();
        assert_eq!(generator.generated_classes.len(), 0);
    }

    #[test]
    fn test_class_name_generation() {
        let generator = CodeGenerator::new();
        
        assert_eq!(generator.generate_class_name("test_behavior").unwrap(), "TestBehavior");
        assert_eq!(generator.generate_class_name("player_manager").unwrap(), "PlayerManager");
        assert_eq!(generator.generate_class_name("UIController").unwrap(), "UIController");
    }

    #[test]
    fn test_unity_method_name_mapping() {
        let generator = CodeGenerator::new();
        
        assert_eq!(generator.map_unity_method_name("start"), Some("Start".to_string()));
        assert_eq!(generator.map_unity_method_name("update"), Some("Update".to_string()));
        assert_eq!(generator.map_unity_method_name("on_player_joined"), Some("OnPlayerJoined".to_string()));
        assert_eq!(generator.map_unity_method_name("custom_method"), None);
    }

    #[test]
    fn test_field_generation() {
        let generator = CodeGenerator::new();
        
        let mut field = StructField::new("player_count".to_string(), RustType::I32);
        field.add_attribute(FieldAttribute::UdonPublic);
        field.add_attribute(FieldAttribute::UdonSync);
        
        let generated = generator.generate_single_field(&field).unwrap();
        
        assert_eq!(generated.name, "playerCount");
        assert_eq!(generated.field_type, "int");
        assert_eq!(generated.visibility, "public");
        assert!(generated.attributes.contains(&"[SerializeField]".to_string()));
        assert!(generated.attributes.contains(&"[UdonSynced]".to_string()));
    }

    #[test]
    fn test_complete_class_generation() {
        let mut generator = CodeGenerator::new();
        
        // Create a test struct
        let mut test_struct = UdonBehaviourStruct::new("TestBehavior".to_string());
        
        // Add a field
        let mut field = StructField::new("player_count".to_string(), RustType::I32);
        field.add_attribute(FieldAttribute::UdonPublic);
        test_struct.add_field(field);
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        let generated = generator.generate_behavior_class(&test_struct).unwrap();
        
        assert_eq!(generated.class_name, "TestBehavior");
        assert!(generated.using_statements.contains(&"using UnityEngine;".to_string()));
        assert!(generated.using_statements.contains(&"using UdonSharp;".to_string()));
        assert_eq!(generated.fields.len(), 1);
        assert_eq!(generated.methods.len(), 1);
        assert!(generated.source_code.contains("public class TestBehavior : UdonSharpBehaviour"));
        assert!(generated.source_code.contains("public int playerCount"));
        assert!(generated.source_code.contains("public override void Start()"));
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_camel_case("test_field"), "testField");
        assert_eq!(to_camel_case("player_count"), "playerCount");
        assert_eq!(to_pascal_case("test_method"), "TestMethod");
        assert_eq!(to_pascal_case("on_player_joined"), "OnPlayerJoined");
    }
}