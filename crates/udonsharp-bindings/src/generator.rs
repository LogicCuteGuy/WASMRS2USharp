//! Binding generator for creating Rust bindings from .asmdef files
//! 
//! This module provides the core functionality for generating Rust bindings
//! from Unity Assembly Definition files.

use crate::asmdef::{AsmdefFile, AsmdefDiscovery, AssemblyInfo, TypeInfo, MethodInfo, PropertyInfo, FieldInfo};
use crate::analyzer::AssemblyAnalyzer;
use crate::compatibility::UdonSharpCompatibilityChecker;
use anyhow::{Result, Context};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

/// Universal binding generation pipeline
pub struct UniversalBindingPipeline {
    asmdef_directories: Vec<String>,
    output_dir: String,
    compatibility_checker: UdonSharpCompatibilityChecker,
    discovery: AsmdefDiscovery,
}

/// Configuration for binding generation
#[derive(Debug, Clone)]
pub struct BindingConfig {
    pub generate_docs: bool,
    pub include_internal_types: bool,
    pub module_prefix: Option<String>,
    pub custom_type_mappings: HashMap<String, String>,
}

/// Generated binding information
#[derive(Debug, Clone)]
pub struct GeneratedBinding {
    pub assembly_name: String,
    pub module_name: String,
    pub file_path: PathBuf,
    pub generated_types: Vec<String>,
}

impl UniversalBindingPipeline {
    /// Create a new binding pipeline
    pub fn new(output_dir: String) -> Self {
        Self {
            asmdef_directories: Vec::new(),
            output_dir,
            compatibility_checker: UdonSharpCompatibilityChecker::new(),
            discovery: AsmdefDiscovery::new(),
        }
    }
    
    /// Create a binding pipeline with custom compatibility checker
    pub fn with_compatibility_checker(output_dir: String, checker: UdonSharpCompatibilityChecker) -> Self {
        Self {
            asmdef_directories: Vec::new(),
            output_dir,
            compatibility_checker: checker,
            discovery: AsmdefDiscovery::new(),
        }
    }
    
    /// Add a directory to scan for .asmdef files
    pub fn add_asmdef_directory(&mut self, directory: String) {
        self.asmdef_directories.push(directory.clone());
        self.discovery.add_search_path(&directory);
    }
    
    /// Scan all directories and generate bindings
    pub fn scan_and_generate_all_bindings(&self) -> Result<Vec<GeneratedBinding>> {
        self.scan_and_generate_bindings_with_config(&BindingConfig::default())
    }
    
    /// Scan and generate bindings with custom configuration
    pub fn scan_and_generate_bindings_with_config(&self, config: &BindingConfig) -> Result<Vec<GeneratedBinding>> {
        log::info!("Starting binding generation process");
        log::info!("Scanning directories: {:?}", self.asmdef_directories);
        log::info!("Output directory: {}", self.output_dir);
        
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create output directory: {}", self.output_dir))?;
        
        // Discover all .asmdef files
        let mut discovery = self.discovery.clone();
        let asmdef_files = discovery.discover_asmdef_files()
            .context("Failed to discover .asmdef files")?;
        
        let mut generated_bindings = Vec::new();
        
        // Process each .asmdef file
        for asmdef_path in asmdef_files {
            match self.process_asmdef_file(&asmdef_path, config) {
                Ok(Some(binding)) => {
                    generated_bindings.push(binding);
                }
                Ok(None) => {
                    log::info!("Skipped incompatible assembly: {}", asmdef_path.display());
                }
                Err(e) => {
                    log::error!("Failed to process {}: {}", asmdef_path.display(), e);
                }
            }
        }
        
        // Generate master module file
        self.generate_master_module_file(&generated_bindings, config)
            .context("Failed to generate master module file")?;
        
        log::info!("Generated {} binding modules", generated_bindings.len());
        Ok(generated_bindings)
    }
    
    /// Process a single .asmdef file
    fn process_asmdef_file(&self, asmdef_path: &Path, config: &BindingConfig) -> Result<Option<GeneratedBinding>> {
        let asmdef = AsmdefFile::from_file(asmdef_path)
            .with_context(|| format!("Failed to parse .asmdef file: {}", asmdef_path.display()))?;
        
        log::info!("Processing assembly: {}", asmdef.name);
        
        // Check compatibility
        let compatibility_result = self.compatibility_checker.check_assembly_compatibility(&asmdef);
        if !compatibility_result.is_compatible {
            log::info!("Skipping incompatible assembly '{}': {:?}", asmdef.name, compatibility_result.reasons);
            return Ok(None);
        }
        
        if !compatibility_result.warnings.is_empty() {
            log::warn!("Assembly '{}' has warnings: {:?}", asmdef.name, compatibility_result.warnings);
        }
        
        // Analyze assembly to extract API information
        let analyzer = AssemblyAnalyzer::new(&asmdef.name)
            .with_context(|| format!("Failed to create analyzer for assembly: {}", asmdef.name))?;
        
        let assembly_info = analyzer.extract_api_information()
            .with_context(|| format!("Failed to extract API information from assembly: {}", asmdef.name))?;
        
        // Generate Rust bindings
        let binding_code = self.generate_binding_for_assembly(&assembly_info, config)
            .with_context(|| format!("Failed to generate bindings for assembly: {}", asmdef.name))?;
        
        // Write binding file
        let module_name = self.get_module_name(&asmdef.name);
        let output_file = format!("{}.rs", module_name);
        let output_path = Path::new(&self.output_dir).join(&output_file);
        
        fs::write(&output_path, binding_code)
            .with_context(|| format!("Failed to write binding file: {}", output_path.display()))?;
        
        let generated_types: Vec<String> = assembly_info.types.iter()
            .filter(|t| self.compatibility_checker.is_type_compatible(t))
            .map(|t| t.name.clone())
            .collect();
        
        log::info!("Generated bindings for assembly '{}' with {} types", asmdef.name, generated_types.len());
        
        Ok(Some(GeneratedBinding {
            assembly_name: asmdef.name,
            module_name,
            file_path: output_path,
            generated_types,
        }))
    }
    
    /// Generate Rust binding code for an assembly
    fn generate_binding_for_assembly(&self, assembly_info: &AssemblyInfo, config: &BindingConfig) -> Result<String> {
        let mut binding_code = String::new();
        
        // Generate file header
        binding_code.push_str(&self.generate_file_header(&assembly_info.name, config));
        
        // Generate use statements
        binding_code.push_str(&self.generate_use_statements());
        
        // Generate bindings for each compatible type
        for type_info in &assembly_info.types {
            if self.compatibility_checker.is_type_compatible(type_info) {
                let type_binding = self.generate_type_binding(type_info, config)
                    .with_context(|| format!("Failed to generate binding for type: {}", type_info.name))?;
                binding_code.push_str(&type_binding);
                binding_code.push_str("\n\n");
            }
        }
        
        Ok(binding_code)
    }
    
    /// Generate file header with documentation and metadata
    fn generate_file_header(&self, assembly_name: &str, config: &BindingConfig) -> String {
        let mut header = String::new();
        
        header.push_str(&format!("// Auto-generated bindings for {}\n", assembly_name));
        header.push_str("// Generated by udonsharp-bindings\n");
        header.push_str("// DO NOT EDIT - This file is automatically generated\n\n");
        
        if config.generate_docs {
            header.push_str(&format!("//! Rust bindings for the {} assembly\n", assembly_name));
            header.push_str("//!\n");
            header.push_str("//! This module provides UdonSharp-compatible Rust bindings for the\n");
            header.push_str(&format!("//! {} assembly. All types and methods in this module\n", assembly_name));
            header.push_str("//! have been verified to be compatible with UdonSharp constraints.\n\n");
        }
        
        header
    }
    
    /// Generate use statements
    fn generate_use_statements(&self) -> String {
        let mut uses = String::new();
        
        uses.push_str("use udonsharp_core::*;\n");
        uses.push_str("use std::collections::HashMap;\n");
        uses.push_str("\n");
        
        uses
    }
    
    /// Generate binding code for a single type
    fn generate_type_binding(&self, type_info: &TypeInfo, config: &BindingConfig) -> Result<String> {
        let mut binding = String::new();
        
        // Generate type documentation
        if config.generate_docs {
            binding.push_str(&format!("/// Rust binding for {}\n", type_info.full_name));
            if let Some(namespace) = &type_info.namespace {
                binding.push_str(&format!("/// Namespace: {}\n", namespace));
            }
        }
        
        // Generate struct definition
        binding.push_str(&format!("#[derive(Debug, Clone)]\n"));
        binding.push_str(&format!("pub struct {} {{\n", type_info.name));
        binding.push_str("    handle: ObjectHandle,\n");
        binding.push_str("}\n\n");
        
        // Generate implementation block
        binding.push_str(&format!("impl {} {{\n", type_info.name));
        
        // Generate constructor if appropriate
        if !type_info.is_abstract && !type_info.is_static {
            binding.push_str(&self.generate_constructor(type_info));
        }
        
        // Generate methods
        for method in &type_info.methods {
            if self.compatibility_checker.is_method_compatible(method) {
                let method_binding = self.generate_method_binding(method, config)?;
                binding.push_str(&method_binding);
            }
        }
        
        // Generate properties
        for property in &type_info.properties {
            if self.compatibility_checker.is_property_compatible(property) {
                let property_bindings = self.generate_property_binding(property, config)?;
                binding.push_str(&property_bindings);
            }
        }
        
        binding.push_str("}\n");
        
        Ok(binding)
    }
    
    /// Generate constructor method
    fn generate_constructor(&self, type_info: &TypeInfo) -> String {
        format!(
            "    /// Create a new instance of {}\n    pub fn new() -> Self {{\n        Self {{\n            handle: ObjectHandle::new(),\n        }}\n    }}\n\n",
            type_info.name
        )
    }
    
    /// Generate method binding
    fn generate_method_binding(&self, method: &MethodInfo, config: &BindingConfig) -> Result<String> {
        let mut binding = String::new();
        
        // Generate method documentation
        if config.generate_docs {
            binding.push_str(&format!("    /// {}\n", method.name));
        }
        
        // Generate udon_binding attribute
        binding.push_str(&format!(
            "    #[udon_binding(\"{}.{}\")]\n",
            method.declaring_type,
            method.name
        ));
        
        // Generate method signature
        let rust_method_name = self.convert_to_rust_naming(&method.name);
        let parameters = self.convert_parameters(&method.parameters)?;
        let return_type = self.convert_return_type(&method.return_type)?;
        
        let method_signature = if method.is_static {
            format!(
                "    pub fn {}({}) -> {} {{\n        todo!(\"Implement {} binding\")\n    }}\n\n",
                rust_method_name,
                parameters,
                return_type,
                method.name
            )
        } else {
            format!(
                "    pub fn {}(&self{}) -> {} {{\n        todo!(\"Implement {} binding\")\n    }}\n\n",
                rust_method_name,
                if parameters.is_empty() { String::new() } else { format!(", {}", parameters) },
                return_type,
                method.name
            )
        };
        
        binding.push_str(&method_signature);
        Ok(binding)
    }
    
    /// Generate property binding (getter and setter)
    fn generate_property_binding(&self, property: &PropertyInfo, config: &BindingConfig) -> Result<String> {
        let mut binding = String::new();
        
        let rust_property_name = self.convert_to_rust_naming(&property.name);
        let rust_type = self.convert_csharp_type_to_rust(&property.property_type);
        
        // Generate getter
        if property.can_read {
            if config.generate_docs {
                binding.push_str(&format!("    /// Get {}\n", property.name));
            }
            binding.push_str(&format!(
                "    #[udon_binding(\"{}.get_{}\")]\n",
                property.declaring_type,
                property.name
            ));
            
            let getter_signature = if property.is_static {
                format!(
                    "    pub fn {}() -> {} {{\n        todo!(\"Implement {} getter\")\n    }}\n\n",
                    rust_property_name,
                    rust_type,
                    property.name
                )
            } else {
                format!(
                    "    pub fn {}(&self) -> {} {{\n        todo!(\"Implement {} getter\")\n    }}\n\n",
                    rust_property_name,
                    rust_type,
                    property.name
                )
            };
            
            binding.push_str(&getter_signature);
        }
        
        // Generate setter
        if property.can_write {
            if config.generate_docs {
                binding.push_str(&format!("    /// Set {}\n", property.name));
            }
            binding.push_str(&format!(
                "    #[udon_binding(\"{}.set_{}\")]\n",
                property.declaring_type,
                property.name
            ));
            
            let setter_name = format!("set_{}", rust_property_name);
            let setter_signature = if property.is_static {
                format!(
                    "    pub fn {}(value: {}) {{\n        todo!(\"Implement {} setter\")\n    }}\n\n",
                    setter_name,
                    rust_type,
                    property.name
                )
            } else {
                format!(
                    "    pub fn {}(&mut self, value: {}) {{\n        todo!(\"Implement {} setter\")\n    }}\n\n",
                    setter_name,
                    rust_type,
                    property.name
                )
            };
            
            binding.push_str(&setter_signature);
        }
        
        Ok(binding)
    }
    
    /// Convert C# method name to Rust naming convention
    fn convert_to_rust_naming(&self, name: &str) -> String {
        // Convert PascalCase to snake_case
        let mut result = String::new();
        let mut chars = name.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch.is_uppercase() && !result.is_empty() {
                // Add underscore before uppercase letters (except at start)
                if let Some(&next_ch) = chars.peek() {
                    if next_ch.is_lowercase() {
                        result.push('_');
                    }
                }
            }
            result.push(ch.to_lowercase().next().unwrap_or(ch));
        }
        
        result
    }
    
    /// Convert method parameters to Rust syntax
    fn convert_parameters(&self, parameters: &[crate::asmdef::ParameterInfo]) -> Result<String> {
        let rust_params: Vec<String> = parameters.iter()
            .map(|param| {
                let rust_type = self.convert_csharp_type_to_rust(&param.parameter_type);
                let param_name = self.convert_to_rust_naming(&param.name);
                format!("{}: {}", param_name, rust_type)
            })
            .collect();
        
        Ok(rust_params.join(", "))
    }
    
    /// Convert return type to Rust syntax
    fn convert_return_type(&self, return_type: &str) -> Result<String> {
        if return_type == "System.Void" || return_type.is_empty() {
            Ok("()".to_string())
        } else {
            Ok(self.convert_csharp_type_to_rust(return_type))
        }
    }
    
    /// Convert C# type to Rust type
    fn convert_csharp_type_to_rust(&self, csharp_type: &str) -> String {
        match csharp_type {
            "System.Boolean" => "bool".to_string(),
            "System.Byte" => "u8".to_string(),
            "System.SByte" => "i8".to_string(),
            "System.Int16" => "i16".to_string(),
            "System.UInt16" => "u16".to_string(),
            "System.Int32" => "i32".to_string(),
            "System.UInt32" => "u32".to_string(),
            "System.Int64" => "i64".to_string(),
            "System.UInt64" => "u64".to_string(),
            "System.Single" => "f32".to_string(),
            "System.Double" => "f64".to_string(),
            "System.Char" => "char".to_string(),
            "System.String" => "String".to_string(),
            "System.Void" => "()".to_string(),
            _ => {
                // For complex types, use the simple name
                if let Some(last_dot) = csharp_type.rfind('.') {
                    csharp_type[last_dot + 1..].to_string()
                } else {
                    csharp_type.to_string()
                }
            }
        }
    }
    
    /// Get module name from assembly name
    fn get_module_name(&self, assembly_name: &str) -> String {
        assembly_name
            .to_lowercase()
            .replace(".", "_")
            .replace("-", "_")
            .replace(" ", "_")
    }
    
    /// Generate master module file that includes all generated bindings
    fn generate_master_module_file(&self, bindings: &[GeneratedBinding], config: &BindingConfig) -> Result<()> {
        let mut mod_content = String::new();
        
        // Generate header
        mod_content.push_str("// Auto-generated module file for all UdonSharp-compatible bindings\n");
        mod_content.push_str("// Generated by udonsharp-bindings\n");
        mod_content.push_str("// DO NOT EDIT - This file is automatically generated\n\n");
        
        if config.generate_docs {
            mod_content.push_str("//! UdonSharp-compatible API bindings\n");
            mod_content.push_str("//!\n");
            mod_content.push_str("//! This module contains auto-generated Rust bindings for UdonSharp-compatible\n");
            mod_content.push_str("//! APIs from Unity, VRChat, and C# system libraries.\n\n");
        }
        
        // Add module declarations
        for binding in bindings {
            mod_content.push_str(&format!("pub mod {};\n", binding.module_name));
        }
        
        // Add re-exports for convenience
        mod_content.push_str("\n// Re-exports for convenience\n");
        for binding in bindings {
            mod_content.push_str(&format!("pub use {}::*;\n", binding.module_name));
        }
        
        let mod_path = Path::new(&self.output_dir).join("mod.rs");
        fs::write(mod_path, mod_content)
            .context("Failed to write master module file")?;
        
        Ok(())
    }
}

impl Default for BindingConfig {
    fn default() -> Self {
        Self {
            generate_docs: true,
            include_internal_types: false,
            module_prefix: None,
            custom_type_mappings: HashMap::new(),
        }
    }
}

/// Placeholder for object handle (to be implemented with actual UdonSharp integration)
#[derive(Debug, Clone)]
pub struct ObjectHandle {
    // Placeholder implementation
}

impl ObjectHandle {
    pub fn new() -> Self {
        Self {}
    }
}