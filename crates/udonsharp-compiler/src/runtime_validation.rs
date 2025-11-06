//! Runtime validation for generated C# code quality and UdonSharp compatibility
//! 
//! This module provides comprehensive validation of generated C# code to ensure
//! it follows UdonSharp conventions, has proper null handling, and meets quality standards.

use crate::code_generator::{GeneratedClass, GeneratedField, GeneratedMethod, CustomEventHandler};
use crate::error_detection::{CompilationError, ErrorType};
use udonsharp_core::error::DiagnosticLevel;
use std::collections::{HashMap, HashSet};
use regex::Regex;

/// Runtime validator for generated C# code
pub struct RuntimeValidator {
    /// C# syntax validation rules
    syntax_rules: Vec<Box<dyn ValidationRule>>,
    /// UdonSharp compatibility rules
    udonsharp_rules: Vec<Box<dyn ValidationRule>>,
    /// Code quality rules
    quality_rules: Vec<Box<dyn ValidationRule>>,
    /// Reserved C# keywords
    reserved_keywords: HashSet<String>,
    /// UdonSharp-specific patterns
    udonsharp_patterns: HashMap<String, Regex>,
}

impl RuntimeValidator {
    /// Create a new runtime validator
    pub fn new() -> Self {
        let mut validator = Self {
            syntax_rules: Vec::new(),
            udonsharp_rules: Vec::new(),
            quality_rules: Vec::new(),
            reserved_keywords: HashSet::new(),
            udonsharp_patterns: HashMap::new(),
        };
        
        validator.initialize_rules();
        validator.initialize_reserved_keywords();
        validator.initialize_udonsharp_patterns();
        validator
    }
    
    /// Initialize validation rules
    fn initialize_rules(&mut self) {
        // C# Syntax Rules
        self.syntax_rules.push(Box::new(CSharpSyntaxRule::new()));
        self.syntax_rules.push(Box::new(NamingConventionRule::new()));
        self.syntax_rules.push(Box::new(AccessModifierRule::new()));
        
        // UdonSharp Compatibility Rules
        self.udonsharp_rules.push(Box::new(UdonSharpInheritanceRule::new()));
        self.udonsharp_rules.push(Box::new(UdonSharpAttributeRule::new()));
        self.udonsharp_rules.push(Box::new(NetworkSyncRule::new()));
        self.udonsharp_rules.push(Box::new(CustomEventRule::new()));
        
        // Code Quality Rules
        self.quality_rules.push(Box::new(NullHandlingRule::new()));
        self.quality_rules.push(Box::new(ErrorHandlingRule::new()));
        self.quality_rules.push(Box::new(PerformanceRule::new()));
        self.quality_rules.push(Box::new(DocumentationRule::new()));
    }
    
    /// Initialize C# reserved keywords
    fn initialize_reserved_keywords(&mut self) {
        let keywords = [
            "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked",
            "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else",
            "enum", "event", "explicit", "extern", "false", "finally", "fixed", "float", "for",
            "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is", "lock",
            "long", "namespace", "new", "null", "object", "operator", "out", "override", "params",
            "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed",
            "short", "sizeof", "stackalloc", "static", "string", "struct", "switch", "this",
            "throw", "true", "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort",
            "using", "virtual", "void", "volatile", "while"
        ];
        
        for keyword in &keywords {
            self.reserved_keywords.insert(keyword.to_string());
        }
    }
    
    /// Initialize UdonSharp-specific patterns
    fn initialize_udonsharp_patterns(&mut self) {
        // Pattern for UdonSharp class inheritance
        self.udonsharp_patterns.insert(
            "udonsharp_inheritance".to_string(),
            Regex::new(r"class\s+\w+\s*:\s*UdonSharpBehaviour").unwrap()
        );
        
        // Pattern for synchronized fields
        self.udonsharp_patterns.insert(
            "udon_synced".to_string(),
            Regex::new(r"\[UdonSynced\]").unwrap()
        );
        
        // Pattern for serialized fields
        self.udonsharp_patterns.insert(
            "serialize_field".to_string(),
            Regex::new(r"\[SerializeField\]").unwrap()
        );
        
        // Pattern for custom events
        self.udonsharp_patterns.insert(
            "custom_event".to_string(),
            Regex::new(r#"SendCustomEvent\s*\(\s*["'][\w]+["']\s*\)"#).unwrap()
        );
        
        // Pattern for GameObject.Find calls
        self.udonsharp_patterns.insert(
            "gameobject_find".to_string(),
            Regex::new(r#"GameObject\.Find\s*\(\s*["'][\w]+["']\s*\)"#).unwrap()
        );
    }
    
    /// Validate a generated C# class
    pub fn validate_generated_class(&self, generated_class: &GeneratedClass) -> ValidationResult {
        let mut issues = Vec::new();
        
        // Validate C# syntax
        for rule in &self.syntax_rules {
            let rule_issues = rule.validate(generated_class);
            issues.extend(rule_issues);
        }
        
        // Validate UdonSharp compatibility
        for rule in &self.udonsharp_rules {
            let rule_issues = rule.validate(generated_class);
            issues.extend(rule_issues);
        }
        
        // Validate code quality
        for rule in &self.quality_rules {
            let rule_issues = rule.validate(generated_class);
            issues.extend(rule_issues);
        }
        
        // Additional source code validation
        let source_issues = self.validate_source_code(&generated_class.source_code);
        issues.extend(source_issues);
        
        let is_valid = issues.iter().all(|issue| issue.severity != DiagnosticLevel::Error);
        let suggestions = self.generate_improvement_suggestions(generated_class);
        
        ValidationResult {
            class_name: generated_class.class_name.clone(),
            issues,
            is_valid,
            suggestions,
        }
    }
    
    /// Validate the complete source code
    fn validate_source_code(&self, source_code: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for basic C# syntax issues
        issues.extend(self.check_syntax_issues(source_code));
        
        // Check UdonSharp-specific patterns
        issues.extend(self.check_udonsharp_patterns(source_code));
        
        // Check for potential runtime issues
        issues.extend(self.check_runtime_safety(source_code));
        
        issues
    }
    
    /// Check for C# syntax issues
    fn check_syntax_issues(&self, source_code: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for balanced braces
        let open_braces = source_code.matches('{').count();
        let close_braces = source_code.matches('}').count();
        if open_braces != close_braces {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::SyntaxError,
                severity: DiagnosticLevel::Error,
                message: format!("Unbalanced braces: {} open, {} close", open_braces, close_braces),
                line_number: None,
                suggestion: Some("Check for missing or extra braces in the generated code".to_string()),
                code_example: None,
            });
        }
        
        // Check for balanced parentheses
        let open_parens = source_code.matches('(').count();
        let close_parens = source_code.matches(')').count();
        if open_parens != close_parens {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::SyntaxError,
                severity: DiagnosticLevel::Error,
                message: format!("Unbalanced parentheses: {} open, {} close", open_parens, close_parens),
                line_number: None,
                suggestion: Some("Check for missing or extra parentheses in method calls".to_string()),
                code_example: None,
            });
        }
        
        // Check for proper semicolon usage
        for (line_num, line) in source_code.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && 
               !trimmed.starts_with("//") && 
               !trimmed.starts_with("/*") &&
               !trimmed.ends_with('{') && 
               !trimmed.ends_with('}') &&
               !trimmed.starts_with('[') &&
               !trimmed.starts_with('#') &&
               !trimmed.contains("class ") &&
               !trimmed.contains("namespace ") &&
               !trimmed.contains("using ") &&
               !trimmed.ends_with(';') {
                
                // Check if this line should end with a semicolon
                if trimmed.contains('=') || 
                   trimmed.contains("return ") ||
                   trimmed.contains("Debug.Log") ||
                   trimmed.contains("RequestSerialization") {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::SyntaxError,
                        severity: DiagnosticLevel::Warning,
                        message: "Statement may be missing semicolon".to_string(),
                        line_number: Some(line_num + 1),
                        suggestion: Some("Add semicolon at the end of the statement".to_string()),
                        code_example: Some(format!("{}; // Add semicolon", trimmed)),
                    });
                }
            }
        }
        
        issues
    }
    
    /// Check UdonSharp-specific patterns
    fn check_udonsharp_patterns(&self, source_code: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for proper UdonSharp inheritance
        if !self.udonsharp_patterns["udonsharp_inheritance"].is_match(source_code) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::UdonSharpCompatibility,
                severity: DiagnosticLevel::Error,
                message: "Class must inherit from UdonSharpBehaviour".to_string(),
                line_number: None,
                suggestion: Some("Ensure class declaration includes ': UdonSharpBehaviour'".to_string()),
                code_example: Some("public class YourClass : UdonSharpBehaviour".to_string()),
            });
        }
        
        // Check for proper using statements
        let required_usings = ["using UnityEngine;", "using UdonSharp;"];
        for using_stmt in &required_usings {
            if !source_code.contains(using_stmt) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::UdonSharpCompatibility,
                    severity: DiagnosticLevel::Warning,
                    message: format!("Missing required using statement: {}", using_stmt),
                    line_number: None,
                    suggestion: Some(format!("Add '{}' at the top of the file", using_stmt)),
                    code_example: Some(using_stmt.to_string()),
                });
            }
        }
        
        // Check for proper synchronized field usage
        if self.udonsharp_patterns["udon_synced"].is_match(source_code) {
            if !source_code.contains("RequestSerialization") {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::NetworkingSafety,
                    severity: DiagnosticLevel::Warning,
                    message: "UdonSynced fields detected but no RequestSerialization calls found".to_string(),
                    line_number: None,
                    suggestion: Some("Add RequestSerialization() calls when modifying synchronized fields".to_string()),
                    code_example: Some("if (Networking.IsMaster) { syncedField = newValue; RequestSerialization(); }".to_string()),
                });
            }
        }
        
        issues
    }
    
    /// Check for runtime safety issues
    fn check_runtime_safety(&self, source_code: &str) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for GameObject.Find without null checks
        if self.udonsharp_patterns["gameobject_find"].is_match(source_code) {
            let lines_with_find: Vec<_> = source_code.lines()
                .enumerate()
                .filter(|(_, line)| self.udonsharp_patterns["gameobject_find"].is_match(line))
                .collect();
            
            for (line_num, line) in lines_with_find {
                // Check if there's a null check nearby
                let context_start = line_num.saturating_sub(2);
                let context_end = (line_num + 3).min(source_code.lines().count());
                let context: String = source_code.lines()
                    .skip(context_start)
                    .take(context_end - context_start)
                    .collect::<Vec<_>>()
                    .join("\n");
                
                if !context.contains("!= null") && !context.contains("== null") {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::NullSafety,
                        severity: DiagnosticLevel::Warning,
                        message: "GameObject.Find call without null check".to_string(),
                        line_number: Some(line_num + 1),
                        suggestion: Some("Add null check after GameObject.Find".to_string()),
                        code_example: Some("var obj = GameObject.Find(\"Name\");\nif (obj != null) { /* use obj */ }".to_string()),
                    });
                }
            }
        }
        
        // Check for array access without bounds checking
        let array_access_pattern = Regex::new(r"\w+\[\w*\]").unwrap();
        for (line_num, line) in source_code.lines().enumerate() {
            if array_access_pattern.is_match(line) && !line.contains("Length") && !line.contains("Count") {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::BoundsSafety,
                    severity: DiagnosticLevel::Info,
                    message: "Array access without bounds checking".to_string(),
                    line_number: Some(line_num + 1),
                    suggestion: Some("Consider adding bounds checking before array access".to_string()),
                    code_example: Some("if (index >= 0 && index < array.Length) { var item = array[index]; }".to_string()),
                });
            }
        }
        
        issues
    }
    
    /// Generate improvement suggestions for the generated class
    fn generate_improvement_suggestions(&self, generated_class: &GeneratedClass) -> Vec<ImprovementSuggestion> {
        let mut suggestions = Vec::new();
        
        // Suggest adding XML documentation
        if !generated_class.source_code.contains("/// <summary>") {
            suggestions.push(ImprovementSuggestion {
                category: SuggestionCategory::Documentation,
                title: "Add XML Documentation".to_string(),
                description: "Consider adding XML documentation comments for better code maintainability".to_string(),
                example: Some("/// <summary>\n/// Description of the class or method\n/// </summary>".to_string()),
                priority: SuggestionPriority::Low,
            });
        }
        
        // Suggest performance optimizations
        if generated_class.source_code.contains("GameObject.Find") {
            suggestions.push(ImprovementSuggestion {
                category: SuggestionCategory::Performance,
                title: "Cache GameObject References".to_string(),
                description: "GameObject.Find is expensive. Consider caching references in Start()".to_string(),
                example: Some("private GameObject cachedObject;\nvoid Start() { cachedObject = GameObject.Find(\"Name\"); }".to_string()),
                priority: SuggestionPriority::Medium,
            });
        }
        
        // Suggest error handling improvements
        if !generated_class.source_code.contains("try") && generated_class.source_code.contains("SendCustomEvent") {
            suggestions.push(ImprovementSuggestion {
                category: SuggestionCategory::ErrorHandling,
                title: "Add Error Handling".to_string(),
                description: "Consider adding try-catch blocks around potentially failing operations".to_string(),
                example: Some("try {\n    SendCustomEvent(\"EventName\");\n} catch (System.Exception e) {\n    Debug.LogError(e.Message);\n}".to_string()),
                priority: SuggestionPriority::Medium,
            });
        }
        
        suggestions
    }
    
    /// Validate multiple generated classes
    pub fn validate_multiple_classes(&self, classes: &[GeneratedClass]) -> MultiClassValidationResult {
        let mut results = Vec::new();
        let mut all_issues = Vec::new();
        
        for class in classes {
            let result = self.validate_generated_class(class);
            all_issues.extend(result.issues.clone());
            results.push(result);
        }
        
        // Cross-class validation
        let cross_class_issues = self.validate_cross_class_dependencies(classes);
        all_issues.extend(cross_class_issues);
        
        let error_count = all_issues.iter().filter(|i| i.severity == DiagnosticLevel::Error).count();
        let warning_count = all_issues.iter().filter(|i| i.severity == DiagnosticLevel::Warning).count();
        
        MultiClassValidationResult {
            class_results: results,
            cross_class_issues: all_issues,
            total_error_count: error_count,
            total_warning_count: warning_count,
            is_valid: error_count == 0,
        }
    }
    
    /// Validate dependencies between classes
    fn validate_cross_class_dependencies(&self, classes: &[GeneratedClass]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for circular dependencies in custom events
        let mut event_dependencies: HashMap<String, Vec<String>> = HashMap::new();
        
        for class in classes {
            let mut class_dependencies = Vec::new();
            
            // Find SendCustomEvent calls to other behaviors
            for line in class.source_code.lines() {
                if line.contains("SendCustomEvent") {
                    // Extract target behavior names (simplified pattern matching)
                    if let Some(start) = line.find("GameObject.Find(\"") {
                        if let Some(end) = line[start + 17..].find('"') {
                            let target_name = &line[start + 17..start + 17 + end];
                            class_dependencies.push(target_name.to_string());
                        }
                    }
                }
            }
            
            event_dependencies.insert(class.class_name.clone(), class_dependencies);
        }
        
        // Check for circular dependencies
        for (class_name, dependencies) in &event_dependencies {
            for dependency in dependencies {
                if let Some(dep_dependencies) = event_dependencies.get(dependency) {
                    if dep_dependencies.contains(class_name) {
                        issues.push(ValidationIssue {
                            issue_type: ValidationIssueType::CircularDependency,
                            severity: DiagnosticLevel::Warning,
                            message: format!("Potential circular dependency between {} and {}", class_name, dependency),
                            line_number: None,
                            suggestion: Some("Consider using events or interfaces to decouple behaviors".to_string()),
                            code_example: None,
                        });
                    }
                }
            }
        }
        
        issues
    }
}

impl Default for RuntimeValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for validation rules
trait ValidationRule: Send + Sync {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue>;
    fn get_rule_name(&self) -> &str;
}

/// C# syntax validation rule
struct CSharpSyntaxRule;

impl CSharpSyntaxRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for CSharpSyntaxRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check class name is valid C# identifier
        if !is_valid_csharp_identifier(&generated_class.class_name) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::SyntaxError,
                severity: DiagnosticLevel::Error,
                message: format!("Invalid C# class name: {}", generated_class.class_name),
                line_number: None,
                suggestion: Some("Use a valid C# identifier for the class name".to_string()),
                code_example: None,
            });
        }
        
        // Check field names
        for field in &generated_class.fields {
            if !is_valid_csharp_identifier(&field.name) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::SyntaxError,
                    severity: DiagnosticLevel::Error,
                    message: format!("Invalid C# field name: {}", field.name),
                    line_number: None,
                    suggestion: Some("Use a valid C# identifier for the field name".to_string()),
                    code_example: None,
                });
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "CSharpSyntax"
    }
}

/// Naming convention validation rule
struct NamingConventionRule;

impl NamingConventionRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for NamingConventionRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check class name follows PascalCase
        if !is_pascal_case(&generated_class.class_name) {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::NamingConvention,
                severity: DiagnosticLevel::Warning,
                message: format!("Class name '{}' should use PascalCase", generated_class.class_name),
                line_number: None,
                suggestion: Some("Use PascalCase for class names (e.g., PlayerManager)".to_string()),
                code_example: None,
            });
        }
        
        // Check method names follow PascalCase
        for method in &generated_class.methods {
            if !is_pascal_case(&method.name) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::NamingConvention,
                    severity: DiagnosticLevel::Warning,
                    message: format!("Method name '{}' should use PascalCase", method.name),
                    line_number: None,
                    suggestion: Some("Use PascalCase for method names (e.g., UpdatePlayerCount)".to_string()),
                    code_example: None,
                });
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "NamingConvention"
    }
}

/// Access modifier validation rule
struct AccessModifierRule;

impl AccessModifierRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for AccessModifierRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check that Unity event methods are public
        let unity_methods = ["Start", "Update", "OnEnable", "OnDisable", "OnTriggerEnter"];
        for method in &generated_class.methods {
            if unity_methods.contains(&method.name.as_str()) {
                if !method.declaration.contains("public ") {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::AccessModifier,
                        severity: DiagnosticLevel::Warning,
                        message: format!("Unity event method '{}' should be public", method.name),
                        line_number: None,
                        suggestion: Some("Make Unity event methods public".to_string()),
                        code_example: Some(format!("public void {}() {{ ... }}", method.name)),
                    });
                }
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "AccessModifier"
    }
}

/// UdonSharp inheritance validation rule
struct UdonSharpInheritanceRule;

impl UdonSharpInheritanceRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for UdonSharpInheritanceRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        if !generated_class.source_code.contains(": UdonSharpBehaviour") {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::UdonSharpCompatibility,
                severity: DiagnosticLevel::Error,
                message: "Class must inherit from UdonSharpBehaviour".to_string(),
                line_number: None,
                suggestion: Some("Add ': UdonSharpBehaviour' to class declaration".to_string()),
                code_example: Some(format!("public class {} : UdonSharpBehaviour", generated_class.class_name)),
            });
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "UdonSharpInheritance"
    }
}

/// UdonSharp attribute validation rule
struct UdonSharpAttributeRule;

impl UdonSharpAttributeRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for UdonSharpAttributeRule {
    fn validate(&self, _generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        // Attribute validation is handled during generation
        Vec::new()
    }
    
    fn get_rule_name(&self) -> &str {
        "UdonSharpAttribute"
    }
}

/// Network synchronization validation rule
struct NetworkSyncRule;

impl NetworkSyncRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for NetworkSyncRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for proper master client validation
        if generated_class.source_code.contains("[UdonSynced]") {
            if !generated_class.source_code.contains("Networking.IsMaster") {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::NetworkingSafety,
                    severity: DiagnosticLevel::Warning,
                    message: "Synchronized fields should have master client validation".to_string(),
                    line_number: None,
                    suggestion: Some("Add Networking.IsMaster checks before modifying synchronized fields".to_string()),
                    code_example: Some("if (Networking.IsMaster) { syncedField = newValue; RequestSerialization(); }".to_string()),
                });
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "NetworkSync"
    }
}

/// Custom event validation rule
struct CustomEventRule;

impl CustomEventRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for CustomEventRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for proper custom event method signatures
        for event in &generated_class.custom_events {
            if !event.declaration.contains("public void") {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::CustomEventSafety,
                    severity: DiagnosticLevel::Warning,
                    message: format!("Custom event method '{}' should be public void", event.method_name),
                    line_number: None,
                    suggestion: Some("Make custom event methods public void".to_string()),
                    code_example: Some(format!("public void {}() {{ ... }}", event.method_name)),
                });
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "CustomEvent"
    }
}

/// Null handling validation rule
struct NullHandlingRule;

impl NullHandlingRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for NullHandlingRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for GameObject references without null checks
        if generated_class.source_code.contains("GameObject") && 
           !generated_class.source_code.contains("!= null") {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::NullSafety,
                severity: DiagnosticLevel::Info,
                message: "Consider adding null checks for GameObject references".to_string(),
                line_number: None,
                suggestion: Some("Add null checks before using GameObject references".to_string()),
                code_example: Some("if (gameObject != null) { /* use gameObject */ }".to_string()),
            });
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "NullHandling"
    }
}

/// Error handling validation rule
struct ErrorHandlingRule;

impl ErrorHandlingRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for ErrorHandlingRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for error logging in catch blocks
        if generated_class.source_code.contains("catch") && 
           !generated_class.source_code.contains("Debug.LogError") {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::ErrorHandling,
                severity: DiagnosticLevel::Info,
                message: "Consider adding error logging in catch blocks".to_string(),
                line_number: None,
                suggestion: Some("Add Debug.LogError calls in exception handlers".to_string()),
                code_example: Some("catch (System.Exception e) { Debug.LogError(e.Message); }".to_string()),
            });
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "ErrorHandling"
    }
}

/// Performance validation rule
struct PerformanceRule;

impl PerformanceRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for PerformanceRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for expensive operations in Update
        for method in &generated_class.methods {
            if method.name == "Update" {
                if method.body.contains("GameObject.Find") {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::Performance,
                        severity: DiagnosticLevel::Warning,
                        message: "GameObject.Find in Update method can impact performance".to_string(),
                        line_number: None,
                        suggestion: Some("Cache GameObject references in Start() instead".to_string()),
                        code_example: Some("void Start() { cachedObject = GameObject.Find(\"Name\"); }".to_string()),
                    });
                }
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "Performance"
    }
}

/// Documentation validation rule
struct DocumentationRule;

impl DocumentationRule {
    fn new() -> Self {
        Self
    }
}

impl ValidationRule for DocumentationRule {
    fn validate(&self, generated_class: &GeneratedClass) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();
        
        // Check for XML documentation on public methods
        for method in &generated_class.methods {
            if method.declaration.contains("public ") && 
               !method.declaration.contains("/// <summary>") {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::Documentation,
                    severity: DiagnosticLevel::Info,
                    message: format!("Public method '{}' lacks XML documentation", method.name),
                    line_number: None,
                    suggestion: Some("Add XML documentation for public methods".to_string()),
                    code_example: Some("/// <summary>\n/// Description of the method\n/// </summary>".to_string()),
                });
            }
        }
        
        issues
    }
    
    fn get_rule_name(&self) -> &str {
        "Documentation"
    }
}

// Helper functions

fn is_valid_csharp_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }
    
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn is_pascal_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    
    let first_char = name.chars().next().unwrap();
    first_char.is_uppercase()
}

/// Validation result for a single class
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub class_name: String,
    pub issues: Vec<ValidationIssue>,
    pub is_valid: bool,
    pub suggestions: Vec<ImprovementSuggestion>,
}

/// Validation result for multiple classes
#[derive(Debug, Clone)]
pub struct MultiClassValidationResult {
    pub class_results: Vec<ValidationResult>,
    pub cross_class_issues: Vec<ValidationIssue>,
    pub total_error_count: usize,
    pub total_warning_count: usize,
    pub is_valid: bool,
}

/// Individual validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub severity: DiagnosticLevel,
    pub message: String,
    pub line_number: Option<usize>,
    pub suggestion: Option<String>,
    pub code_example: Option<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationIssueType {
    SyntaxError,
    NamingConvention,
    AccessModifier,
    UdonSharpCompatibility,
    NetworkingSafety,
    CustomEventSafety,
    NullSafety,
    BoundsSafety,
    ErrorHandling,
    Performance,
    Documentation,
    CircularDependency,
}

/// Improvement suggestion
#[derive(Debug, Clone)]
pub struct ImprovementSuggestion {
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub example: Option<String>,
    pub priority: SuggestionPriority,
}

/// Categories of improvement suggestions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionCategory {
    Performance,
    ErrorHandling,
    Documentation,
    CodeQuality,
    UdonSharpBestPractices,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionPriority {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_generator::*;

    fn create_test_class() -> GeneratedClass {
        GeneratedClass {
            class_name: "TestBehavior".to_string(),
            namespace: None,
            using_statements: vec!["using UnityEngine;".to_string(), "using UdonSharp;".to_string()],
            class_attributes: vec!["[UdonBehaviourSyncMode(BehaviourSyncMode.None)]".to_string()],
            fields: vec![],
            methods: vec![],
            custom_events: vec![],
            source_code: "using UnityEngine;\nusing UdonSharp;\n\n[UdonBehaviourSyncMode(BehaviourSyncMode.None)]\npublic class TestBehavior : UdonSharpBehaviour\n{\n    public override void Start()\n    {\n        // Initialize behavior\n    }\n}".to_string(),
        }
    }

    #[test]
    fn test_validator_creation() {
        let validator = RuntimeValidator::new();
        assert!(!validator.syntax_rules.is_empty());
        assert!(!validator.udonsharp_rules.is_empty());
        assert!(!validator.quality_rules.is_empty());
    }

    #[test]
    fn test_valid_class_validation() {
        let validator = RuntimeValidator::new();
        let test_class = create_test_class();
        
        let result = validator.validate_generated_class(&test_class);
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_class_name() {
        let validator = RuntimeValidator::new();
        let mut test_class = create_test_class();
        test_class.class_name = "123InvalidName".to_string(); // Invalid C# identifier
        
        let result = validator.validate_generated_class(&test_class);
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.issue_type == ValidationIssueType::SyntaxError));
    }

    #[test]
    fn test_missing_inheritance() {
        let validator = RuntimeValidator::new();
        let mut test_class = create_test_class();
        test_class.source_code = "public class TestBehavior { }".to_string(); // Missing inheritance
        
        let result = validator.validate_generated_class(&test_class);
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.issue_type == ValidationIssueType::UdonSharpCompatibility));
    }

    #[test]
    fn test_syntax_validation() {
        let validator = RuntimeValidator::new();
        let mut test_class = create_test_class();
        test_class.source_code = "public class TestBehavior { // Missing closing brace".to_string();
        
        let result = validator.validate_generated_class(&test_class);
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.issue_type == ValidationIssueType::SyntaxError));
    }

    #[test]
    fn test_multiple_class_validation() {
        let validator = RuntimeValidator::new();
        let test_classes = vec![create_test_class(), create_test_class()];
        
        let result = validator.validate_multiple_classes(&test_classes);
        assert!(result.is_valid);
        assert_eq!(result.class_results.len(), 2);
    }
}