//! C# code transformation system for applying OOP patterns
//! 
//! This module provides functionality to transform generated C# code
//! to apply object-oriented programming patterns.

use crate::analyzer::{OopAnalysisResult, ClassInfo, MethodInfo, FieldInfo};
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet};
use std::io::Write;

/// Enhanced wasm2usharp converter with OOP transformations
pub struct EnhancedWasm2USharp {
    /// Configuration for the conversion
    config: ConversionConfig,
    /// Transformation pipeline
    transformation_pipeline: OopTransformationPipeline,
}

impl EnhancedWasm2USharp {
    /// Create a new enhanced converter
    pub fn new() -> Self {
        let mut pipeline = OopTransformationPipeline::new();
        
        // Add default transformations
        pipeline.add_transformation(Box::new(ClassStructureTransformation::new()));
        pipeline.add_transformation(Box::new(MethodOrganizationTransformation::new()));
        pipeline.add_transformation(Box::new(InheritanceTransformation::new()));
        pipeline.add_transformation(Box::new(UdonSharpAttributeTransformation::new()));
        
        Self {
            config: ConversionConfig::default(),
            transformation_pipeline: pipeline,
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ConversionConfig) -> Self {
        let mut converter = Self::new();
        converter.config = config;
        converter
    }
    
    /// Convert WASM to UdonSharp with OOP transformations
    pub fn convert_with_oop(&self, wasm_bytes: &[u8], analysis: &OopAnalysisResult) -> Result<ConversionResult> {
        // First, use the original wasm2usharp to generate base C# code
        let base_code = self.generate_base_code(wasm_bytes)?;
        
        // Apply OOP transformations based on analysis
        let transformed_code = self.apply_oop_transformations(&base_code, analysis)?;
        
        // Generate multiple files if needed
        let files = self.organize_into_files(&transformed_code, analysis)?;
        
        Ok(ConversionResult {
            main_class: files.get("main").cloned().unwrap_or_default(),
            additional_files: files.into_iter()
                .filter(|(name, _)| name != "main")
                .collect(),
            analysis_result: analysis.clone(),
        })
    }
    
    /// Generate base C# code using built-in WASM to C# conversion
    fn generate_base_code(&self, wasm_bytes: &[u8]) -> Result<String> {
        // For now, generate a basic C# class structure
        // This will be replaced with actual WASM parsing and conversion later
        let mut code = String::new();
        
        // Add using statements
        code.push_str("using UnityEngine;\n");
        code.push_str("using VRC.SDKBase;\n");
        code.push_str("using VRC.Udon;\n");
        code.push_str("using UdonSharp;\n\n");
        
        // Add namespace if configured
        if let Some(namespace) = &self.config.namespace {
            code.push_str(&format!("namespace {}\n{{\n", namespace));
        }
        
        // Generate basic class structure
        code.push_str(&format!("[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\n"));
        code.push_str(&format!("public class {} : UdonSharpBehaviour\n{{\n", self.config.class_name));
        
        // Add basic Start method
        code.push_str("    public void Start()\n    {\n        // Generated from WASM\n    }\n");
        
        code.push_str("}\n");
        
        // Close namespace if opened
        if self.config.namespace.is_some() {
            code.push_str("}\n");
        }
        
        Ok(code)
    }
    
    /// Apply OOP transformations to the base code
    fn apply_oop_transformations(&self, base_code: &str, analysis: &OopAnalysisResult) -> Result<String> {
        let mut context = TransformationContext {
            analysis: analysis.clone(),
            config: self.config.clone(),
            generated_classes: HashMap::new(),
            method_mappings: HashMap::new(),
        };
        
        self.transformation_pipeline.apply_transformations(base_code, &mut context)
    }
    
    /// Organize transformed code into multiple files
    fn organize_into_files(&self, code: &str, analysis: &OopAnalysisResult) -> Result<HashMap<String, String>> {
        let mut files = HashMap::new();
        
        // Main class always goes to "main"
        let main_class_code = self.extract_main_class(code)?;
        files.insert("main".to_string(), main_class_code);
        
        // Generate separate files for each detected class
        for class in &analysis.classes {
            if class.name != self.config.class_name {
                let class_code = self.generate_class_file(class, analysis)?;
                files.insert(format!("{}.cs", class.name), class_code);
            }
        }
        
        Ok(files)
    }
    
    /// Extract main class code
    fn extract_main_class(&self, code: &str) -> Result<String> {
        // For now, return the entire code as main class
        // This could be enhanced to extract only the main class
        Ok(code.to_string())
    }
    
    /// Generate code for a specific class
    fn generate_class_file(&self, class: &ClassInfo, analysis: &OopAnalysisResult) -> Result<String> {
        let mut code = String::new();
        
        // Add namespace if configured
        if let Some(namespace) = &self.config.namespace {
            code.push_str(&format!("namespace {}\n{{\n", namespace));
        }
        
        // Add using statements
        code.push_str("using UnityEngine;\n");
        code.push_str("using VRC.SDKBase;\n");
        code.push_str("using VRC.Udon;\n");
        code.push_str("using UdonSharp;\n\n");
        
        // Generate class declaration
        code.push_str(&format!("[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\n"));
        code.push_str(&format!("public class {} : UdonSharpBehaviour\n{{\n", class.name));
        
        // Generate fields
        for field in &class.fields {
            code.push_str(&self.generate_field_code(field)?);
        }
        
        // Generate methods
        let class_methods: Vec<_> = analysis.methods.iter()
            .filter(|m| m.class.as_ref() == Some(&class.name))
            .collect();
            
        for method in class_methods {
            code.push_str(&self.generate_method_code(method)?);
        }
        
        code.push_str("}\n");
        
        // Close namespace if opened
        if self.config.namespace.is_some() {
            code.push_str("}\n");
        }
        
        Ok(code)
    }
    
    /// Generate field code
    fn generate_field_code(&self, field: &FieldInfo) -> Result<String> {
        let visibility = if field.is_public { "public" } else { "private" };
        Ok(format!("    {} {} {};\n", visibility, field.field_type, field.name))
    }
    
    /// Generate method code
    fn generate_method_code(&self, method: &MethodInfo) -> Result<String> {
        let mut code = String::new();
        
        // Add UdonSharp attributes if needed
        if method.name == "Start" || method.name == "Update" {
            // These are UdonSharp lifecycle methods
        } else {
            code.push_str("    [UdonSharpMethod]\n");
        }
        
        // Generate method signature
        let return_type = method.return_type.as_deref().unwrap_or("void");
        let params = method.parameters.join(", ");
        
        code.push_str(&format!(
            "    public {} {}({})\n    {{\n        // Method implementation\n    }}\n\n",
            return_type, method.name, params
        ));
        
        Ok(code)
    }
}

/// Configuration for the conversion process
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    pub class_name: String,
    pub namespace: Option<String>,
    pub generate_separate_files: bool,
    pub udonsharp_attributes: bool,
    pub inheritance_support: bool,
}

impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            class_name: "GeneratedUdonSharp".to_string(),
            namespace: None,
            generate_separate_files: true,
            udonsharp_attributes: true,
            inheritance_support: true,
        }
    }
}

/// Result of the conversion process
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub main_class: String,
    pub additional_files: HashMap<String, String>,
    pub analysis_result: OopAnalysisResult,
}

/// Transformation pipeline for applying OOP patterns to C# code
pub struct OopTransformationPipeline {
    transformations: Vec<Box<dyn OopTransformation>>,
}

impl OopTransformationPipeline {
    /// Create a new transformation pipeline
    pub fn new() -> Self {
        Self {
            transformations: Vec::new(),
        }
    }
    
    /// Add a transformation to the pipeline
    pub fn add_transformation(&mut self, transformation: Box<dyn OopTransformation>) {
        self.transformations.push(transformation);
    }
    
    /// Apply all transformations to the C# code
    pub fn apply_transformations(&self, code: &str, context: &mut TransformationContext) -> Result<String> {
        let mut result = code.to_string();
        
        for transformation in &self.transformations {
            result = transformation.transform(&result, context)?;
        }
        
        Ok(result)
    }
}

/// Context for transformations
pub struct TransformationContext {
    pub analysis: OopAnalysisResult,
    pub config: ConversionConfig,
    pub generated_classes: HashMap<String, String>,
    pub method_mappings: HashMap<String, String>,
}

/// Trait for OOP transformations
pub trait OopTransformation {
    fn transform(&self, code: &str, context: &mut TransformationContext) -> Result<String>;
}

/// Transformation for organizing code into class structures
pub struct ClassStructureTransformation {
    // Configuration for class structure transformation
}

impl ClassStructureTransformation {
    pub fn new() -> Self {
        Self {}
    }
}

impl OopTransformation for ClassStructureTransformation {
    fn transform(&self, code: &str, context: &mut TransformationContext) -> Result<String> {
        let mut result = code.to_string();
        
        // Transform flat function-based code into class-based structure
        for class in &context.analysis.classes {
            result = self.wrap_functions_in_class(&result, class)?;
        }
        
        Ok(result)
    }
}

impl ClassStructureTransformation {
    fn wrap_functions_in_class(&self, code: &str, class: &ClassInfo) -> Result<String> {
        // This is a simplified implementation
        // In a real implementation, this would parse the C# code and restructure it
        let mut result = code.to_string();
        
        // Add class wrapper if not already present
        if !result.contains(&format!("class {}", class.name)) {
            // Insert class declaration
            let class_header = format!(
                "\n[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\npublic class {} : UdonSharpBehaviour\n{{\n",
                class.name
            );
            
            // Find a good insertion point (after using statements)
            if let Some(pos) = result.find("public class") {
                result.insert_str(pos, &class_header);
            }
        }
        
        Ok(result)
    }
}

/// Transformation for organizing methods within classes
pub struct MethodOrganizationTransformation;

impl MethodOrganizationTransformation {
    pub fn new() -> Self {
        Self
    }
}

impl OopTransformation for MethodOrganizationTransformation {
    fn transform(&self, code: &str, context: &mut TransformationContext) -> Result<String> {
        let mut result = code.to_string();
        
        // Group methods by class and organize them
        for method in &context.analysis.methods {
            if let Some(class_name) = &method.class {
                result = self.organize_method_in_class(&result, method, class_name)?;
            }
        }
        
        Ok(result)
    }
}

impl MethodOrganizationTransformation {
    fn organize_method_in_class(&self, code: &str, method: &MethodInfo, class_name: &str) -> Result<String> {
        // This would reorganize methods within their appropriate classes
        // For now, return the code unchanged
        Ok(code.to_string())
    }
}

/// Transformation for handling inheritance patterns
pub struct InheritanceTransformation;

impl InheritanceTransformation {
    pub fn new() -> Self {
        Self
    }
}

impl OopTransformation for InheritanceTransformation {
    fn transform(&self, code: &str, context: &mut TransformationContext) -> Result<String> {
        let mut result = code.to_string();
        
        // Apply inheritance transformations
        for inheritance in &context.analysis.inheritance_relationships {
            result = self.apply_inheritance(&result, inheritance)?;
        }
        
        Ok(result)
    }
}

impl InheritanceTransformation {
    fn apply_inheritance(&self, code: &str, inheritance: &crate::analyzer::InheritanceInfo) -> Result<String> {
        // Transform composition patterns into inheritance where appropriate
        // This is a simplified implementation
        Ok(code.to_string())
    }
}

/// Transformation for adding UdonSharp-specific attributes
pub struct UdonSharpAttributeTransformation;

impl UdonSharpAttributeTransformation {
    pub fn new() -> Self {
        Self
    }
}

impl OopTransformation for UdonSharpAttributeTransformation {
    fn transform(&self, code: &str, context: &mut TransformationContext) -> Result<String> {
        let mut result = code.to_string();
        
        if context.config.udonsharp_attributes {
            // Add UdonSharp attributes to classes and methods
            result = self.add_udonsharp_attributes(&result)?;
        }
        
        Ok(result)
    }
}

impl UdonSharpAttributeTransformation {
    fn add_udonsharp_attributes(&self, code: &str) -> Result<String> {
        let mut result = code.to_string();
        
        // Add [UdonBehaviourSyncMode] to classes that don't have it
        if !result.contains("[UdonBehaviourSyncMode") {
            result = result.replace(
                "public class",
                "[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]\npublic class"
            );
        }
        
        // Add using statements if not present
        if !result.contains("using UdonSharp;") {
            let using_statements = "using UnityEngine;\nusing VRC.SDKBase;\nusing VRC.Udon;\nusing UdonSharp;\n\n";
            if let Some(pos) = result.find("namespace") {
                result.insert_str(pos, using_statements);
            } else if let Some(pos) = result.find("public class") {
                result.insert_str(pos, using_statements);
            }
        }
        
        Ok(result)
    }
}
/// Advanced C Sharp code transformation system
pub struct CSharpTransformationSystem {
    /// Parser for C# code analysis
    parser: CSharpParser,
    /// Code generator for creating transformed output
    generator: CSharpCodeGenerator,
}

impl CSharpTransformationSystem {
    /// Create a new transformation system
    pub fn new() -> Self {
        Self {
            parser: CSharpParser::new(),
            generator: CSharpCodeGenerator::new(),
        }
    }
    
    /// Transform C# code with OOP patterns
    pub fn transform_code(&mut self, code: &str, analysis: &OopAnalysisResult) -> Result<TransformedCode> {
        // Parse the input C# code
        let parsed_code = self.parser.parse(code)?;
        
        // Apply transformations based on OOP analysis
        let transformed_ast = self.apply_oop_transformations(parsed_code, analysis)?;
        
        // Generate the final C# code
        let output_code = self.generator.generate(&transformed_ast)?;
        
        Ok(output_code)
    }
    
    /// Apply OOP transformations to the parsed AST
    fn apply_oop_transformations(&mut self, mut ast: CSharpAst, analysis: &OopAnalysisResult) -> Result<CSharpAst> {
        // Transform classes
        for class_info in &analysis.classes {
            ast = self.transform_class_structure(ast, class_info)?;
        }
        
        // Transform methods
        for method_info in &analysis.methods {
            ast = self.transform_method_structure(ast, method_info)?;
        }
        
        // Apply inheritance transformations
        for inheritance in &analysis.inheritance_relationships {
            ast = self.transform_inheritance(ast, inheritance)?;
        }
        
        Ok(ast)
    }
    
    /// Transform class structure in the AST
    fn transform_class_structure(&mut self, mut ast: CSharpAst, class_info: &ClassInfo) -> Result<CSharpAst> {
        // Find or create the class in the AST
        let class_node = ast.find_or_create_class(&class_info.name)?;
        
        // Add fields to the class
        for field in &class_info.fields {
            let field_node = CSharpFieldNode {
                name: field.name.clone(),
                field_type: field.field_type.clone(),
                visibility: if field.is_public { Visibility::Public } else { Visibility::Private },
                attributes: self.generate_field_attributes(field)?,
            };
            class_node.add_field(field_node);
        }
        
        // Add UdonSharp attributes to the class
        class_node.add_attribute(CSharpAttribute {
            name: "UdonBehaviourSyncMode".to_string(),
            parameters: vec!["BehaviourSyncMode.Manual".to_string()],
        });
        
        // Set base class to UdonSharpBehaviour
        class_node.set_base_class("UdonSharpBehaviour".to_string());
        
        Ok(ast)
    }
    
    /// Transform method structure in the AST
    fn transform_method_structure(&mut self, mut ast: CSharpAst, method_info: &MethodInfo) -> Result<CSharpAst> {
        if let Some(class_name) = &method_info.class {
            let class_node = ast.find_or_create_class(class_name)?;
            
            let method_node = CSharpMethodNode {
                name: method_info.name.clone(),
                return_type: method_info.return_type.clone().unwrap_or("void".to_string()),
                parameters: self.convert_parameters(&method_info.parameters)?,
                visibility: Visibility::Public,
                is_static: method_info.is_static,
                is_virtual: method_info.is_virtual,
                attributes: self.generate_method_attributes(method_info)?,
                body: self.generate_method_body(method_info)?,
            };
            
            class_node.add_method(method_node);
        }
        
        Ok(ast)
    }
    
    /// Transform inheritance relationships
    fn transform_inheritance(&mut self, mut ast: CSharpAst, inheritance: &crate::analyzer::InheritanceInfo) -> Result<CSharpAst> {
        let child_class = ast.find_or_create_class(&inheritance.child)?;
        
        match inheritance.relationship_type {
            crate::analyzer::InheritanceType::Implementation => {
                child_class.set_base_class(inheritance.parent.clone());
            }
            crate::analyzer::InheritanceType::Interface => {
                child_class.add_interface(inheritance.parent.clone());
            }
            crate::analyzer::InheritanceType::Composition => {
                // Add the parent as a field for composition
                let field_node = CSharpFieldNode {
                    name: format!("_{}", inheritance.parent.to_lowercase()),
                    field_type: inheritance.parent.clone(),
                    visibility: Visibility::Private,
                    attributes: Vec::new(),
                };
                child_class.add_field(field_node);
            }
        }
        
        Ok(ast)
    }
    
    /// Generate field attributes based on field info
    fn generate_field_attributes(&self, field: &FieldInfo) -> Result<Vec<CSharpAttribute>> {
        let mut attributes = Vec::new();
        
        if field.is_public {
            attributes.push(CSharpAttribute {
                name: "UdonSynced".to_string(),
                parameters: Vec::new(),
            });
        }
        
        Ok(attributes)
    }
    
    /// Generate method attributes based on method info
    fn generate_method_attributes(&self, method: &MethodInfo) -> Result<Vec<CSharpAttribute>> {
        let mut attributes = Vec::new();
        
        // Add UdonSharp lifecycle method attributes
        match method.name.as_str() {
            "Start" | "Update" | "FixedUpdate" | "LateUpdate" => {
                // These are Unity lifecycle methods, no special attributes needed
            }
            _ => {
                if !method.is_static {
                    attributes.push(CSharpAttribute {
                        name: "UdonSharpMethod".to_string(),
                        parameters: Vec::new(),
                    });
                }
            }
        }
        
        Ok(attributes)
    }
    
    /// Generate method body based on method info
    fn generate_method_body(&self, method: &MethodInfo) -> Result<String> {
        // This is a placeholder implementation
        // In a real implementation, this would generate appropriate method bodies
        // based on the WASM analysis and function mapping
        
        match method.name.as_str() {
            "Start" => Ok("        // Initialization code\n".to_string()),
            "Update" => Ok("        // Update logic\n".to_string()),
            _ => Ok("        // Method implementation\n".to_string()),
        }
    }
    
    /// Convert parameter types to C# method parameters
    fn convert_parameters(&self, parameters: &[String]) -> Result<Vec<CSharpParameter>> {
        parameters.iter().enumerate().map(|(i, param_type)| {
            Ok(CSharpParameter {
                name: format!("param{}", i),
                param_type: param_type.clone(),
                is_ref: false,
                is_out: false,
            })
        }).collect()
    }
}

/// Simple C# parser for basic code analysis
pub struct CSharpParser {
    // Parser state
}

impl CSharpParser {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Parse C# code into an AST
    pub fn parse(&mut self, code: &str) -> Result<CSharpAst> {
        let mut ast = CSharpAst::new();
        
        // This is a simplified parser implementation
        // In a real implementation, this would use a proper C# parser
        
        // Extract namespace
        if let Some(namespace) = self.extract_namespace(code) {
            ast.namespace = Some(namespace);
        }
        
        // Extract using statements
        ast.using_statements = self.extract_using_statements(code);
        
        // Extract classes (simplified)
        let classes = self.extract_classes(code)?;
        for class in classes {
            ast.classes.insert(class.name.clone(), class);
        }
        
        Ok(ast)
    }
    
    /// Extract namespace from code
    fn extract_namespace(&self, code: &str) -> Option<String> {
        for line in code.lines() {
            let line = line.trim();
            if line.starts_with("namespace ") {
                return Some(line[10..].trim_end_matches('{').trim().to_string());
            }
        }
        None
    }
    
    /// Extract using statements
    fn extract_using_statements(&self, code: &str) -> Vec<String> {
        let mut using_statements = Vec::new();
        
        for line in code.lines() {
            let line = line.trim();
            if line.starts_with("using ") && line.ends_with(';') {
                let using_stmt = line[6..line.len()-1].trim().to_string();
                using_statements.push(using_stmt);
            }
        }
        
        using_statements
    }
    
    /// Extract classes from code (simplified)
    fn extract_classes(&self, code: &str) -> Result<Vec<CSharpClassNode>> {
        let mut classes = Vec::new();
        
        // This is a very simplified class extraction
        // A real implementation would use proper parsing
        
        for line in code.lines() {
            let line = line.trim();
            if line.starts_with("public class ") {
                let class_name = line[13..].split_whitespace().next().unwrap_or("Unknown");
                classes.push(CSharpClassNode {
                    name: class_name.to_string(),
                    base_class: None,
                    interfaces: Vec::new(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                    attributes: Vec::new(),
                });
            }
        }
        
        Ok(classes)
    }
}

/// C# code generator
pub struct CSharpCodeGenerator {
    // Generator configuration
}

impl CSharpCodeGenerator {
    pub fn new() -> Self {
        Self {}
    }
    
    /// Generate C# code from AST
    pub fn generate(&mut self, ast: &CSharpAst) -> Result<TransformedCode> {
        let mut main_file = String::new();
        let mut additional_files = HashMap::new();
        
        // Generate using statements
        for using_stmt in &ast.using_statements {
            main_file.push_str(&format!("using {};\n", using_stmt));
        }
        main_file.push('\n');
        
        // Generate namespace if present
        let namespace_indent = if let Some(namespace) = &ast.namespace {
            main_file.push_str(&format!("namespace {}\n{{\n", namespace));
            "    "
        } else {
            ""
        };
        
        // Generate classes
        for (class_name, class_node) in &ast.classes {
            let class_code = self.generate_class(class_node, namespace_indent)?;
            
            if class_name == "Main" || ast.classes.len() == 1 {
                main_file.push_str(&class_code);
            } else {
                additional_files.insert(format!("{}.cs", class_name), class_code);
            }
        }
        
        // Close namespace if opened
        if ast.namespace.is_some() {
            main_file.push_str("}\n");
        }
        
        Ok(TransformedCode {
            main_file,
            additional_files,
        })
    }
    
    /// Generate code for a single class
    fn generate_class(&mut self, class_node: &CSharpClassNode, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Generate class attributes
        for attribute in &class_node.attributes {
            code.push_str(&format!("{}[{}]\n", indent, self.generate_attribute(attribute)));
        }
        
        // Generate class declaration
        code.push_str(&format!("{}public class {}", indent, class_node.name));
        
        if let Some(base_class) = &class_node.base_class {
            code.push_str(&format!(" : {}", base_class));
        }
        
        if !class_node.interfaces.is_empty() {
            let interfaces = class_node.interfaces.join(", ");
            if class_node.base_class.is_some() {
                code.push_str(&format!(", {}", interfaces));
            } else {
                code.push_str(&format!(" : {}", interfaces));
            }
        }
        
        code.push_str("\n");
        code.push_str(&format!("{}{{\n", indent));
        
        // Generate fields
        for field in &class_node.fields {
            code.push_str(&self.generate_field(field, &format!("{}    ", indent))?);
        }
        
        if !class_node.fields.is_empty() {
            code.push('\n');
        }
        
        // Generate methods
        for method in &class_node.methods {
            code.push_str(&self.generate_method(method, &format!("{}    ", indent))?);
        }
        
        code.push_str(&format!("{}}}\n\n", indent));
        
        Ok(code)
    }
    
    /// Generate attribute code
    fn generate_attribute(&self, attribute: &CSharpAttribute) -> String {
        if attribute.parameters.is_empty() {
            attribute.name.clone()
        } else {
            format!("{}({})", attribute.name, attribute.parameters.join(", "))
        }
    }
    
    /// Generate field code
    fn generate_field(&mut self, field: &CSharpFieldNode, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Generate field attributes
        for attribute in &field.attributes {
            code.push_str(&format!("{}[{}]\n", indent, self.generate_attribute(attribute)));
        }
        
        // Generate field declaration
        let visibility = match field.visibility {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Protected => "protected",
        };
        
        code.push_str(&format!(
            "{}{} {} {};\n",
            indent, visibility, field.field_type, field.name
        ));
        
        Ok(code)
    }
    
    /// Generate method code
    fn generate_method(&mut self, method: &CSharpMethodNode, indent: &str) -> Result<String> {
        let mut code = String::new();
        
        // Generate method attributes
        for attribute in &method.attributes {
            code.push_str(&format!("{}[{}]\n", indent, self.generate_attribute(attribute)));
        }
        
        // Generate method signature
        let visibility = match method.visibility {
            Visibility::Public => "public",
            Visibility::Private => "private",
            Visibility::Protected => "protected",
        };
        
        let modifiers = if method.is_static {
            "static "
        } else if method.is_virtual {
            "virtual "
        } else {
            ""
        };
        
        let parameters: Vec<String> = method.parameters.iter().map(|p| {
            format!("{} {}", p.param_type, p.name)
        }).collect();
        
        code.push_str(&format!(
            "{}{} {}{} {}({})\n{}{{ \n{}{}}}\n\n",
            indent,
            visibility,
            modifiers,
            method.return_type,
            method.name,
            parameters.join(", "),
            indent,
            method.body,
            indent
        ));
        
        Ok(code)
    }
}

/// Simplified C# AST representation
#[derive(Debug, Clone)]
pub struct CSharpAst {
    pub namespace: Option<String>,
    pub using_statements: Vec<String>,
    pub classes: HashMap<String, CSharpClassNode>,
}

impl CSharpAst {
    pub fn new() -> Self {
        Self {
            namespace: None,
            using_statements: Vec::new(),
            classes: HashMap::new(),
        }
    }
    
    pub fn find_or_create_class(&mut self, name: &str) -> Result<&mut CSharpClassNode> {
        if !self.classes.contains_key(name) {
            self.classes.insert(name.to_string(), CSharpClassNode {
                name: name.to_string(),
                base_class: None,
                interfaces: Vec::new(),
                fields: Vec::new(),
                methods: Vec::new(),
                attributes: Vec::new(),
            });
        }
        
        Ok(self.classes.get_mut(name).unwrap())
    }
}

/// C# class node in the AST
#[derive(Debug, Clone)]
pub struct CSharpClassNode {
    pub name: String,
    pub base_class: Option<String>,
    pub interfaces: Vec<String>,
    pub fields: Vec<CSharpFieldNode>,
    pub methods: Vec<CSharpMethodNode>,
    pub attributes: Vec<CSharpAttribute>,
}

impl CSharpClassNode {
    pub fn add_field(&mut self, field: CSharpFieldNode) {
        self.fields.push(field);
    }
    
    pub fn add_method(&mut self, method: CSharpMethodNode) {
        self.methods.push(method);
    }
    
    pub fn add_attribute(&mut self, attribute: CSharpAttribute) {
        self.attributes.push(attribute);
    }
    
    pub fn set_base_class(&mut self, base_class: String) {
        self.base_class = Some(base_class);
    }
    
    pub fn add_interface(&mut self, interface: String) {
        self.interfaces.push(interface);
    }
}

/// C# field node
#[derive(Debug, Clone)]
pub struct CSharpFieldNode {
    pub name: String,
    pub field_type: String,
    pub visibility: Visibility,
    pub attributes: Vec<CSharpAttribute>,
}

/// C# method node
#[derive(Debug, Clone)]
pub struct CSharpMethodNode {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<CSharpParameter>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_virtual: bool,
    pub attributes: Vec<CSharpAttribute>,
    pub body: String,
}

/// C# parameter
#[derive(Debug, Clone)]
pub struct CSharpParameter {
    pub name: String,
    pub param_type: String,
    pub is_ref: bool,
    pub is_out: bool,
}

/// C# attribute
#[derive(Debug, Clone)]
pub struct CSharpAttribute {
    pub name: String,
    pub parameters: Vec<String>,
}

/// Visibility modifier
#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

/// Result of code transformation
#[derive(Debug, Clone)]
pub struct TransformedCode {
    pub main_file: String,
    pub additional_files: HashMap<String, String>,
}