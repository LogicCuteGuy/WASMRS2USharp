//! File splitting and organization system for generated C# code
//! 
//! This module provides functionality to split generated C# code into
//! multiple organized files with proper dependency management.

use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use regex::Regex;
use udonsharp_core::error::{UdonSharpError, UdonSharpResult};
use udonsharp_core::multi_behavior_errors::{MultiBehaviorErrorHandler, ErrorCategory};

/// File splitter for organizing generated C# code
pub struct FileSplitter {
    splitting_strategy: SplittingStrategy,
    parser: CSharpCodeParser,
    dependency_analyzer: DependencyAnalyzer,
    error_handler: MultiBehaviorErrorHandler,
}

impl FileSplitter {
    /// Create a new file splitter with the specified strategy
    pub fn new(strategy: SplittingStrategy) -> Self {
        Self {
            splitting_strategy: strategy,
            parser: CSharpCodeParser::new(),
            dependency_analyzer: DependencyAnalyzer::new(),
            error_handler: MultiBehaviorErrorHandler::new(),
        }
    }
    
    /// Split C# code into multiple files with proper organization
    pub fn split_code(&self, code: &str) -> UdonSharpResult<HashMap<String, CSharpFile>> {
        // Parse the input code to understand its structure
        let parsed_code = self.parser.parse(code)
            .map_err(|e| UdonSharpError::behavior_splitting(format!("Failed to parse C# code: {}", e)))?;
        
        // Analyze dependencies between classes and methods
        let dependencies = self.dependency_analyzer.analyze(&parsed_code)
            .map_err(|e| UdonSharpError::behavior_splitting(format!("Failed to analyze dependencies: {}", e)))?;
        
        // Check for circular dependencies before splitting
        if let Err(circular_error) = self.check_circular_dependencies(&dependencies) {
            return Err(circular_error);
        }
        
        // Apply the splitting strategy
        let split_files = match self.splitting_strategy {
            SplittingStrategy::ByClass => self.split_by_class(&parsed_code, &dependencies)?,
            SplittingStrategy::ByNamespace => self.split_by_namespace(&parsed_code, &dependencies)?,
            SplittingStrategy::BySize(max_size) => self.split_by_size(&parsed_code, &dependencies, max_size)?,
        };
        
        // Generate proper using statements and organize content
        self.organize_files(split_files, &dependencies)
            .map_err(|e| UdonSharpError::behavior_splitting(format!("Failed to organize files: {}", e)))
    }
    
    /// Check for circular dependencies in the dependency graph
    fn check_circular_dependencies(&self, dependencies: &DependencyGraph) -> UdonSharpResult<()> {
        let cycles = dependencies.detect_cycles();
        
        if !cycles.is_empty() {
            let cycle_descriptions: Vec<String> = cycles.iter()
                .map(|cycle| cycle.join(" -> "))
                .collect();
            
            let all_behaviors: HashSet<String> = cycles.iter()
                .flat_map(|cycle| cycle.iter().cloned())
                .collect();
            
            return Err(UdonSharpError::circular_dependency(
                cycle_descriptions.join("; "),
                all_behaviors.into_iter().collect()
            ));
        }
        
        Ok(())
    }
    
    /// Validate behavior splitting configuration
    pub fn validate_splitting_config(&self, behaviors: &[String]) -> UdonSharpResult<()> {
        // Check for duplicate behavior names
        let mut seen_names = HashSet::new();
        for behavior in behaviors {
            if !seen_names.insert(behavior.clone()) {
                return Err(UdonSharpError::invalid_attribute_with_suggestion(
                    format!("Duplicate behavior name: {}", behavior),
                    "udon_behaviour".to_string(),
                    behavior.clone(),
                    "Ensure all behavior names are unique. Consider adding a suffix or using more descriptive names.".to_string()
                ));
            }
        }
        
        // Check for invalid behavior names
        for behavior in behaviors {
            if !self.is_valid_behavior_name(behavior) {
                return Err(UdonSharpError::invalid_attribute_with_suggestion(
                    format!("Invalid behavior name: {}", behavior),
                    "udon_behaviour".to_string(),
                    behavior.clone(),
                    "Behavior names must be valid C# class names (start with letter, contain only letters/numbers/underscores).".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// Check if a behavior name is valid
    fn is_valid_behavior_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        
        // Must start with a letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' {
            return false;
        }
        
        // Must contain only letters, numbers, and underscores
        name.chars().all(|c| c.is_alphanumeric() || c == '_')
    }
    
    /// Generate user-friendly error report
    pub fn generate_error_report(&self, error: &UdonSharpError) -> String {
        self.error_handler.generate_user_report(error)
    }
    
    /// Split code by class - each class gets its own file
    fn split_by_class(&self, parsed_code: &ParsedCSharpCode, dependencies: &DependencyGraph) -> Result<HashMap<String, CSharpFileContent>> {
        let mut files = HashMap::new();
        
        for class in &parsed_code.classes {
            let file_name = format!("{}.cs", class.name);
            let mut file_content = CSharpFileContent {
                classes: vec![class.clone()],
                methods: Vec::new(),
                fields: Vec::new(),
                enums: Vec::new(),
                interfaces: Vec::new(),
                namespace: parsed_code.namespace.clone(),
                dependencies: dependencies.get_class_dependencies(&class.name),
            };
            
            // Add methods that belong to this class
            for method in &parsed_code.methods {
                if method.class_name.as_ref() == Some(&class.name) {
                    file_content.methods.push(method.clone());
                }
            }
            
            // Add fields that belong to this class
            for field in &parsed_code.fields {
                if field.class_name.as_ref() == Some(&class.name) {
                    file_content.fields.push(field.clone());
                }
            }
            
            files.insert(file_name, file_content);
        }
        
        // Handle standalone methods and fields (not in any class)
        let standalone_methods: Vec<_> = parsed_code.methods.iter()
            .filter(|m| m.class_name.is_none())
            .cloned()
            .collect();
            
        let standalone_fields: Vec<_> = parsed_code.fields.iter()
            .filter(|f| f.class_name.is_none())
            .cloned()
            .collect();
            
        if !standalone_methods.is_empty() || !standalone_fields.is_empty() {
            files.insert("Utilities.cs".to_string(), CSharpFileContent {
                classes: Vec::new(),
                methods: standalone_methods,
                fields: standalone_fields,
                enums: parsed_code.enums.clone(),
                interfaces: parsed_code.interfaces.clone(),
                namespace: parsed_code.namespace.clone(),
                dependencies: HashSet::new(),
            });
        }
        
        Ok(files)
    }
    
    /// Split code by namespace - each namespace gets its own file
    fn split_by_namespace(&self, parsed_code: &ParsedCSharpCode, dependencies: &DependencyGraph) -> Result<HashMap<String, CSharpFileContent>> {
        let mut files = HashMap::new();
        let mut namespace_contents: HashMap<String, CSharpFileContent> = HashMap::new();
        
        // Group content by namespace
        let default_namespace = parsed_code.namespace.clone().unwrap_or_else(|| "Global".to_string());
        
        for class in &parsed_code.classes {
            let namespace = class.namespace.clone().unwrap_or_else(|| default_namespace.clone());
            let entry = namespace_contents.entry(namespace.clone()).or_insert_with(|| CSharpFileContent {
                classes: Vec::new(),
                methods: Vec::new(),
                fields: Vec::new(),
                enums: Vec::new(),
                interfaces: Vec::new(),
                namespace: Some(namespace.clone()),
                dependencies: HashSet::new(),
            });
            
            entry.classes.push(class.clone());
            entry.dependencies.extend(dependencies.get_class_dependencies(&class.name));
        }
        
        // Add methods and fields to their respective namespaces
        for method in &parsed_code.methods {
            let namespace = method.namespace.clone().unwrap_or_else(|| default_namespace.clone());
            if let Some(content) = namespace_contents.get_mut(&namespace) {
                content.methods.push(method.clone());
            }
        }
        
        for field in &parsed_code.fields {
            let namespace = field.namespace.clone().unwrap_or_else(|| default_namespace.clone());
            if let Some(content) = namespace_contents.get_mut(&namespace) {
                content.fields.push(field.clone());
            }
        }
        
        // Convert to files with proper names
        for (namespace, content) in namespace_contents {
            let file_name = format!("{}.cs", namespace.replace(".", "_"));
            files.insert(file_name, content);
        }
        
        Ok(files)
    }
    
    /// Split code by size - keep files under the specified size limit
    fn split_by_size(&self, parsed_code: &ParsedCSharpCode, dependencies: &DependencyGraph, max_size: usize) -> Result<HashMap<String, CSharpFileContent>> {
        let mut files = HashMap::new();
        let mut current_file_index = 1;
        let mut current_file = CSharpFileContent {
            classes: Vec::new(),
            methods: Vec::new(),
            fields: Vec::new(),
            enums: Vec::new(),
            interfaces: Vec::new(),
            namespace: parsed_code.namespace.clone(),
            dependencies: HashSet::new(),
        };
        
        // Estimate size of current file content
        let mut current_size = 0;
        
        // Add classes one by one, checking size limits
        for class in &parsed_code.classes {
            let class_size = self.estimate_class_size(class);
            
            if current_size + class_size > max_size && !current_file.classes.is_empty() {
                // Save current file and start a new one
                let file_name = format!("Generated_{}.cs", current_file_index);
                files.insert(file_name, current_file);
                
                current_file_index += 1;
                current_file = CSharpFileContent {
                    classes: Vec::new(),
                    methods: Vec::new(),
                    fields: Vec::new(),
                    enums: Vec::new(),
                    interfaces: Vec::new(),
                    namespace: parsed_code.namespace.clone(),
                    dependencies: HashSet::new(),
                };
                current_size = 0;
            }
            
            current_file.classes.push(class.clone());
            current_file.dependencies.extend(dependencies.get_class_dependencies(&class.name));
            current_size += class_size;
        }
        
        // Add remaining content to the last file
        if !current_file.classes.is_empty() || !parsed_code.methods.is_empty() || !parsed_code.fields.is_empty() {
            current_file.methods.extend(parsed_code.methods.clone());
            current_file.fields.extend(parsed_code.fields.clone());
            current_file.enums.extend(parsed_code.enums.clone());
            current_file.interfaces.extend(parsed_code.interfaces.clone());
            
            let file_name = format!("Generated_{}.cs", current_file_index);
            files.insert(file_name, current_file);
        }
        
        Ok(files)
    }
    
    /// Estimate the size of a class in characters
    fn estimate_class_size(&self, class: &ParsedClass) -> usize {
        // Rough estimation based on class structure
        let base_size = 200; // Basic class declaration overhead
        let method_size = class.methods.len() * 150; // Average method size
        let field_size = class.fields.len() * 50; // Average field size
        let property_size = class.properties.len() * 100; // Average property size
        
        base_size + method_size + field_size + property_size
    }
    
    /// Organize files with proper using statements and dependencies
    fn organize_files(&self, file_contents: HashMap<String, CSharpFileContent>, dependencies: &DependencyGraph) -> Result<HashMap<String, CSharpFile>> {
        let mut organized_files = HashMap::new();
        
        for (file_name, content) in file_contents {
            let using_statements = self.generate_using_statements(&content, dependencies)?;
            let organized_content = self.generate_file_content(&content)?;
            
            let csharp_file = CSharpFile {
                name: file_name.clone(),
                content: organized_content,
                using_statements,
                namespace: content.namespace.clone(),
                dependencies: content.dependencies.clone(),
            };
            
            organized_files.insert(file_name, csharp_file);
        }
        
        Ok(organized_files)
    }
    
    /// Generate appropriate using statements for a file
    fn generate_using_statements(&self, content: &CSharpFileContent, dependencies: &DependencyGraph) -> Result<Vec<String>> {
        let mut using_statements = HashSet::new();
        
        // Standard UdonSharp using statements
        using_statements.insert("UnityEngine".to_string());
        using_statements.insert("VRC.SDKBase".to_string());
        using_statements.insert("VRC.Udon".to_string());
        using_statements.insert("UdonSharp".to_string());
        
        // Add System namespaces based on content analysis
        if self.content_uses_collections(content) {
            using_statements.insert("System.Collections.Generic".to_string());
        }
        
        if self.content_uses_linq(content) {
            using_statements.insert("System.Linq".to_string());
        }
        
        // Add dependencies from other files
        for dependency in &content.dependencies {
            if let Some(namespace) = dependencies.get_namespace_for_type(dependency) {
                using_statements.insert(namespace);
            }
        }
        
        let mut sorted_statements: Vec<_> = using_statements.into_iter().collect();
        sorted_statements.sort();
        
        Ok(sorted_statements)
    }
    
    /// Check if content uses collections
    fn content_uses_collections(&self, content: &CSharpFileContent) -> bool {
        // Check for common collection types in fields and method signatures
        let collection_types = ["List", "Dictionary", "HashSet", "Array"];
        
        for class in &content.classes {
            for field in &class.fields {
                if collection_types.iter().any(|&t| field.field_type.contains(t)) {
                    return true;
                }
            }
        }
        
        for method in &content.methods {
            if collection_types.iter().any(|&t| method.return_type.contains(t)) {
                return true;
            }
            for param in &method.parameters {
                if collection_types.iter().any(|&t| param.contains(t)) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check if content uses LINQ
    fn content_uses_linq(&self, _content: &CSharpFileContent) -> bool {
        // For now, assume LINQ is not used in UdonSharp
        // This could be enhanced to detect LINQ usage patterns
        false
    }
    
    /// Generate the actual file content
    fn generate_file_content(&self, content: &CSharpFileContent) -> Result<String> {
        let mut file_content = String::new();
        
        // Add namespace declaration if present
        let indent = if let Some(namespace) = &content.namespace {
            file_content.push_str(&format!("namespace {}\n{{\n", namespace));
            "    "
        } else {
            ""
        };
        
        // Add enums
        for enum_def in &content.enums {
            file_content.push_str(&self.generate_enum_code(enum_def, indent)?);
            file_content.push_str("\n");
        }
        
        // Add interfaces
        for interface in &content.interfaces {
            file_content.push_str(&self.generate_interface_code(interface, indent)?);
            file_content.push_str("\n");
        }
        
        // Add classes
        for class in &content.classes {
            file_content.push_str(&self.generate_class_code(class, indent)?);
            file_content.push_str("\n");
        }
        
        // Add standalone methods (if any)
        if !content.methods.is_empty() {
            file_content.push_str(&format!("{}public static class Utilities\n{}{{\n", indent, indent));
            for method in &content.methods {
                if method.class_name.is_none() {
                    file_content.push_str(&self.generate_method_code(method, &format!("{}    ", indent))?);
                }
            }
            file_content.push_str(&format!("{}}}\n\n", indent));
        }
        
        // Close namespace if opened
        if content.namespace.is_some() {
            file_content.push_str("}\n");
        }
        
        Ok(file_content)
    }
    
    /// Generate enum code
    fn generate_enum_code(&self, enum_def: &ParsedEnum, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        code.push_str(&format!("{}public enum {}\n{}{{\n", indent, enum_def.name, indent));
        
        for (i, value) in enum_def.values.iter().enumerate() {
            let comma = if i < enum_def.values.len() - 1 { "," } else { "" };
            code.push_str(&format!("{}    {}{}\n", indent, value, comma));
        }
        
        code.push_str(&format!("{}}}\n", indent));
        
        Ok(code)
    }
    
    /// Generate interface code
    fn generate_interface_code(&self, interface: &ParsedInterface, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        code.push_str(&format!("{}public interface {}\n{}{{\n", indent, interface.name, indent));
        
        for method in &interface.methods {
            code.push_str(&format!(
                "{}    {} {}({});\n",
                indent,
                method.return_type,
                method.name,
                method.parameters.join(", ")
            ));
        }
        
        code.push_str(&format!("{}}}\n", indent));
        
        Ok(code)
    }
    
    /// Generate class code
    fn generate_class_code(&self, class: &ParsedClass, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Add class attributes
        for attribute in &class.attributes {
            code.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Class declaration
        code.push_str(&format!("{}public class {}", indent, class.name));
        
        if let Some(base_class) = &class.base_class {
            code.push_str(&format!(" : {}", base_class));
        }
        
        if !class.interfaces.is_empty() {
            let interfaces = class.interfaces.join(", ");
            if class.base_class.is_some() {
                code.push_str(&format!(", {}", interfaces));
            } else {
                code.push_str(&format!(" : {}", interfaces));
            }
        }
        
        code.push_str(&format!("\n{}{{\n", indent));
        
        // Add fields
        for field in &class.fields {
            code.push_str(&self.generate_field_code(field, &format!("{}    ", indent))?);
        }
        
        if !class.fields.is_empty() {
            code.push_str("\n");
        }
        
        // Add properties
        for property in &class.properties {
            code.push_str(&self.generate_property_code(property, &format!("{}    ", indent))?);
        }
        
        if !class.properties.is_empty() {
            code.push_str("\n");
        }
        
        // Add methods
        for method in &class.methods {
            code.push_str(&self.generate_method_code(method, &format!("{}    ", indent))?);
        }
        
        code.push_str(&format!("{}}}\n", indent));
        
        Ok(code)
    }
    
    /// Generate field code
    fn generate_field_code(&self, field: &ParsedField, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Add field attributes
        for attribute in &field.attributes {
            code.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Field declaration
        code.push_str(&format!(
            "{}{} {} {};\n",
            indent,
            field.visibility,
            field.field_type,
            field.name
        ));
        
        Ok(code)
    }
    
    /// Generate property code
    fn generate_property_code(&self, property: &ParsedProperty, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Add property attributes
        for attribute in &property.attributes {
            code.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Property declaration
        code.push_str(&format!(
            "{}{} {} {} {{ get; set; }}\n",
            indent,
            property.visibility,
            property.property_type,
            property.name
        ));
        
        Ok(code)
    }
    
    /// Generate method code
    fn generate_method_code(&self, method: &ParsedMethod, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Add method attributes
        for attribute in &method.attributes {
            code.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Method signature
        let modifiers = if method.is_static { "static " } else { "" };
        let parameters = method.parameters.join(", ");
        
        code.push_str(&format!(
            "{}{} {}{} {}({})\n{}{{ \n{}    // Method implementation\n{}}}\n\n",
            indent,
            method.visibility,
            modifiers,
            method.return_type,
            method.name,
            parameters,
            indent,
            indent,
            indent
        ));
        
        Ok(code)
    }
}

/// Strategy for splitting files
#[derive(Debug, Clone)]
pub enum SplittingStrategy {
    ByClass,
    ByNamespace,
    BySize(usize),
}

/// Represents a generated C# file with complete metadata
#[derive(Debug, Clone)]
pub struct CSharpFile {
    pub name: String,
    pub content: String,
    pub using_statements: Vec<String>,
    pub namespace: Option<String>,
    pub dependencies: HashSet<String>,
}
/// C# code parser for analyzing code structure
pub struct CSharpCodeParser {
    class_regex: Regex,
    method_regex: Regex,
    field_regex: Regex,
    using_regex: Regex,
    namespace_regex: Regex,
}

impl CSharpCodeParser {
    pub fn new() -> Self {
        Self {
            class_regex: Regex::new(r"(?m)^\s*(?:\[.*?\]\s*)*public\s+class\s+(\w+)(?:\s*:\s*([^{]+))?").unwrap(),
            method_regex: Regex::new(r"(?m)^\s*(?:\[.*?\]\s*)*(?:public|private|protected)\s+(?:static\s+)?(?:virtual\s+)?(\w+)\s+(\w+)\s*\(([^)]*)\)").unwrap(),
            field_regex: Regex::new(r"(?m)^\s*(?:\[.*?\]\s*)*(?:public|private|protected)\s+(\w+)\s+(\w+);").unwrap(),
            using_regex: Regex::new(r"(?m)^\s*using\s+([^;]+);").unwrap(),
            namespace_regex: Regex::new(r"(?m)^\s*namespace\s+([^{]+)").unwrap(),
        }
    }
    
    /// Parse C# code into structured representation
    pub fn parse(&self, code: &str) -> Result<ParsedCSharpCode> {
        let mut parsed = ParsedCSharpCode {
            namespace: self.extract_namespace(code),
            using_statements: self.extract_using_statements(code),
            classes: self.extract_classes(code)?,
            methods: self.extract_methods(code)?,
            fields: self.extract_fields(code)?,
            enums: self.extract_enums(code)?,
            interfaces: self.extract_interfaces(code)?,
        };
        
        // Associate methods and fields with their classes
        self.associate_members_with_classes(&mut parsed)?;
        
        Ok(parsed)
    }
    
    /// Extract namespace from code
    fn extract_namespace(&self, code: &str) -> Option<String> {
        self.namespace_regex.captures(code)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }
    
    /// Extract using statements
    fn extract_using_statements(&self, code: &str) -> Vec<String> {
        self.using_regex.captures_iter(code)
            .filter_map(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
            .collect()
    }
    
    /// Extract classes from code
    fn extract_classes(&self, code: &str) -> Result<Vec<ParsedClass>> {
        let mut classes = Vec::new();
        
        for caps in self.class_regex.captures_iter(code) {
            let class_name = caps.get(1).unwrap().as_str().to_string();
            let inheritance = caps.get(2).map(|m| m.as_str().trim().to_string());
            
            let (base_class, interfaces) = self.parse_inheritance(&inheritance);
            
            let class = ParsedClass {
                name: class_name,
                namespace: None, // Will be set later
                base_class,
                interfaces,
                attributes: self.extract_class_attributes(code, &caps.get(0).unwrap().as_str())?,
                fields: Vec::new(), // Will be populated later
                methods: Vec::new(), // Will be populated later
                properties: Vec::new(), // Will be populated later
            };
            
            classes.push(class);
        }
        
        Ok(classes)
    }
    
    /// Parse inheritance string into base class and interfaces
    fn parse_inheritance(&self, inheritance: &Option<String>) -> (Option<String>, Vec<String>) {
        if let Some(inheritance_str) = inheritance {
            let parts: Vec<&str> = inheritance_str.split(',').map(|s| s.trim()).collect();
            if !parts.is_empty() {
                let base_class = Some(parts[0].to_string());
                let interfaces = parts[1..].iter().map(|s| s.to_string()).collect();
                return (base_class, interfaces);
            }
        }
        (None, Vec::new())
    }
    
    /// Extract class attributes
    fn extract_class_attributes(&self, code: &str, class_declaration: &str) -> Result<Vec<String>> {
        // Find attributes before the class declaration
        let attribute_regex = Regex::new(r"\[([^\]]+)\]")?;
        let mut attributes = Vec::new();
        
        for caps in attribute_regex.captures_iter(class_declaration) {
            if let Some(attr) = caps.get(1) {
                attributes.push(attr.as_str().to_string());
            }
        }
        
        Ok(attributes)
    }
    
    /// Extract methods from code
    fn extract_methods(&self, code: &str) -> Result<Vec<ParsedMethod>> {
        let mut methods = Vec::new();
        
        for caps in self.method_regex.captures_iter(code) {
            let return_type = caps.get(1).unwrap().as_str().to_string();
            let method_name = caps.get(2).unwrap().as_str().to_string();
            let parameters_str = caps.get(3).unwrap().as_str();
            
            let parameters = self.parse_parameters(parameters_str)?;
            
            let method = ParsedMethod {
                name: method_name,
                class_name: None, // Will be set later
                namespace: None, // Will be set later
                return_type,
                parameters,
                visibility: "public".to_string(), // Simplified for now
                is_static: false, // Will be detected later
                attributes: Vec::new(), // Will be extracted later
            };
            
            methods.push(method);
        }
        
        Ok(methods)
    }
    
    /// Parse method parameters
    fn parse_parameters(&self, parameters_str: &str) -> Result<Vec<String>> {
        if parameters_str.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        let parameters: Vec<String> = parameters_str
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
            
        Ok(parameters)
    }
    
    /// Extract fields from code
    fn extract_fields(&self, code: &str) -> Result<Vec<ParsedField>> {
        let mut fields = Vec::new();
        
        for caps in self.field_regex.captures_iter(code) {
            let field_type = caps.get(1).unwrap().as_str().to_string();
            let field_name = caps.get(2).unwrap().as_str().to_string();
            
            let field = ParsedField {
                name: field_name,
                class_name: None, // Will be set later
                namespace: None, // Will be set later
                field_type,
                visibility: "public".to_string(), // Simplified for now
                attributes: Vec::new(), // Will be extracted later
            };
            
            fields.push(field);
        }
        
        Ok(fields)
    }
    
    /// Extract enums from code (simplified implementation)
    fn extract_enums(&self, code: &str) -> Result<Vec<ParsedEnum>> {
        let enum_regex = Regex::new(r"(?m)^\s*public\s+enum\s+(\w+)\s*\{([^}]+)\}")?;
        let mut enums = Vec::new();
        
        for caps in enum_regex.captures_iter(code) {
            let enum_name = caps.get(1).unwrap().as_str().to_string();
            let enum_body = caps.get(2).unwrap().as_str();
            
            let values: Vec<String> = enum_body
                .split(',')
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect();
            
            enums.push(ParsedEnum {
                name: enum_name,
                values,
            });
        }
        
        Ok(enums)
    }
    
    /// Extract interfaces from code (simplified implementation)
    fn extract_interfaces(&self, code: &str) -> Result<Vec<ParsedInterface>> {
        let interface_regex = Regex::new(r"(?m)^\s*public\s+interface\s+(\w+)\s*\{([^}]+)\}")?;
        let mut interfaces = Vec::new();
        
        for caps in interface_regex.captures_iter(code) {
            let interface_name = caps.get(1).unwrap().as_str().to_string();
            let interface_body = caps.get(2).unwrap().as_str();
            
            // Extract method signatures from interface
            let methods = self.extract_interface_methods(interface_body)?;
            
            interfaces.push(ParsedInterface {
                name: interface_name,
                methods,
            });
        }
        
        Ok(interfaces)
    }
    
    /// Extract method signatures from interface body
    fn extract_interface_methods(&self, interface_body: &str) -> Result<Vec<ParsedInterfaceMethod>> {
        let method_regex = Regex::new(r"(?m)^\s*(\w+)\s+(\w+)\s*\(([^)]*)\);")?;
        let mut methods = Vec::new();
        
        for caps in method_regex.captures_iter(interface_body) {
            let return_type = caps.get(1).unwrap().as_str().to_string();
            let method_name = caps.get(2).unwrap().as_str().to_string();
            let parameters_str = caps.get(3).unwrap().as_str();
            
            let parameters = self.parse_parameters(parameters_str)?;
            
            methods.push(ParsedInterfaceMethod {
                name: method_name,
                return_type,
                parameters,
            });
        }
        
        Ok(methods)
    }
    
    /// Associate methods and fields with their containing classes
    fn associate_members_with_classes(&self, parsed: &mut ParsedCSharpCode) -> Result<()> {
        // This is a simplified implementation
        // In a real parser, this would use proper AST analysis
        
        for class in &mut parsed.classes {
            // Find methods that belong to this class
            let class_methods: Vec<_> = parsed.methods.iter()
                .filter(|m| self.method_belongs_to_class(m, &class.name))
                .cloned()
                .collect();
            
            class.methods = class_methods;
            
            // Find fields that belong to this class
            let class_fields: Vec<_> = parsed.fields.iter()
                .filter(|f| self.field_belongs_to_class(f, &class.name))
                .cloned()
                .collect();
            
            class.fields = class_fields;
        }
        
        Ok(())
    }
    
    /// Check if a method belongs to a specific class (simplified heuristic)
    fn method_belongs_to_class(&self, _method: &ParsedMethod, _class_name: &str) -> bool {
        // This is a placeholder implementation
        // A real implementation would analyze the code structure
        true
    }
    
    /// Check if a field belongs to a specific class (simplified heuristic)
    fn field_belongs_to_class(&self, _field: &ParsedField, _class_name: &str) -> bool {
        // This is a placeholder implementation
        // A real implementation would analyze the code structure
        true
    }
}

/// Dependency analyzer for understanding relationships between code elements
pub struct DependencyAnalyzer {
    type_usage_regex: Regex,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            type_usage_regex: Regex::new(r"\b([A-Z][a-zA-Z0-9_]*)\b").unwrap(),
        }
    }
    
    /// Analyze dependencies in parsed code
    pub fn analyze(&self, parsed_code: &ParsedCSharpCode) -> Result<DependencyGraph> {
        let mut graph = DependencyGraph::new();
        
        // Analyze class dependencies
        for class in &parsed_code.classes {
            let dependencies = self.analyze_class_dependencies(class)?;
            graph.add_class_dependencies(&class.name, dependencies);
        }
        
        // Analyze method dependencies
        for method in &parsed_code.methods {
            let dependencies = self.analyze_method_dependencies(method)?;
            if let Some(class_name) = &method.class_name {
                graph.add_method_dependencies(class_name, &method.name, dependencies);
            }
        }
        
        Ok(graph)
    }
    
    /// Analyze dependencies for a specific class
    fn analyze_class_dependencies(&self, class: &ParsedClass) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        
        // Add base class as dependency
        if let Some(base_class) = &class.base_class {
            dependencies.insert(base_class.clone());
        }
        
        // Add interfaces as dependencies
        for interface in &class.interfaces {
            dependencies.insert(interface.clone());
        }
        
        // Analyze field types
        for field in &class.fields {
            let field_dependencies = self.extract_type_dependencies(&field.field_type)?;
            dependencies.extend(field_dependencies);
        }
        
        // Analyze method signatures
        for method in &class.methods {
            let method_dependencies = self.extract_type_dependencies(&method.return_type)?;
            dependencies.extend(method_dependencies);
            
            for param in &method.parameters {
                let param_dependencies = self.extract_type_dependencies(param)?;
                dependencies.extend(param_dependencies);
            }
        }
        
        Ok(dependencies)
    }
    
    /// Analyze dependencies for a specific method
    fn analyze_method_dependencies(&self, method: &ParsedMethod) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        
        // Analyze return type
        let return_dependencies = self.extract_type_dependencies(&method.return_type)?;
        dependencies.extend(return_dependencies);
        
        // Analyze parameters
        for param in &method.parameters {
            let param_dependencies = self.extract_type_dependencies(param)?;
            dependencies.extend(param_dependencies);
        }
        
        Ok(dependencies)
    }
    
    /// Extract type dependencies from a type string
    fn extract_type_dependencies(&self, type_str: &str) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        
        // Extract all type names from the type string
        for caps in self.type_usage_regex.captures_iter(type_str) {
            if let Some(type_name) = caps.get(1) {
                let type_name_str = type_name.as_str().to_string();
                
                // Filter out primitive types and common system types
                if !self.is_primitive_type(&type_name_str) {
                    dependencies.insert(type_name_str);
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Check if a type is a primitive type that doesn't need dependency tracking
    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(type_name, 
            "bool" | "byte" | "sbyte" | "short" | "ushort" | 
            "int" | "uint" | "long" | "ulong" | "float" | 
            "double" | "char" | "string" | "object" | "void"
        )
    }
}

/// Dependency graph for tracking relationships between code elements
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    class_dependencies: HashMap<String, HashSet<String>>,
    method_dependencies: HashMap<String, HashMap<String, HashSet<String>>>,
    type_namespaces: HashMap<String, String>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            class_dependencies: HashMap::new(),
            method_dependencies: HashMap::new(),
            type_namespaces: HashMap::new(),
        }
    }
    
    /// Add dependencies for a class
    pub fn add_class_dependencies(&mut self, class_name: &str, dependencies: HashSet<String>) {
        self.class_dependencies.insert(class_name.to_string(), dependencies);
    }
    
    /// Add dependencies for a method
    pub fn add_method_dependencies(&mut self, class_name: &str, method_name: &str, dependencies: HashSet<String>) {
        let class_methods = self.method_dependencies.entry(class_name.to_string()).or_insert_with(HashMap::new);
        class_methods.insert(method_name.to_string(), dependencies);
    }
    
    /// Get dependencies for a class
    pub fn get_class_dependencies(&self, class_name: &str) -> HashSet<String> {
        self.class_dependencies.get(class_name).cloned().unwrap_or_default()
    }
    
    /// Get namespace for a type
    pub fn get_namespace_for_type(&self, type_name: &str) -> Option<String> {
        self.type_namespaces.get(type_name).cloned()
    }
    
    /// Add namespace mapping for a type
    pub fn add_type_namespace(&mut self, type_name: String, namespace: String) {
        self.type_namespaces.insert(type_name, namespace);
    }
    
    /// Detect circular dependencies in the graph
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for class_name in self.class_dependencies.keys() {
            if !visited.contains(class_name) {
                self.dfs_detect_cycle(
                    class_name,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles
                );
            }
        }
        
        cycles
    }
    
    /// Depth-first search to detect cycles
    fn dfs_detect_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());
        
        if let Some(dependencies) = self.class_dependencies.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    self.dfs_detect_cycle(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    // Found a cycle - extract the cycle from the path
                    if let Some(cycle_start) = path.iter().position(|x| x == dep) {
                        let cycle = path[cycle_start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }
        
        rec_stack.remove(node);
        path.pop();
    }
}

/// Parsed C# code structure
#[derive(Debug, Clone)]
pub struct ParsedCSharpCode {
    pub namespace: Option<String>,
    pub using_statements: Vec<String>,
    pub classes: Vec<ParsedClass>,
    pub methods: Vec<ParsedMethod>,
    pub fields: Vec<ParsedField>,
    pub enums: Vec<ParsedEnum>,
    pub interfaces: Vec<ParsedInterface>,
}

/// Parsed class information
#[derive(Debug, Clone)]
pub struct ParsedClass {
    pub name: String,
    pub namespace: Option<String>,
    pub base_class: Option<String>,
    pub interfaces: Vec<String>,
    pub attributes: Vec<String>,
    pub fields: Vec<ParsedField>,
    pub methods: Vec<ParsedMethod>,
    pub properties: Vec<ParsedProperty>,
}

/// Parsed method information
#[derive(Debug, Clone)]
pub struct ParsedMethod {
    pub name: String,
    pub class_name: Option<String>,
    pub namespace: Option<String>,
    pub return_type: String,
    pub parameters: Vec<String>,
    pub visibility: String,
    pub is_static: bool,
    pub attributes: Vec<String>,
}

/// Parsed field information
#[derive(Debug, Clone)]
pub struct ParsedField {
    pub name: String,
    pub class_name: Option<String>,
    pub namespace: Option<String>,
    pub field_type: String,
    pub visibility: String,
    pub attributes: Vec<String>,
}

/// Parsed property information
#[derive(Debug, Clone)]
pub struct ParsedProperty {
    pub name: String,
    pub property_type: String,
    pub visibility: String,
    pub attributes: Vec<String>,
}

/// Parsed enum information
#[derive(Debug, Clone)]
pub struct ParsedEnum {
    pub name: String,
    pub values: Vec<String>,
}

/// Parsed interface information
#[derive(Debug, Clone)]
pub struct ParsedInterface {
    pub name: String,
    pub methods: Vec<ParsedInterfaceMethod>,
}

/// Parsed interface method information
#[derive(Debug, Clone)]
pub struct ParsedInterfaceMethod {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<String>,
}

/// File content before final organization
#[derive(Debug, Clone)]
struct CSharpFileContent {
    pub classes: Vec<ParsedClass>,
    pub methods: Vec<ParsedMethod>,
    pub fields: Vec<ParsedField>,
    pub enums: Vec<ParsedEnum>,
    pub interfaces: Vec<ParsedInterface>,
    pub namespace: Option<String>,
    pub dependencies: HashSet<String>,
}