//! C# file generation system with template support
//! 
//! This module provides functionality to generate clean, well-organized C# files
//! with proper using statements, namespaces, and UdonSharp attributes.

use crate::splitter::{CSharpFile, ParsedClass, ParsedMethod, ParsedField, ParsedProperty};
use crate::analyzer::{BehaviorUnit, InterBehaviorCall, CallType};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};

/// C# file generator with template system
pub struct CSharpFileGenerator {
    template_engine: TemplateEngine,
    config: FileGenerationConfig,
}

/// Multi-behavior file generator for creating multiple UdonBehaviour classes
pub struct MultiBehaviorFileGenerator {
    file_generator: CSharpFileGenerator,
    behavior_config: MultiBehaviorConfig,
}

/// Configuration for multi-behavior file generation
#[derive(Debug, Clone)]
pub struct MultiBehaviorConfig {
    /// Base namespace for generated classes
    pub namespace: Option<String>,
    /// Whether to generate SharedRuntime class
    pub generate_shared_runtime: bool,
    /// Naming convention for behavior classes
    pub naming_convention: BehaviorNamingConvention,
    /// Whether to include debug information
    pub include_debug_info: bool,
    /// Custom templates for behavior classes
    pub custom_templates: HashMap<String, String>,
}

impl Default for MultiBehaviorConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            generate_shared_runtime: true,
            naming_convention: BehaviorNamingConvention::PascalCase,
            include_debug_info: true,
            custom_templates: HashMap::new(),
        }
    }
}

/// Naming convention for behavior classes
#[derive(Debug, Clone)]
pub enum BehaviorNamingConvention {
    /// PascalCase: PlayerManager
    PascalCase,
    /// PascalCaseWithSuffix: PlayerManagerBehaviour
    PascalCaseWithSuffix,
    /// Custom format string with {name} placeholder
    Custom(String),
}

impl CSharpFileGenerator {
    /// Create a new file generator with default configuration
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config: FileGenerationConfig::default(),
        }
    }
    
    /// Create a file generator with custom configuration
    pub fn with_config(config: FileGenerationConfig) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config,
        }
    }
    
    /// Generate a complete C# file from the provided data
    pub fn generate_file(&self, file_data: &CSharpFile) -> Result<GeneratedCSharpFile> {
        let mut content = String::new();
        
        // Generate file header comment
        if self.config.include_header_comment {
            content.push_str(&self.generate_header_comment(file_data)?);
        }
        
        // Generate using statements
        content.push_str(&self.generate_using_statements(&file_data.using_statements)?);
        content.push('\n');
        
        // Generate namespace wrapper if needed
        let namespace_indent = if let Some(namespace) = &file_data.namespace {
            content.push_str(&format!("namespace {}\n{{\n", namespace));
            "    "
        } else {
            ""
        };
        
        // Generate the main content
        content.push_str(&file_data.content);
        
        // Close namespace if opened
        if file_data.namespace.is_some() {
            content.push_str("}\n");
        }
        
        // Apply final formatting
        let formatted_content = self.apply_formatting(&content)?;
        
        Ok(GeneratedCSharpFile {
            name: file_data.name.clone(),
            content: formatted_content,
            using_statements: file_data.using_statements.clone(),
            namespace: file_data.namespace.clone(),
            dependencies: file_data.dependencies.clone(),
            metadata: FileMetadata {
                generated_at: chrono::Utc::now(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                line_count: content.lines().count(),
                character_count: content.len(),
            },
        })
    }
    
    /// Generate multiple files from a collection
    pub fn generate_files(&self, files: &HashMap<String, CSharpFile>) -> Result<HashMap<String, GeneratedCSharpFile>> {
        let mut generated_files = HashMap::new();
        
        for (file_name, file_data) in files {
            let generated_file = self.generate_file(file_data)
                .with_context(|| format!("Failed to generate file: {}", file_name))?;
            generated_files.insert(file_name.clone(), generated_file);
        }
        
        Ok(generated_files)
    }
    
    /// Generate a class file using templates
    pub fn generate_class_file(&self, class: &ParsedClass, namespace: Option<&str>) -> Result<GeneratedCSharpFile> {
        let template_data = ClassTemplateData {
            class: class.clone(),
            namespace: namespace.map(|s| s.to_string()),
            using_statements: self.generate_class_using_statements(class)?,
            config: self.config.clone(),
        };
        
        let content = self.template_engine.render_class_template(&template_data)?;
        
        let file_data = CSharpFile {
            name: format!("{}.cs", class.name),
            content,
            using_statements: template_data.using_statements.clone(),
            namespace: template_data.namespace.clone(),
            dependencies: self.extract_class_dependencies(class)?,
        };
        
        self.generate_file(&file_data)
    }
    
    /// Generate header comment for the file
    fn generate_header_comment(&self, file_data: &CSharpFile) -> Result<String> {
        let mut comment = String::new();
        
        comment.push_str("//\n");
        comment.push_str(&format!("// Generated C# file: {}\n", file_data.name));
        comment.push_str("// This file was automatically generated by the Rust to UdonSharp compiler.\n");
        comment.push_str("// Do not modify this file directly as changes will be overwritten.\n");
        comment.push_str("//\n");
        
        if let Some(namespace) = &file_data.namespace {
            comment.push_str(&format!("// Namespace: {}\n", namespace));
        }
        
        if !file_data.dependencies.is_empty() {
            comment.push_str("// Dependencies:\n");
            for dependency in &file_data.dependencies {
                comment.push_str(&format!("//   - {}\n", dependency));
            }
        }
        
        comment.push_str("//\n\n");
        
        Ok(comment)
    }
    
    /// Generate using statements with proper organization
    fn generate_using_statements(&self, using_statements: &[String]) -> Result<String> {
        if using_statements.is_empty() {
            return Ok(String::new());
        }
        
        let mut organized_statements = self.organize_using_statements(using_statements)?;
        
        // Add standard UdonSharp using statements if not present
        let standard_usings = [
            "UnityEngine",
            "VRC.SDKBase", 
            "VRC.Udon",
            "UdonSharp",
        ];
        
        for standard_using in &standard_usings {
            if !organized_statements.system_usings.contains(*standard_using) &&
               !organized_statements.third_party_usings.contains(*standard_using) &&
               !organized_statements.project_usings.contains(*standard_using) {
                organized_statements.third_party_usings.insert(standard_using.to_string());
            }
        }
        
        let mut result = String::new();
        
        // System usings first
        if !organized_statements.system_usings.is_empty() {
            let mut system_usings: Vec<_> = organized_statements.system_usings.into_iter().collect();
            system_usings.sort();
            for using_stmt in system_usings {
                result.push_str(&format!("using {};\n", using_stmt));
            }
            result.push('\n');
        }
        
        // Third-party usings
        if !organized_statements.third_party_usings.is_empty() {
            let mut third_party_usings: Vec<_> = organized_statements.third_party_usings.into_iter().collect();
            third_party_usings.sort();
            for using_stmt in third_party_usings {
                result.push_str(&format!("using {};\n", using_stmt));
            }
            result.push('\n');
        }
        
        // Project usings last
        if !organized_statements.project_usings.is_empty() {
            let mut project_usings: Vec<_> = organized_statements.project_usings.into_iter().collect();
            project_usings.sort();
            for using_stmt in project_usings {
                result.push_str(&format!("using {};\n", using_stmt));
            }
            result.push('\n');
        }
        
        Ok(result)
    }
    
    /// Organize using statements into categories
    fn organize_using_statements(&self, using_statements: &[String]) -> Result<OrganizedUsingStatements> {
        let mut organized = OrganizedUsingStatements {
            system_usings: HashSet::new(),
            third_party_usings: HashSet::new(),
            project_usings: HashSet::new(),
        };
        
        for using_stmt in using_statements {
            if using_stmt.starts_with("System") {
                organized.system_usings.insert(using_stmt.clone());
            } else if using_stmt.starts_with("UnityEngine") || 
                      using_stmt.starts_with("VRC") || 
                      using_stmt.starts_with("UdonSharp") {
                organized.third_party_usings.insert(using_stmt.clone());
            } else {
                organized.project_usings.insert(using_stmt.clone());
            }
        }
        
        Ok(organized)
    }
    
    /// Generate using statements for a specific class
    fn generate_class_using_statements(&self, class: &ParsedClass) -> Result<Vec<String>> {
        let mut using_statements = HashSet::new();
        
        // Standard UdonSharp usings
        using_statements.insert("UnityEngine".to_string());
        using_statements.insert("VRC.SDKBase".to_string());
        using_statements.insert("VRC.Udon".to_string());
        using_statements.insert("UdonSharp".to_string());
        
        // Analyze class content for additional usings
        if self.class_uses_collections(class) {
            using_statements.insert("System.Collections.Generic".to_string());
        }
        
        if self.class_uses_linq(class) {
            using_statements.insert("System.Linq".to_string());
        }
        
        if self.class_uses_text(class) {
            using_statements.insert("System.Text".to_string());
        }
        
        let mut sorted_statements: Vec<_> = using_statements.into_iter().collect();
        sorted_statements.sort();
        
        Ok(sorted_statements)
    }
    
    /// Check if class uses collections
    fn class_uses_collections(&self, class: &ParsedClass) -> bool {
        let collection_types = ["List", "Dictionary", "HashSet", "Array", "IEnumerable"];
        
        // Check fields
        for field in &class.fields {
            if collection_types.iter().any(|&t| field.field_type.contains(t)) {
                return true;
            }
        }
        
        // Check methods
        for method in &class.methods {
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
    
    /// Check if class uses LINQ
    fn class_uses_linq(&self, _class: &ParsedClass) -> bool {
        // For UdonSharp, LINQ is generally not supported
        // This could be enhanced to detect specific patterns
        false
    }
    
    /// Check if class uses text processing
    fn class_uses_text(&self, class: &ParsedClass) -> bool {
        let text_types = ["StringBuilder", "Regex", "Encoding"];
        
        // Check fields
        for field in &class.fields {
            if text_types.iter().any(|&t| field.field_type.contains(t)) {
                return true;
            }
        }
        
        // Check methods
        for method in &class.methods {
            if text_types.iter().any(|&t| method.return_type.contains(t)) {
                return true;
            }
            for param in &method.parameters {
                if text_types.iter().any(|&t| param.contains(t)) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Extract dependencies from a class
    fn extract_class_dependencies(&self, class: &ParsedClass) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        
        // Add base class
        if let Some(base_class) = &class.base_class {
            dependencies.insert(base_class.clone());
        }
        
        // Add interfaces
        for interface in &class.interfaces {
            dependencies.insert(interface.clone());
        }
        
        // Add field type dependencies
        for field in &class.fields {
            let field_deps = self.extract_type_dependencies(&field.field_type)?;
            dependencies.extend(field_deps);
        }
        
        // Add method dependencies
        for method in &class.methods {
            let return_deps = self.extract_type_dependencies(&method.return_type)?;
            dependencies.extend(return_deps);
            
            for param in &method.parameters {
                let param_deps = self.extract_type_dependencies(param)?;
                dependencies.extend(param_deps);
            }
        }
        
        Ok(dependencies)
    }
    
    /// Extract type dependencies from a type string
    fn extract_type_dependencies(&self, type_str: &str) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        
        // Simple regex-based extraction
        let type_regex = regex::Regex::new(r"\b([A-Z][a-zA-Z0-9_]*)\b")?;
        
        for caps in type_regex.captures_iter(type_str) {
            if let Some(type_name) = caps.get(1) {
                let type_name_str = type_name.as_str().to_string();
                
                // Filter out primitive types
                if !self.is_primitive_type(&type_name_str) {
                    dependencies.insert(type_name_str);
                }
            }
        }
        
        Ok(dependencies)
    }
    
    /// Check if a type is primitive
    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(type_name, 
            "bool" | "byte" | "sbyte" | "short" | "ushort" | 
            "int" | "uint" | "long" | "ulong" | "float" | 
            "double" | "char" | "string" | "object" | "void"
        )
    }
    
    /// Generate multiple UdonBehaviour class files from behavior units
    pub fn generate_multiple_behavior_files(&self, behavior_units: &[BehaviorUnit]) -> Result<HashMap<String, GeneratedCSharpFile>> {
        let mut generated_files = HashMap::new();
        
        for behavior_unit in behavior_units {
            let file_name = self.generate_behavior_class_name(&behavior_unit.name);
            let class_data = self.create_behavior_class_data(behavior_unit)?;
            let generated_file = self.generate_class_file(&class_data, None)?;
            generated_files.insert(file_name, generated_file);
        }
        
        Ok(generated_files)
    }
    
    /// Generate class name for a behavior unit
    fn generate_behavior_class_name(&self, behavior_name: &str) -> String {
        // Convert behavior name to PascalCase and ensure it's a valid C# class name
        let sanitized_name = behavior_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();
            
        // Ensure it starts with a capital letter
        let class_name = if sanitized_name.is_empty() {
            "UdonBehaviour".to_string()
        } else {
            let mut chars = sanitized_name.chars();
            match chars.next() {
                None => "UdonBehaviour".to_string(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };
        
        format!("{}.cs", class_name)
    }
    
    /// Create ParsedClass data from BehaviorUnit
    fn create_behavior_class_data(&self, behavior_unit: &BehaviorUnit) -> Result<ParsedClass> {
        let class_name = behavior_unit.name.clone();
        
        // Create fields for inter-behavior communication
        let mut fields = Vec::new();
        for call in &behavior_unit.inter_behavior_calls {
            if call.call_type == CallType::Direct {
                let field = ParsedField {
                    name: format!("_{}_reference", call.target_behavior.to_lowercase()),
                    class_name: Some(class_name.clone()),
                    namespace: None,
                    field_type: call.target_behavior.clone(),
                    visibility: "private".to_string(),
                    attributes: vec!["SerializeField".to_string()],
                };
                fields.push(field);
            }
        }
        
        // Create methods for Unity lifecycle events
        let mut methods = Vec::new();
        for event in &behavior_unit.unity_events {
            let method = ParsedMethod {
                name: event.clone(),
                class_name: Some(class_name.clone()),
                namespace: None,
                return_type: "void".to_string(),
                parameters: self.get_unity_event_parameters(event),
                visibility: "public".to_string(),
                is_static: false,
                attributes: Vec::new(),
            };
            methods.push(method);
        }
        
        // Add entry function method
        let entry_method = ParsedMethod {
            name: behavior_unit.entry_function.clone(),
            class_name: Some(class_name.clone()),
            namespace: None,
            return_type: "void".to_string(),
            parameters: Vec::new(),
            visibility: "public".to_string(),
            is_static: false,
            attributes: vec!["UdonSharpMethod".to_string()],
        };
        methods.push(entry_method);
        
        // Add inter-behavior communication methods
        for call in &behavior_unit.inter_behavior_calls {
            let method_name = format!("Call{}{}", call.target_behavior, call.function_name);
            let method = ParsedMethod {
                name: method_name,
                class_name: Some(class_name.clone()),
                namespace: None,
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "private".to_string(),
                is_static: false,
                attributes: Vec::new(),
            };
            methods.push(method);
        }
        
        Ok(ParsedClass {
            name: class_name,
            namespace: None,
            base_class: Some("UdonSharpBehaviour".to_string()),
            interfaces: Vec::new(),
            attributes: vec![
                "UdonBehaviourSyncMode(BehaviourSyncMode.Manual)".to_string()
            ],
            fields,
            methods,
            properties: Vec::new(),
        })
    }
    
    /// Get parameters for Unity event methods
    fn get_unity_event_parameters(&self, event_name: &str) -> Vec<String> {
        match event_name {
            "OnTriggerEnter" | "OnTriggerExit" | "OnTriggerStay" => {
                vec!["Collider other".to_string()]
            },
            "OnCollisionEnter" | "OnCollisionExit" | "OnCollisionStay" => {
                vec!["Collision collision".to_string()]
            },
            "OnPlayerJoined" | "OnPlayerLeft" => {
                vec!["VRCPlayerApi player".to_string()]
            },
            _ => Vec::new(),
        }
    }
    
    /// Apply final formatting to the generated content
    fn apply_formatting(&self, content: &str) -> Result<String> {
        let mut formatted = content.to_string();
        
        if self.config.normalize_line_endings {
            formatted = formatted.replace("\r\n", "\n");
        }
        
        if self.config.trim_trailing_whitespace {
            formatted = formatted
                .lines()
                .map(|line| line.trim_end())
                .collect::<Vec<_>>()
                .join("\n");
        }
        
        if self.config.ensure_final_newline && !formatted.ends_with('\n') {
            formatted.push('\n');
        }
        
        Ok(formatted)
    }
}

/// Template engine for generating C# code
pub struct TemplateEngine {
    templates: HashMap<TemplateType, String>,
    custom_templates: HashMap<String, String>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        templates.insert(TemplateType::UdonSharpClass, Self::default_udonsharp_class_template());
        templates.insert(TemplateType::StandardClass, Self::default_standard_class_template());
        templates.insert(TemplateType::MainClass, Self::default_main_class_template());
        templates.insert(TemplateType::Method, Self::default_method_template());
        templates.insert(TemplateType::Field, Self::default_field_template());
        templates.insert(TemplateType::Property, Self::default_property_template());
        templates.insert(TemplateType::UdonSharpMethod, Self::default_udonsharp_method_template());
        templates.insert(TemplateType::LifecycleMethod, Self::default_lifecycle_method_template());
        
        Self {
            templates,
            custom_templates: HashMap::new(),
        }
    }
    
    /// Create template engine with custom templates
    pub fn with_custom_templates(custom_templates: HashMap<String, String>) -> Self {
        let mut engine = Self::new();
        engine.custom_templates = custom_templates;
        engine
    }
    
    /// Add or update a custom template
    pub fn add_custom_template(&mut self, name: String, template: String) {
        self.custom_templates.insert(name, template);
    }
    
    /// Get a template by type
    pub fn get_template(&self, template_type: TemplateType) -> Option<&String> {
        self.templates.get(&template_type)
    }
    
    /// Get a custom template by name
    pub fn get_custom_template(&self, name: &str) -> Option<&String> {
        self.custom_templates.get(name)
    }
    
    /// Render a template with variable substitution
    pub fn render_template(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        let mut result = template.to_string();
        
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }
    
    /// Render a template by type with variables
    pub fn render_template_by_type(&self, template_type: TemplateType, variables: &HashMap<String, String>) -> Result<String> {
        let template = self.get_template(template_type)
            .ok_or_else(|| anyhow::anyhow!("Template not found for type: {:?}", template_type))?;
        
        self.render_template(template, variables)
    }
    
    /// Render a custom template with variables
    pub fn render_custom_template(&self, template_name: &str, variables: &HashMap<String, String>) -> Result<String> {
        let template = self.get_custom_template(template_name)
            .ok_or_else(|| anyhow::anyhow!("Custom template not found: {}", template_name))?;
        
        self.render_template(template, variables)
    }
    
    /// Render a class using the template
    pub fn render_class_template(&self, data: &ClassTemplateData) -> Result<String> {
        let mut content = String::new();
        
        // Generate class attributes
        for attribute in &data.class.attributes {
            content.push_str(&format!("[{}]\n", attribute));
        }
        
        // If no UdonBehaviourSyncMode attribute, add default
        if !data.class.attributes.iter().any(|attr| attr.contains("UdonBehaviourSyncMode")) {
            content.push_str("[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\n");
        }
        
        // Generate class declaration
        content.push_str(&format!("public class {}", data.class.name));
        
        // Add inheritance
        if let Some(base_class) = &data.class.base_class {
            content.push_str(&format!(" : {}", base_class));
        } else if data.config.default_base_class_for_udonsharp {
            content.push_str(" : UdonSharpBehaviour");
        }
        
        if !data.class.interfaces.is_empty() {
            let interfaces = data.class.interfaces.join(", ");
            if data.class.base_class.is_some() || data.config.default_base_class_for_udonsharp {
                content.push_str(&format!(", {}", interfaces));
            } else {
                content.push_str(&format!(" : {}", interfaces));
            }
        }
        
        content.push_str("\n{\n");
        
        // Generate fields
        for field in &data.class.fields {
            content.push_str(&self.render_field_template(field, "    ")?);
        }
        
        if !data.class.fields.is_empty() {
            content.push('\n');
        }
        
        // Generate properties
        for property in &data.class.properties {
            content.push_str(&self.render_property_template(property, "    ")?);
        }
        
        if !data.class.properties.is_empty() {
            content.push('\n');
        }
        
        // Generate methods
        for method in &data.class.methods {
            content.push_str(&self.render_method_template(method, "    ")?);
        }
        
        content.push_str("}\n");
        
        Ok(content)
    }
    
    /// Render a method using the template
    pub fn render_method_template(&self, method: &ParsedMethod, indent: &str) -> Result<String> {
        let mut content = String::new();
        
        // Add method attributes
        for attribute in &method.attributes {
            content.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Add UdonSharp method attribute if needed
        if !method.attributes.iter().any(|attr| attr.contains("UdonSharp")) &&
           !self.is_unity_lifecycle_method(&method.name) {
            content.push_str(&format!("{}[UdonSharpMethod]\n", indent));
        }
        
        // Generate method signature
        let modifiers = if method.is_static { "static " } else { "" };
        let parameters = method.parameters.join(", ");
        
        content.push_str(&format!(
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
        
        Ok(content)
    }
    
    /// Render a field using the template
    pub fn render_field_template(&self, field: &ParsedField, indent: &str) -> Result<String> {
        let mut content = String::new();
        
        // Add field attributes
        for attribute in &field.attributes {
            content.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Add UdonSynced attribute for public fields if not present
        if field.visibility == "public" && 
           !field.attributes.iter().any(|attr| attr.contains("UdonSynced")) {
            content.push_str(&format!("{}[UdonSynced]\n", indent));
        }
        
        // Generate field declaration
        content.push_str(&format!(
            "{}{} {} {};\n",
            indent,
            field.visibility,
            field.field_type,
            field.name
        ));
        
        Ok(content)
    }
    
    /// Render a property using the template
    pub fn render_property_template(&self, property: &ParsedProperty, indent: &str) -> Result<String> {
        let mut content = String::new();
        
        // Add property attributes
        for attribute in &property.attributes {
            content.push_str(&format!("{}[{}]\n", indent, attribute));
        }
        
        // Generate property declaration
        content.push_str(&format!(
            "{}{} {} {} {{ get; set; }}\n",
            indent,
            property.visibility,
            property.property_type,
            property.name
        ));
        
        Ok(content)
    }
    
    /// Check if a method is a Unity lifecycle method
    fn is_unity_lifecycle_method(&self, method_name: &str) -> bool {
        matches!(method_name, 
            "Start" | "Update" | "FixedUpdate" | "LateUpdate" | 
            "Awake" | "OnEnable" | "OnDisable" | "OnDestroy" |
            "OnTriggerEnter" | "OnTriggerExit" | "OnTriggerStay" |
            "OnCollisionEnter" | "OnCollisionExit" | "OnCollisionStay"
        )
    }
    
    /// Default UdonSharp class template
    fn default_udonsharp_class_template() -> String {
        r#"[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
public class {class_name} : UdonSharpBehaviour
{
{fields}

{properties}

{methods}
}"#.to_string()
    }
    
    /// Default standard class template
    fn default_standard_class_template() -> String {
        r#"public class {class_name}{inheritance}
{
{fields}

{properties}

{methods}
}"#.to_string()
    }
    
    /// Default main class template
    fn default_main_class_template() -> String {
        r#"[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
public class {class_name} : UdonSharpBehaviour
{
{component_fields}

    private bool _initialized = false;

{lifecycle_methods}

{wasm_integration_methods}

{udonsharp_methods}
}"#.to_string()
    }
    
    /// Default method template
    fn default_method_template() -> String {
        r#"    public {return_type} {method_name}({parameters})
    {
        {method_body}
    }"#.to_string()
    }
    
    /// Default UdonSharp method template
    fn default_udonsharp_method_template() -> String {
        r#"    [UdonSharpMethod]
    public {return_type} {method_name}({parameters})
    {
        {method_body}
    }"#.to_string()
    }
    
    /// Default lifecycle method template
    fn default_lifecycle_method_template() -> String {
        r#"    void {method_name}()
    {
        {method_body}
    }"#.to_string()
    }
    
    /// Default field template
    fn default_field_template() -> String {
        r#"    {attributes}
    {visibility} {field_type} {field_name};"#.to_string()
    }
    
    /// Default property template
    fn default_property_template() -> String {
        r#"    {attributes}
    {visibility} {property_type} {property_name} { get; set; }"#.to_string()
    }
    
    /// Create a template builder for complex template construction
    pub fn create_template_builder() -> TemplateBuilder {
        TemplateBuilder::new()
    }
    
    /// Load templates from a directory
    pub fn load_templates_from_directory(directory: &str) -> Result<HashMap<String, String>> {
        let mut templates = HashMap::new();
        
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "template") {
                let template_name = path.file_stem()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid template file name"))?
                    .to_string();
                
                let template_content = std::fs::read_to_string(&path)?;
                templates.insert(template_name, template_content);
            }
        }
        
        Ok(templates)
    }
    
    /// Save templates to a directory
    pub fn save_templates_to_directory(&self, directory: &str) -> Result<()> {
        std::fs::create_dir_all(directory)?;
        
        for (template_type, template_content) in &self.templates {
            let filename = format!("{:?}.template", template_type).to_lowercase();
            let filepath = format!("{}/{}", directory, filename);
            std::fs::write(filepath, template_content)?;
        }
        
        for (template_name, template_content) in &self.custom_templates {
            let filename = format!("{}.template", template_name);
            let filepath = format!("{}/{}", directory, filename);
            std::fs::write(filepath, template_content)?;
        }
        
        Ok(())
    }
}

/// Configuration for file generation
#[derive(Debug, Clone)]
pub struct FileGenerationConfig {
    pub include_header_comment: bool,
    pub normalize_line_endings: bool,
    pub trim_trailing_whitespace: bool,
    pub ensure_final_newline: bool,
    pub default_base_class_for_udonsharp: bool,
    pub auto_add_udonsharp_attributes: bool,
    pub organize_using_statements: bool,
}

impl Default for FileGenerationConfig {
    fn default() -> Self {
        Self {
            include_header_comment: true,
            normalize_line_endings: true,
            trim_trailing_whitespace: true,
            ensure_final_newline: true,
            default_base_class_for_udonsharp: true,
            auto_add_udonsharp_attributes: true,
            organize_using_statements: true,
        }
    }
}

/// Generated C# file with metadata
#[derive(Debug, Clone)]
pub struct GeneratedCSharpFile {
    pub name: String,
    pub content: String,
    pub using_statements: Vec<String>,
    pub namespace: Option<String>,
    pub dependencies: HashSet<String>,
    pub metadata: FileMetadata,
}

/// Metadata about a generated file
#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub generator_version: String,
    pub line_count: usize,
    pub character_count: usize,
}

/// Template data for class generation
#[derive(Debug, Clone)]
pub struct ClassTemplateData {
    pub class: ParsedClass,
    pub namespace: Option<String>,
    pub using_statements: Vec<String>,
    pub config: FileGenerationConfig,
}

/// Organized using statements
#[derive(Debug, Clone)]
struct OrganizedUsingStatements {
    pub system_usings: HashSet<String>,
    pub third_party_usings: HashSet<String>,
    pub project_usings: HashSet<String>,
}

/// Main class generator for creating UdonSharp entry point classes
pub struct MainClassGenerator {
    template_engine: TemplateEngine,
    config: MainClassConfig,
}

impl MainClassGenerator {
    /// Create a new main class generator with default configuration
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config: MainClassConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: MainClassConfig) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config,
        }
    }
    
    /// Generate the main entry point class that integrates with wasm2usharp
    pub fn generate_main_class(&self, class_name: &str, components: &[ComponentReference]) -> Result<GeneratedCSharpFile> {
        let main_class_data = MainClassData {
            class_name: class_name.to_string(),
            components: components.to_vec(),
            config: self.config.clone(),
        };
        
        let content = self.render_main_class_template(&main_class_data)?;
        
        let using_statements = self.generate_main_class_using_statements(&main_class_data)?;
        
        let file_data = CSharpFile {
            name: format!("{}.cs", class_name),
            content,
            using_statements,
            namespace: self.config.namespace.clone(),
            dependencies: self.extract_main_class_dependencies(&main_class_data)?,
        };
        
        let generator = CSharpFileGenerator::with_config(FileGenerationConfig {
            include_header_comment: true,
            normalize_line_endings: true,
            trim_trailing_whitespace: true,
            ensure_final_newline: true,
            default_base_class_for_udonsharp: true,
            auto_add_udonsharp_attributes: true,
            organize_using_statements: true,
        });
        
        generator.generate_file(&file_data)
    }
    
    /// Render the main class template
    fn render_main_class_template(&self, data: &MainClassData) -> Result<String> {
        let mut content = String::new();
        
        // Add class attributes
        content.push_str("[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\n");
        
        // Generate class declaration
        content.push_str(&format!("public class {} : UdonSharpBehaviour\n{{\n", data.class_name));
        
        // Generate component fields
        for component in &data.components {
            content.push_str(&self.render_component_field(component)?);
        }
        
        if !data.components.is_empty() {
            content.push('\n');
        }
        
        // Generate initialization flag
        content.push_str("    private bool _initialized = false;\n\n");
        
        // Generate Start method with wasm2usharp integration
        content.push_str(&self.render_start_method(data)?);
        
        // Generate Update method if needed
        if data.config.generate_update_method {
            content.push_str(&self.render_update_method(data)?);
        }
        
        // Generate component initialization methods
        content.push_str(&self.render_component_initialization_methods(data)?);
        
        // Generate wasm2usharp integration methods
        content.push_str(&self.render_wasm_integration_methods(data)?);
        
        // Generate UdonSharp lifecycle methods
        content.push_str(&self.render_udonsharp_lifecycle_methods(data)?);
        
        content.push_str("}\n");
        
        Ok(content)
    }
    
    /// Render component field declaration
    fn render_component_field(&self, component: &ComponentReference) -> Result<String> {
        let mut field = String::new();
        
        // Add UdonSynced attribute if the component should be synced
        if component.is_synced {
            field.push_str("    [UdonSynced]\n");
        }
        
        field.push_str(&format!(
            "    public {} {};\n",
            component.component_type,
            component.field_name
        ));
        
        Ok(field)
    }
    
    /// Render the Start method with wasm2usharp integration
    fn render_start_method(&self, data: &MainClassData) -> Result<String> {
        let mut method = String::new();
        
        method.push_str("    void Start()\n    {\n");
        method.push_str("        if (_initialized) return;\n\n");
        
        // Initialize components
        method.push_str("        // Initialize components\n");
        method.push_str("        InitializeComponents();\n\n");
        
        // Call wasm2usharp initialization
        method.push_str("        // Initialize WASM runtime\n");
        method.push_str("        try\n        {\n");
        method.push_str("            w2us_init();\n");
        method.push_str("            _initialized = true;\n");
        method.push_str("            Debug.Log(\"[{}] WASM runtime initialized successfully\");\n");
        method.push_str("        }\n");
        method.push_str("        catch (System.Exception ex)\n        {\n");
        method.push_str("            Debug.LogError($\"[{}] Failed to initialize WASM runtime: {{ex.Message}}\");\n");
        method.push_str("        }\n\n");
        
        // Call wasm2usharp start
        method.push_str("        // Start WASM execution\n");
        method.push_str("        if (_initialized)\n        {\n");
        method.push_str("            try\n            {\n");
        method.push_str("                w2us_start();\n");
        method.push_str("            }\n");
        method.push_str("            catch (System.Exception ex)\n            {\n");
        method.push_str("                Debug.LogError($\"[{}] Error in WASM start: {{ex.Message}}\");\n");
        method.push_str("            }\n");
        method.push_str("        }\n");
        
        method.push_str("    }\n\n");
        
        // Replace placeholders with actual class name
        let method = method.replace("{}", &data.class_name);
        
        Ok(method)
    }
    
    /// Render the Update method if needed
    fn render_update_method(&self, data: &MainClassData) -> Result<String> {
        let mut method = String::new();
        
        method.push_str("    void Update()\n    {\n");
        method.push_str("        if (!_initialized) return;\n\n");
        
        method.push_str("        // Call WASM update if available\n");
        method.push_str("        try\n        {\n");
        method.push_str("            // Note: Add w2us_update() call here if your WASM module exports an update function\n");
        method.push_str("        }\n");
        method.push_str("        catch (System.Exception ex)\n        {\n");
        method.push_str(&format!("            Debug.LogError($\"[{}] Error in WASM update: {{ex.Message}}\");\n", data.class_name));
        method.push_str("        }\n");
        
        method.push_str("    }\n\n");
        
        Ok(method)
    }
    
    /// Render component initialization methods
    fn render_component_initialization_methods(&self, data: &MainClassData) -> Result<String> {
        let mut methods = String::new();
        
        methods.push_str("    private void InitializeComponents()\n    {\n");
        
        for component in &data.components {
            methods.push_str(&format!(
                "        // Initialize {}\n",
                component.field_name
            ));
            
            match component.initialization_method {
                ComponentInitializationMethod::FindByName => {
                    methods.push_str(&format!(
                        "        if ({} == null)\n",
                        component.field_name
                    ));
                    methods.push_str(&format!(
                        "            {} = GameObject.Find(\"{}\")?.GetComponent<{}>();\n",
                        component.field_name,
                        component.game_object_name.as_ref().unwrap_or(&component.field_name),
                        component.component_type
                    ));
                }
                ComponentInitializationMethod::FindByTag => {
                    methods.push_str(&format!(
                        "        if ({} == null)\n",
                        component.field_name
                    ));
                    methods.push_str(&format!(
                        "            {} = GameObject.FindWithTag(\"{}\")?.GetComponent<{}>();\n",
                        component.field_name,
                        component.tag.as_ref().unwrap_or(&"Untagged".to_string()),
                        component.component_type
                    ));
                }
                ComponentInitializationMethod::GetComponent => {
                    methods.push_str(&format!(
                        "        if ({} == null)\n",
                        component.field_name
                    ));
                    methods.push_str(&format!(
                        "            {} = GetComponent<{}>();\n",
                        component.field_name,
                        component.component_type
                    ));
                }
                ComponentInitializationMethod::Manual => {
                    methods.push_str(&format!(
                        "        // {} requires manual assignment in the inspector\n",
                        component.field_name
                    ));
                }
            }
            
            methods.push_str(&format!(
                "        if ({} == null)\n",
                component.field_name
            ));
            methods.push_str(&format!(
                "            Debug.LogWarning($\"[{}] {} component not found or assigned\");\n\n",
                data.class_name,
                component.field_name
            ));
        }
        
        methods.push_str("    }\n\n");
        
        Ok(methods)
    }
    
    /// Render wasm2usharp integration methods
    fn render_wasm_integration_methods(&self, data: &MainClassData) -> Result<String> {
        let mut methods = String::new();
        
        // Add wasm2usharp entry point declarations
        methods.push_str("    // WASM entry points - these will be generated by wasm2usharp\n");
        methods.push_str("    // The actual implementations will be in the generated WASM conversion files\n\n");
        
        methods.push_str("    /// <summary>\n");
        methods.push_str("    /// Initialize the WASM runtime - called once at startup\n");
        methods.push_str("    /// This method will be implemented by the wasm2usharp generated code\n");
        methods.push_str("    /// </summary>\n");
        methods.push_str("    private void w2us_init()\n    {\n");
        methods.push_str("        // This method body will be replaced by wasm2usharp generated code\n");
        methods.push_str("        // It handles WASM module initialization, memory setup, and function table setup\n");
        methods.push_str("    }\n\n");
        
        methods.push_str("    /// <summary>\n");
        methods.push_str("    /// Start WASM execution - called after initialization\n");
        methods.push_str("    /// This method will be implemented by the wasm2usharp generated code\n");
        methods.push_str("    /// </summary>\n");
        methods.push_str("    private void w2us_start()\n    {\n");
        methods.push_str("        // This method body will be replaced by wasm2usharp generated code\n");
        methods.push_str("        // It calls the main entry point of the compiled Rust code\n");
        methods.push_str("    }\n\n");
        
        if data.config.generate_wasm_helper_methods {
            methods.push_str("    /// <summary>\n");
            methods.push_str("    /// Helper method to call WASM functions safely\n");
            methods.push_str("    /// </summary>\n");
            methods.push_str("    private void CallWasmFunction(string functionName)\n    {\n");
            methods.push_str("        if (!_initialized)\n        {\n");
            methods.push_str(&format!("            Debug.LogWarning($\"[{}] Cannot call WASM function '{{functionName}}' - runtime not initialized\");\n", data.class_name));
            methods.push_str("            return;\n");
            methods.push_str("        }\n\n");
            methods.push_str("        try\n        {\n");
            methods.push_str("            // WASM function calls will be implemented by wasm2usharp\n");
            methods.push_str("        }\n");
            methods.push_str("        catch (System.Exception ex)\n        {\n");
            methods.push_str(&format!("            Debug.LogError($\"[{}] Error calling WASM function '{{functionName}}': {{ex.Message}}\");\n", data.class_name));
            methods.push_str("        }\n");
            methods.push_str("    }\n\n");
        }
        
        Ok(methods)
    }
    
    /// Render UdonSharp lifecycle methods
    fn render_udonsharp_lifecycle_methods(&self, data: &MainClassData) -> Result<String> {
        let mut methods = String::new();
        
        if data.config.generate_networking_methods {
            methods.push_str("    // UdonSharp networking lifecycle methods\n\n");
            
            methods.push_str("    public override void OnPlayerJoined(VRCPlayerApi player)\n    {\n");
            methods.push_str("        if (!_initialized) return;\n\n");
            methods.push_str(&format!("        Debug.Log($\"[{}] Player joined: {{player.displayName}}\");\n", data.class_name));
            methods.push_str("        \n");
            methods.push_str("        // Call WASM player joined handler if available\n");
            methods.push_str("        try\n        {\n");
            methods.push_str("            // Add w2us_on_player_joined(player) call here if your WASM module exports this function\n");
            methods.push_str("        }\n");
            methods.push_str("        catch (System.Exception ex)\n        {\n");
            methods.push_str(&format!("            Debug.LogError($\"[{}] Error in WASM player joined handler: {{ex.Message}}\");\n", data.class_name));
            methods.push_str("        }\n");
            methods.push_str("    }\n\n");
            
            methods.push_str("    public override void OnPlayerLeft(VRCPlayerApi player)\n    {\n");
            methods.push_str("        if (!_initialized) return;\n\n");
            methods.push_str(&format!("        Debug.Log($\"[{}] Player left: {{player.displayName}}\");\n", data.class_name));
            methods.push_str("        \n");
            methods.push_str("        // Call WASM player left handler if available\n");
            methods.push_str("        try\n        {\n");
            methods.push_str("            // Add w2us_on_player_left(player) call here if your WASM module exports this function\n");
            methods.push_str("        }\n");
            methods.push_str("        catch (System.Exception ex)\n        {\n");
            methods.push_str(&format!("            Debug.LogError($\"[{}] Error in WASM player left handler: {{ex.Message}}\");\n", data.class_name));
            methods.push_str("        }\n");
            methods.push_str("    }\n\n");
        }
        
        if data.config.generate_interaction_methods {
            methods.push_str("    // UdonSharp interaction methods\n\n");
            
            methods.push_str("    public override void Interact()\n    {\n");
            methods.push_str("        if (!_initialized) return;\n\n");
            methods.push_str(&format!("        Debug.Log($\"[{}] Interact called by {{Networking.LocalPlayer.displayName}}\");\n", data.class_name));
            methods.push_str("        \n");
            methods.push_str("        // Call WASM interact handler if available\n");
            methods.push_str("        try\n        {\n");
            methods.push_str("            // Add w2us_on_interact() call here if your WASM module exports this function\n");
            methods.push_str("        }\n");
            methods.push_str("        catch (System.Exception ex)\n        {\n");
            methods.push_str(&format!("            Debug.LogError($\"[{}] Error in WASM interact handler: {{ex.Message}}\");\n", data.class_name));
            methods.push_str("        }\n");
            methods.push_str("    }\n\n");
        }
        
        Ok(methods)
    }
    
    /// Generate using statements for the main class
    fn generate_main_class_using_statements(&self, data: &MainClassData) -> Result<Vec<String>> {
        let mut using_statements = HashSet::new();
        
        // Standard UdonSharp usings
        using_statements.insert("UnityEngine".to_string());
        using_statements.insert("VRC.SDKBase".to_string());
        using_statements.insert("VRC.Udon".to_string());
        using_statements.insert("UdonSharp".to_string());
        
        // Add component-specific usings
        for component in &data.components {
            if let Some(namespace) = &component.namespace {
                using_statements.insert(namespace.clone());
            }
        }
        
        // Add system usings if needed
        if data.config.generate_networking_methods || data.config.generate_interaction_methods {
            using_statements.insert("System".to_string());
        }
        
        let mut sorted_statements: Vec<_> = using_statements.into_iter().collect();
        sorted_statements.sort();
        
        Ok(sorted_statements)
    }
    
    /// Extract dependencies for the main class
    fn extract_main_class_dependencies(&self, data: &MainClassData) -> Result<HashSet<String>> {
        let mut dependencies = HashSet::new();
        
        // Add UdonSharpBehaviour as base class dependency
        dependencies.insert("UdonSharpBehaviour".to_string());
        
        // Add component dependencies
        for component in &data.components {
            dependencies.insert(component.component_type.clone());
        }
        
        // Add VRChat API dependencies if networking methods are generated
        if data.config.generate_networking_methods {
            dependencies.insert("VRCPlayerApi".to_string());
        }
        
        Ok(dependencies)
    }
}

/// Configuration for main class generation
#[derive(Debug, Clone)]
pub struct MainClassConfig {
    pub namespace: Option<String>,
    pub generate_update_method: bool,
    pub generate_networking_methods: bool,
    pub generate_interaction_methods: bool,
    pub generate_wasm_helper_methods: bool,
}

impl Default for MainClassConfig {
    fn default() -> Self {
        Self {
            namespace: None,
            generate_update_method: true,
            generate_networking_methods: true,
            generate_interaction_methods: true,
            generate_wasm_helper_methods: true,
        }
    }
}

/// Data for main class template rendering
#[derive(Debug, Clone)]
pub struct MainClassData {
    pub class_name: String,
    pub components: Vec<ComponentReference>,
    pub config: MainClassConfig,
}

/// Reference to a component that should be managed by the main class
#[derive(Debug, Clone)]
pub struct ComponentReference {
    pub field_name: String,
    pub component_type: String,
    pub namespace: Option<String>,
    pub is_synced: bool,
    pub initialization_method: ComponentInitializationMethod,
    pub game_object_name: Option<String>,
    pub tag: Option<String>,
}

/// Methods for initializing components
#[derive(Debug, Clone)]
pub enum ComponentInitializationMethod {
    /// Find by GameObject name
    FindByName,
    /// Find by GameObject tag
    FindByTag,
    /// Get component from the same GameObject
    GetComponent,
    /// Manual assignment in inspector
    Manual,
}

/// Types of templates supported by the template engine
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TemplateType {
    /// Standard UdonSharp class template
    UdonSharpClass,
    /// Standard C# class template
    StandardClass,
    /// Main entry point class template
    MainClass,
    /// Method template
    Method,
    /// UdonSharp-specific method template
    UdonSharpMethod,
    /// Unity lifecycle method template
    LifecycleMethod,
    /// Field template
    Field,
    /// Property template
    Property,
}

/// Template builder for constructing complex templates
pub struct TemplateBuilder {
    template: String,
    variables: HashMap<String, String>,
}

impl TemplateBuilder {
    /// Create a new template builder
    pub fn new() -> Self {
        Self {
            template: String::new(),
            variables: HashMap::new(),
        }
    }
    
    /// Start with a base template
    pub fn with_template(mut self, template: String) -> Self {
        self.template = template;
        self
    }
    
    /// Add a variable substitution
    pub fn with_variable(mut self, key: String, value: String) -> Self {
        self.variables.insert(key, value);
        self
    }
    
    /// Add multiple variables
    pub fn with_variables(mut self, variables: HashMap<String, String>) -> Self {
        self.variables.extend(variables);
        self
    }
    
    /// Append content to the template
    pub fn append(mut self, content: &str) -> Self {
        self.template.push_str(content);
        self
    }
    
    /// Prepend content to the template
    pub fn prepend(mut self, content: &str) -> Self {
        self.template = format!("{}{}", content, self.template);
        self
    }
    
    /// Insert content at a specific placeholder
    pub fn insert_at_placeholder(mut self, placeholder: &str, content: &str) -> Self {
        let placeholder_with_braces = format!("{{{}}}", placeholder);
        self.template = self.template.replace(&placeholder_with_braces, content);
        self
    }
    
    /// Build the final template with variable substitution
    pub fn build(self) -> Result<String> {
        let mut result = self.template;
        
        for (key, value) in &self.variables {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        Ok(result)
    }
    
    /// Build and return both the template and remaining variables
    pub fn build_with_variables(self) -> Result<(String, HashMap<String, String>)> {
        let variables = self.variables.clone();
        let template = self.build()?;
        Ok((template, variables))
    }
}

impl MultiBehaviorFileGenerator {
    /// Create a new multi-behavior file generator
    pub fn new() -> Self {
        Self {
            file_generator: CSharpFileGenerator::new(),
            behavior_config: MultiBehaviorConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: MultiBehaviorConfig) -> Self {
        let file_config = FileGenerationConfig {
            include_header_comment: config.include_debug_info,
            normalize_line_endings: true,
            trim_trailing_whitespace: true,
            ensure_final_newline: true,
            default_base_class_for_udonsharp: true,
            auto_add_udonsharp_attributes: true,
            organize_using_statements: true,
        };
        
        Self {
            file_generator: CSharpFileGenerator::with_config(file_config),
            behavior_config: config,
        }
    }
    
    /// Generate multiple UdonBehaviour class files from behavior units
    pub fn generate_behavior_files(&self, behavior_units: &[BehaviorUnit]) -> Result<HashMap<String, GeneratedCSharpFile>> {
        let mut generated_files = HashMap::new();
        
        // Generate individual behavior class files
        for behavior_unit in behavior_units {
            let class_name = self.generate_behavior_class_name(&behavior_unit.name);
            let file_name = format!("{}.cs", class_name);
            
            let class_data = self.create_behavior_class_data(behavior_unit, behavior_units)?;
            let generated_file = self.file_generator.generate_class_file(&class_data, self.behavior_config.namespace.as_deref())?;
            
            generated_files.insert(file_name, generated_file);
        }
        
        // Generate SharedRuntime class if enabled
        if self.behavior_config.generate_shared_runtime {
            let shared_runtime_file = self.generate_shared_runtime_file(behavior_units)?;
            generated_files.insert("SharedRuntime.cs".to_string(), shared_runtime_file);
        }
        
        Ok(generated_files)
    }
    
    /// Generate class name for a behavior unit following naming convention
    pub fn generate_behavior_class_name(&self, behavior_name: &str) -> String {
        match &self.behavior_config.naming_convention {
            BehaviorNamingConvention::PascalCase => {
                self.to_pascal_case(behavior_name)
            }
            BehaviorNamingConvention::PascalCaseWithSuffix => {
                format!("{}Behaviour", self.to_pascal_case(behavior_name))
            }
            BehaviorNamingConvention::Custom(format) => {
                format.replace("{name}", &self.to_pascal_case(behavior_name))
            }
        }
    }
    
    /// Convert string to PascalCase
    fn to_pascal_case(&self, input: &str) -> String {
        // Handle different input formats: snake_case, camelCase, kebab-case
        let words: Vec<&str> = input
            .split(|c| c == '_' || c == '-' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
        
        if words.is_empty() {
            return "Behavior".to_string();
        }
        
        words.iter()
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
    
    /// Create ParsedClass data from BehaviorUnit
    fn create_behavior_class_data(&self, behavior_unit: &BehaviorUnit, all_behavior_units: &[BehaviorUnit]) -> Result<ParsedClass> {
        let class_name = self.generate_behavior_class_name(&behavior_unit.name);
        
        // Create fields for inter-behavior communication
        let mut fields = Vec::new();
        
        // Add reference fields for direct inter-behavior calls
        let mut referenced_behaviors = HashSet::new();
        for call in &behavior_unit.inter_behavior_calls {
            if call.call_type == CallType::Direct && !referenced_behaviors.contains(&call.target_behavior) {
                referenced_behaviors.insert(call.target_behavior.clone());
                
                let field = ParsedField {
                    name: format!("_{}_reference", self.to_snake_case(&call.target_behavior)),
                    class_name: Some(class_name.clone()),
                    namespace: self.behavior_config.namespace.clone(),
                    field_type: self.generate_behavior_class_name(&call.target_behavior),
                    visibility: "private".to_string(),
                    attributes: vec!["SerializeField".to_string()],
                };
                fields.push(field);
            }
        }
        
        // Add SharedRuntime reference if needed
        if self.behavior_config.generate_shared_runtime && !behavior_unit.shared_dependencies.is_empty() {
            let field = ParsedField {
                name: "_sharedRuntime".to_string(),
                class_name: Some(class_name.clone()),
                namespace: self.behavior_config.namespace.clone(),
                field_type: "SharedRuntime".to_string(),
                visibility: "private".to_string(),
                attributes: vec!["SerializeField".to_string()],
            };
            fields.push(field);
        }
        
        // Add initialization flag
        let init_field = ParsedField {
            name: "_initialized".to_string(),
            class_name: Some(class_name.clone()),
            namespace: self.behavior_config.namespace.clone(),
            field_type: "bool".to_string(),
            visibility: "private".to_string(),
            attributes: Vec::new(),
        };
        fields.push(init_field);
        
        // Create methods for Unity lifecycle events using the specialized mapper
        let mut methods = Vec::new();
        
        let lifecycle_mapper = UnityLifecycleMapper::new();
        let lifecycle_methods = lifecycle_mapper.map_lifecycle_methods(behavior_unit)?;
        methods.extend(lifecycle_methods);
        
        // Add entry function method if not already covered by Unity events
        if !behavior_unit.unity_events.contains(&behavior_unit.entry_function) {
            let entry_method = ParsedMethod {
                name: behavior_unit.entry_function.clone(),
                class_name: Some(class_name.clone()),
                namespace: self.behavior_config.namespace.clone(),
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                is_static: false,
                attributes: vec!["UdonSharpMethod".to_string()],
            };
            methods.push(entry_method);
        }
        
        // Generate inter-behavior communication methods using the specialized generator
        let comm_generator = InterBehaviorCommunicationGenerator::new();
        let comm_methods = comm_generator.generate_communication_methods(behavior_unit, all_behavior_units)?;
        methods.extend(comm_methods);
        
        Ok(ParsedClass {
            name: class_name,
            namespace: self.behavior_config.namespace.clone(),
            base_class: Some("UdonSharpBehaviour".to_string()),
            interfaces: Vec::new(),
            attributes: vec![
                "UdonBehaviourSyncMode(BehaviourSyncMode.Manual)".to_string()
            ],
            fields,
            methods,
            properties: Vec::new(),
        })
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
    
    /// Get parameters for Unity event methods
    fn get_unity_event_parameters(&self, event_name: &str) -> Vec<String> {
        match event_name {
            "OnTriggerEnter" | "OnTriggerExit" | "OnTriggerStay" => {
                vec!["Collider other".to_string()]
            },
            "OnCollisionEnter" | "OnCollisionExit" | "OnCollisionStay" => {
                vec!["Collision collision".to_string()]
            },
            "OnPlayerJoined" | "OnPlayerLeft" => {
                vec!["VRCPlayerApi player".to_string()]
            },
            "OnPickup" | "OnDrop" => {
                vec!["VRCPickup pickup".to_string()]
            },
            "OnOwnershipTransferred" => {
                vec!["VRCPlayerApi player".to_string()]
            },
            _ => Vec::new(),
        }
    }
    
    /// Get attributes for Unity event methods
    fn get_unity_event_attributes(&self, event_name: &str) -> Vec<String> {
        match event_name {
            "Start" | "Update" | "FixedUpdate" | "LateUpdate" | 
            "Awake" | "OnEnable" | "OnDisable" | "OnDestroy" => {
                // Unity lifecycle methods don't need UdonSharp attributes
                Vec::new()
            },
            "OnTriggerEnter" | "OnTriggerExit" | "OnTriggerStay" |
            "OnCollisionEnter" | "OnCollisionExit" | "OnCollisionStay" => {
                // Physics events don't need special attributes
                Vec::new()
            },
            "OnPlayerJoined" | "OnPlayerLeft" | "OnOwnershipTransferred" => {
                // VRChat networking events
                Vec::new()
            },
            _ => {
                // Custom events need UdonSharpMethod attribute
                vec!["UdonSharpMethod".to_string()]
            }
        }
    }
    
    /// Generate method name for inter-behavior communication
    fn generate_inter_behavior_method_name(&self, call: &InterBehaviorCall) -> String {
        match call.call_type {
            CallType::Direct => {
                format!("Call{}{}", 
                    self.to_pascal_case(&call.target_behavior),
                    self.to_pascal_case(&call.function_name)
                )
            },
            CallType::Event => {
                format!("Send{}Event", 
                    self.to_pascal_case(&call.function_name)
                )
            },
            CallType::Network => {
                format!("SendNetwork{}Event", 
                    self.to_pascal_case(&call.function_name)
                )
            },
        }
    }
    
    /// Generate SharedRuntime class file
    fn generate_shared_runtime_file(&self, behavior_units: &[BehaviorUnit]) -> Result<GeneratedCSharpFile> {
        let shared_runtime_config = SharedRuntimeConfig {
            class_name: "SharedRuntime".to_string(),
            namespace: self.behavior_config.namespace.clone(),
            is_static_class: true,
            include_initialization: true,
            include_utilities: true,
            custom_template: None,
        };
        
        let shared_runtime_generator = SharedRuntimeGenerator::with_config(shared_runtime_config);
        
        // Convert shared dependencies to SharedFunction objects
        let mut additional_functions = Vec::new();
        let mut all_shared_functions = HashSet::new();
        
        for behavior_unit in behavior_units {
            all_shared_functions.extend(behavior_unit.shared_dependencies.iter().cloned());
        }
        
        // Create SharedFunction objects for each shared dependency
        for func_name in &all_shared_functions {
            additional_functions.push(SharedFunction {
                name: func_name.clone(),
                return_type: "void".to_string(), // Simplified - would need actual analysis
                parameters: Vec::new(), // Simplified - would need actual analysis
                description: Some(format!("Shared function from WASM: {}", func_name)),
                is_static: true,
            });
        }
        
        shared_runtime_generator.generate_shared_runtime(behavior_units, &additional_functions)
    }
    
    /// Generate all files for a multi-behavior project
    pub fn generate_all_files(&self, behavior_units: &[BehaviorUnit], shared_functions: &[String]) -> Result<MultiBehaviorGenerationResult> {
        let mut behavior_files = HashMap::new();
        let mut shared_runtime_file = None;
        let mut generation_metadata = MultiBehaviorGenerationMetadata {
            total_behaviors: behavior_units.len(),
            total_files: 0,
            shared_functions_count: shared_functions.len(),
            inter_behavior_calls: 0,
            generation_time: chrono::Utc::now(),
        };
        
        // Generate behavior class files
        for behavior_unit in behavior_units {
            let class_name = self.generate_behavior_class_name(&behavior_unit.name);
            let file_name = format!("{}.cs", class_name);
            
            let class_data = self.create_behavior_class_data(behavior_unit, behavior_units)?;
            let generated_file = self.file_generator.generate_class_file(&class_data, self.behavior_config.namespace.as_deref())?;
            
            behavior_files.insert(behavior_unit.name.clone(), generated_file);
            generation_metadata.inter_behavior_calls += behavior_unit.inter_behavior_calls.len();
        }
        
        // Generate SharedRuntime if needed
        if self.behavior_config.generate_shared_runtime && !shared_functions.is_empty() {
            shared_runtime_file = Some(self.generate_shared_runtime_file(behavior_units)?);
        }
        
        generation_metadata.total_files = behavior_files.len() + if shared_runtime_file.is_some() { 1 } else { 0 };
        
        Ok(MultiBehaviorGenerationResult {
            behavior_files,
            shared_runtime_file,
            metadata: generation_metadata,
        })
    }
}

/// Result of multi-behavior file generation
#[derive(Debug)]
pub struct MultiBehaviorGenerationResult {
    /// Generated behavior class files (behavior_name -> generated_file)
    pub behavior_files: HashMap<String, GeneratedCSharpFile>,
    /// Generated SharedRuntime file (if enabled)
    pub shared_runtime_file: Option<GeneratedCSharpFile>,
    /// Generation metadata
    pub metadata: MultiBehaviorGenerationMetadata,
}

/// Metadata about the multi-behavior generation process
#[derive(Debug)]
pub struct MultiBehaviorGenerationMetadata {
    /// Total number of behaviors processed
    pub total_behaviors: usize,
    /// Total number of files generated
    pub total_files: usize,
    /// Number of shared functions
    pub shared_functions_count: usize,
    /// Total inter-behavior calls
    pub inter_behavior_calls: usize,
    /// Generation timestamp
    pub generation_time: chrono::DateTime<chrono::Utc>,
}

/// Specialized generator for SharedRuntime class
pub struct SharedRuntimeGenerator {
    template_engine: TemplateEngine,
    config: SharedRuntimeConfig,
}

/// Configuration for SharedRuntime generation
#[derive(Debug, Clone)]
pub struct SharedRuntimeConfig {
    /// Class name for the shared runtime
    pub class_name: String,
    /// Namespace for the shared runtime
    pub namespace: Option<String>,
    /// Whether to generate as static class
    pub is_static_class: bool,
    /// Whether to include initialization methods
    pub include_initialization: bool,
    /// Whether to include utility methods
    pub include_utilities: bool,
    /// Custom template for shared runtime
    pub custom_template: Option<String>,
}

impl Default for SharedRuntimeConfig {
    fn default() -> Self {
        Self {
            class_name: "SharedRuntime".to_string(),
            namespace: None,
            is_static_class: true,
            include_initialization: true,
            include_utilities: true,
            custom_template: None,
        }
    }
}

impl SharedRuntimeGenerator {
    /// Create a new SharedRuntime generator
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config: SharedRuntimeConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: SharedRuntimeConfig) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config,
        }
    }
    
    /// Generate SharedRuntime class from behavior units and shared functions
    pub fn generate_shared_runtime(&self, behavior_units: &[BehaviorUnit], additional_shared_functions: &[SharedFunction]) -> Result<GeneratedCSharpFile> {
        // Collect shared functions from behavior units
        let mut all_shared_functions = HashSet::new();
        let mut shared_data_structures = HashSet::new();
        
        for behavior_unit in behavior_units {
            all_shared_functions.extend(behavior_unit.shared_dependencies.iter().cloned());
            
            // Extract shared data from inter-behavior calls
            for call in &behavior_unit.inter_behavior_calls {
                if matches!(call.call_type, CallType::Direct) {
                    shared_data_structures.insert(format!("{}Reference", call.target_behavior));
                }
            }
        }
        
        // Create SharedRuntime class structure
        let shared_runtime_class = self.create_shared_runtime_class(
            &all_shared_functions.into_iter().collect::<Vec<_>>(),
            additional_shared_functions,
            &shared_data_structures.into_iter().collect::<Vec<_>>(),
        )?;
        
        // Generate the file
        let file_generator = CSharpFileGenerator::new();
        file_generator.generate_class_file(&shared_runtime_class, self.config.namespace.as_deref())
    }
    
    /// Create the ParsedClass structure for SharedRuntime
    fn create_shared_runtime_class(&self, shared_function_names: &[String], additional_functions: &[SharedFunction], shared_data: &[String]) -> Result<ParsedClass> {
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        
        // Add shared data fields
        for data_name in shared_data {
            let field = ParsedField {
                name: format!("_shared{}", data_name),
                class_name: Some(self.config.class_name.clone()),
                namespace: self.config.namespace.clone(),
                field_type: "object".to_string(), // Simplified - would need type analysis
                visibility: "private".to_string(),
                attributes: if self.config.is_static_class {
                    vec!["UdonSynced".to_string()]
                } else {
                    vec!["UdonSynced".to_string(), "SerializeField".to_string()]
                },
            };
            fields.push(field);
        }
        
        // Add initialization method if enabled
        if self.config.include_initialization {
            let init_method = ParsedMethod {
                name: "Initialize".to_string(),
                class_name: Some(self.config.class_name.clone()),
                namespace: self.config.namespace.clone(),
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                is_static: self.config.is_static_class,
                attributes: vec!["UdonSharpMethod".to_string()],
            };
            methods.push(init_method);
        }
        
        // Add shared function wrappers
        for func_name in shared_function_names {
            let method = ParsedMethod {
                name: func_name.clone(),
                class_name: Some(self.config.class_name.clone()),
                namespace: self.config.namespace.clone(),
                return_type: "void".to_string(), // Simplified
                parameters: Vec::new(), // Simplified
                visibility: "public".to_string(),
                is_static: self.config.is_static_class,
                attributes: vec!["UdonSharpMethod".to_string()],
            };
            methods.push(method);
        }
        
        // Add additional shared functions
        for shared_func in additional_functions {
            let method = ParsedMethod {
                name: shared_func.name.clone(),
                class_name: Some(self.config.class_name.clone()),
                namespace: self.config.namespace.clone(),
                return_type: shared_func.return_type.clone(),
                parameters: shared_func.parameters.clone(),
                visibility: "public".to_string(),
                is_static: self.config.is_static_class,
                attributes: vec!["UdonSharpMethod".to_string()],
            };
            methods.push(method);
        }
        
        // Add utility methods if enabled
        if self.config.include_utilities {
            self.add_utility_methods(&mut methods)?;
        }
        
        Ok(ParsedClass {
            name: self.config.class_name.clone(),
            namespace: self.config.namespace.clone(),
            base_class: if self.config.is_static_class { None } else { Some("UdonSharpBehaviour".to_string()) },
            interfaces: Vec::new(),
            attributes: if self.config.is_static_class {
                Vec::new()
            } else {
                vec!["UdonBehaviourSyncMode(BehaviourSyncMode.Manual)".to_string()]
            },
            fields,
            methods,
            properties: Vec::new(),
        })
    }
    
    /// Add common utility methods to SharedRuntime
    fn add_utility_methods(&self, methods: &mut Vec<ParsedMethod>) -> Result<()> {
        // Add logging utility
        let log_method = ParsedMethod {
            name: "LogMessage".to_string(),
            class_name: Some(self.config.class_name.clone()),
            namespace: self.config.namespace.clone(),
            return_type: "void".to_string(),
            parameters: vec!["string message".to_string()],
            visibility: "public".to_string(),
            is_static: self.config.is_static_class,
            attributes: vec!["UdonSharpMethod".to_string()],
        };
        methods.push(log_method);
        
        // Add error handling utility
        let error_method = ParsedMethod {
            name: "LogError".to_string(),
            class_name: Some(self.config.class_name.clone()),
            namespace: self.config.namespace.clone(),
            return_type: "void".to_string(),
            parameters: vec!["string error".to_string()],
            visibility: "public".to_string(),
            is_static: self.config.is_static_class,
            attributes: vec!["UdonSharpMethod".to_string()],
        };
        methods.push(error_method);
        
        // Add data validation utility
        let validate_method = ParsedMethod {
            name: "ValidateData".to_string(),
            class_name: Some(self.config.class_name.clone()),
            namespace: self.config.namespace.clone(),
            return_type: "bool".to_string(),
            parameters: vec!["object data".to_string()],
            visibility: "public".to_string(),
            is_static: self.config.is_static_class,
            attributes: vec!["UdonSharpMethod".to_string()],
        };
        methods.push(validate_method);
        
        // Add behavior reference management
        let get_behavior_method = ParsedMethod {
            name: "GetBehaviorReference".to_string(),
            class_name: Some(self.config.class_name.clone()),
            namespace: self.config.namespace.clone(),
            return_type: "UdonSharpBehaviour".to_string(),
            parameters: vec!["string behaviorName".to_string()],
            visibility: "public".to_string(),
            is_static: self.config.is_static_class,
            attributes: vec!["UdonSharpMethod".to_string()],
        };
        methods.push(get_behavior_method);
        
        Ok(())
    }
    
    /// Generate SharedRuntime template with custom content
    pub fn generate_with_template(&self, template_data: &SharedRuntimeTemplateData) -> Result<GeneratedCSharpFile> {
        let default_template = self.get_default_shared_runtime_template();
        let template = self.config.custom_template.as_ref()
            .unwrap_or(&default_template);
        
        let mut variables = HashMap::new();
        variables.insert("class_name".to_string(), self.config.class_name.clone());
        variables.insert("namespace".to_string(), self.config.namespace.clone().unwrap_or_default());
        variables.insert("base_class".to_string(), if self.config.is_static_class { 
            String::new() 
        } else { 
            " : UdonSharpBehaviour".to_string() 
        });
        variables.insert("static_modifier".to_string(), if self.config.is_static_class { 
            "static ".to_string() 
        } else { 
            String::new() 
        });
        
        // Add shared functions
        let shared_functions_code = template_data.shared_functions.iter()
            .map(|func| self.generate_shared_function_code(func))
            .collect::<Result<Vec<_>>>()?
            .join("\n\n");
        variables.insert("shared_functions".to_string(), shared_functions_code);
        
        // Add shared data
        let shared_data_code = template_data.shared_data.iter()
            .map(|data| self.generate_shared_data_code(data))
            .collect::<Result<Vec<_>>>()?
            .join("\n");
        variables.insert("shared_data".to_string(), shared_data_code);
        
        let content = self.template_engine.render_template(template, &variables)?;
        let line_count = content.lines().count();
        let character_count = content.len();
        
        Ok(GeneratedCSharpFile {
            name: format!("{}.cs", self.config.class_name),
            content,
            using_statements: self.get_shared_runtime_using_statements(),
            namespace: self.config.namespace.clone(),
            dependencies: self.get_shared_runtime_dependencies(),
            metadata: FileMetadata {
                generated_at: chrono::Utc::now(),
                generator_version: env!("CARGO_PKG_VERSION").to_string(),
                line_count,
                character_count,
            },
        })
    }
    
    /// Get default SharedRuntime template
    fn get_default_shared_runtime_template(&self) -> String {
        if self.config.is_static_class {
            r#"/// <summary>
/// Shared runtime utilities for multi-behavior UdonSharp projects
/// This class contains shared functions and data structures used across multiple behaviors
/// </summary>
public {static_modifier}class {class_name}{base_class}
{
{shared_data}

    /// <summary>
    /// Initialize the shared runtime system
    /// </summary>
    [UdonSharpMethod]
    public {static_modifier}void Initialize()
    {
        // Initialize shared data structures
        LogMessage("SharedRuntime initialized");
    }

{shared_functions}

    /// <summary>
    /// Log a message to the Unity console
    /// </summary>
    [UdonSharpMethod]
    public {static_modifier}void LogMessage(string message)
    {
        Debug.Log($"[SharedRuntime] {message}");
    }

    /// <summary>
    /// Log an error to the Unity console
    /// </summary>
    [UdonSharpMethod]
    public {static_modifier}void LogError(string error)
    {
        Debug.LogError($"[SharedRuntime] {error}");
    }

    /// <summary>
    /// Validate data integrity
    /// </summary>
    [UdonSharpMethod]
    public {static_modifier}bool ValidateData(object data)
    {
        return data != null;
    }
}"#.to_string()
        } else {
            r#"/// <summary>
/// Shared runtime utilities for multi-behavior UdonSharp projects
/// This class contains shared functions and data structures used across multiple behaviors
/// </summary>
[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
public class {class_name} : UdonSharpBehaviour
{
{shared_data}

    private bool _initialized = false;

    void Start()
    {
        Initialize();
    }

    /// <summary>
    /// Initialize the shared runtime system
    /// </summary>
    [UdonSharpMethod]
    public void Initialize()
    {
        if (_initialized) return;
        
        // Initialize shared data structures
        _initialized = true;
        LogMessage("SharedRuntime initialized");
    }

{shared_functions}

    /// <summary>
    /// Log a message to the Unity console
    /// </summary>
    [UdonSharpMethod]
    public void LogMessage(string message)
    {
        Debug.Log($"[SharedRuntime] {message}");
    }

    /// <summary>
    /// Log an error to the Unity console
    /// </summary>
    [UdonSharpMethod]
    public void LogError(string error)
    {
        Debug.LogError($"[SharedRuntime] {error}");
    }

    /// <summary>
    /// Validate data integrity
    /// </summary>
    [UdonSharpMethod]
    public bool ValidateData(object data)
    {
        return data != null;
    }
}"#.to_string()
        }
    }
    
    /// Generate code for a shared function
    fn generate_shared_function_code(&self, func: &SharedFunction) -> Result<String> {
        let static_modifier = if self.config.is_static_class { "static " } else { "" };
        let parameters = func.parameters.join(", ");
        
        Ok(format!(
            r#"    /// <summary>
    /// {description}
    /// </summary>
    [UdonSharpMethod]
    public {static_modifier}{return_type} {name}({parameters})
    {{
        // Shared function implementation
        // This will be replaced with actual WASM-generated code
        {default_implementation}
    }}"#,
            description = func.description.as_deref().unwrap_or(&format!("Shared function: {}", func.name)),
            static_modifier = static_modifier,
            return_type = func.return_type,
            name = func.name,
            parameters = parameters,
            default_implementation = self.generate_default_implementation(&func.return_type)
        ))
    }
    
    /// Generate code for shared data
    fn generate_shared_data_code(&self, data: &SharedData) -> Result<String> {
        let static_modifier = if self.config.is_static_class { "static " } else { "" };
        let attributes = if self.config.is_static_class {
            "[UdonSynced]".to_string()
        } else {
            "[UdonSynced, SerializeField]".to_string()
        };
        
        Ok(format!(
            r#"    /// <summary>
    /// {description}
    /// </summary>
    {attributes}
    public {static_modifier}{data_type} {name};"#,
            description = data.description.as_deref().unwrap_or(&format!("Shared data: {}", data.name)),
            attributes = attributes,
            static_modifier = static_modifier,
            data_type = data.data_type,
            name = data.name
        ))
    }
    
    /// Generate default implementation for a function based on return type
    fn generate_default_implementation(&self, return_type: &str) -> String {
        match return_type {
            "void" => String::new(),
            "bool" => "return false;".to_string(),
            "int" | "uint" | "short" | "ushort" | "byte" | "sbyte" => "return 0;".to_string(),
            "long" | "ulong" => "return 0L;".to_string(),
            "float" => "return 0.0f;".to_string(),
            "double" => "return 0.0;".to_string(),
            "string" => "return string.Empty;".to_string(),
            _ => "return null;".to_string(),
        }
    }
    
    /// Get using statements for SharedRuntime
    fn get_shared_runtime_using_statements(&self) -> Vec<String> {
        vec![
            "UnityEngine".to_string(),
            "VRC.SDKBase".to_string(),
            "VRC.Udon".to_string(),
            "UdonSharp".to_string(),
            "System.Collections.Generic".to_string(),
        ]
    }
    
    /// Get dependencies for SharedRuntime
    fn get_shared_runtime_dependencies(&self) -> HashSet<String> {
        let mut deps = HashSet::new();
        if !self.config.is_static_class {
            deps.insert("UdonSharpBehaviour".to_string());
        }
        deps.insert("Debug".to_string());
        deps
    }
}

/// Template data for SharedRuntime generation
#[derive(Debug, Clone)]
pub struct SharedRuntimeTemplateData {
    /// Shared functions to include
    pub shared_functions: Vec<SharedFunction>,
    /// Shared data structures
    pub shared_data: Vec<SharedData>,
    /// Additional template variables
    pub custom_variables: HashMap<String, String>,
}

/// Represents a shared function
#[derive(Debug, Clone)]
pub struct SharedFunction {
    /// Function name
    pub name: String,
    /// Return type
    pub return_type: String,
    /// Parameters
    pub parameters: Vec<String>,
    /// Function description
    pub description: Option<String>,
    /// Whether the function is static
    pub is_static: bool,
}

/// Represents shared data
#[derive(Debug, Clone)]
pub struct SharedData {
    /// Data name
    pub name: String,
    /// Data type
    pub data_type: String,
    /// Data description
    pub description: Option<String>,
    /// Whether the data should be synced
    pub is_synced: bool,
}

/// Generator for inter-behavior communication code
pub struct InterBehaviorCommunicationGenerator {
    template_engine: TemplateEngine,
    config: InterBehaviorConfig,
}

/// Configuration for inter-behavior communication generation
#[derive(Debug, Clone)]
pub struct InterBehaviorConfig {
    /// Whether to use SendCustomEvent for cross-behavior calls
    pub use_custom_events: bool,
    /// Whether to use networking for remote calls
    pub enable_networking: bool,
    /// Timeout for network calls in seconds
    pub network_timeout: f32,
    /// Whether to include error handling
    pub include_error_handling: bool,
    /// Custom event prefix
    pub event_prefix: String,
}

impl Default for InterBehaviorConfig {
    fn default() -> Self {
        Self {
            use_custom_events: true,
            enable_networking: true,
            network_timeout: 5.0,
            include_error_handling: true,
            event_prefix: "UdonSharp_".to_string(),
        }
    }
}

impl InterBehaviorCommunicationGenerator {
    /// Create a new inter-behavior communication generator
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config: InterBehaviorConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: InterBehaviorConfig) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config,
        }
    }
    
    /// Generate inter-behavior communication methods for a behavior unit
    pub fn generate_communication_methods(&self, behavior_unit: &BehaviorUnit, all_behaviors: &[BehaviorUnit]) -> Result<Vec<ParsedMethod>> {
        let mut methods = Vec::new();
        
        // Generate methods for each inter-behavior call
        for call in &behavior_unit.inter_behavior_calls {
            let method = self.generate_communication_method(call, behavior_unit, all_behaviors)?;
            methods.push(method);
        }
        
        // Generate event handlers for incoming calls
        let event_handlers = self.generate_event_handlers(behavior_unit, all_behaviors)?;
        methods.extend(event_handlers);
        
        // Generate initialization method for setting up references
        let init_method = self.generate_initialization_method(behavior_unit)?;
        methods.push(init_method);
        
        Ok(methods)
    }
    
    /// Generate a communication method for a specific inter-behavior call
    fn generate_communication_method(&self, call: &InterBehaviorCall, source_behavior: &BehaviorUnit, all_behaviors: &[BehaviorUnit]) -> Result<ParsedMethod> {
        let method_name = self.generate_communication_method_name(call);
        let method_body = self.generate_communication_method_body(call, source_behavior, all_behaviors)?;
        
        Ok(ParsedMethod {
            name: method_name,
            class_name: Some(source_behavior.name.clone()),
            namespace: None,
            return_type: "void".to_string(),
            parameters: self.get_communication_method_parameters(call),
            visibility: "private".to_string(),
            is_static: false,
            attributes: vec!["UdonSharpMethod".to_string()],
        })
    }
    
    /// Generate method name for inter-behavior communication
    fn generate_communication_method_name(&self, call: &InterBehaviorCall) -> String {
        match call.call_type {
            CallType::Direct => {
                format!("Call{}{}", 
                    self.to_pascal_case(&call.target_behavior),
                    self.to_pascal_case(&call.function_name)
                )
            },
            CallType::Event => {
                format!("Send{}Event", 
                    self.to_pascal_case(&call.function_name)
                )
            },
            CallType::Network => {
                format!("SendNetwork{}Event", 
                    self.to_pascal_case(&call.function_name)
                )
            },
        }
    }
    
    /// Generate method body for inter-behavior communication
    fn generate_communication_method_body(&self, call: &InterBehaviorCall, source_behavior: &BehaviorUnit, all_behaviors: &[BehaviorUnit]) -> Result<String> {
        match call.call_type {
            CallType::Direct => self.generate_direct_call_body(call, source_behavior),
            CallType::Event => self.generate_event_call_body(call, source_behavior, all_behaviors),
            CallType::Network => self.generate_network_call_body(call, source_behavior, all_behaviors),
        }
    }
    
    /// Generate body for direct method calls
    fn generate_direct_call_body(&self, call: &InterBehaviorCall, _source_behavior: &BehaviorUnit) -> Result<String> {
        let reference_field = format!("_{}_reference", self.to_snake_case(&call.target_behavior));
        let target_method = &call.function_name;
        
        let mut body = String::new();
        
        if self.config.include_error_handling {
            body.push_str(&format!(
                r#"        if ({reference_field} == null)
        {{
            Debug.LogError($"[{{GetType().Name}}] {target_behavior} reference not set for {target_method}");
            return;
        }}

        try
        {{
            {reference_field}.{target_method}();
        }}
        catch (System.Exception ex)
        {{
            Debug.LogError($"[{{GetType().Name}}] Error calling {target_behavior}.{target_method}: {{ex.Message}}");
        }}"#,
                reference_field = reference_field,
                target_behavior = call.target_behavior,
                target_method = target_method
            ));
        } else {
            body.push_str(&format!(
                r#"        if ({reference_field} != null)
        {{
            {reference_field}.{target_method}();
        }}"#,
                reference_field = reference_field,
                target_method = target_method
            ));
        }
        
        Ok(body)
    }
    
    /// Generate body for event-based calls
    fn generate_event_call_body(&self, call: &InterBehaviorCall, _source_behavior: &BehaviorUnit, all_behaviors: &[BehaviorUnit]) -> Result<String> {
        let event_name = format!("{}{}", self.config.event_prefix, call.function_name);
        let target_behavior_obj = format!("_{}_reference", self.to_snake_case(&call.target_behavior));
        
        // Find the target behavior to get its GameObject reference
        let target_behavior = all_behaviors.iter()
            .find(|b| b.name == call.target_behavior);
        
        let mut body = String::new();
        
        if self.config.include_error_handling {
            body.push_str(&format!(
                r#"        if ({target_behavior_obj} == null)
        {{
            Debug.LogError($"[{{GetType().Name}}] {target_behavior} reference not set for event {event_name}");
            return;
        }}

        try
        {{
            {target_behavior_obj}.SendCustomEvent("{event_name}");
        }}
        catch (System.Exception ex)
        {{
            Debug.LogError($"[{{GetType().Name}}] Error sending event {event_name} to {target_behavior}: {{ex.Message}}");
        }}"#,
                target_behavior_obj = target_behavior_obj,
                target_behavior = call.target_behavior,
                event_name = event_name
            ));
        } else {
            body.push_str(&format!(
                r#"        if ({target_behavior_obj} != null)
        {{
            {target_behavior_obj}.SendCustomEvent("{event_name}");
        }}"#,
                target_behavior_obj = target_behavior_obj,
                event_name = event_name
            ));
        }
        
        Ok(body)
    }
    
    /// Generate body for network-based calls
    fn generate_network_call_body(&self, call: &InterBehaviorCall, _source_behavior: &BehaviorUnit, _all_behaviors: &[BehaviorUnit]) -> Result<String> {
        let event_name = format!("{}{}", self.config.event_prefix, call.function_name);
        let target_behavior_obj = format!("_{}_reference", self.to_snake_case(&call.target_behavior));
        
        let mut body = String::new();
        
        if self.config.include_error_handling {
            body.push_str(&format!(
                r#"        if ({target_behavior_obj} == null)
        {{
            Debug.LogError($"[{{GetType().Name}}] {target_behavior} reference not set for network event {event_name}");
            return;
        }}

        if (!Networking.IsOwner(gameObject))
        {{
            Debug.LogWarning($"[{{GetType().Name}}] Cannot send network event {event_name} - not owner");
            return;
        }}

        try
        {{
            {target_behavior_obj}.SendCustomNetworkEvent(VRC.Udon.Common.Interfaces.NetworkEventTarget.All, "{event_name}");
        }}
        catch (System.Exception ex)
        {{
            Debug.LogError($"[{{GetType().Name}}] Error sending network event {event_name} to {target_behavior}: {{ex.Message}}");
        }}"#,
                target_behavior_obj = target_behavior_obj,
                target_behavior = call.target_behavior,
                event_name = event_name
            ));
        } else {
            body.push_str(&format!(
                r#"        if ({target_behavior_obj} != null && Networking.IsOwner(gameObject))
        {{
            {target_behavior_obj}.SendCustomNetworkEvent(VRC.Udon.Common.Interfaces.NetworkEventTarget.All, "{event_name}");
        }}"#,
                target_behavior_obj = target_behavior_obj,
                event_name = event_name
            ));
        }
        
        Ok(body)
    }
    
    /// Generate event handlers for incoming inter-behavior calls
    fn generate_event_handlers(&self, behavior_unit: &BehaviorUnit, all_behaviors: &[BehaviorUnit]) -> Result<Vec<ParsedMethod>> {
        let mut handlers = Vec::new();
        
        // Find all calls that target this behavior
        for other_behavior in all_behaviors {
            if other_behavior.name == behavior_unit.name {
                continue; // Skip self
            }
            
            for call in &other_behavior.inter_behavior_calls {
                if call.target_behavior == behavior_unit.name && 
                   matches!(call.call_type, CallType::Event | CallType::Network) {
                    
                    let handler = self.generate_event_handler(call, behavior_unit)?;
                    handlers.push(handler);
                }
            }
        }
        
        Ok(handlers)
    }
    
    /// Generate a single event handler method
    fn generate_event_handler(&self, call: &InterBehaviorCall, target_behavior: &BehaviorUnit) -> Result<ParsedMethod> {
        let event_name = format!("{}{}", self.config.event_prefix, call.function_name);
        let handler_body = self.generate_event_handler_body(call, target_behavior)?;
        
        Ok(ParsedMethod {
            name: event_name.clone(),
            class_name: Some(target_behavior.name.clone()),
            namespace: None,
            return_type: "void".to_string(),
            parameters: Vec::new(),
            visibility: "public".to_string(),
            is_static: false,
            attributes: vec!["UdonSharpMethod".to_string()],
        })
    }
    
    /// Generate body for event handler
    fn generate_event_handler_body(&self, call: &InterBehaviorCall, _target_behavior: &BehaviorUnit) -> Result<String> {
        let mut body = String::new();
        
        if self.config.include_error_handling {
            body.push_str(&format!(
                r#"        try
        {{
            // Handle event from {source_behavior}
            {function_name}();
        }}
        catch (System.Exception ex)
        {{
            Debug.LogError($"[{{GetType().Name}}] Error handling event {function_name} from {source_behavior}: {{ex.Message}}");
        }}"#,
                source_behavior = call.source_behavior,
                function_name = call.function_name
            ));
        } else {
            body.push_str(&format!(
                r#"        // Handle event from {source_behavior}
        {function_name}();"#,
                source_behavior = call.source_behavior,
                function_name = call.function_name
            ));
        }
        
        Ok(body)
    }
    
    /// Generate initialization method for setting up behavior references
    fn generate_initialization_method(&self, behavior_unit: &BehaviorUnit) -> Result<ParsedMethod> {
        let mut body = String::new();
        
        body.push_str("        // Initialize behavior references\n");
        
        // Get unique target behaviors
        let mut target_behaviors = HashSet::new();
        for call in &behavior_unit.inter_behavior_calls {
            if call.call_type == CallType::Direct {
                target_behaviors.insert(&call.target_behavior);
            }
        }
        
        for target_behavior in target_behaviors {
            let reference_field = format!("_{}_reference", self.to_snake_case(target_behavior));
            let class_name = self.to_pascal_case(target_behavior);
            
            body.push_str(&format!(
                r#"        if ({reference_field} == null)
        {{
            {reference_field} = GameObject.FindObjectOfType<{class_name}>();
            if ({reference_field} == null)
            {{
                Debug.LogWarning($"[{{GetType().Name}}] Could not find {class_name} reference");
            }}
        }}
"#,
                reference_field = reference_field,
                class_name = class_name
            ));
        }
        
        Ok(ParsedMethod {
            name: "InitializeBehaviorReferences".to_string(),
            class_name: Some(behavior_unit.name.clone()),
            namespace: None,
            return_type: "void".to_string(),
            parameters: Vec::new(),
            visibility: "private".to_string(),
            is_static: false,
            attributes: vec!["UdonSharpMethod".to_string()],
        })
    }
    
    /// Get parameters for communication methods
    fn get_communication_method_parameters(&self, _call: &InterBehaviorCall) -> Vec<String> {
        // For now, return empty parameters
        // In a real implementation, this would analyze the target function signature
        Vec::new()
    }
    
    /// Convert string to PascalCase
    fn to_pascal_case(&self, input: &str) -> String {
        let words: Vec<&str> = input
            .split(|c| c == '_' || c == '-' || c == ' ')
            .filter(|s| !s.is_empty())
            .collect();
        
        words.iter()
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
}

/// Generator for Unity lifecycle method mapping
pub struct UnityLifecycleMapper {
    config: UnityLifecycleConfig,
}

/// Configuration for Unity lifecycle mapping
#[derive(Debug, Clone)]
pub struct UnityLifecycleConfig {
    /// Whether to generate all lifecycle methods or only used ones
    pub generate_all_methods: bool,
    /// Whether to include VRChat-specific events
    pub include_vrchat_events: bool,
    /// Whether to include physics events
    pub include_physics_events: bool,
    /// Whether to include UI events
    pub include_ui_events: bool,
    /// Custom event mappings (rust_function_name -> unity_event_name)
    pub custom_mappings: HashMap<String, String>,
}

impl Default for UnityLifecycleConfig {
    fn default() -> Self {
        Self {
            generate_all_methods: false,
            include_vrchat_events: true,
            include_physics_events: true,
            include_ui_events: false,
            custom_mappings: HashMap::new(),
        }
    }
}

impl UnityLifecycleMapper {
    /// Create a new Unity lifecycle mapper
    pub fn new() -> Self {
        Self {
            config: UnityLifecycleConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: UnityLifecycleConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// Map Rust functions to Unity lifecycle methods
    pub fn map_lifecycle_methods(&self, behavior_unit: &BehaviorUnit) -> Result<Vec<ParsedMethod>> {
        let mut methods = Vec::new();
        
        // Map explicit Unity events from the behavior unit
        for event in &behavior_unit.unity_events {
            let method = self.create_unity_lifecycle_method(event, behavior_unit)?;
            methods.push(method);
        }
        
        // Map Rust function names to Unity events based on naming conventions
        for function_name in &behavior_unit.local_functions {
            if let Some(unity_event) = self.infer_unity_event_from_function_name(function_name) {
                // Only add if not already present
                if !behavior_unit.unity_events.contains(&unity_event) {
                    let method = self.create_unity_lifecycle_method(&unity_event, behavior_unit)?;
                    methods.push(method);
                }
            }
        }
        
        // Add standard lifecycle methods if configured
        if self.config.generate_all_methods {
            methods.extend(self.generate_standard_lifecycle_methods(behavior_unit)?);
        }
        
        Ok(methods)
    }
    
    /// Create a Unity lifecycle method
    fn create_unity_lifecycle_method(&self, event_name: &str, behavior_unit: &BehaviorUnit) -> Result<ParsedMethod> {
        let method_info = self.get_unity_method_info(event_name);
        
        Ok(ParsedMethod {
            name: event_name.to_string(),
            class_name: Some(behavior_unit.name.clone()),
            namespace: None,
            return_type: method_info.return_type,
            parameters: method_info.parameters,
            visibility: method_info.visibility,
            is_static: false,
            attributes: method_info.attributes,
        })
    }
    
    /// Get method information for a Unity event
    fn get_unity_method_info(&self, event_name: &str) -> UnityMethodInfo {
        match event_name {
            // Standard Unity lifecycle
            "Awake" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(), // Unity lifecycle methods don't have explicit visibility
                attributes: Vec::new(),
            },
            "Start" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "Update" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "FixedUpdate" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "LateUpdate" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnEnable" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnDisable" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnDestroy" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            
            // Physics events
            "OnTriggerEnter" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["Collider other".to_string()],
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnTriggerExit" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["Collider other".to_string()],
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnTriggerStay" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["Collider other".to_string()],
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnCollisionEnter" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["Collision collision".to_string()],
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnCollisionExit" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["Collision collision".to_string()],
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            "OnCollisionStay" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["Collision collision".to_string()],
                visibility: "void".to_string(),
                attributes: Vec::new(),
            },
            
            // VRChat events
            "OnPlayerJoined" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["VRCPlayerApi player".to_string()],
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnPlayerLeft" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["VRCPlayerApi player".to_string()],
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnOwnershipTransferred" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["VRCPlayerApi player".to_string()],
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnPickup" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnDrop" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnPickupUseDown" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnPickupUseUp" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            
            // UI events (if enabled)
            "OnPointerClick" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["PointerEventData eventData".to_string()],
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnPointerEnter" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["PointerEventData eventData".to_string()],
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            "OnPointerExit" => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: vec!["PointerEventData eventData".to_string()],
                visibility: "public".to_string(),
                attributes: Vec::new(),
            },
            
            // Default for custom events
            _ => UnityMethodInfo {
                return_type: "void".to_string(),
                parameters: Vec::new(),
                visibility: "public".to_string(),
                attributes: vec!["UdonSharpMethod".to_string()],
            },
        }
    }
    
    /// Infer Unity event from Rust function name
    fn infer_unity_event_from_function_name(&self, function_name: &str) -> Option<String> {
        // Check custom mappings first
        if let Some(unity_event) = self.config.custom_mappings.get(function_name) {
            return Some(unity_event.clone());
        }
        
        // Convert common Rust naming patterns to Unity events
        let lower_name = function_name.to_lowercase();
        
        // Direct matches
        if lower_name.contains("start") && !lower_name.contains("_") {
            return Some("Start".to_string());
        }
        if lower_name.contains("update") && !lower_name.contains("_") {
            return Some("Update".to_string());
        }
        if lower_name.contains("awake") {
            return Some("Awake".to_string());
        }
        
        // Pattern-based matching
        if lower_name.ends_with("_start") || lower_name.ends_with("start") {
            return Some("Start".to_string());
        }
        if lower_name.ends_with("_update") || lower_name.ends_with("update") {
            return Some("Update".to_string());
        }
        if lower_name.ends_with("_awake") || lower_name.ends_with("awake") {
            return Some("Awake".to_string());
        }
        
        // Physics events
        if lower_name.contains("trigger_enter") || lower_name.contains("triggerenter") {
            return Some("OnTriggerEnter".to_string());
        }
        if lower_name.contains("trigger_exit") || lower_name.contains("triggerexit") {
            return Some("OnTriggerExit".to_string());
        }
        if lower_name.contains("collision_enter") || lower_name.contains("collisionenter") {
            return Some("OnCollisionEnter".to_string());
        }
        
        // VRChat events
        if lower_name.contains("player_joined") || lower_name.contains("playerjoined") {
            return Some("OnPlayerJoined".to_string());
        }
        if lower_name.contains("player_left") || lower_name.contains("playerleft") {
            return Some("OnPlayerLeft".to_string());
        }
        if lower_name.contains("pickup") && !lower_name.contains("use") {
            return Some("OnPickup".to_string());
        }
        if lower_name.contains("drop") {
            return Some("OnDrop".to_string());
        }
        
        None
    }
    
    /// Generate standard lifecycle methods
    fn generate_standard_lifecycle_methods(&self, behavior_unit: &BehaviorUnit) -> Result<Vec<ParsedMethod>> {
        let mut methods = Vec::new();
        
        let standard_methods = vec!["Start", "Update"];
        
        for method_name in standard_methods {
            // Only add if not already present
            if !behavior_unit.unity_events.contains(&method_name.to_string()) {
                let method = self.create_unity_lifecycle_method(method_name, behavior_unit)?;
                methods.push(method);
            }
        }
        
        Ok(methods)
    }
    
    /// Generate method body for Unity lifecycle methods
    pub fn generate_lifecycle_method_body(&self, event_name: &str, behavior_unit: &BehaviorUnit, rust_function_name: Option<&str>) -> String {
        let mut body = String::new();
        
        // Add initialization check for Start method
        if event_name == "Start" {
            body.push_str("        if (!_initialized)\n");
            body.push_str("        {\n");
            body.push_str("            InitializeBehaviorReferences();\n");
            body.push_str("            _initialized = true;\n");
            body.push_str("        }\n\n");
        }
        
        // Call the corresponding Rust function if available
        if let Some(rust_func) = rust_function_name {
            body.push_str(&format!("        // Call Rust function: {}\n", rust_func));
            body.push_str(&format!("        {}();\n", rust_func));
        } else {
            // Generate default implementation
            match event_name {
                "Start" => {
                    body.push_str("        // Behavior initialization complete\n");
                    body.push_str(&format!("        Debug.Log($\"[{}] Started\");\n", behavior_unit.name));
                },
                "Update" => {
                    body.push_str("        // Update logic here\n");
                },
                "OnTriggerEnter" => {
                    body.push_str("        // Handle trigger enter\n");
                    body.push_str(&format!("        Debug.Log($\"[{}] Trigger entered by {{other.name}}\");\n", behavior_unit.name));
                },
                "OnPlayerJoined" => {
                    body.push_str("        // Handle player joined\n");
                    body.push_str(&format!("        Debug.Log($\"[{}] Player joined: {{player.displayName}}\");\n", behavior_unit.name));
                },
                _ => {
                    body.push_str(&format!("        // {} event handler\n", event_name));
                }
            }
        }
        
        body
    }
    
    /// Create a mapping from Rust functions to Unity events
    pub fn create_function_to_event_mapping(&self, behavior_unit: &BehaviorUnit) -> HashMap<String, String> {
        let mut mapping = HashMap::new();
        
        // Add explicit mappings from unity_events
        for (i, event) in behavior_unit.unity_events.iter().enumerate() {
            if i < behavior_unit.local_functions.len() {
                let rust_function = behavior_unit.local_functions.iter().nth(i).unwrap();
                mapping.insert(rust_function.clone(), event.clone());
            }
        }
        
        // Add inferred mappings
        for function_name in &behavior_unit.local_functions {
            if !mapping.contains_key(function_name) {
                if let Some(unity_event) = self.infer_unity_event_from_function_name(function_name) {
                    mapping.insert(function_name.clone(), unity_event);
                }
            }
        }
        
        mapping
    }
    
    /// Validate Unity event mappings
    pub fn validate_mappings(&self, behavior_unit: &BehaviorUnit) -> Result<Vec<MappingValidationError>> {
        let mut errors = Vec::new();
        
        // Check for conflicting mappings
        let mapping = self.create_function_to_event_mapping(behavior_unit);
        let mut event_to_functions: HashMap<String, Vec<String>> = HashMap::new();
        
        for (function, event) in &mapping {
            event_to_functions.entry(event.clone()).or_insert_with(Vec::new).push(function.clone());
        }
        
        for (event, functions) in event_to_functions {
            if functions.len() > 1 {
                errors.push(MappingValidationError {
                    error_type: MappingErrorType::ConflictingMappings,
                    message: format!("Multiple functions mapped to event '{}': {:?}", event, functions),
                    event_name: Some(event),
                    function_names: functions,
                });
            }
        }
        
        // Check for unsupported events
        for event in &behavior_unit.unity_events {
            if !self.is_supported_unity_event(event) {
                errors.push(MappingValidationError {
                    error_type: MappingErrorType::UnsupportedEvent,
                    message: format!("Unsupported Unity event: '{}'", event),
                    event_name: Some(event.clone()),
                    function_names: Vec::new(),
                });
            }
        }
        
        Ok(errors)
    }
    
    /// Check if a Unity event is supported
    fn is_supported_unity_event(&self, event_name: &str) -> bool {
        matches!(event_name,
            "Awake" | "Start" | "Update" | "FixedUpdate" | "LateUpdate" |
            "OnEnable" | "OnDisable" | "OnDestroy" |
            "OnTriggerEnter" | "OnTriggerExit" | "OnTriggerStay" |
            "OnCollisionEnter" | "OnCollisionExit" | "OnCollisionStay" |
            "OnPlayerJoined" | "OnPlayerLeft" | "OnOwnershipTransferred" |
            "OnPickup" | "OnDrop" | "OnPickupUseDown" | "OnPickupUseUp" |
            "OnPointerClick" | "OnPointerEnter" | "OnPointerExit"
        ) || self.config.custom_mappings.values().any(|v| v == event_name)
    }
}

/// Information about a Unity method
#[derive(Debug, Clone)]
struct UnityMethodInfo {
    return_type: String,
    parameters: Vec<String>,
    visibility: String,
    attributes: Vec<String>,
}

/// Validation error for Unity event mappings
#[derive(Debug, Clone)]
pub struct MappingValidationError {
    pub error_type: MappingErrorType,
    pub message: String,
    pub event_name: Option<String>,
    pub function_names: Vec<String>,
}

/// Types of mapping validation errors
#[derive(Debug, Clone)]
pub enum MappingErrorType {
    ConflictingMappings,
    UnsupportedEvent,
    MissingFunction,
    InvalidSignature,
}