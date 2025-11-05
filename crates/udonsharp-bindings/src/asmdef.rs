//! Assembly Definition (.asmdef) file parsing and analysis
//! 
//! This module provides functionality to parse Unity .asmdef files and extract
//! API information for binding generation.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::{Result, Context};
use walkdir::WalkDir;

/// Represents a Unity Assembly Definition file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AsmdefFile {
    pub name: String,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(rename = "includePlatforms", default)]
    pub include_platforms: Vec<String>,
    #[serde(rename = "excludePlatforms", default)]
    pub exclude_platforms: Vec<String>,
    #[serde(rename = "allowUnsafeCode", default)]
    pub allow_unsafe_code: bool,
    #[serde(rename = "overrideReferences", default)]
    pub override_references: bool,
    #[serde(rename = "precompiledReferences", default)]
    pub precompiled_references: Vec<String>,
    #[serde(rename = "autoReferenced", default = "default_auto_referenced")]
    pub auto_referenced: bool,
    #[serde(rename = "defineConstraints", default)]
    pub define_constraints: Vec<String>,
    #[serde(rename = "versionDefines", default)]
    pub version_defines: Vec<VersionDefine>,
    #[serde(rename = "noEngineReferences", default)]
    pub no_engine_references: bool,
}

/// Version define constraint for conditional compilation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VersionDefine {
    pub name: String,
    pub expression: String,
    pub define: String,
}

/// Information about an assembly extracted from .asmdef and reflection
#[derive(Debug, Clone)]
pub struct AssemblyInfo {
    pub name: String,
    pub version: String,
    pub types: Vec<TypeInfo>,
    pub dependencies: Vec<String>,
    pub asmdef_path: PathBuf,
}

/// Information about a type within an assembly
#[derive(Debug, Clone)]
pub struct TypeInfo {
    pub name: String,
    pub namespace: Option<String>,
    pub full_name: String,
    pub is_public: bool,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_generic: bool,
    pub generic_constraints: Vec<String>,
    pub base_type: Option<String>,
    pub interfaces: Vec<String>,
    pub methods: Vec<MethodInfo>,
    pub properties: Vec<PropertyInfo>,
    pub fields: Vec<FieldInfo>,
    pub events: Vec<EventInfo>,
}

/// Information about a method within a type
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub declaring_type: String,
    pub is_public: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_abstract: bool,
    pub is_generic: bool,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: String,
    pub has_ref_parameters: bool,
    pub has_out_parameters: bool,
}

/// Information about a property within a type
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    pub name: String,
    pub declaring_type: String,
    pub property_type: String,
    pub is_public: bool,
    pub is_static: bool,
    pub can_read: bool,
    pub can_write: bool,
}

/// Information about a field within a type
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub declaring_type: String,
    pub field_type: String,
    pub is_public: bool,
    pub is_static: bool,
    pub is_readonly: bool,
    pub is_const: bool,
}

/// Information about an event within a type
#[derive(Debug, Clone)]
pub struct EventInfo {
    pub name: String,
    pub declaring_type: String,
    pub event_type: String,
    pub is_public: bool,
    pub is_static: bool,
}

/// Information about a method parameter
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub parameter_type: String,
    pub is_ref: bool,
    pub is_out: bool,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// File discovery system for scanning directories for .asmdef files
#[derive(Clone)]
pub struct AsmdefDiscovery {
    search_paths: Vec<PathBuf>,
    discovered_files: HashMap<String, PathBuf>,
}

impl AsmdefDiscovery {
    /// Create a new discovery system
    pub fn new() -> Self {
        Self {
            search_paths: Vec::new(),
            discovered_files: HashMap::new(),
        }
    }
    
    /// Add a directory to search for .asmdef files
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }
    
    /// Scan all search paths for .asmdef files
    pub fn discover_asmdef_files(&mut self) -> Result<Vec<PathBuf>> {
        self.discovered_files.clear();
        let mut all_files = Vec::new();
        
        for search_path in &self.search_paths {
            if !search_path.exists() {
                log::warn!("Search path does not exist: {}", search_path.display());
                continue;
            }
            
            let files = self.scan_directory(search_path)
                .with_context(|| format!("Failed to scan directory: {}", search_path.display()))?;
            
            for file_path in files {
                // Parse the .asmdef file to get its name
                match AsmdefFile::from_file(&file_path) {
                    Ok(asmdef) => {
                        if let Some(existing_path) = self.discovered_files.get(&asmdef.name) {
                            log::warn!(
                                "Duplicate assembly definition found: {} at {} and {}",
                                asmdef.name,
                                existing_path.display(),
                                file_path.display()
                            );
                        } else {
                            self.discovered_files.insert(asmdef.name.clone(), file_path.clone());
                            all_files.push(file_path);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to parse .asmdef file {}: {}", file_path.display(), e);
                    }
                }
            }
        }
        
        log::info!("Discovered {} .asmdef files", all_files.len());
        Ok(all_files)
    }
    
    /// Get the path for a specific assembly by name
    pub fn get_assembly_path(&self, name: &str) -> Option<&PathBuf> {
        self.discovered_files.get(name)
    }
    
    /// Get all discovered assemblies
    pub fn get_all_assemblies(&self) -> &HashMap<String, PathBuf> {
        &self.discovered_files
    }
    
    /// Scan a single directory for .asmdef files
    fn scan_directory(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut asmdef_files = Vec::new();
        
        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "asmdef") {
                asmdef_files.push(path.to_path_buf());
            }
        }
        
        Ok(asmdef_files)
    }
}

impl AsmdefFile {
    /// Parse an .asmdef file from a string
    pub fn parse(content: &str) -> Result<Self> {
        serde_json::from_str(content)
            .with_context(|| "Failed to parse .asmdef JSON content")
    }
    
    /// Load an .asmdef file from disk
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read .asmdef file: {}", path.display()))?;
        Self::parse(&content)
    }
    
    /// Check if this assembly is likely to be UdonSharp-compatible based on basic criteria
    pub fn is_likely_udonsharp_compatible(&self) -> bool {
        // Skip editor-only assemblies
        if self.name.contains("Editor") || self.name.contains("Test") {
            return false;
        }
        
        // Skip if explicitly excluded from runtime platforms
        if !self.include_platforms.is_empty() && !self.include_platforms.contains(&"Any".to_string()) {
            return false;
        }
        
        // Check for known incompatible references
        let incompatible_refs = [
            "System.Threading",
            "System.IO",
            "System.Net",
            "System.Reflection",
            "UnityEditor",
        ];
        
        for reference in &self.references {
            if incompatible_refs.iter().any(|&incompatible| reference.contains(incompatible)) {
                return false;
            }
        }
        
        true
    }
}

impl Default for AsmdefDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Default value for auto_referenced field
fn default_auto_referenced() -> bool {
    true
}