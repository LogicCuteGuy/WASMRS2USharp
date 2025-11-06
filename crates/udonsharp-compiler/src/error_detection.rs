//! Comprehensive compilation error detection system for multi-behavior patterns
//! 
//! This module provides detailed error detection and analysis for UdonBehaviour
//! compilation, focusing on trait implementations, attribute usage, and type validation.

use crate::multi_behavior::{UdonBehaviourStruct, StructField, FieldAttribute, RustType};
use crate::trait_validator::{TraitValidator, ValidationError};
use crate::struct_analyzer::AnalysisError;
use udonsharp_core::error::{UdonSharpError, Diagnostic, DiagnosticLevel};
use std::collections::{HashMap, HashSet};

/// Comprehensive error detection system for multi-behavior compilation
pub struct CompilationErrorDetector {
    /// Trait validator for UdonBehaviour implementations
    trait_validator: TraitValidator,
    /// Supported Rust types for UdonSharp
    supported_types: HashSet<String>,
    /// Valid attribute combinations
    valid_attribute_combinations: HashMap<String, Vec<Vec<FieldAttribute>>>,
    /// Error patterns and their solutions
    error_patterns: HashMap<String, ErrorSolution>,
}

impl CompilationErrorDetector {
    /// Create a new compilation error detector
    pub fn new() -> Self {
        let mut detector = Self {
            trait_validator: TraitValidator::new(),
            supported_types: HashSet::new(),
            valid_attribute_combinations: HashMap::new(),
            error_patterns: HashMap::new(),
        };
        
        detector.initialize_supported_types();
        detector.initialize_valid_attribute_combinations();
        detector.initialize_error_patterns();
        detector
    }
    
    /// Initialize supported Rust types for UdonSharp compilation
    fn initialize_supported_types(&mut self) {
        // Basic types
        self.supported_types.insert("bool".to_string());
        self.supported_types.insert("i8".to_string());
        self.supported_types.insert("i16".to_string());
        self.supported_types.insert("i32".to_string());
        self.supported_types.insert("i64".to_string());
        self.supported_types.insert("u8".to_string());
        self.supported_types.insert("u16".to_string());
        self.supported_types.insert("u32".to_string());
        self.supported_types.insert("u64".to_string());
        self.supported_types.insert("f32".to_string());
        self.supported_types.insert("f64".to_string());
        self.supported_types.insert("char".to_string());
        self.supported_types.insert("String".to_string());
        
        // Unity types
        self.supported_types.insert("Vector2".to_string());
        self.supported_types.insert("Vector3".to_string());
        self.supported_types.insert("Vector4".to_string());
        self.supported_types.insert("Quaternion".to_string());
        self.supported_types.insert("Color".to_string());
        self.supported_types.insert("Color32".to_string());
        self.supported_types.insert("GameObject".to_string());
        self.supported_types.insert("Transform".to_string());
        
        // VRChat types
        self.supported_types.insert("VRCPlayerApi".to_string());
        
        // Generic types (handled specially)
        self.supported_types.insert("Option".to_string());
        self.supported_types.insert("Vec".to_string());
        self.supported_types.insert("HashMap".to_string());
    }
    
    /// Initialize valid field attribute combinations
    fn initialize_valid_attribute_combinations(&mut self) {
        // udon_public can be combined with header and tooltip
        self.valid_attribute_combinations.insert(
            "udon_public".to_string(),
            vec![
                vec![FieldAttribute::UdonPublic],
                vec![FieldAttribute::UdonPublic, FieldAttribute::Header("".to_string())],
                vec![FieldAttribute::UdonPublic, FieldAttribute::Tooltip("".to_string())],
                vec![FieldAttribute::UdonPublic, FieldAttribute::Header("".to_string()), FieldAttribute::Tooltip("".to_string())],
            ]
        );
        
        // udon_sync can be combined with udon_public
        self.valid_attribute_combinations.insert(
            "udon_sync".to_string(),
            vec![
                vec![FieldAttribute::UdonSync],
                vec![FieldAttribute::UdonSync, FieldAttribute::UdonPublic],
                vec![FieldAttribute::UdonSync, FieldAttribute::UdonPublic, FieldAttribute::Header("".to_string())],
                vec![FieldAttribute::UdonSync, FieldAttribute::UdonPublic, FieldAttribute::Tooltip("".to_string())],
            ]
        );
        
        // header and tooltip are decorative and can be combined with others
        self.valid_attribute_combinations.insert(
            "header".to_string(),
            vec![
                vec![FieldAttribute::Header("".to_string())],
                vec![FieldAttribute::Header("".to_string()), FieldAttribute::Tooltip("".to_string())],
            ]
        );
    }
    
    /// Initialize error patterns and their solutions
    fn initialize_error_patterns(&mut self) {
        self.error_patterns.insert(
            "missing_trait_implementation".to_string(),
            ErrorSolution {
                pattern: "missing.*trait.*implementation".to_string(),
                category: ErrorCategory::TraitImplementation,
                severity: DiagnosticLevel::Error,
                solution: "Add 'impl UdonBehaviour for YourStruct { ... }' with required methods".to_string(),
                code_example: Some(r#"
impl UdonBehaviour for YourStruct {
    fn start(&mut self) {
        // Initialize your behavior
    }
}
"#.to_string()),
                documentation_link: Some("docs/api-reference.md#udon-behaviour-trait".to_string()),
            }
        );
        
        self.error_patterns.insert(
            "invalid_attribute_combination".to_string(),
            ErrorSolution {
                pattern: "invalid.*attribute.*combination".to_string(),
                category: ErrorCategory::AttributeUsage,
                severity: DiagnosticLevel::Error,
                solution: "Check attribute compatibility and remove conflicting attributes".to_string(),
                code_example: Some(r#"
// Correct usage:
#[udon_public]
#[header("Player Settings")]
pub player_name: String,

// Incorrect usage:
#[udon_sync]
#[udon_sync] // Duplicate attribute
pub score: i32,
"#.to_string()),
                documentation_link: Some("docs/api-reference.md#field-attributes".to_string()),
            }
        );
        
        self.error_patterns.insert(
            "unsupported_type".to_string(),
            ErrorSolution {
                pattern: "unsupported.*type".to_string(),
                category: ErrorCategory::TypeValidation,
                severity: DiagnosticLevel::Error,
                solution: "Use UdonSharp-compatible types or wrap in supported containers".to_string(),
                code_example: Some(r#"
// Supported types:
pub score: i32,
pub position: Vector3,
pub target: Option<GameObject>,

// Unsupported types:
pub complex_data: HashMap<String, Vec<CustomStruct>>, // Too complex
pub callback: fn() -> (), // Function pointers not supported
"#.to_string()),
                documentation_link: Some("docs/api-reference.md#supported-types".to_string()),
            }
        );
    }
    
    /// Detect missing UdonBehaviour trait implementations
    pub fn detect_missing_trait_implementations(&self, structs: &[UdonBehaviourStruct]) -> Vec<CompilationError> {
        let mut errors = Vec::new();
        
        for udon_struct in structs {
            if udon_struct.trait_impl.is_none() {
                errors.push(CompilationError {
                    error_type: ErrorType::MissingTraitImplementation,
                    struct_name: Some(udon_struct.name.clone()),
                    field_name: None,
                    method_name: None,
                    message: format!("Struct '{}' must implement the UdonBehaviour trait", udon_struct.name),
                    suggestion: Some(format!(
                        "Add the following implementation:\n\nimpl UdonBehaviour for {} {{\n    fn start(&mut self) {{\n        // Initialize your behavior\n    }}\n}}",
                        udon_struct.name
                    )),
                    code_example: Some(self.generate_trait_implementation_example(&udon_struct.name)),
                    severity: DiagnosticLevel::Error,
                    source_location: None,
                });
            } else {
                // Check for missing required methods
                let missing_methods = self.trait_validator.get_missing_required_methods(udon_struct);
                if !missing_methods.is_empty() {
                    errors.push(CompilationError {
                        error_type: ErrorType::MissingRequiredMethods,
                        struct_name: Some(udon_struct.name.clone()),
                        field_name: None,
                        method_name: None,
                        message: format!(
                            "Struct '{}' is missing required UdonBehaviour methods: {}",
                            udon_struct.name,
                            missing_methods.join(", ")
                        ),
                        suggestion: Some(format!(
                            "Add the missing methods to your UdonBehaviour implementation:\n{}",
                            self.trait_validator.generate_implementation_guidance(&udon_struct.name, &missing_methods)
                        )),
                        code_example: Some(self.generate_missing_methods_example(&udon_struct.name, &missing_methods)),
                        severity: DiagnosticLevel::Error,
                        source_location: None,
                    });
                }
            }
        }
        
        errors
    }
    
    /// Validate field attribute usage and combinations
    pub fn validate_field_attributes(&self, structs: &[UdonBehaviourStruct]) -> Vec<CompilationError> {
        let mut errors = Vec::new();
        
        for udon_struct in structs {
            for field in &udon_struct.fields {
                // Check for duplicate attributes
                let duplicate_errors = self.check_duplicate_attributes(udon_struct, field);
                errors.extend(duplicate_errors);
                
                // Check for invalid attribute combinations
                let combination_errors = self.check_invalid_attribute_combinations(udon_struct, field);
                errors.extend(combination_errors);
                
                // Check attribute-specific validation
                let specific_errors = self.validate_specific_attributes(udon_struct, field);
                errors.extend(specific_errors);
            }
        }
        
        errors
    }
    
    /// Check for duplicate field attributes
    fn check_duplicate_attributes(&self, udon_struct: &UdonBehaviourStruct, field: &StructField) -> Vec<CompilationError> {
        let mut errors = Vec::new();
        let mut seen_attributes = HashSet::new();
        
        for attribute in &field.attributes {
            let attr_key = self.get_attribute_key(attribute);
            if seen_attributes.contains(&attr_key) {
                errors.push(CompilationError {
                    error_type: ErrorType::DuplicateAttribute,
                    struct_name: Some(udon_struct.name.clone()),
                    field_name: Some(field.name.clone()),
                    method_name: None,
                    message: format!(
                        "Duplicate attribute '{}' on field '{}' in struct '{}'",
                        attr_key, field.name, udon_struct.name
                    ),
                    suggestion: Some("Remove the duplicate attribute".to_string()),
                    code_example: Some(format!(
                        "// Correct:\n#[{}]\npub {}: {},\n\n// Incorrect:\n#[{}]\n#[{}] // Duplicate\npub {}: {},",
                        attr_key, field.name, self.format_type(&field.field_type),
                        attr_key, attr_key, field.name, self.format_type(&field.field_type)
                    )),
                    severity: DiagnosticLevel::Error,
                    source_location: None,
                });
            }
            seen_attributes.insert(attr_key);
        }
        
        errors
    }
    
    /// Check for invalid attribute combinations
    fn check_invalid_attribute_combinations(&self, udon_struct: &UdonBehaviourStruct, field: &StructField) -> Vec<CompilationError> {
        let mut errors = Vec::new();
        
        // Check if udon_sync is used with incompatible types
        let has_sync = field.attributes.iter().any(|attr| matches!(attr, FieldAttribute::UdonSync));
        if has_sync && !self.is_type_syncable(&field.field_type) {
            errors.push(CompilationError {
                error_type: ErrorType::InvalidAttributeUsage,
                struct_name: Some(udon_struct.name.clone()),
                field_name: Some(field.name.clone()),
                method_name: None,
                message: format!(
                    "Field '{}' with type '{}' cannot use #[udon_sync] attribute",
                    field.name, self.format_type(&field.field_type)
                ),
                suggestion: Some("Use a syncable type or remove the #[udon_sync] attribute".to_string()),
                code_example: Some(format!(
                    "// Syncable types:\npub score: i32,\npub position: Vector3,\npub is_active: bool,\n\n// Non-syncable types:\n// pub complex_data: HashMap<String, Vec<i32>>, // Too complex for sync"
                )),
                severity: DiagnosticLevel::Error,
                source_location: None,
            });
        }
        
        errors
    }
    
    /// Validate specific attribute requirements
    fn validate_specific_attributes(&self, udon_struct: &UdonBehaviourStruct, field: &StructField) -> Vec<CompilationError> {
        let mut errors = Vec::new();
        
        for attribute in &field.attributes {
            match attribute {
                FieldAttribute::Header(text) => {
                    if text.is_empty() {
                        errors.push(CompilationError {
                            error_type: ErrorType::InvalidAttributeUsage,
                            struct_name: Some(udon_struct.name.clone()),
                            field_name: Some(field.name.clone()),
                            method_name: None,
                            message: format!(
                                "Header attribute on field '{}' cannot have empty text",
                                field.name
                            ),
                            suggestion: Some("Provide descriptive text for the header".to_string()),
                            code_example: Some(format!(
                                "#[header(\"Player Settings\")]\npub {}: {},",
                                field.name, self.format_type(&field.field_type)
                            )),
                            severity: DiagnosticLevel::Warning,
                            source_location: None,
                        });
                    }
                }
                FieldAttribute::Tooltip(text) => {
                    if text.is_empty() {
                        errors.push(CompilationError {
                            error_type: ErrorType::InvalidAttributeUsage,
                            struct_name: Some(udon_struct.name.clone()),
                            field_name: Some(field.name.clone()),
                            method_name: None,
                            message: format!(
                                "Tooltip attribute on field '{}' cannot have empty text",
                                field.name
                            ),
                            suggestion: Some("Provide helpful tooltip text".to_string()),
                            code_example: Some(format!(
                                "#[tooltip(\"The player's current score\")]\npub {}: {},",
                                field.name, self.format_type(&field.field_type)
                            )),
                            severity: DiagnosticLevel::Warning,
                            source_location: None,
                        });
                    }
                }
                _ => {} // Other attributes are valid by default
            }
        }
        
        errors
    }
    
    /// Check for unsupported Rust features and types
    pub fn check_unsupported_features(&self, structs: &[UdonBehaviourStruct]) -> Vec<CompilationError> {
        let mut errors = Vec::new();
        
        for udon_struct in structs {
            // Check struct fields for unsupported types
            for field in &udon_struct.fields {
                if !self.is_type_supported(&field.field_type) {
                    let alternatives = self.get_type_alternatives(&field.field_type);
                    errors.push(CompilationError {
                        error_type: ErrorType::UnsupportedType,
                        struct_name: Some(udon_struct.name.clone()),
                        field_name: Some(field.name.clone()),
                        method_name: None,
                        message: format!(
                            "Unsupported type '{}' for field '{}' in struct '{}'",
                            self.format_type(&field.field_type), field.name, udon_struct.name
                        ),
                        suggestion: Some(format!(
                            "Consider using one of these supported alternatives: {}",
                            alternatives.join(", ")
                        )),
                        code_example: Some(self.generate_type_alternative_example(&field.field_type, &alternatives)),
                        severity: DiagnosticLevel::Error,
                        source_location: None,
                    });
                }
            }
            
            // Check methods for unsupported features
            for method in &udon_struct.methods {
                if method.is_async {
                    errors.push(CompilationError {
                        error_type: ErrorType::UnsupportedFeature,
                        struct_name: Some(udon_struct.name.clone()),
                        field_name: None,
                        method_name: Some(method.name.clone()),
                        message: format!(
                            "Async method '{}' is not supported in UdonBehaviour trait implementations",
                            method.name
                        ),
                        suggestion: Some("Remove the 'async' keyword and use synchronous code".to_string()),
                        code_example: Some(format!(
                            "// Correct:\nfn {}(&mut self) {{\n    // Synchronous implementation\n}}\n\n// Incorrect:\nasync fn {}(&mut self) {{\n    // Async not supported\n}}",
                            method.name, method.name
                        )),
                        severity: DiagnosticLevel::Error,
                        source_location: None,
                    });
                }
            }
        }
        
        errors
    }
    
    /// Generate comprehensive error report
    pub fn generate_error_report(&self, structs: &[UdonBehaviourStruct]) -> CompilationErrorReport {
        let mut all_errors = Vec::new();
        
        // Detect all types of errors
        all_errors.extend(self.detect_missing_trait_implementations(structs));
        all_errors.extend(self.validate_field_attributes(structs));
        all_errors.extend(self.check_unsupported_features(structs));
        
        // Categorize errors
        let mut errors_by_category = HashMap::new();
        for error in &all_errors {
            let category = self.categorize_error(error);
            errors_by_category.entry(category).or_insert_with(Vec::new).push(error.clone());
        }
        
        // Generate statistics
        let error_count = all_errors.iter().filter(|e| e.severity == DiagnosticLevel::Error).count();
        let warning_count = all_errors.iter().filter(|e| e.severity == DiagnosticLevel::Warning).count();
        
        CompilationErrorReport {
            errors: all_errors,
            errors_by_category,
            error_count,
            warning_count,
            structs_analyzed: structs.len(),
            has_blocking_errors: error_count > 0,
        }
    }
    
    // Helper methods
    
    fn get_attribute_key(&self, attribute: &FieldAttribute) -> String {
        match attribute {
            FieldAttribute::UdonPublic => "udon_public".to_string(),
            FieldAttribute::UdonSync => "udon_sync".to_string(),
            FieldAttribute::Header(_) => "header".to_string(),
            FieldAttribute::Tooltip(_) => "tooltip".to_string(),
        }
    }
    
    fn is_type_syncable(&self, rust_type: &RustType) -> bool {
        match rust_type {
            RustType::Bool | RustType::I32 | RustType::F32 | RustType::String => true,
            RustType::Vector2 | RustType::Vector3 | RustType::Quaternion | RustType::Color => true,
            RustType::Option(inner) => self.is_type_syncable(inner),
            _ => false,
        }
    }
    
    fn is_type_supported(&self, rust_type: &RustType) -> bool {
        match rust_type {
            RustType::Bool | RustType::I8 | RustType::I16 | RustType::I32 | RustType::I64 => true,
            RustType::U8 | RustType::U16 | RustType::U32 | RustType::U64 => true,
            RustType::F32 | RustType::F64 | RustType::Char | RustType::String => true,
            RustType::Vector2 | RustType::Vector3 | RustType::Vector4 => true,
            RustType::Quaternion | RustType::Color | RustType::Color32 => true,
            RustType::GameObject | RustType::Transform | RustType::VRCPlayerApi => true,
            RustType::Option(inner) => self.is_type_supported(inner),
            RustType::Vec(inner) => self.is_type_supported(inner),
            RustType::HashMap(key, value) => self.is_type_supported(key) && self.is_type_supported(value),
            RustType::Custom(_) => true, // Allow custom types, they'll be validated later
            RustType::Unit => true,
            _ => false,
        }
    }
    
    fn get_type_alternatives(&self, rust_type: &RustType) -> Vec<String> {
        match rust_type {
            RustType::I128 | RustType::U128 => vec!["i64".to_string(), "u64".to_string()],
            _ => vec!["i32".to_string(), "String".to_string(), "Vector3".to_string()],
        }
    }
    
    fn format_type(&self, rust_type: &RustType) -> String {
        format!("{:?}", rust_type)
    }
    
    fn generate_trait_implementation_example(&self, struct_name: &str) -> String {
        format!(
            r#"impl UdonBehaviour for {} {{
    fn start(&mut self) {{
        // Initialize your behavior here
        // This method is called when the behavior starts
    }}
    
    // Optional: Add other Unity event methods
    fn update(&mut self) {{
        // Called every frame
    }}
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {{
        // Called when a player joins
    }}
}}"#,
            struct_name
        )
    }
    
    fn generate_missing_methods_example(&self, struct_name: &str, missing_methods: &[String]) -> String {
        let mut example = format!("impl UdonBehaviour for {} {{\n", struct_name);
        
        for method in missing_methods {
            match method.as_str() {
                "start" => {
                    example.push_str("    fn start(&mut self) {\n");
                    example.push_str("        // Initialize your behavior\n");
                    example.push_str("    }\n\n");
                }
                "update" => {
                    example.push_str("    fn update(&mut self) {\n");
                    example.push_str("        // Update logic called every frame\n");
                    example.push_str("    }\n\n");
                }
                _ => {
                    example.push_str(&format!("    fn {}(&mut self) {{\n", method));
                    example.push_str(&format!("        // Implement {} logic\n", method));
                    example.push_str("    }\n\n");
                }
            }
        }
        
        example.push('}');
        example
    }
    
    fn generate_type_alternative_example(&self, _rust_type: &RustType, alternatives: &[String]) -> String {
        format!(
            "// Consider using one of these supported types:\n{}",
            alternatives.iter()
                .map(|alt| format!("pub field_name: {},", alt))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
    
    fn categorize_error(&self, error: &CompilationError) -> ErrorCategory {
        match error.error_type {
            ErrorType::MissingTraitImplementation | ErrorType::MissingRequiredMethods => ErrorCategory::TraitImplementation,
            ErrorType::InvalidAttributeUsage | ErrorType::DuplicateAttribute => ErrorCategory::AttributeUsage,
            ErrorType::UnsupportedType | ErrorType::UnsupportedFeature => ErrorCategory::TypeValidation,
        }
    }
}

impl Default for CompilationErrorDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a compilation error with detailed information
#[derive(Debug, Clone)]
pub struct CompilationError {
    pub error_type: ErrorType,
    pub struct_name: Option<String>,
    pub field_name: Option<String>,
    pub method_name: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
    pub code_example: Option<String>,
    pub severity: DiagnosticLevel,
    pub source_location: Option<SourceLocation>,
}

/// Types of compilation errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    MissingTraitImplementation,
    MissingRequiredMethods,
    InvalidAttributeUsage,
    DuplicateAttribute,
    UnsupportedType,
    UnsupportedFeature,
}

/// Error categories for organization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    TraitImplementation,
    AttributeUsage,
    TypeValidation,
}

/// Source location information
#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// Error solution with examples and documentation
#[derive(Debug, Clone)]
pub struct ErrorSolution {
    pub pattern: String,
    pub category: ErrorCategory,
    pub severity: DiagnosticLevel,
    pub solution: String,
    pub code_example: Option<String>,
    pub documentation_link: Option<String>,
}

/// Comprehensive compilation error report
#[derive(Debug, Clone)]
pub struct CompilationErrorReport {
    pub errors: Vec<CompilationError>,
    pub errors_by_category: HashMap<ErrorCategory, Vec<CompilationError>>,
    pub error_count: usize,
    pub warning_count: usize,
    pub structs_analyzed: usize,
    pub has_blocking_errors: bool,
}

impl CompilationErrorReport {
    /// Check if compilation should be blocked
    pub fn should_block_compilation(&self) -> bool {
        self.has_blocking_errors
    }
    
    /// Get errors by category
    pub fn get_errors_by_category(&self, category: ErrorCategory) -> Vec<&CompilationError> {
        self.errors_by_category.get(&category).map(|errors| errors.iter().collect()).unwrap_or_default()
    }
    
    /// Generate summary text
    pub fn generate_summary(&self) -> String {
        if self.error_count == 0 && self.warning_count == 0 {
            format!("✅ Analysis completed successfully for {} struct(s)", self.structs_analyzed)
        } else {
            format!(
                "📊 Analysis completed: {} error(s), {} warning(s) in {} struct(s)",
                self.error_count, self.warning_count, self.structs_analyzed
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_behavior::*;

    fn create_test_struct(name: &str) -> UdonBehaviourStruct {
        UdonBehaviourStruct::new(name.to_string())
    }

    #[test]
    fn test_detector_creation() {
        let detector = CompilationErrorDetector::new();
        assert!(!detector.supported_types.is_empty());
        assert!(!detector.valid_attribute_combinations.is_empty());
    }

    #[test]
    fn test_missing_trait_implementation_detection() {
        let detector = CompilationErrorDetector::new();
        let test_struct = create_test_struct("TestBehavior");
        let structs = vec![test_struct];
        
        let errors = detector.detect_missing_trait_implementations(&structs);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].error_type, ErrorType::MissingTraitImplementation);
    }

    #[test]
    fn test_duplicate_attribute_detection() {
        let detector = CompilationErrorDetector::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        let mut field = StructField::new("test_field".to_string(), RustType::I32);
        field.add_attribute(FieldAttribute::UdonPublic);
        field.add_attribute(FieldAttribute::UdonPublic); // Duplicate
        test_struct.add_field(field);
        
        let structs = vec![test_struct];
        let errors = detector.validate_field_attributes(&structs);
        
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.error_type == ErrorType::DuplicateAttribute));
    }

    #[test]
    fn test_unsupported_type_detection() {
        let detector = CompilationErrorDetector::new();
        let mut test_struct = create_test_struct("TestBehavior");
        
        let field = StructField::new("test_field".to_string(), RustType::I128); // Unsupported
        test_struct.add_field(field);
        
        let structs = vec![test_struct];
        let errors = detector.check_unsupported_features(&structs);
        
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.error_type == ErrorType::UnsupportedType));
    }

    #[test]
    fn test_error_report_generation() {
        let detector = CompilationErrorDetector::new();
        let test_struct = create_test_struct("TestBehavior");
        let structs = vec![test_struct];
        
        let report = detector.generate_error_report(&structs);
        assert_eq!(report.structs_analyzed, 1);
        assert!(report.has_blocking_errors);
    }
}