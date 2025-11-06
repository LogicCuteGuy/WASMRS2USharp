//! Inter-behavior communication system for UdonSharp multi-behavior patterns
//! 
//! This module provides functionality for managing communication between multiple
//! UdonBehaviour structs through GameObject references, custom events, and safe
//! parameter passing.

use crate::multi_behavior::{
    UdonBehaviourStruct, StructField, StructMethod, RustType, FieldAttribute,
    MethodAttribute, RustToCSharpTypeMapper, is_valid_csharp_identifier
};
use crate::code_generator::{GeneratedClass, GeneratedField, GeneratedMethod, GeneratedParameter, GenerationError, GenerationResult};
use std::collections::{HashMap, HashSet};

/// Result type for inter-behavior communication operations
pub type CommunicationResult<T> = Result<T, CommunicationError>;

/// Errors that can occur during inter-behavior communication setup
#[derive(Debug, Clone)]
pub enum CommunicationError {
    /// Invalid GameObject reference configuration
    InvalidGameObjectReference { behavior: String, field: String, reason: String },
    /// Custom event configuration error
    InvalidCustomEvent { behavior: String, event: String, reason: String },
    /// Parameter validation error
    ParameterValidationError { event: String, parameter: String, reason: String },
    /// Missing required component
    MissingComponent { component: String, reason: String },
}

impl std::fmt::Display for CommunicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommunicationError::InvalidGameObjectReference { behavior, field, reason } => {
                write!(f, "Invalid GameObject reference in behavior '{}', field '{}': {}", behavior, field, reason)
            }
            CommunicationError::InvalidCustomEvent { behavior, event, reason } => {
                write!(f, "Invalid custom event in behavior '{}', event '{}': {}", behavior, event, reason)
            }
            CommunicationError::ParameterValidationError { event, parameter, reason } => {
                write!(f, "Parameter validation error in event '{}', parameter '{}': {}", event, parameter, reason)
            }
            CommunicationError::MissingComponent { component, reason } => {
                write!(f, "Missing component '{}': {}", component, reason)
            }
        }
    }
}

impl std::error::Error for CommunicationError {}

/// Manager for GameObject references between behaviors
pub struct GameObjectReferenceManager {
    /// Type mapper for Rust to C# conversion
    type_mapper: RustToCSharpTypeMapper,
    /// Discovered GameObject references
    gameobject_references: HashMap<String, Vec<GameObjectReference>>,
    /// Reference validation rules
    validation_rules: Vec<Box<dyn GameObjectValidationRule>>,
}

impl GameObjectReferenceManager {
    /// Create a new GameObject reference manager
    pub fn new() -> Self {
        let mut manager = Self {
            type_mapper: RustToCSharpTypeMapper::new(),
            gameobject_references: HashMap::new(),
            validation_rules: Vec::new(),
        };
        
        manager.initialize_validation_rules();
        manager
    }

    /// Initialize default validation rules
    fn initialize_validation_rules(&mut self) {
        self.validation_rules.push(Box::new(NullReferenceValidationRule));
        self.validation_rules.push(Box::new(ComponentExistenceValidationRule));
        self.validation_rules.push(Box::new(CircularReferenceValidationRule));
    }

    /// Analyze GameObject references in a behavior
    pub fn analyze_gameobject_references(&mut self, behavior: &UdonBehaviourStruct) -> CommunicationResult<Vec<GameObjectReference>> {
        let mut references = Vec::new();

        for field in &behavior.fields {
            if let Some(reference) = self.extract_gameobject_reference(behavior, field)? {
                references.push(reference);
            }
        }

        // Store references for this behavior
        self.gameobject_references.insert(behavior.name.clone(), references.clone());

        Ok(references)
    }

    /// Extract GameObject reference from a field
    fn extract_gameobject_reference(&self, behavior: &UdonBehaviourStruct, field: &StructField) -> CommunicationResult<Option<GameObjectReference>> {
        if !self.type_mapper.is_gameobject_reference(&field.field_type) {
            return Ok(None);
        }

        let reference_type = self.determine_reference_type(field);
        let target_behavior = self.infer_target_behavior(field);
        let initialization_strategy = self.determine_initialization_strategy(field);

        let reference = GameObjectReference {
            source_behavior: behavior.name.clone(),
            field_name: field.name.clone(),
            field_type: field.field_type.clone(),
            reference_type,
            target_behavior,
            initialization_strategy,
            is_optional: matches!(field.field_type, RustType::Option(_)),
            validation_rules: self.get_applicable_validation_rules(field),
        };

        // Validate the reference
        self.validate_gameobject_reference(&reference)?;

        Ok(Some(reference))
    }

    /// Determine the type of GameObject reference
    fn determine_reference_type(&self, field: &StructField) -> GameObjectReferenceType {
        let field_name_lower = field.name.to_lowercase();

        if field_name_lower.contains("manager") || field_name_lower.contains("controller") {
            GameObjectReferenceType::Manager
        } else if field_name_lower.contains("ui") || field_name_lower.contains("interface") {
            GameObjectReferenceType::UI
        } else if field_name_lower.contains("player") {
            GameObjectReferenceType::Player
        } else if field_name_lower.contains("target") || field_name_lower.contains("destination") {
            GameObjectReferenceType::Target
        } else {
            GameObjectReferenceType::Generic
        }
    }

    /// Infer target behavior from field name or attributes
    fn infer_target_behavior(&self, field: &StructField) -> Option<String> {
        let field_name = &field.name;
        
        // Try to infer from field name patterns
        if field_name.ends_with("_manager") {
            let behavior_name = field_name.strip_suffix("_manager").unwrap();
            Some(format!("{}Manager", to_pascal_case(behavior_name)))
        } else if field_name.ends_with("_controller") {
            let behavior_name = field_name.strip_suffix("_controller").unwrap();
            Some(format!("{}Controller", to_pascal_case(behavior_name)))
        } else if field_name.starts_with("target_") {
            let behavior_name = field_name.strip_prefix("target_").unwrap();
            Some(to_pascal_case(behavior_name))
        } else {
            // Default to field name as behavior name
            Some(to_pascal_case(field_name))
        }
    }

    /// Determine initialization strategy for the reference
    fn determine_initialization_strategy(&self, field: &StructField) -> InitializationStrategy {
        // Check if field has specific attributes that indicate initialization strategy
        if field.attributes.iter().any(|attr| matches!(attr, FieldAttribute::UdonPublic)) {
            InitializationStrategy::Inspector
        } else {
            InitializationStrategy::FindByName
        }
    }

    /// Get applicable validation rules for a field
    fn get_applicable_validation_rules(&self, field: &StructField) -> Vec<String> {
        let mut rules = vec!["null_check".to_string()];
        
        if !matches!(field.field_type, RustType::Option(_)) {
            rules.push("required_reference".to_string());
        }
        
        rules.push("component_existence".to_string());
        rules
    }

    /// Validate a GameObject reference
    fn validate_gameobject_reference(&self, reference: &GameObjectReference) -> CommunicationResult<()> {
        for rule in &self.validation_rules {
            rule.validate(reference)?;
        }
        Ok(())
    }

    /// Generate GameObject reference fields for a behavior
    pub fn generate_gameobject_fields(&self, behavior: &UdonBehaviourStruct) -> CommunicationResult<Vec<GeneratedField>> {
        let mut fields = Vec::new();

        if let Some(references) = self.gameobject_references.get(&behavior.name) {
            for reference in references {
                let field = self.generate_gameobject_field(reference)?;
                fields.push(field);
            }
        }

        Ok(fields)
    }

    /// Generate a single GameObject reference field
    fn generate_gameobject_field(&self, reference: &GameObjectReference) -> CommunicationResult<GeneratedField> {
        let field_name = to_camel_case(&reference.field_name);
        let csharp_type = self.type_mapper.map_type(&reference.field_type)
            .map_err(|reason| CommunicationError::InvalidGameObjectReference {
                behavior: reference.source_behavior.clone(),
                field: reference.field_name.clone(),
                reason,
            })?;

        let mut attributes = Vec::new();
        let visibility = match reference.initialization_strategy {
            InitializationStrategy::Inspector => {
                attributes.push("[SerializeField]".to_string());
                attributes.push("[Tooltip(\"GameObject reference for inter-behavior communication\")]".to_string());
                "public".to_string()
            }
            InitializationStrategy::FindByName => {
                "private".to_string()
            }
        };

        let default_value = if reference.is_optional {
            Some("null".to_string())
        } else {
            None
        };

        let declaration = self.generate_gameobject_field_declaration(
            &attributes,
            &visibility,
            &csharp_type,
            &field_name,
            &default_value,
            reference,
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

    /// Generate GameObject field declaration with validation
    fn generate_gameobject_field_declaration(
        &self,
        attributes: &[String],
        visibility: &str,
        field_type: &str,
        field_name: &str,
        default_value: &Option<String>,
        reference: &GameObjectReference,
    ) -> String {
        let mut lines = Vec::new();

        // Add documentation comment
        lines.push(format!("    /// <summary>"));
        lines.push(format!("    /// GameObject reference to {} behavior", 
            reference.target_behavior.as_deref().unwrap_or("target")));
        lines.push(format!("    /// Initialization: {:?}", reference.initialization_strategy));
        lines.push(format!("    /// </summary>"));

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

    /// Generate GameObject initialization code for Start() method
    pub fn generate_gameobject_initialization(&self, behavior: &UdonBehaviourStruct) -> CommunicationResult<Vec<String>> {
        let mut initialization_code = Vec::new();

        if let Some(references) = self.gameobject_references.get(&behavior.name) {
            for reference in references {
                if matches!(reference.initialization_strategy, InitializationStrategy::FindByName) {
                    let code = self.generate_find_gameobject_code(reference)?;
                    initialization_code.extend(code);
                }
            }
        }

        Ok(initialization_code)
    }

    /// Generate GameObject.Find() code for a reference
    fn generate_find_gameobject_code(&self, reference: &GameObjectReference) -> CommunicationResult<Vec<String>> {
        let field_name = to_camel_case(&reference.field_name);
        let target_name = reference.target_behavior.as_deref().unwrap_or(&reference.field_name);
        
        let mut code = Vec::new();
        
        code.push(format!("        // Initialize {} GameObject reference", field_name));
        code.push(format!("        if ({} == null)", field_name));
        code.push("        {".to_string());
        code.push(format!("            {} = GameObject.Find(\"{}\");", field_name, target_name));
        
        if !reference.is_optional {
            code.push(format!("            if ({} == null)", field_name));
            code.push("            {".to_string());
            code.push(format!("                Debug.LogError($\"Failed to find required GameObject '{}' for behavior '{}'\");", target_name, reference.source_behavior));
            code.push("            }".to_string());
        }
        
        // Add component validation
        if let Some(target_behavior) = &reference.target_behavior {
            code.push(format!("            else"));
            code.push("            {".to_string());
            code.push(format!("                var targetBehaviour = {}.GetComponent<UdonBehaviour>();", field_name));
            code.push("                if (targetBehaviour == null)".to_string());
            code.push("                {".to_string());
            code.push(format!("                    Debug.LogWarning($\"GameObject '{{{}}}' does not have an UdonBehaviour component\", {}.name);", field_name, field_name));
            code.push("                }".to_string());
            code.push("            }".to_string());
        }
        
        code.push("        }".to_string());
        code.push("".to_string());

        Ok(code)
    }

    /// Get all GameObject references for a behavior
    pub fn get_gameobject_references(&self, behavior_name: &str) -> Option<&Vec<GameObjectReference>> {
        self.gameobject_references.get(behavior_name)
    }

    /// Validate all GameObject references
    pub fn validate_all_references(&self) -> CommunicationResult<Vec<String>> {
        let mut warnings = Vec::new();

        for (behavior_name, references) in &self.gameobject_references {
            for reference in references {
                match self.validate_gameobject_reference(reference) {
                    Ok(()) => {}
                    Err(e) => {
                        warnings.push(format!("Behavior '{}': {}", behavior_name, e));
                    }
                }
            }
        }

        Ok(warnings)
    }
}

impl Default for GameObjectReferenceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a GameObject reference between behaviors
#[derive(Debug, Clone)]
pub struct GameObjectReference {
    /// Source behavior that holds the reference
    pub source_behavior: String,
    /// Field name in the source behavior
    pub field_name: String,
    /// Rust type of the field
    pub field_type: RustType,
    /// Type of reference
    pub reference_type: GameObjectReferenceType,
    /// Target behavior name (if known)
    pub target_behavior: Option<String>,
    /// How the reference should be initialized
    pub initialization_strategy: InitializationStrategy,
    /// Whether the reference is optional
    pub is_optional: bool,
    /// Validation rules to apply
    pub validation_rules: Vec<String>,
}

/// Type of GameObject reference
#[derive(Debug, Clone, PartialEq)]
pub enum GameObjectReferenceType {
    /// Reference to a manager behavior
    Manager,
    /// Reference to a UI behavior
    UI,
    /// Reference to a player-related behavior
    Player,
    /// Reference to a target behavior
    Target,
    /// Generic GameObject reference
    Generic,
}

/// Strategy for initializing GameObject references
#[derive(Debug, Clone, PartialEq)]
pub enum InitializationStrategy {
    /// Set through Unity Inspector
    Inspector,
    /// Find by GameObject name
    FindByName,
}

/// Trait for GameObject reference validation rules
pub trait GameObjectValidationRule {
    /// Validate a GameObject reference
    fn validate(&self, reference: &GameObjectReference) -> CommunicationResult<()>;
    
    /// Get the name of this validation rule
    fn rule_name(&self) -> &'static str;
}

/// Validation rule for null reference checking
pub struct NullReferenceValidationRule;

impl GameObjectValidationRule for NullReferenceValidationRule {
    fn validate(&self, reference: &GameObjectReference) -> CommunicationResult<()> {
        if !reference.is_optional && matches!(reference.initialization_strategy, InitializationStrategy::FindByName) {
            if reference.target_behavior.is_none() {
                return Err(CommunicationError::InvalidGameObjectReference {
                    behavior: reference.source_behavior.clone(),
                    field: reference.field_name.clone(),
                    reason: "Required GameObject reference has no target behavior specified".to_string(),
                });
            }
        }
        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "null_reference_check"
    }
}

/// Validation rule for component existence
pub struct ComponentExistenceValidationRule;

impl GameObjectValidationRule for ComponentExistenceValidationRule {
    fn validate(&self, reference: &GameObjectReference) -> CommunicationResult<()> {
        // This rule would be enforced at runtime, but we can check for obvious issues
        if let Some(target_behavior) = &reference.target_behavior {
            if !is_valid_csharp_identifier(target_behavior) {
                return Err(CommunicationError::InvalidGameObjectReference {
                    behavior: reference.source_behavior.clone(),
                    field: reference.field_name.clone(),
                    reason: format!("Target behavior name '{}' is not a valid identifier", target_behavior),
                });
            }
        }
        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "component_existence_check"
    }
}

/// Validation rule for circular reference detection
pub struct CircularReferenceValidationRule;

impl GameObjectValidationRule for CircularReferenceValidationRule {
    fn validate(&self, reference: &GameObjectReference) -> CommunicationResult<()> {
        // Check for obvious self-references
        if let Some(target_behavior) = &reference.target_behavior {
            if target_behavior == &reference.source_behavior {
                return Err(CommunicationError::InvalidGameObjectReference {
                    behavior: reference.source_behavior.clone(),
                    field: reference.field_name.clone(),
                    reason: "GameObject reference creates a self-reference".to_string(),
                });
            }
        }
        Ok(())
    }

    fn rule_name(&self) -> &'static str {
        "circular_reference_check"
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::{StructField, FieldAttribute};

    fn create_test_behavior(name: &str) -> UdonBehaviourStruct {
        UdonBehaviourStruct::new(name.to_string())
    }

    fn create_gameobject_field(name: &str, is_optional: bool) -> StructField {
        let field_type = if is_optional {
            RustType::Option(Box::new(RustType::GameObject))
        } else {
            RustType::GameObject
        };
        
        StructField::new(name.to_string(), field_type)
    }

    #[test]
    fn test_gameobject_reference_manager_creation() {
        let manager = GameObjectReferenceManager::new();
        assert_eq!(manager.gameobject_references.len(), 0);
        assert_eq!(manager.validation_rules.len(), 3);
    }

    #[test]
    fn test_gameobject_reference_extraction() {
        let mut manager = GameObjectReferenceManager::new();
        let mut behavior = create_test_behavior("TestBehavior");
        
        let field = create_gameobject_field("player_manager", false);
        behavior.add_field(field);
        
        let references = manager.analyze_gameobject_references(&behavior).unwrap();
        
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].field_name, "player_manager");
        assert_eq!(references[0].target_behavior, Some("PlayerManager".to_string()));
        assert!(!references[0].is_optional);
    }

    #[test]
    fn test_optional_gameobject_reference() {
        let mut manager = GameObjectReferenceManager::new();
        let mut behavior = create_test_behavior("TestBehavior");
        
        let field = create_gameobject_field("optional_target", true);
        behavior.add_field(field);
        
        let references = manager.analyze_gameobject_references(&behavior).unwrap();
        
        assert_eq!(references.len(), 1);
        assert!(references[0].is_optional);
    }

    #[test]
    fn test_reference_type_determination() {
        let manager = GameObjectReferenceManager::new();
        
        let manager_field = create_gameobject_field("game_manager", false);
        let ui_field = create_gameobject_field("ui_controller", false);
        let player_field = create_gameobject_field("player_data", false);
        let target_field = create_gameobject_field("target_object", false);
        let generic_field = create_gameobject_field("some_object", false);
        
        assert_eq!(manager.determine_reference_type(&manager_field), GameObjectReferenceType::Manager);
        assert_eq!(manager.determine_reference_type(&ui_field), GameObjectReferenceType::UI);
        assert_eq!(manager.determine_reference_type(&player_field), GameObjectReferenceType::Player);
        assert_eq!(manager.determine_reference_type(&target_field), GameObjectReferenceType::Target);
        assert_eq!(manager.determine_reference_type(&generic_field), GameObjectReferenceType::Generic);
    }

    #[test]
    fn test_initialization_strategy() {
        let manager = GameObjectReferenceManager::new();
        
        let mut public_field = create_gameobject_field("public_reference", false);
        public_field.add_attribute(FieldAttribute::UdonPublic);
        
        let private_field = create_gameobject_field("private_reference", false);
        
        assert_eq!(manager.determine_initialization_strategy(&public_field), InitializationStrategy::Inspector);
        assert_eq!(manager.determine_initialization_strategy(&private_field), InitializationStrategy::FindByName);
    }

    #[test]
    fn test_gameobject_field_generation() {
        let mut manager = GameObjectReferenceManager::new();
        let mut behavior = create_test_behavior("TestBehavior");
        
        let field = create_gameobject_field("target_manager", false);
        behavior.add_field(field);
        
        let _references = manager.analyze_gameobject_references(&behavior).unwrap();
        let fields = manager.generate_gameobject_fields(&behavior).unwrap();
        
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].name, "targetManager");
        assert_eq!(fields[0].field_type, "GameObject");
        assert_eq!(fields[0].visibility, "private");
    }

    #[test]
    fn test_gameobject_initialization_code() {
        let mut manager = GameObjectReferenceManager::new();
        let mut behavior = create_test_behavior("TestBehavior");
        
        let field = create_gameobject_field("target_manager", false);
        behavior.add_field(field);
        
        let _references = manager.analyze_gameobject_references(&behavior).unwrap();
        let init_code = manager.generate_gameobject_initialization(&behavior).unwrap();
        
        assert!(!init_code.is_empty());
        assert!(init_code.iter().any(|line| line.contains("GameObject.Find")));
        assert!(init_code.iter().any(|line| line.contains("targetManager")));
    }

    #[test]
    fn test_validation_rules() {
        let null_rule = NullReferenceValidationRule;
        let component_rule = ComponentExistenceValidationRule;
        let circular_rule = CircularReferenceValidationRule;
        
        assert_eq!(null_rule.rule_name(), "null_reference_check");
        assert_eq!(component_rule.rule_name(), "component_existence_check");
        assert_eq!(circular_rule.rule_name(), "circular_reference_check");
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_camel_case("test_field"), "testField");
        assert_eq!(to_camel_case("player_manager"), "playerManager");
        assert_eq!(to_pascal_case("test_behavior"), "TestBehavior");
        assert_eq!(to_pascal_case("ui_controller"), "UiController");
    }
}

/// Manager for custom event routing between behaviors
pub struct CustomEventRouter {
    /// Type mapper for parameter conversion
    type_mapper: RustToCSharpTypeMapper,
    /// Discovered custom events
    custom_events: HashMap<String, Vec<CustomEventDefinition>>,
    /// Event routing mappings
    event_routes: HashMap<String, Vec<EventRoute>>,
    /// Parameter validation rules
    parameter_validators: Vec<Box<dyn ParameterValidator>>,
}

impl CustomEventRouter {
    /// Create a new custom event router
    pub fn new() -> Self {
        let mut router = Self {
            type_mapper: RustToCSharpTypeMapper::new(),
            custom_events: HashMap::new(),
            event_routes: HashMap::new(),
            parameter_validators: Vec::new(),
        };
        
        router.initialize_parameter_validators();
        router
    }

    /// Initialize parameter validation rules
    fn initialize_parameter_validators(&mut self) {
        self.parameter_validators.push(Box::new(TypeCompatibilityValidator));
        self.parameter_validators.push(Box::new(NullSafetyValidator));
        self.parameter_validators.push(Box::new(UdonSharpCompatibilityValidator));
    }

    /// Analyze custom events in a behavior
    pub fn analyze_custom_events(&mut self, behavior: &UdonBehaviourStruct) -> CommunicationResult<Vec<CustomEventDefinition>> {
        let mut events = Vec::new();

        for method in &behavior.methods {
            if let Some(event) = self.extract_custom_event(behavior, method)? {
                events.push(event);
            }
        }

        // Store events for this behavior
        self.custom_events.insert(behavior.name.clone(), events.clone());

        Ok(events)
    }

    /// Extract custom event from a method
    fn extract_custom_event(&self, behavior: &UdonBehaviourStruct, method: &StructMethod) -> CommunicationResult<Option<CustomEventDefinition>> {
        // Check if method has custom event attribute
        let event_name = method.attributes.iter()
            .find_map(|attr| match attr {
                MethodAttribute::UdonEvent(name) => Some(name.clone()),
            });

        if let Some(event_name) = event_name {
            let parameters = self.convert_method_parameters(&method.parameters)?;
            let validation_rules = self.determine_validation_rules(&parameters);

            let event = CustomEventDefinition {
                behavior_name: behavior.name.clone(),
                method_name: method.name.clone(),
                event_name,
                parameters,
                return_type: method.return_type.clone(),
                validation_rules,
                is_networked: self.is_networked_event(behavior, method),
            };

            // Validate the event definition
            self.validate_custom_event(&event)?;

            Ok(Some(event))
        } else {
            Ok(None)
        }
    }

    /// Convert method parameters to event parameters
    fn convert_method_parameters(&self, parameters: &[crate::multi_behavior::MethodParameter]) -> CommunicationResult<Vec<EventParameter>> {
        let mut event_params = Vec::new();

        for param in parameters {
            let csharp_type = self.type_mapper.map_type(&param.param_type)
                .map_err(|reason| CommunicationError::ParameterValidationError {
                    event: "unknown".to_string(),
                    parameter: param.name.clone(),
                    reason,
                })?;

            let event_param = EventParameter {
                name: param.name.clone(),
                rust_type: param.param_type.clone(),
                csharp_type,
                is_optional: matches!(param.param_type, RustType::Option(_)),
                validation_rules: self.get_parameter_validation_rules(&param.param_type),
            };

            event_params.push(event_param);
        }

        Ok(event_params)
    }

    /// Determine validation rules for parameters
    fn determine_validation_rules(&self, parameters: &[EventParameter]) -> Vec<String> {
        let mut rules = vec!["parameter_count_check".to_string()];
        
        for param in parameters {
            if !param.is_optional {
                rules.push(format!("required_parameter_{}", param.name));
            }
            
            if matches!(param.rust_type, RustType::GameObject | RustType::VRCPlayerApi) {
                rules.push(format!("null_check_{}", param.name));
            }
        }
        
        rules
    }

    /// Get validation rules for a parameter type
    fn get_parameter_validation_rules(&self, param_type: &RustType) -> Vec<String> {
        let mut rules = Vec::new();
        
        match param_type {
            RustType::GameObject => {
                rules.push("null_check".to_string());
                rules.push("component_check".to_string());
            }
            RustType::VRCPlayerApi => {
                rules.push("player_validity_check".to_string());
            }
            RustType::String => {
                rules.push("string_null_or_empty_check".to_string());
            }
            RustType::Option(_) => {
                rules.push("optional_parameter".to_string());
            }
            _ => {}
        }
        
        rules
    }

    /// Check if an event should be networked
    fn is_networked_event(&self, behavior: &UdonBehaviourStruct, method: &StructMethod) -> bool {
        // Events are networked if the behavior has networking capabilities
        // or if the method name suggests networking
        behavior.has_networking() || 
        method.name.contains("sync") || 
        method.name.contains("network") ||
        method.name.contains("broadcast")
    }

    /// Validate a custom event definition
    fn validate_custom_event(&self, event: &CustomEventDefinition) -> CommunicationResult<()> {
        // Validate event name
        if !is_valid_csharp_identifier(&event.event_name) {
            return Err(CommunicationError::InvalidCustomEvent {
                behavior: event.behavior_name.clone(),
                event: event.event_name.clone(),
                reason: "Event name must be a valid C# identifier".to_string(),
            });
        }

        // Validate parameters
        for param in &event.parameters {
            for validator in &self.parameter_validators {
                validator.validate(param, event)?;
            }
        }

        Ok(())
    }

    /// Generate SendCustomEvent method calls
    pub fn generate_send_custom_event_calls(&self, behavior: &UdonBehaviourStruct, gameobject_refs: &[GameObjectReference]) -> CommunicationResult<Vec<String>> {
        let mut method_calls = Vec::new();

        // Generate helper methods for sending events to each GameObject reference
        for reference in gameobject_refs {
            if let Some(target_behavior) = &reference.target_behavior {
                let methods = self.generate_event_sender_methods(behavior, reference, target_behavior)?;
                method_calls.extend(methods);
            }
        }

        Ok(method_calls)
    }

    /// Generate event sender methods for a specific GameObject reference
    fn generate_event_sender_methods(&self, behavior: &UdonBehaviourStruct, reference: &GameObjectReference, target_behavior: &str) -> CommunicationResult<Vec<String>> {
        let mut methods = Vec::new();

        // Get events that could be sent to the target behavior
        if let Some(target_events) = self.custom_events.get(target_behavior) {
            for event in target_events {
                let method = self.generate_send_event_method(behavior, reference, event)?;
                methods.push(method);
            }
        }

        // Generate generic SendCustomEvent method
        let generic_method = self.generate_generic_send_event_method(reference)?;
        methods.push(generic_method);

        Ok(methods)
    }

    /// Generate a specific SendCustomEvent method for an event
    fn generate_send_event_method(&self, _behavior: &UdonBehaviourStruct, reference: &GameObjectReference, event: &CustomEventDefinition) -> CommunicationResult<String> {
        let field_name = to_camel_case(&reference.field_name);
        let method_name = format!("Send{}To{}", to_pascal_case(&event.event_name), to_pascal_case(&reference.field_name));
        
        let mut method_lines = Vec::new();
        
        // Generate method signature
        let param_list = event.parameters.iter()
            .map(|p| format!("{} {}", p.csharp_type, to_camel_case(&p.name)))
            .collect::<Vec<_>>()
            .join(", ");

        method_lines.push(format!("    /// <summary>"));
        method_lines.push(format!("    /// Send {} event to {}", event.event_name, reference.field_name));
        method_lines.push(format!("    /// </summary>"));
        
        if !event.parameters.is_empty() {
            for param in &event.parameters {
                method_lines.push(format!("    /// <param name=\"{}\">{}</param>", 
                    to_camel_case(&param.name), 
                    self.get_parameter_description(&param.rust_type)));
            }
        }
        
        method_lines.push(format!("    public void {}({})", method_name, param_list));
        method_lines.push("    {".to_string());

        // Add validation code
        method_lines.push(format!("        // Validate {} reference", field_name));
        method_lines.push(format!("        if ({} == null)", field_name));
        method_lines.push("        {".to_string());
        method_lines.push(format!("            Debug.LogError(\"Cannot send {} event: {} is null\");", event.event_name, field_name));
        method_lines.push("            return;".to_string());
        method_lines.push("        }".to_string());
        method_lines.push("".to_string());

        // Add parameter validation
        for param in &event.parameters {
            let param_name = to_camel_case(&param.name);
            let validation_code = self.generate_parameter_validation(&param_name, &param.rust_type, &event.event_name);
            method_lines.extend(validation_code);
        }

        // Get UdonBehaviour component
        method_lines.push(format!("        // Get UdonBehaviour component from {}", field_name));
        method_lines.push(format!("        var targetBehaviour = {}.GetComponent<UdonBehaviour>();", field_name));
        method_lines.push("        if (targetBehaviour == null)".to_string());
        method_lines.push("        {".to_string());
        method_lines.push(format!("            Debug.LogError($\"GameObject '{{{}}}' does not have an UdonBehaviour component\", {}.name);", field_name, field_name));
        method_lines.push("            return;".to_string());
        method_lines.push("        }".to_string());
        method_lines.push("".to_string());

        // Handle parameters if any
        if !event.parameters.is_empty() {
            method_lines.push("        // Set event parameters".to_string());
            for param in &event.parameters {
                let param_name = to_camel_case(&param.name);
                method_lines.push(format!("        targetBehaviour.SetProgramVariable(\"{}_param\", {});", param.name, param_name));
            }
            method_lines.push("".to_string());
        }

        // Send the custom event
        method_lines.push(format!("        // Send custom event"));
        method_lines.push(format!("        targetBehaviour.SendCustomEvent(\"{}\");", event.event_name));

        method_lines.push("    }".to_string());

        Ok(method_lines.join("\n"))
    }

    /// Generate generic SendCustomEvent method
    fn generate_generic_send_event_method(&self, reference: &GameObjectReference) -> CommunicationResult<String> {
        let field_name = to_camel_case(&reference.field_name);
        let method_name = format!("SendCustomEventTo{}", to_pascal_case(&reference.field_name));
        
        let mut method_lines = Vec::new();
        
        method_lines.push(format!("    /// <summary>"));
        method_lines.push(format!("    /// Send a custom event to {}", reference.field_name));
        method_lines.push(format!("    /// </summary>"));
        method_lines.push(format!("    /// <param name=\"eventName\">Name of the custom event to send</param>"));
        method_lines.push(format!("    public void {}(string eventName)", method_name));
        method_lines.push("    {".to_string());

        // Add validation
        method_lines.push("        // Validate parameters".to_string());
        method_lines.push(format!("        if ({} == null)", field_name));
        method_lines.push("        {".to_string());
        method_lines.push(format!("            Debug.LogError(\"Cannot send custom event: {} is null\");", field_name));
        method_lines.push("            return;".to_string());
        method_lines.push("        }".to_string());
        method_lines.push("".to_string());
        
        method_lines.push("        if (string.IsNullOrEmpty(eventName))".to_string());
        method_lines.push("        {".to_string());
        method_lines.push("            Debug.LogError(\"Cannot send custom event: eventName is null or empty\");".to_string());
        method_lines.push("            return;".to_string());
        method_lines.push("        }".to_string());
        method_lines.push("".to_string());

        // Get UdonBehaviour and send event
        method_lines.push(format!("        // Get UdonBehaviour component and send event"));
        method_lines.push(format!("        var targetBehaviour = {}.GetComponent<UdonBehaviour>();", field_name));
        method_lines.push("        if (targetBehaviour != null)".to_string());
        method_lines.push("        {".to_string());
        method_lines.push("            targetBehaviour.SendCustomEvent(eventName);".to_string());
        method_lines.push("        }".to_string());
        method_lines.push("        else".to_string());
        method_lines.push("        {".to_string());
        method_lines.push(format!("            Debug.LogError($\"GameObject '{{{}}}' does not have an UdonBehaviour component\", {}.name);", field_name, field_name));
        method_lines.push("        }".to_string());

        method_lines.push("    }".to_string());

        Ok(method_lines.join("\n"))
    }

    /// Generate parameter validation code
    fn generate_parameter_validation(&self, param_name: &str, param_type: &RustType, event_name: &str) -> Vec<String> {
        let mut validation = Vec::new();

        match param_type {
            RustType::GameObject => {
                validation.push(format!("        // Validate {} parameter", param_name));
                validation.push(format!("        if ({} == null)", param_name));
                validation.push("        {".to_string());
                validation.push(format!("            Debug.LogWarning(\"Event '{}': {} parameter is null\");", event_name, param_name));
                validation.push("            return;".to_string());
                validation.push("        }".to_string());
                validation.push("".to_string());
            }
            RustType::VRCPlayerApi => {
                validation.push(format!("        // Validate {} parameter", param_name));
                validation.push(format!("        if (!Utilities.IsValid({}))", param_name));
                validation.push("        {".to_string());
                validation.push(format!("            Debug.LogWarning(\"Event '{}': {} parameter is not a valid player\");", event_name, param_name));
                validation.push("            return;".to_string());
                validation.push("        }".to_string());
                validation.push("".to_string());
            }
            RustType::String => {
                validation.push(format!("        // Validate {} parameter", param_name));
                validation.push(format!("        if (string.IsNullOrEmpty({}))", param_name));
                validation.push("        {".to_string());
                validation.push(format!("            Debug.LogWarning(\"Event '{}': {} parameter is null or empty\");", event_name, param_name));
                validation.push("            return;".to_string());
                validation.push("        }".to_string());
                validation.push("".to_string());
            }
            RustType::Option(_) => {
                validation.push(format!("        // {} is optional, no validation needed", param_name));
                validation.push("".to_string());
            }
            _ => {
                // For other types, just add a comment
                validation.push(format!("        // {} parameter validated (type: {:?})", param_name, param_type));
                validation.push("".to_string());
            }
        }

        validation
    }

    /// Get parameter description for documentation
    fn get_parameter_description(&self, param_type: &RustType) -> String {
        match param_type {
            RustType::GameObject => "GameObject reference".to_string(),
            RustType::VRCPlayerApi => "VRChat player reference".to_string(),
            RustType::String => "String parameter".to_string(),
            RustType::I32 => "Integer value".to_string(),
            RustType::F32 => "Float value".to_string(),
            RustType::Bool => "Boolean value".to_string(),
            RustType::Vector3 => "3D vector position".to_string(),
            RustType::Option(inner) => format!("Optional {}", self.get_parameter_description(inner)),
            _ => format!("Parameter of type {:?}", param_type),
        }
    }

    /// Get all custom events for a behavior
    pub fn get_custom_events(&self, behavior_name: &str) -> Option<&Vec<CustomEventDefinition>> {
        self.custom_events.get(behavior_name)
    }

    /// Generate event routing table
    pub fn generate_event_routing_table(&self, behaviors: &[UdonBehaviourStruct]) -> CommunicationResult<HashMap<String, Vec<EventRoute>>> {
        let mut routing_table = HashMap::new();

        for behavior in behaviors {
            let mut routes = Vec::new();

            if let Some(events) = self.custom_events.get(&behavior.name) {
                for event in events {
                    let route = EventRoute {
                        source_behavior: behavior.name.clone(),
                        event_name: event.event_name.clone(),
                        target_behaviors: self.find_potential_targets(&event.event_name, behaviors),
                        parameter_mapping: self.generate_parameter_mapping(&event.parameters),
                    };
                    routes.push(route);
                }
            }

            routing_table.insert(behavior.name.clone(), routes);
        }

        Ok(routing_table)
    }

    /// Find potential target behaviors for an event
    fn find_potential_targets(&self, event_name: &str, behaviors: &[UdonBehaviourStruct]) -> Vec<String> {
        let mut targets = Vec::new();

        for behavior in behaviors {
            if let Some(events) = self.custom_events.get(&behavior.name) {
                if events.iter().any(|e| e.event_name == event_name) {
                    targets.push(behavior.name.clone());
                }
            }
        }

        targets
    }

    /// Generate parameter mapping for event routing
    fn generate_parameter_mapping(&self, parameters: &[EventParameter]) -> HashMap<String, String> {
        let mut mapping = HashMap::new();

        for param in parameters {
            mapping.insert(param.name.clone(), param.csharp_type.clone());
        }

        mapping
    }
}

impl Default for CustomEventRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a custom event definition
#[derive(Debug, Clone)]
pub struct CustomEventDefinition {
    /// Behavior that defines this event
    pub behavior_name: String,
    /// Method name in Rust
    pub method_name: String,
    /// Event name for UdonSharp
    pub event_name: String,
    /// Event parameters
    pub parameters: Vec<EventParameter>,
    /// Return type (usually Unit for events)
    pub return_type: RustType,
    /// Validation rules to apply
    pub validation_rules: Vec<String>,
    /// Whether this event should be networked
    pub is_networked: bool,
}

/// Represents an event parameter
#[derive(Debug, Clone)]
pub struct EventParameter {
    /// Parameter name
    pub name: String,
    /// Rust type
    pub rust_type: RustType,
    /// C# type
    pub csharp_type: String,
    /// Whether parameter is optional
    pub is_optional: bool,
    /// Validation rules for this parameter
    pub validation_rules: Vec<String>,
}

/// Represents an event route between behaviors
#[derive(Debug, Clone)]
pub struct EventRoute {
    /// Source behavior that sends the event
    pub source_behavior: String,
    /// Event name
    pub event_name: String,
    /// Potential target behaviors
    pub target_behaviors: Vec<String>,
    /// Parameter type mapping
    pub parameter_mapping: HashMap<String, String>,
}

/// Trait for parameter validation
pub trait ParameterValidator {
    /// Validate a parameter in the context of an event
    fn validate(&self, parameter: &EventParameter, event: &CustomEventDefinition) -> CommunicationResult<()>;
    
    /// Get validator name
    fn validator_name(&self) -> &'static str;
}

/// Validator for type compatibility
pub struct TypeCompatibilityValidator;

impl ParameterValidator for TypeCompatibilityValidator {
    fn validate(&self, parameter: &EventParameter, _event: &CustomEventDefinition) -> CommunicationResult<()> {
        if !parameter.rust_type.is_udonsharp_compatible() {
            return Err(CommunicationError::ParameterValidationError {
                event: _event.event_name.clone(),
                parameter: parameter.name.clone(),
                reason: format!("Type {:?} is not compatible with UdonSharp", parameter.rust_type),
            });
        }
        Ok(())
    }

    fn validator_name(&self) -> &'static str {
        "type_compatibility"
    }
}

/// Validator for null safety
pub struct NullSafetyValidator;

impl ParameterValidator for NullSafetyValidator {
    fn validate(&self, parameter: &EventParameter, event: &CustomEventDefinition) -> CommunicationResult<()> {
        if !parameter.is_optional && matches!(parameter.rust_type, RustType::GameObject | RustType::VRCPlayerApi) {
            // This is fine, we'll generate null checks at runtime
        }
        Ok(())
    }

    fn validator_name(&self) -> &'static str {
        "null_safety"
    }
}

/// Validator for UdonSharp compatibility
pub struct UdonSharpCompatibilityValidator;

impl ParameterValidator for UdonSharpCompatibilityValidator {
    fn validate(&self, parameter: &EventParameter, event: &CustomEventDefinition) -> CommunicationResult<()> {
        // Check for UdonSharp-specific limitations
        match &parameter.rust_type {
            RustType::HashMap(_, _) => {
                return Err(CommunicationError::ParameterValidationError {
                    event: event.event_name.clone(),
                    parameter: parameter.name.clone(),
                    reason: "HashMap parameters are not supported in UdonSharp custom events. Use arrays instead.".to_string(),
                });
            }
            RustType::Vec(inner) => {
                if !inner.is_udonsharp_compatible() {
                    return Err(CommunicationError::ParameterValidationError {
                        event: event.event_name.clone(),
                        parameter: parameter.name.clone(),
                        reason: format!("Vec<{:?}> is not supported in UdonSharp", inner),
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn validator_name(&self) -> &'static str {
        "udonsharp_compatibility"
    }
}

/// Manager for safe parameter passing between behaviors
pub struct EventParameterHandler {
    /// Type mapper for parameter conversion
    type_mapper: RustToCSharpTypeMapper,
    /// Parameter serialization strategies
    serialization_strategies: HashMap<RustType, Box<dyn ParameterSerializationStrategy>>,
    /// Parameter validation cache
    validation_cache: HashMap<String, ParameterValidationResult>,
}

impl EventParameterHandler {
    /// Create a new event parameter handler
    pub fn new() -> Self {
        let mut handler = Self {
            type_mapper: RustToCSharpTypeMapper::new(),
            serialization_strategies: HashMap::new(),
            validation_cache: HashMap::new(),
        };
        
        handler.initialize_serialization_strategies();
        handler
    }

    /// Initialize parameter serialization strategies
    fn initialize_serialization_strategies(&mut self) {
        // Basic types use direct passing
        self.serialization_strategies.insert(RustType::Bool, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::I32, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::F32, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::String, Box::new(DirectPassingStrategy));
        
        // Unity types use direct passing
        self.serialization_strategies.insert(RustType::Vector3, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::Quaternion, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::Color, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::GameObject, Box::new(DirectPassingStrategy));
        self.serialization_strategies.insert(RustType::VRCPlayerApi, Box::new(DirectPassingStrategy));
        
        // Complex types need special handling
        // Note: We'll add specific strategies for Vec, Option, etc. as needed
    }

    /// Generate parameter handling code for custom events
    pub fn generate_parameter_handling(&self, event: &CustomEventDefinition) -> CommunicationResult<ParameterHandlingCode> {
        let mut sender_code = Vec::new();
        let mut receiver_code = Vec::new();
        let mut validation_code = Vec::new();

        // Generate code for each parameter
        for (index, param) in event.parameters.iter().enumerate() {
            let param_sender = self.generate_parameter_sender_code(param, index)?;
            let param_receiver = self.generate_parameter_receiver_code(param, index)?;
            let param_validation = self.generate_parameter_validation_code(param, &event.event_name)?;

            sender_code.extend(param_sender);
            receiver_code.extend(param_receiver);
            validation_code.extend(param_validation);
        }

        Ok(ParameterHandlingCode {
            sender_code,
            receiver_code,
            validation_code,
            parameter_count: event.parameters.len(),
            requires_serialization: self.requires_serialization(&event.parameters),
        })
    }

    /// Generate code for sending parameters
    fn generate_parameter_sender_code(&self, param: &EventParameter, index: usize) -> CommunicationResult<Vec<String>> {
        let mut code = Vec::new();
        let param_name = to_camel_case(&param.name);
        let param_key = format!("param_{}", index);

        match &param.rust_type {
            RustType::Option(inner) => {
                // Handle optional parameters
                code.push(format!("        // Handle optional parameter {}", param_name));
                code.push(format!("        if ({} != null)", param_name));
                code.push("        {".to_string());
                code.push(format!("            targetBehaviour.SetProgramVariable(\"{}\", {});", param_key, param_name));
                code.push(format!("            targetBehaviour.SetProgramVariable(\"{}_hasValue\", true);", param_key));
                code.push("        }".to_string());
                code.push("        else".to_string());
                code.push("        {".to_string());
                code.push(format!("            targetBehaviour.SetProgramVariable(\"{}_hasValue\", false);", param_key));
                code.push("        }".to_string());
            }
            RustType::Vec(inner) => {
                // Handle array parameters
                let inner_type = self.type_mapper.map_type(inner)
                    .map_err(|reason| CommunicationError::ParameterValidationError {
                        event: "unknown".to_string(),
                        parameter: param.name.clone(),
                        reason,
                    })?;
                
                code.push(format!("        // Handle array parameter {}", param_name));
                code.push(format!("        if ({} != null)", param_name));
                code.push("        {".to_string());
                code.push(format!("            targetBehaviour.SetProgramVariable(\"{}\", {});", param_key, param_name));
                code.push(format!("            targetBehaviour.SetProgramVariable(\"{}_length\", {}.Length);", param_key, param_name));
                code.push("        }".to_string());
                code.push("        else".to_string());
                code.push("        {".to_string());
                code.push(format!("            targetBehaviour.SetProgramVariable(\"{}_length\", 0);", param_key));
                code.push("        }".to_string());
            }
            _ => {
                // Handle direct parameters
                code.push(format!("        // Set parameter {}", param_name));
                code.push(format!("        targetBehaviour.SetProgramVariable(\"{}\", {});", param_key, param_name));
            }
        }

        Ok(code)
    }

    /// Generate code for receiving parameters
    fn generate_parameter_receiver_code(&self, param: &EventParameter, index: usize) -> CommunicationResult<Vec<String>> {
        let mut code = Vec::new();
        let param_name = to_camel_case(&param.name);
        let param_key = format!("param_{}", index);

        code.push(format!("        // Receive parameter {}", param_name));

        match &param.rust_type {
            RustType::Option(inner) => {
                let inner_type = self.type_mapper.map_type(inner)
                    .map_err(|reason| CommunicationError::ParameterValidationError {
                        event: "unknown".to_string(),
                        parameter: param.name.clone(),
                        reason,
                    })?;

                code.push(format!("        {} {} = null;", inner_type, param_name));
                code.push(format!("        bool {}_hasValue = (bool)GetProgramVariable(\"{}_hasValue\");", param_name, param_key));
                code.push(format!("        if ({}_hasValue)", param_name));
                code.push("        {".to_string());
                code.push(format!("            {} = ({})GetProgramVariable(\"{}\");", param_name, inner_type, param_key));
                code.push("        }".to_string());
            }
            RustType::Vec(inner) => {
                let inner_type = self.type_mapper.map_type(inner)
                    .map_err(|reason| CommunicationError::ParameterValidationError {
                        event: "unknown".to_string(),
                        parameter: param.name.clone(),
                        reason,
                    })?;

                code.push(format!("        {}[] {} = null;", inner_type, param_name));
                code.push(format!("        int {}_length = (int)GetProgramVariable(\"{}_length\");", param_name, param_key));
                code.push(format!("        if ({}_length > 0)", param_name));
                code.push("        {".to_string());
                code.push(format!("            {} = ({}[])GetProgramVariable(\"{}\");", param_name, inner_type, param_key));
                code.push("        }".to_string());
            }
            _ => {
                code.push(format!("        {} {} = ({})GetProgramVariable(\"{}\");", 
                    param.csharp_type, param_name, param.csharp_type, param_key));
            }
        }

        Ok(code)
    }

    /// Generate parameter validation code
    fn generate_parameter_validation_code(&self, param: &EventParameter, event_name: &str) -> CommunicationResult<Vec<String>> {
        let mut code = Vec::new();
        let param_name = to_camel_case(&param.name);

        // Generate validation based on parameter type and rules
        for rule in &param.validation_rules {
            match rule.as_str() {
                "null_check" => {
                    code.push(format!("        // Null check for {}", param_name));
                    code.push(format!("        if ({} == null)", param_name));
                    code.push("        {".to_string());
                    code.push(format!("            Debug.LogWarning($\"Event '{}': Parameter '{}' is null\");", event_name, param.name));
                    if !param.is_optional {
                        code.push("            return; // Required parameter is null".to_string());
                    }
                    code.push("        }".to_string());
                }
                "component_check" => {
                    if matches!(param.rust_type, RustType::GameObject) {
                        code.push(format!("        // Component check for {}", param_name));
                        code.push(format!("        if ({} != null)", param_name));
                        code.push("        {".to_string());
                        code.push(format!("            var component = {}.GetComponent<UdonBehaviour>();", param_name));
                        code.push("            if (component == null)".to_string());
                        code.push("            {".to_string());
                        code.push(format!("                Debug.LogWarning($\"Event '{}': GameObject parameter '{}' has no UdonBehaviour component\");", event_name, param.name));
                        code.push("            }".to_string());
                        code.push("        }".to_string());
                    }
                }
                "player_validity_check" => {
                    if matches!(param.rust_type, RustType::VRCPlayerApi) {
                        code.push(format!("        // Player validity check for {}", param_name));
                        code.push(format!("        if ({} != null && !Utilities.IsValid({}))", param_name, param_name));
                        code.push("        {".to_string());
                        code.push(format!("            Debug.LogWarning($\"Event '{}': Player parameter '{}' is not valid\");", event_name, param.name));
                        if !param.is_optional {
                            code.push("            return; // Invalid player parameter".to_string());
                        }
                        code.push("        }".to_string());
                    }
                }
                "string_null_or_empty_check" => {
                    if matches!(param.rust_type, RustType::String) {
                        code.push(format!("        // String validation for {}", param_name));
                        code.push(format!("        if (string.IsNullOrEmpty({}))", param_name));
                        code.push("        {".to_string());
                        code.push(format!("            Debug.LogWarning($\"Event '{}': String parameter '{}' is null or empty\");", event_name, param.name));
                        if !param.is_optional {
                            code.push("            return; // Required string parameter is empty".to_string());
                        }
                        code.push("        }".to_string());
                    }
                }
                _ => {
                    // Custom validation rule
                    code.push(format!("        // Custom validation rule: {}", rule));
                }
            }
        }

        Ok(code)
    }

    /// Check if parameters require serialization
    fn requires_serialization(&self, parameters: &[EventParameter]) -> bool {
        parameters.iter().any(|p| {
            matches!(p.rust_type, 
                RustType::Vec(_) | 
                RustType::HashMap(_, _) | 
                RustType::Custom(_)
            )
        })
    }

    /// Generate parameter type checking code
    pub fn generate_type_checking_code(&self, parameters: &[EventParameter]) -> CommunicationResult<Vec<String>> {
        let mut code = Vec::new();

        code.push("        // Parameter type checking".to_string());
        
        for (index, param) in parameters.iter().enumerate() {
            let param_key = format!("param_{}", index);
            let param_name = to_camel_case(&param.name);

            code.push(format!("        // Check type for parameter {}", param_name));
            code.push(format!("        var {}_value = GetProgramVariable(\"{}\");", param_name, param_key));
            code.push(format!("        if ({}_value != null && !({}_value is {}))", param_name, param_name, param.csharp_type));
            code.push("        {".to_string());
            code.push(format!("            Debug.LogError($\"Parameter '{}' expected type {} but got {{{}._value.GetType()}}\");", param.name, param.csharp_type, param_name));
            code.push("            return;".to_string());
            code.push("        }".to_string());
        }

        Ok(code)
    }

    /// Generate parameter serialization helpers
    pub fn generate_serialization_helpers(&self, parameters: &[EventParameter]) -> CommunicationResult<Vec<String>> {
        let mut helpers = Vec::new();

        for param in parameters {
            if let Some(strategy) = self.serialization_strategies.get(&param.rust_type) {
                let helper_code = strategy.generate_serialization_helper(param)?;
                helpers.extend(helper_code);
            }
        }

        Ok(helpers)
    }

    /// Validate parameter compatibility between sender and receiver
    pub fn validate_parameter_compatibility(&self, sender_params: &[EventParameter], receiver_params: &[EventParameter]) -> CommunicationResult<Vec<String>> {
        let mut warnings = Vec::new();

        if sender_params.len() != receiver_params.len() {
            warnings.push(format!("Parameter count mismatch: sender has {}, receiver has {}", 
                sender_params.len(), receiver_params.len()));
        }

        for (i, (sender, receiver)) in sender_params.iter().zip(receiver_params.iter()).enumerate() {
            if sender.csharp_type != receiver.csharp_type {
                warnings.push(format!("Parameter {} type mismatch: sender has {}, receiver has {}", 
                    i, sender.csharp_type, receiver.csharp_type));
            }

            if sender.name != receiver.name {
                warnings.push(format!("Parameter {} name mismatch: sender has '{}', receiver has '{}'", 
                    i, sender.name, receiver.name));
            }
        }

        Ok(warnings)
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }
}

impl Default for EventParameterHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of parameter handling code generation
#[derive(Debug, Clone)]
pub struct ParameterHandlingCode {
    /// Code for sending parameters
    pub sender_code: Vec<String>,
    /// Code for receiving parameters
    pub receiver_code: Vec<String>,
    /// Code for validating parameters
    pub validation_code: Vec<String>,
    /// Number of parameters
    pub parameter_count: usize,
    /// Whether serialization is required
    pub requires_serialization: bool,
}

/// Result of parameter validation
#[derive(Debug, Clone)]
pub struct ParameterValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Trait for parameter serialization strategies
pub trait ParameterSerializationStrategy {
    /// Generate serialization helper code
    fn generate_serialization_helper(&self, parameter: &EventParameter) -> CommunicationResult<Vec<String>>;
    
    /// Get strategy name
    fn strategy_name(&self) -> &'static str;
}

/// Direct passing strategy for simple types
pub struct DirectPassingStrategy;

impl ParameterSerializationStrategy for DirectPassingStrategy {
    fn generate_serialization_helper(&self, parameter: &EventParameter) -> CommunicationResult<Vec<String>> {
        // Direct passing doesn't need helper code
        Ok(Vec::new())
    }

    fn strategy_name(&self) -> &'static str {
        "direct_passing"
    }
}

/// JSON serialization strategy for complex types
pub struct JsonSerializationStrategy;

impl ParameterSerializationStrategy for JsonSerializationStrategy {
    fn generate_serialization_helper(&self, parameter: &EventParameter) -> CommunicationResult<Vec<String>> {
        let mut code = Vec::new();
        let param_name = to_camel_case(&parameter.name);

        code.push(format!("    /// <summary>"));
        code.push(format!("    /// Serialize {} parameter to JSON", param_name));
        code.push(format!("    /// </summary>"));
        code.push(format!("    private string Serialize{}({})", to_pascal_case(&parameter.name), parameter.csharp_type));
        code.push("    {".to_string());
        code.push("        try".to_string());
        code.push("        {".to_string());
        code.push(format!("            return JsonUtility.ToJson({});", param_name));
        code.push("        }".to_string());
        code.push("        catch (System.Exception ex)".to_string());
        code.push("        {".to_string());
        code.push(format!("            Debug.LogError($\"Failed to serialize {}: {{ex.Message}}\");", param_name));
        code.push("            return null;".to_string());
        code.push("        }".to_string());
        code.push("    }".to_string());

        Ok(code)
    }

    fn strategy_name(&self) -> &'static str {
        "json_serialization"
    }
}

/// Main inter-behavior communication coordinator
pub struct InterBehaviorCommunicationCoordinator {
    /// GameObject reference manager
    pub gameobject_manager: GameObjectReferenceManager,
    /// Custom event router
    pub event_router: CustomEventRouter,
    /// Parameter handler
    pub parameter_handler: EventParameterHandler,
}

impl InterBehaviorCommunicationCoordinator {
    /// Create a new inter-behavior communication coordinator
    pub fn new() -> Self {
        Self {
            gameobject_manager: GameObjectReferenceManager::new(),
            event_router: CustomEventRouter::new(),
            parameter_handler: EventParameterHandler::new(),
        }
    }

    /// Analyze and set up communication for multiple behaviors
    pub fn setup_communication(&mut self, behaviors: &[UdonBehaviourStruct]) -> CommunicationResult<CommunicationSetup> {
        let mut gameobject_references = HashMap::new();
        let mut custom_events = HashMap::new();
        let mut parameter_handling = HashMap::new();

        // Analyze each behavior
        for behavior in behaviors {
            // Analyze GameObject references
            let refs = self.gameobject_manager.analyze_gameobject_references(behavior)?;
            gameobject_references.insert(behavior.name.clone(), refs);

            // Analyze custom events
            let events = self.event_router.analyze_custom_events(behavior)?;
            custom_events.insert(behavior.name.clone(), events.clone());

            // Generate parameter handling for each event
            for event in &events {
                let handling = self.parameter_handler.generate_parameter_handling(event)?;
                parameter_handling.insert(format!("{}::{}", behavior.name, event.event_name), handling);
            }
        }

        // Generate event routing table
        let routing_table = self.event_router.generate_event_routing_table(behaviors)?;

        // Validate communication setup
        let validation_results = self.validate_communication_setup(behaviors, &gameobject_references, &custom_events)?;

        Ok(CommunicationSetup {
            gameobject_references,
            custom_events,
            parameter_handling,
            routing_table,
            validation_results,
        })
    }

    /// Validate the entire communication setup
    fn validate_communication_setup(
        &self,
        behaviors: &[UdonBehaviourStruct],
        gameobject_refs: &HashMap<String, Vec<GameObjectReference>>,
        custom_events: &HashMap<String, Vec<CustomEventDefinition>>,
    ) -> CommunicationResult<CommunicationValidationResults> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Validate GameObject references
        let ref_warnings = self.gameobject_manager.validate_all_references()?;
        warnings.extend(ref_warnings);

        // Validate event compatibility
        for behavior in behaviors {
            if let Some(refs) = gameobject_refs.get(&behavior.name) {
                for reference in refs {
                    if let Some(target_behavior) = &reference.target_behavior {
                        if !behaviors.iter().any(|b| b.name == *target_behavior) {
                            warnings.push(format!("Behavior '{}' references non-existent behavior '{}'", 
                                behavior.name, target_behavior));
                        }
                    }
                }
            }
        }

        // Check for orphaned events (events that are sent but never received)
        for (behavior_name, events) in custom_events {
            for event in events {
                let has_receivers = behaviors.iter().any(|b| {
                    if let Some(target_events) = custom_events.get(&b.name) {
                        target_events.iter().any(|e| e.event_name == event.event_name && b.name != *behavior_name)
                    } else {
                        false
                    }
                });

                if !has_receivers {
                    warnings.push(format!("Event '{}' in behavior '{}' has no receivers", 
                        event.event_name, behavior_name));
                }
            }
        }

        let is_valid = errors.is_empty();
        Ok(CommunicationValidationResults {
            errors,
            warnings,
            is_valid,
        })
    }

    /// Generate complete communication code for a behavior
    pub fn generate_communication_code(&self, behavior: &UdonBehaviourStruct, setup: &CommunicationSetup) -> CommunicationResult<String> {
        let mut code_sections = Vec::new();

        // Generate GameObject reference fields
        if let Some(refs) = setup.gameobject_references.get(&behavior.name) {
            if !refs.is_empty() {
                code_sections.push("    // GameObject References".to_string());
                let fields = self.gameobject_manager.generate_gameobject_fields(behavior)?;
                for field in fields {
                    code_sections.push(field.declaration);
                }
                code_sections.push("".to_string());
            }
        }

        // Generate custom event sender methods
        if let Some(refs) = setup.gameobject_references.get(&behavior.name) {
            if !refs.is_empty() {
                code_sections.push("    // Custom Event Sender Methods".to_string());
                let sender_methods = self.event_router.generate_send_custom_event_calls(behavior, refs)?;
                code_sections.extend(sender_methods);
                code_sections.push("".to_string());
            }
        }

        // Generate GameObject initialization code for Start method
        if let Some(refs) = setup.gameobject_references.get(&behavior.name) {
            if !refs.is_empty() {
                let init_code = self.gameobject_manager.generate_gameobject_initialization(behavior)?;
                if !init_code.is_empty() {
                    code_sections.push("    // GameObject Reference Initialization (add to Start method)".to_string());
                    code_sections.extend(init_code);
                    code_sections.push("".to_string());
                }
            }
        }

        Ok(code_sections.join("\n"))
    }
}

impl Default for InterBehaviorCommunicationCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete communication setup for multiple behaviors
#[derive(Debug, Clone)]
pub struct CommunicationSetup {
    /// GameObject references by behavior
    pub gameobject_references: HashMap<String, Vec<GameObjectReference>>,
    /// Custom events by behavior
    pub custom_events: HashMap<String, Vec<CustomEventDefinition>>,
    /// Parameter handling by event
    pub parameter_handling: HashMap<String, ParameterHandlingCode>,
    /// Event routing table
    pub routing_table: HashMap<String, Vec<EventRoute>>,
    /// Validation results
    pub validation_results: CommunicationValidationResults,
}

/// Results of communication validation
#[derive(Debug, Clone)]
pub struct CommunicationValidationResults {
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Whether the setup is valid
    pub is_valid: bool,
}