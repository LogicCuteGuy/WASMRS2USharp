//! Simple test file for code_generator functionality

#[cfg(test)]
mod tests {
    use crate::code_generator::*;
    use crate::multi_behavior::*;

    #[test]
    fn test_code_generator_creation() {
        let generator = CodeGenerator::new();
        assert_eq!(generator.get_all_generated_classes().len(), 0);
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
        use crate::code_generator::{to_camel_case, to_pascal_case};
        assert_eq!(to_camel_case("test_field"), "testField");
        assert_eq!(to_camel_case("player_count"), "playerCount");
        assert_eq!(to_pascal_case("test_method"), "TestMethod");
        assert_eq!(to_pascal_case("on_player_joined"), "OnPlayerJoined");
    }

    // Additional comprehensive tests for Requirements 2.2, 2.3, 3.1, 3.2, 4.3

    #[test]
    fn test_field_types_and_attributes() {
        let generator = CodeGenerator::new();
        
        // Test different field types with various attributes
        let test_cases = vec![
            (RustType::Bool, vec![FieldAttribute::UdonPublic], "bool", "public", vec!["[SerializeField]"]),
            (RustType::I32, vec![FieldAttribute::UdonSync], "int", "private", vec!["[UdonSynced]"]),
            (RustType::F32, vec![], "float", "private", vec![]),
            (RustType::String, vec![FieldAttribute::UdonPublic, FieldAttribute::UdonSync], "string", "public", vec!["[SerializeField]", "[UdonSynced]"]),
            (RustType::GameObject, vec![FieldAttribute::UdonPublic], "GameObject", "public", vec!["[SerializeField]"]),
            (RustType::Vector3, vec![FieldAttribute::UdonSync], "Vector3", "private", vec!["[UdonSynced]"]),
        ];
        
        for (rust_type, attributes, expected_type, expected_visibility, expected_attributes) in test_cases {
            let mut field = StructField::new("test_field".to_string(), rust_type);
            for attr in attributes {
                field.add_attribute(attr);
            }
            
            let generated = generator.generate_single_field(&field).unwrap();
            
            assert_eq!(generated.field_type, expected_type);
            assert_eq!(generated.visibility, expected_visibility);
            for expected_attr in expected_attributes {
                assert!(generated.attributes.contains(&expected_attr.to_string()), 
                       "Missing attribute {} for type {:?}", expected_attr, rust_type);
            }
        }
    }

    #[test]
    fn test_unity_event_method_generation() {
        let generator = CodeGenerator::new();
        
        // Test all Unity event methods
        let unity_methods = vec![
            ("start", "Start"),
            ("update", "Update"),
            ("fixed_update", "FixedUpdate"),
            ("late_update", "LateUpdate"),
            ("on_enable", "OnEnable"),
            ("on_disable", "OnDisable"),
            ("on_player_joined", "OnPlayerJoined"),
            ("on_player_left", "OnPlayerLeft"),
            ("on_player_respawn", "OnPlayerRespawn"),
            ("on_post_deserialization", "OnDeserialization"),
        ];
        
        for (rust_method, expected_csharp) in unity_methods {
            let mapped = generator.map_unity_method_name(rust_method);
            assert_eq!(mapped, Some(expected_csharp.to_string()), 
                      "Failed to map {} to {}", rust_method, expected_csharp);
        }
    }

    #[test]
    fn test_custom_event_handler_generation() {
        let generator = CodeGenerator::new();
        
        // Create a struct with custom event handlers
        let mut test_struct = UdonBehaviourStruct::new("EventBehavior".to_string());
        
        // Add custom event methods
        let mut event_method1 = StructMethod::new("on_score_changed".to_string());
        event_method1.add_attribute(MethodAttribute::UdonEvent("OnScoreChanged".to_string()));
        event_method1.add_parameter(MethodParameter::new("new_score".to_string(), RustType::I32));
        test_struct.add_method(event_method1);
        
        let mut event_method2 = StructMethod::new("on_game_state_changed".to_string());
        event_method2.add_attribute(MethodAttribute::UdonEvent("OnGameStateChanged".to_string()));
        event_method2.add_parameter(MethodParameter::new("state".to_string(), RustType::String));
        test_struct.add_method(event_method2);
        
        // Add basic trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        let generated = generator.generate_behavior_class(&test_struct).unwrap();
        
        // Verify custom event handlers are generated
        assert!(generated.source_code.contains("public void OnScoreChanged(int newScore)"));
        assert!(generated.source_code.contains("public void OnGameStateChanged(string state)"));
        
        // Verify custom events are in the custom_events list
        assert_eq!(generated.custom_events.len(), 2);
        assert!(generated.custom_events.iter().any(|e| e.name == "OnScoreChanged"));
        assert!(generated.custom_events.iter().any(|e| e.name == "OnGameStateChanged"));
    }

    #[test]
    fn test_complex_type_generation() {
        let generator = CodeGenerator::new();
        
        // Test complex types
        let complex_types = vec![
            (RustType::Option(Box::new(RustType::GameObject)), "GameObject"),
            (RustType::Vec(Box::new(RustType::String)), "string[]"),
            (RustType::HashMap(Box::new(RustType::String), Box::new(RustType::I32)), "Dictionary<string, int>"),
            (RustType::Option(Box::new(RustType::Vec(Box::new(RustType::F32)))), "float[]"),
        ];
        
        for (rust_type, expected_csharp) in complex_types {
            let mut field = StructField::new("complex_field".to_string(), rust_type.clone());
            let generated = generator.generate_single_field(&field).unwrap();
            
            assert_eq!(generated.field_type, expected_csharp, 
                      "Failed to convert {:?} to {}", rust_type, expected_csharp);
        }
    }

    #[test]
    fn test_gameobject_reference_generation() {
        let generator = CodeGenerator::new();
        
        // Create struct with GameObject references for inter-behavior communication
        let mut test_struct = UdonBehaviourStruct::new("GameManager".to_string());
        
        // Add GameObject reference fields
        let mut ui_ref = StructField::new("ui_controller_ref".to_string(), 
                                         RustType::Option(Box::new(RustType::GameObject)));
        ui_ref.add_attribute(FieldAttribute::UdonPublic);
        test_struct.add_field(ui_ref);
        
        let mut score_ref = StructField::new("score_tracker_ref".to_string(), 
                                           RustType::Option(Box::new(RustType::GameObject)));
        score_ref.add_attribute(FieldAttribute::UdonPublic);
        test_struct.add_field(score_ref);
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        let generated = generator.generate_behavior_class(&test_struct).unwrap();
        
        // Verify GameObject references are generated correctly
        assert!(generated.source_code.contains("public GameObject uiControllerRef"));
        assert!(generated.source_code.contains("public GameObject scoreTrackerRef"));
        assert!(generated.source_code.contains("[SerializeField]"));
    }

    #[test]
    fn test_networking_field_generation() {
        let generator = CodeGenerator::new();
        
        // Create struct with synchronized fields
        let mut test_struct = UdonBehaviourStruct::new("NetworkedBehavior".to_string());
        
        // Add synchronized fields
        let mut sync_field1 = StructField::new("player_count".to_string(), RustType::I32);
        sync_field1.add_attribute(FieldAttribute::UdonSync);
        test_struct.add_field(sync_field1);
        
        let mut sync_field2 = StructField::new("game_state".to_string(), RustType::String);
        sync_field2.add_attribute(FieldAttribute::UdonSync);
        sync_field2.add_attribute(FieldAttribute::UdonPublic);
        test_struct.add_field(sync_field2);
        
        // Add OnDeserialization method
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.add_method("on_post_deserialization".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        let generated = generator.generate_behavior_class(&test_struct).unwrap();
        
        // Verify synchronized fields have correct attributes
        assert!(generated.source_code.contains("[UdonSynced]"));
        assert!(generated.source_code.contains("private int playerCount"));
        assert!(generated.source_code.contains("public string gameState"));
        
        // Verify OnDeserialization method is generated
        assert!(generated.source_code.contains("public override void OnDeserialization()"));
    }

    #[test]
    fn test_method_parameter_generation() {
        let generator = CodeGenerator::new();
        
        // Test method parameter conversion
        let parameter_tests = vec![
            (RustType::I32, "int"),
            (RustType::F32, "float"),
            (RustType::Bool, "bool"),
            (RustType::String, "string"),
            (RustType::GameObject, "GameObject"),
            (RustType::Vector3, "Vector3"),
            (RustType::Option(Box::new(RustType::I32)), "int"),
            (RustType::Vec(Box::new(RustType::String)), "string[]"),
        ];
        
        for (rust_type, expected_csharp) in parameter_tests {
            let param = MethodParameter::new("test_param".to_string(), rust_type.clone());
            let csharp_type = generator.convert_parameter_type(&param);
            
            assert_eq!(csharp_type, expected_csharp, 
                      "Failed to convert parameter type {:?} to {}", rust_type, expected_csharp);
        }
    }

    #[test]
    fn test_class_inheritance_and_using_statements() {
        let generator = CodeGenerator::new();
        
        // Create a comprehensive test struct
        let mut test_struct = UdonBehaviourStruct::new("ComprehensiveBehavior".to_string());
        
        // Add various field types to trigger different using statements
        let mut unity_field = StructField::new("transform_ref".to_string(), RustType::Transform);
        test_struct.add_field(unity_field);
        
        let mut vrchat_field = StructField::new("player_api".to_string(), RustType::VRCPlayerApi);
        test_struct.add_field(vrchat_field);
        
        let mut collection_field = StructField::new("string_list".to_string(), 
                                                   RustType::Vec(Box::new(RustType::String)));
        test_struct.add_field(collection_field);
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        let generated = generator.generate_behavior_class(&test_struct).unwrap();
        
        // Verify proper using statements are included
        assert!(generated.using_statements.contains(&"using UnityEngine;".to_string()));
        assert!(generated.using_statements.contains(&"using UdonSharp;".to_string()));
        assert!(generated.using_statements.contains(&"using VRC.SDKBase;".to_string()));
        assert!(generated.using_statements.contains(&"using System.Collections.Generic;".to_string()));
        
        // Verify class inheritance
        assert!(generated.source_code.contains("public class ComprehensiveBehavior : UdonSharpBehaviour"));
    }

    #[test]
    fn test_error_handling_in_generation() {
        let generator = CodeGenerator::new();
        
        // Test invalid struct name
        let invalid_struct = UdonBehaviourStruct::new("123InvalidName".to_string());
        let result = generator.generate_behavior_class(&invalid_struct);
        assert!(result.is_err(), "Should fail for invalid struct name");
        
        // Test struct without trait implementation
        let mut no_trait_struct = UdonBehaviourStruct::new("NoTraitBehavior".to_string());
        let field = StructField::new("test_field".to_string(), RustType::I32);
        no_trait_struct.add_field(field);
        
        let result = generator.generate_behavior_class(&no_trait_struct);
        assert!(result.is_err(), "Should fail for struct without trait implementation");
    }

    #[test]
    fn test_field_name_conversion() {
        let generator = CodeGenerator::new();
        
        // Test field name conversion from snake_case to camelCase
        let name_tests = vec![
            ("player_count", "playerCount"),
            ("ui_controller_ref", "uiControllerRef"),
            ("game_state", "gameState"),
            ("is_active", "isActive"),
            ("max_health_points", "maxHealthPoints"),
            ("simple", "simple"),
            ("UPPERCASE", "UPPERCASE"), // Should remain unchanged
        ];
        
        for (rust_name, expected_csharp) in name_tests {
            let field = StructField::new(rust_name.to_string(), RustType::I32);
            let generated = generator.generate_single_field(&field).unwrap();
            
            assert_eq!(generated.name, expected_csharp, 
                      "Failed to convert field name {} to {}", rust_name, expected_csharp);
        }
    }

    #[test]
    fn test_method_signature_generation() {
        let generator = CodeGenerator::new();
        
        // Create a struct with various method signatures
        let mut test_struct = UdonBehaviourStruct::new("MethodTestBehavior".to_string());
        
        // Method with no parameters
        let mut no_param_method = StructMethod::new("simple_method".to_string());
        no_param_method.add_attribute(MethodAttribute::UdonEvent("SimpleMethod".to_string()));
        test_struct.add_method(no_param_method);
        
        // Method with single parameter
        let mut single_param_method = StructMethod::new("single_param_method".to_string());
        single_param_method.add_attribute(MethodAttribute::UdonEvent("SingleParamMethod".to_string()));
        single_param_method.add_parameter(MethodParameter::new("value".to_string(), RustType::I32));
        test_struct.add_method(single_param_method);
        
        // Method with multiple parameters
        let mut multi_param_method = StructMethod::new("multi_param_method".to_string());
        multi_param_method.add_attribute(MethodAttribute::UdonEvent("MultiParamMethod".to_string()));
        multi_param_method.add_parameter(MethodParameter::new("name".to_string(), RustType::String));
        multi_param_method.add_parameter(MethodParameter::new("score".to_string(), RustType::I32));
        multi_param_method.add_parameter(MethodParameter::new("is_winner".to_string(), RustType::Bool));
        test_struct.add_method(multi_param_method);
        
        // Add trait implementation
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        test_struct.set_trait_impl(trait_impl);
        
        let generated = generator.generate_behavior_class(&test_struct).unwrap();
        
        // Verify method signatures
        assert!(generated.source_code.contains("public void SimpleMethod()"));
        assert!(generated.source_code.contains("public void SingleParamMethod(int value)"));
        assert!(generated.source_code.contains("public void MultiParamMethod(string name, int score, bool isWinner)"));
    }
}