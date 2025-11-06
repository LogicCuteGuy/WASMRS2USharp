//! UdonBehaviour trait validation for ensuring proper implementation
//! 
//! This module provides validation logic to ensure that structs with
//! #[derive(UdonBehaviour)] properly implement the UdonBehaviour trait
//! with all required methods.

use crate::multi_behavior::{UdonBehaviourStruct, UdonBehaviourTraitImpl, StructMethod};
use std::collections::HashSet;

/// Result type for trait validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Errors that can occur during trait validation
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Missing trait implementation entirely
    MissingTraitImplementation { struct_name: String },
    /// Missing required methods from trait implementation
    MissingRequiredMethods { struct_name: String, missing_methods: Vec<String> },
    /// Invalid method signature for trait method
    InvalidMethodSignature { struct_name: String, method_name: String, expected: String, found: String },
    /// Method implementation has incorrect visibility
    InvalidMethodVisibility { struct_name: String, method_name: String, expected: String, found: String },
    /// Async methods are not supported for UdonBehaviour trait
    AsyncMethodNotSupported { struct_name: String, method_name: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingTraitImplementation { struct_name } => {
                write!(f, "Struct '{}' must implement the UdonBehaviour trait", struct_name)
            }
            ValidationError::MissingRequiredMethods { struct_name, missing_methods } => {
                write!(f, "Struct '{}' is missing required UdonBehaviour methods: {}", 
                       struct_name, missing_methods.join(", "))
            }
            ValidationError::InvalidMethodSignature { struct_name, method_name, expected, found } => {
                write!(f, "Invalid signature for method '{}::{}'. Expected: {}, Found: {}", 
                       struct_name, method_name, expected, found)
            }
            ValidationError::InvalidMethodVisibility { struct_name, method_name, expected, found } => {
                write!(f, "Invalid visibility for method '{}::{}'. Expected: {}, Found: {}", 
                       struct_name, method_name, expected, found)
            }
            ValidationError::AsyncMethodNotSupported { struct_name, method_name } => {
                write!(f, "Async method '{}::{}' is not supported for UdonBehaviour trait", 
                       struct_name, method_name)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validator for UdonBehaviour trait implementations
pub struct TraitValidator {
    /// Required methods that must be implemented
    required_methods: HashSet<String>,
    /// Optional methods that can be implemented
    optional_methods: HashSet<String>,
    /// Unity event methods that have specific signatures
    unity_event_methods: HashSet<String>,
}

impl TraitValidator {
    /// Create a new trait validator with default UdonBehaviour requirements
    pub fn new() -> Self {
        let mut required_methods = HashSet::new();
        required_methods.insert("start".to_string());

        let mut optional_methods = HashSet::new();
        optional_methods.insert("update".to_string());
        optional_methods.insert("fixed_update".to_string());
        optional_methods.insert("late_update".to_string());
        optional_methods.insert("on_enable".to_string());
        optional_methods.insert("on_disable".to_string());
        optional_methods.insert("on_destroy".to_string());
        optional_methods.insert("on_trigger_enter".to_string());
        optional_methods.insert("on_trigger_exit".to_string());
        optional_methods.insert("on_trigger_stay".to_string());
        optional_methods.insert("on_collision_enter".to_string());
        optional_methods.insert("on_collision_exit".to_string());
        optional_methods.insert("on_collision_stay".to_string());
        optional_methods.insert("on_player_joined".to_string());
        optional_methods.insert("on_player_left".to_string());
        optional_methods.insert("on_player_trigger_enter".to_string());
        optional_methods.insert("on_player_trigger_exit".to_string());
        optional_methods.insert("on_player_trigger_stay".to_string());
        optional_methods.insert("on_player_collision_enter".to_string());
        optional_methods.insert("on_player_collision_exit".to_string());
        optional_methods.insert("on_player_collision_stay".to_string());
        optional_methods.insert("on_pickup".to_string());
        optional_methods.insert("on_drop".to_string());
        optional_methods.insert("on_pickup_use_down".to_string());
        optional_methods.insert("on_pickup_use_up".to_string());
        optional_methods.insert("on_station_entered".to_string());
        optional_methods.insert("on_station_exited".to_string());
        optional_methods.insert("on_post_deserialization".to_string());

        let mut unity_event_methods = HashSet::new();
        unity_event_methods.insert("start".to_string());
        unity_event_methods.insert("update".to_string());
        unity_event_methods.insert("fixed_update".to_string());
        unity_event_methods.insert("late_update".to_string());
        unity_event_methods.insert("on_enable".to_string());
        unity_event_methods.insert("on_disable".to_string());
        unity_event_methods.insert("on_destroy".to_string());
        unity_event_methods.insert("on_trigger_enter".to_string());
        unity_event_methods.insert("on_trigger_exit".to_string());
        unity_event_methods.insert("on_trigger_stay".to_string());
        unity_event_methods.insert("on_collision_enter".to_string());
        unity_event_methods.insert("on_collision_exit".to_string());
        unity_event_methods.insert("on_collision_stay".to_string());
        unity_event_methods.insert("on_player_joined".to_string());
        unity_event_methods.insert("on_player_left".to_string());
        unity_event_methods.insert("on_pickup".to_string());
        unity_event_methods.insert("on_drop".to_string());
        unity_event_methods.insert("on_pickup_use_down".to_string());
        unity_event_methods.insert("on_pickup_use_up".to_string());
        unity_event_methods.insert("on_station_entered".to_string());
        unity_event_methods.insert("on_station_exited".to_string());
        unity_event_methods.insert("on_post_deserialization".to_string());

        Self {
            required_methods,
            optional_methods,
            unity_event_methods,
        }
    }

    /// Validate a UdonBehaviour struct implementation
    pub fn validate_struct(&self, udon_struct: &UdonBehaviourStruct) -> ValidationResult<()> {
        // Check if trait implementation exists
        let trait_impl = udon_struct.trait_impl.as_ref()
            .ok_or_else(|| ValidationError::MissingTraitImplementation {
                struct_name: udon_struct.name.clone(),
            })?;

        // Validate required methods are implemented
        self.validate_required_methods(&udon_struct.name, trait_impl)?;

        // Validate method signatures for implemented methods
        self.validate_method_signatures(udon_struct)?;

        // Validate method visibility and async constraints
        self.validate_method_constraints(udon_struct)?;

        Ok(())
    }

    /// Validate that all required methods are implemented
    fn validate_required_methods(&self, struct_name: &str, trait_impl: &UdonBehaviourTraitImpl) -> ValidationResult<()> {
        let missing_methods: Vec<String> = self.required_methods
            .iter()
            .filter(|method| !trait_impl.implemented_methods.contains(*method))
            .cloned()
            .collect();

        if !missing_methods.is_empty() {
            return Err(ValidationError::MissingRequiredMethods {
                struct_name: struct_name.to_string(),
                missing_methods,
            });
        }

        Ok(())
    }

    /// Validate method signatures match expected UdonBehaviour patterns
    fn validate_method_signatures(&self, udon_struct: &UdonBehaviourStruct) -> ValidationResult<()> {
        for method in &udon_struct.methods {
            if self.is_trait_method(&method.name) {
                self.validate_unity_event_signature(&udon_struct.name, method)?;
            }
        }
        Ok(())
    }

    /// Validate method constraints (visibility, async, etc.)
    fn validate_method_constraints(&self, udon_struct: &UdonBehaviourStruct) -> ValidationResult<()> {
        for method in &udon_struct.methods {
            if self.is_trait_method(&method.name) {
                // Check if method is async (not allowed for UdonBehaviour trait methods)
                if method.is_async {
                    return Err(ValidationError::AsyncMethodNotSupported {
                        struct_name: udon_struct.name.clone(),
                        method_name: method.name.clone(),
                    });
                }

                // Unity event methods should be public (will be generated as public in C#)
                if self.unity_event_methods.contains(&method.name) {
                    // Note: We don't enforce visibility here since the generated C# will be public
                    // regardless of Rust visibility
                }
            }
        }
        Ok(())
    }

    /// Validate Unity event method signature
    fn validate_unity_event_signature(&self, struct_name: &str, method: &StructMethod) -> ValidationResult<()> {
        let expected_signature = self.get_expected_signature(&method.name);
        let actual_signature = self.format_method_signature(method);

        // For most Unity events, we expect no parameters and void return
        match method.name.as_str() {
            "start" | "update" | "fixed_update" | "late_update" | 
            "on_enable" | "on_disable" | "on_destroy" => {
                if !method.parameters.is_empty() {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_trigger_enter" | "on_trigger_exit" | "on_trigger_stay" => {
                // These methods should have one Collider parameter
                if method.parameters.len() != 1 {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_collision_enter" | "on_collision_exit" | "on_collision_stay" => {
                // These methods should have one Collision parameter
                if method.parameters.len() != 1 {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_player_joined" | "on_player_left" => {
                // These methods should have one VRCPlayerApi parameter
                if method.parameters.len() != 1 {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_player_trigger_enter" | "on_player_trigger_exit" | "on_player_trigger_stay" => {
                // These methods should have one VRCPlayerApi parameter
                if method.parameters.len() != 1 {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_player_collision_enter" | "on_player_collision_exit" | "on_player_collision_stay" => {
                // These methods should have one VRCPlayerApi parameter
                if method.parameters.len() != 1 {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_pickup" | "on_drop" => {
                // These methods should have no parameters
                if !method.parameters.is_empty() {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_pickup_use_down" | "on_pickup_use_up" => {
                // These methods should have no parameters
                if !method.parameters.is_empty() {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_station_entered" | "on_station_exited" => {
                // These methods should have one VRCPlayerApi parameter
                if method.parameters.len() != 1 {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            "on_post_deserialization" => {
                // This method should have no parameters
                if !method.parameters.is_empty() {
                    return Err(ValidationError::InvalidMethodSignature {
                        struct_name: struct_name.to_string(),
                        method_name: method.name.clone(),
                        expected: expected_signature,
                        found: actual_signature,
                    });
                }
            }
            _ => {
                // Custom methods - less strict validation
            }
        }

        Ok(())
    }

    /// Check if a method name is a UdonBehaviour trait method
    fn is_trait_method(&self, method_name: &str) -> bool {
        self.required_methods.contains(method_name) || 
        self.optional_methods.contains(method_name)
    }

    /// Get expected signature for a method
    fn get_expected_signature(&self, method_name: &str) -> String {
        match method_name {
            "start" | "update" | "fixed_update" | "late_update" | 
            "on_enable" | "on_disable" | "on_destroy" => {
                "fn {}(&mut self)".to_string()
            }
            "on_trigger_enter" | "on_trigger_exit" | "on_trigger_stay" => {
                "fn {}(&mut self, other: Collider)".to_string()
            }
            "on_collision_enter" | "on_collision_exit" | "on_collision_stay" => {
                "fn {}(&mut self, collision: Collision)".to_string()
            }
            "on_player_joined" | "on_player_left" |
            "on_player_trigger_enter" | "on_player_trigger_exit" | "on_player_trigger_stay" |
            "on_player_collision_enter" | "on_player_collision_exit" | "on_player_collision_stay" |
            "on_station_entered" | "on_station_exited" => {
                "fn {}(&mut self, player: VRCPlayerApi)".to_string()
            }
            "on_pickup" | "on_drop" | "on_pickup_use_down" | "on_pickup_use_up" |
            "on_post_deserialization" => {
                "fn {}(&mut self)".to_string()
            }
            _ => {
                "fn {}(&mut self, ...)".to_string()
            }
        }
    }

    /// Format a method signature for display
    fn format_method_signature(&self, method: &StructMethod) -> String {
        let params: Vec<String> = method.parameters.iter()
            .map(|p| format!("{}: {:?}", p.name, p.param_type))
            .collect();
        
        format!("fn {}(&mut self{}{})", 
                method.name,
                if params.is_empty() { "" } else { ", " },
                params.join(", "))
    }

    /// Generate helpful error messages with implementation guidance
    pub fn generate_implementation_guidance(&self, struct_name: &str, missing_methods: &[String]) -> String {
        let mut guidance = format!("To fix the missing UdonBehaviour trait implementation for '{}':\n\n", struct_name);
        
        guidance.push_str("impl UdonBehaviour for ");
        guidance.push_str(struct_name);
        guidance.push_str(" {\n");
        
        for method in missing_methods {
            let signature = self.get_expected_signature(method);
            guidance.push_str("    ");
            guidance.push_str(&signature.replace("{}", method));
            guidance.push_str(" {\n");
            guidance.push_str("        // TODO: Implement ");
            guidance.push_str(method);
            guidance.push_str(" logic\n");
            guidance.push_str("    }\n\n");
        }
        
        guidance.push_str("}\n");
        guidance
    }

    /// Validate multiple structs and collect all errors
    pub fn validate_multiple_structs(&self, structs: &[UdonBehaviourStruct]) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        for udon_struct in structs {
            if let Err(error) = self.validate_struct(udon_struct) {
                errors.push(error);
            }
        }
        
        errors
    }

    /// Check if a struct has complete trait implementation
    pub fn is_implementation_complete(&self, udon_struct: &UdonBehaviourStruct) -> bool {
        if let Some(trait_impl) = &udon_struct.trait_impl {
            self.required_methods.iter()
                .all(|method| trait_impl.implemented_methods.contains(method))
        } else {
            false
        }
    }

    /// Get list of missing required methods
    pub fn get_missing_required_methods(&self, udon_struct: &UdonBehaviourStruct) -> Vec<String> {
        if let Some(trait_impl) = &udon_struct.trait_impl {
            self.required_methods.iter()
                .filter(|method| !trait_impl.implemented_methods.contains(*method))
                .cloned()
                .collect()
        } else {
            self.required_methods.iter().cloned().collect()
        }
    }

    /// Get list of implemented optional methods
    pub fn get_implemented_optional_methods(&self, udon_struct: &UdonBehaviourStruct) -> Vec<String> {
        if let Some(trait_impl) = &udon_struct.trait_impl {
            trait_impl.implemented_methods.iter()
                .filter(|method| self.optional_methods.contains(*method))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for TraitValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::{RustType, MethodParameter};

    fn create_test_struct(name: &str) -> UdonBehaviourStruct {
        UdonBehaviourStruct::new(name.to_string())
    }

    fn create_test_method(name: &str) -> StructMethod {
        StructMethod::new(name.to_string(), RustType::Unit)
    }

    #[test]
    fn test_validator_creation() {
        let validator = TraitValidator::new();
        assert!(validator.required_methods.contains("start"));
        assert!(validator.optional_methods.contains("update"));
        assert!(validator.unity_event_methods.contains("start"));
    }

    #[test]
    fn test_missing_trait_implementation() {
        let validator = TraitValidator::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        let result = validator.validate_struct(&test_struct);
        assert!(result.is_err());
        
        if let Err(ValidationError::MissingTraitImplementation { struct_name }) = result {
            assert_eq!(struct_name, "TestBehavior");
        } else {
            panic!("Expected MissingTraitImplementation error");
        }
    }

    #[test]
    fn test_missing_required_methods() {
        let validator = TraitValidator::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        // Add empty trait implementation
        let trait_impl = UdonBehaviourTraitImpl::new();
        test_struct.set_trait_impl(trait_impl);
        
        let result = validator.validate_struct(&test_struct);
        assert!(result.is_err());
        
        if let Err(ValidationError::MissingRequiredMethods { struct_name, missing_methods }) = result {
            assert_eq!(struct_name, "TestBehavior");
            assert!(missing_methods.contains(&"start".to_string()));
        } else {
            panic!("Expected MissingRequiredMethods error");
        }
    }

    #[test]
    fn test_valid_implementation() {
        let validator = TraitValidator::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        // Add trait implementation with required method
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        // Add the start method
        let start_method = create_test_method("start");
        test_struct.add_method(start_method);
        
        let result = validator.validate_struct(&test_struct);
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_method_not_supported() {
        let validator = TraitValidator::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        test_struct.set_trait_impl(trait_impl);
        
        // Add async start method
        let mut start_method = create_test_method("start");
        start_method.set_async(true);
        test_struct.add_method(start_method);
        
        let result = validator.validate_struct(&test_struct);
        assert!(result.is_err());
        
        if let Err(ValidationError::AsyncMethodNotSupported { struct_name, method_name }) = result {
            assert_eq!(struct_name, "TestBehavior");
            assert_eq!(method_name, "start");
        } else {
            panic!("Expected AsyncMethodNotSupported error");
        }
    }

    #[test]
    fn test_method_signature_validation() {
        let validator = TraitValidator::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("on_player_joined".to_string());
        test_struct.set_trait_impl(trait_impl);
        
        // Add method with wrong signature (should have VRCPlayerApi parameter)
        let mut method = create_test_method("on_player_joined");
        // Add wrong parameter type
        let param = MethodParameter::new("player".to_string(), RustType::I32);
        method.add_parameter(param);
        test_struct.add_method(method);
        
        // This should pass basic validation since we're not strictly checking parameter types yet
        let result = validator.validate_struct(&test_struct);
        // For now, this passes because we only check parameter count, not types
        assert!(result.is_ok());
    }

    #[test]
    fn test_implementation_guidance() {
        let validator = TraitValidator::new();
        let missing_methods = vec!["start".to_string(), "update".to_string()];
        
        let guidance = validator.generate_implementation_guidance("TestBehavior", &missing_methods);
        
        assert!(guidance.contains("impl UdonBehaviour for TestBehavior"));
        assert!(guidance.contains("fn start(&mut self)"));
        assert!(guidance.contains("fn update(&mut self)"));
    }

    #[test]
    fn test_multiple_struct_validation() {
        let validator = TraitValidator::new();
        
        // Create two structs, one valid and one invalid
        let mut valid_struct = create_test_struct("ValidBehavior");
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        valid_struct.set_trait_impl(trait_impl);
        valid_struct.add_method(create_test_method("start"));
        
        let invalid_struct = create_test_struct("InvalidBehavior");
        
        let structs = vec![valid_struct, invalid_struct];
        let errors = validator.validate_multiple_structs(&structs);
        
        assert_eq!(errors.len(), 1);
        if let ValidationError::MissingTraitImplementation { struct_name } = &errors[0] {
            assert_eq!(struct_name, "InvalidBehavior");
        } else {
            panic!("Expected MissingTraitImplementation error");
        }
    }

    #[test]
    fn test_is_implementation_complete() {
        let validator = TraitValidator::new();
        
        // Test incomplete implementation
        let incomplete_struct = create_test_struct("IncompleteBehavior");
        assert!(!validator.is_implementation_complete(&incomplete_struct));
        
        // Test complete implementation
        let mut complete_struct = create_test_struct("CompleteBehavior");
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        complete_struct.set_trait_impl(trait_impl);
        
        assert!(validator.is_implementation_complete(&complete_struct));
    }

    #[test]
    fn test_get_missing_required_methods() {
        let validator = TraitValidator::new();
        let test_struct = create_test_struct("TestBehavior");
        
        let missing = validator.get_missing_required_methods(&test_struct);
        assert!(missing.contains(&"start".to_string()));
    }

    #[test]
    fn test_get_implemented_optional_methods() {
        let validator = TraitValidator::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.add_method("update".to_string());
        trait_impl.add_method("on_enable".to_string());
        test_struct.set_trait_impl(trait_impl);
        
        let optional = validator.get_implemented_optional_methods(&test_struct);
        assert!(optional.contains(&"update".to_string()));
        assert!(optional.contains(&"on_enable".to_string()));
        assert!(!optional.contains(&"start".to_string())); // start is required, not optional
    }
}