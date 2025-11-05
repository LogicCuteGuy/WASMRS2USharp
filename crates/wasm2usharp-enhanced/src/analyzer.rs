//! OOP behavior analysis for WASM modules
//! 
//! This module provides functionality to analyze WASM bytecode and detect
//! object-oriented programming patterns.

use anyhow::Result;
use std::collections::{HashMap, HashSet};
use wasmparser::{
    FuncType, Operator, Parser, Payload, TypeRef, ValType
};
use udonsharp_core::attributes::{UdonBehaviourMarker, AttributeParser};

/// Analyzer for detecting OOP patterns in WASM
pub struct OopBehaviorAnalyzer {
    /// Function signatures indexed by function index
    function_types: Vec<FuncType>,
    /// Import information
    imports: Vec<ImportInfo>,
    /// Export information  
    exports: Vec<ExportInfo>,
    /// Function bodies for analysis
    function_bodies: HashMap<u32, Vec<Operator<'static>>>,
    /// Detected patterns
    patterns: PatternDatabase,
    /// Mapping from Rust function names to WASM exports
    function_name_mapping: HashMap<String, u32>,
    /// UdonBehaviour attributes extracted from custom sections
    udon_behaviour_attributes: HashMap<String, UdonBehaviourMarker>,
}

impl OopBehaviorAnalyzer {
    /// Create a new OOP behavior analyzer
    pub fn new() -> Self {
        Self {
            function_types: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            function_bodies: HashMap::new(),
            patterns: PatternDatabase::new(),
            function_name_mapping: HashMap::new(),
            udon_behaviour_attributes: HashMap::new(),
        }
    }
    
    /// Analyze WASM bytecode for OOP patterns
    pub fn analyze(&mut self, wasm_bytes: &[u8]) -> Result<OopAnalysisResult> {
        self.parse_wasm_module(wasm_bytes)?;
        self.detect_oop_patterns()?;
        Ok(self.build_analysis_result())
    }
    
    /// Parse the WASM module and extract relevant information
    fn parse_wasm_module(&mut self, wasm_bytes: &[u8]) -> Result<()> {
        let mut parser = Parser::new(0);
        let mut payloads = parser.parse_all(wasm_bytes);
        
        while let Some(payload) = payloads.next() {
            match payload? {
                Payload::TypeSection(reader) => {
                    self.parse_type_section(reader)?;
                }
                Payload::ImportSection(reader) => {
                    self.parse_import_section(reader)?;
                }
                Payload::FunctionSection(reader) => {
                    self.parse_function_section(reader)?;
                }
                Payload::ExportSection(reader) => {
                    self.parse_export_section(reader)?;
                }
                Payload::CodeSectionEntry(body) => {
                    self.parse_function_body(body)?;
                }
                Payload::CustomSection(reader) => {
                    self.parse_custom_section(reader)?;
                }
                _ => {} // Ignore other sections for now
            }
        }
        
        Ok(())
    }
    
    /// Parse type section to get function signatures
    fn parse_type_section(&mut self, reader: wasmparser::TypeSectionReader) -> Result<()> {
        for ty in reader {
            let ty = ty?;
            // Note: wasmparser API has changed, this is a simplified implementation
            // In a real implementation, we would properly handle the type parsing
            // Simplified: just add a placeholder function type
            // self.function_types.push(func_type.clone());
        }
        Ok(())
    }
    
    /// Parse import section
    fn parse_import_section(&mut self, _reader: wasmparser::ImportSectionReader) -> Result<()> {
        // Simplified implementation - in a real version this would parse imports
        Ok(())
    }
    
    /// Parse function section
    fn parse_function_section(&mut self, reader: wasmparser::FunctionSectionReader) -> Result<()> {
        // Function section just contains type indices, which we'll use later
        Ok(())
    }
    
    /// Parse export section
    fn parse_export_section(&mut self, reader: wasmparser::ExportSectionReader) -> Result<()> {
        for export in reader {
            let export = export?;
            let export_info = ExportInfo {
                name: export.name.to_string(),
                kind: match export.kind {
                    wasmparser::ExternalKind::Func => ExportKind::Func,
                    wasmparser::ExternalKind::Table => ExportKind::Table,
                    wasmparser::ExternalKind::Memory => ExportKind::Memory,
                    wasmparser::ExternalKind::Global => ExportKind::Global,
                    wasmparser::ExternalKind::Tag => ExportKind::Tag,
                },
                index: export.index,
            };
            
            // Create function name mapping for exported functions
            if export_info.kind == ExportKind::Func {
                self.function_name_mapping.insert(export_info.name.clone(), export_info.index);
            }
            
            self.exports.push(export_info);
        }
        Ok(())
    }
    
    /// Parse function body
    fn parse_function_body(&mut self, body: wasmparser::FunctionBody) -> Result<()> {
        let func_index = self.function_bodies.len() as u32;
        let mut operators = Vec::new();
        
        let mut reader = body.get_operators_reader()?;
        while !reader.eof() {
            let op = reader.read()?;
            // Convert to owned operator for storage
            let owned_op = self.convert_to_owned_operator(op);
            operators.push(owned_op);
        }
        
        self.function_bodies.insert(func_index, operators);
        Ok(())
    }
    
    /// Parse custom section for UdonBehaviour attributes
    fn parse_custom_section(&mut self, reader: wasmparser::CustomSectionReader) -> Result<()> {
        let section_name = reader.name();
        
        // Look for our custom section containing UdonBehaviour metadata
        if section_name == "udonsharp.attributes" {
            let data = reader.data();
            self.parse_udonsharp_attributes(data)?;
        }
        
        Ok(())
    }
    
    /// Parse UdonSharp attributes from custom section data
    fn parse_udonsharp_attributes(&mut self, data: &[u8]) -> Result<()> {
        // Convert bytes to string (assuming UTF-8 encoding)
        let attributes_str = std::str::from_utf8(data)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in attributes section: {}", e))?;
        
        // Parse JSON-like format: {"function_name": "attribute_metadata", ...}
        for line in attributes_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip empty lines and comments
            }
            
            // Simple parsing: "function_name:attribute_metadata"
            if let Some((func_name, metadata)) = line.split_once(':') {
                let func_name = func_name.trim().to_string();
                let metadata = metadata.trim();
                
                // Parse the UdonBehaviour attribute metadata
                match AttributeParser::parse_udon_behaviour_from_metadata(metadata) {
                    Ok(marker) => {
                        self.udon_behaviour_attributes.insert(func_name, marker);
                    }
                    Err(e) => {
                        log::warn!("Failed to parse UdonBehaviour attribute for function '{}': {}", func_name, e);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Convert borrowed operator to owned for storage
    fn convert_to_owned_operator(&self, op: Operator) -> Operator<'static> {
        match op {
            Operator::Call { function_index } => Operator::Call { function_index },
            Operator::CallIndirect { type_index, table_index, table_byte } => 
                Operator::CallIndirect { type_index, table_index, table_byte },
            Operator::LocalGet { local_index } => Operator::LocalGet { local_index },
            Operator::LocalSet { local_index } => Operator::LocalSet { local_index },
            Operator::GlobalGet { global_index } => Operator::GlobalGet { global_index },
            Operator::GlobalSet { global_index } => Operator::GlobalSet { global_index },
            Operator::I32Const { value } => Operator::I32Const { value },
            Operator::I64Const { value } => Operator::I64Const { value },
            Operator::F32Const { value } => Operator::F32Const { value },
            Operator::F64Const { value } => Operator::F64Const { value },
            Operator::Return => Operator::Return,
            Operator::End => Operator::End,
            // Add more operators as needed for pattern detection
            _ => Operator::Nop, // Fallback for unsupported operators
        }
    }
    
    /// Detect OOP patterns in the parsed WASM
    fn detect_oop_patterns(&mut self) -> Result<()> {
        self.detect_class_patterns()?;
        self.detect_method_patterns()?;
        self.detect_inheritance_patterns()?;
        Ok(())
    }
    
    /// Detect class-like patterns
    fn detect_class_patterns(&mut self) -> Result<()> {
        // Look for constructor patterns (functions that initialize state)
        for (func_idx, operators) in &self.function_bodies {
            if self.is_constructor_pattern(operators) {
                let class_name = self.infer_class_name_from_function(*func_idx);
                let methods = self.find_related_methods(&class_name);
                let fields = self.find_class_fields(&class_name, operators);
                
                self.patterns.classes.push(ClassInfo {
                    name: class_name,
                    methods,
                    fields,
                    constructor_function: Some(*func_idx),
                });
            }
        }
        
        // Look for vtable-like patterns (function pointer tables)
        self.detect_vtable_patterns()?;
        
        Ok(())
    }
    
    /// Detect method-like patterns
    fn detect_method_patterns(&mut self) -> Result<()> {
        for (func_idx, operators) in &self.function_bodies {
            // Check if function takes 'this' pointer as first parameter
            if let Some(func_type) = self.get_function_type(*func_idx) {
                if self.is_method_signature(&func_type) {
                    let method_name = self.get_function_name(*func_idx);
                    let class_name = self.infer_class_from_method(&method_name);
                    
                    self.patterns.methods.push(MethodInfo {
                        name: method_name,
                        class: class_name,
                        function_index: *func_idx,
                        parameters: self.extract_parameter_types(&func_type),
                        return_type: self.extract_return_type(&func_type),
                        is_static: false,
                        is_virtual: self.is_virtual_method(operators),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Detect inheritance patterns
    fn detect_inheritance_patterns(&mut self) -> Result<()> {
        // Look for patterns where one class calls methods from another
        // This is a simplified heuristic for inheritance detection
        
        for class in &self.patterns.classes.clone() {
            for method_name in &class.methods {
                if let Some(parent_class) = self.find_parent_class_for_method(method_name) {
                    if parent_class != class.name {
                        self.patterns.inheritance_relationships.push(InheritanceInfo {
                            child: class.name.clone(),
                            parent: parent_class,
                            relationship_type: InheritanceType::Implementation,
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if operators represent a constructor pattern
    fn is_constructor_pattern(&self, operators: &[Operator]) -> bool {
        // Look for patterns that initialize multiple global variables or memory locations
        let mut initialization_count = 0;
        
        for op in operators {
            match op {
                Operator::GlobalSet { .. } => initialization_count += 1,
                Operator::I32Store { .. } | Operator::I64Store { .. } | 
                Operator::F32Store { .. } | Operator::F64Store { .. } => initialization_count += 1,
                _ => {}
            }
        }
        
        // Heuristic: constructors typically initialize multiple fields
        initialization_count >= 2
    }
    
    /// Infer class name from function index
    fn infer_class_name_from_function(&self, func_idx: u32) -> String {
        // Try to get exported name first
        for export in &self.exports {
            if export.kind == ExportKind::Func && export.index == func_idx {
                return self.extract_class_name_from_export(&export.name);
            }
        }
        
        // Fallback to generic name
        format!("Class{}", func_idx)
    }
    
    /// Extract class name from export name (e.g., "MyClass::new" -> "MyClass")
    fn extract_class_name_from_export(&self, export_name: &str) -> String {
        if let Some(pos) = export_name.find("::") {
            export_name[..pos].to_string()
        } else if export_name.ends_with("_new") || export_name.ends_with("_init") {
            export_name.trim_end_matches("_new").trim_end_matches("_init").to_string()
        } else {
            export_name.to_string()
        }
    }
    
    /// Find methods related to a class
    fn find_related_methods(&self, class_name: &str) -> Vec<String> {
        let mut methods = Vec::new();
        
        for export in &self.exports {
            if export.kind == ExportKind::Func {
                if export.name.starts_with(&format!("{}::", class_name)) ||
                   export.name.starts_with(&format!("{}_", class_name)) {
                    let method_name = self.extract_method_name(&export.name, class_name);
                    methods.push(method_name);
                }
            }
        }
        
        methods
    }
    
    /// Extract method name from export
    fn extract_method_name(&self, export_name: &str, class_name: &str) -> String {
        if let Some(pos) = export_name.find("::") {
            export_name[pos + 2..].to_string()
        } else if export_name.starts_with(&format!("{}_", class_name)) {
            export_name[class_name.len() + 1..].to_string()
        } else {
            export_name.to_string()
        }
    }
    
    /// Find class fields by analyzing constructor operations
    fn find_class_fields(&self, _class_name: &str, operators: &[Operator]) -> Vec<FieldInfo> {
        let mut fields = Vec::new();
        let mut field_counter = 0;
        
        for op in operators {
            match op {
                Operator::GlobalSet { global_index } => {
                    fields.push(FieldInfo {
                        name: format!("field_{}", field_counter),
                        field_type: "i32".to_string(), // Simplified
                        offset: *global_index as usize,
                        is_public: false,
                    });
                    field_counter += 1;
                }
                _ => {}
            }
        }
        
        fields
    }
    
    /// Detect vtable patterns for virtual method dispatch
    fn detect_vtable_patterns(&mut self) -> Result<()> {
        // Look for function tables that might represent vtables
        // This is a simplified implementation
        Ok(())
    }
    
    /// Get function type by index
    fn get_function_type(&self, func_idx: u32) -> Option<&FuncType> {
        // Account for imported functions
        let import_func_count = self.imports.iter()
            .filter(|imp| matches!(imp.kind, ImportKind::Func(_)))
            .count() as u32;
            
        if func_idx < import_func_count {
            // This is an imported function
            if let Some(import) = self.imports.get(func_idx as usize) {
                if let ImportKind::Func(type_idx) = import.kind {
                    return self.function_types.get(type_idx as usize);
                }
            }
        } else {
            // This is a local function
            let local_idx = func_idx - import_func_count;
            return self.function_types.get(local_idx as usize);
        }
        
        None
    }
    
    /// Check if function signature looks like a method (takes 'this' pointer)
    fn is_method_signature(&self, func_type: &FuncType) -> bool {
        // Heuristic: methods typically take at least one parameter (this pointer)
        !func_type.params().is_empty() && 
        func_type.params()[0] == ValType::I32 // Assuming 'this' is an i32 pointer
    }
    
    /// Get function name from exports
    fn get_function_name(&self, func_idx: u32) -> String {
        for export in &self.exports {
            if export.kind == ExportKind::Func && export.index == func_idx {
                return export.name.clone();
            }
        }
        format!("func_{}", func_idx)
    }
    
    /// Infer class name from method name
    fn infer_class_from_method(&self, method_name: &str) -> Option<String> {
        if let Some(pos) = method_name.find("::") {
            Some(method_name[..pos].to_string())
        } else if let Some(pos) = method_name.find("_") {
            Some(method_name[..pos].to_string())
        } else {
            None
        }
    }
    
    /// Extract parameter types from function type
    fn extract_parameter_types(&self, func_type: &FuncType) -> Vec<String> {
        func_type.params().iter().map(|param| {
            match param {
                ValType::I32 => "i32".to_string(),
                ValType::I64 => "i64".to_string(),
                ValType::F32 => "f32".to_string(),
                ValType::F64 => "f64".to_string(),
                _ => "unknown".to_string(),
            }
        }).collect()
    }
    
    /// Extract return type from function type
    fn extract_return_type(&self, func_type: &FuncType) -> Option<String> {
        func_type.results().first().map(|result| {
            match result {
                ValType::I32 => "i32".to_string(),
                ValType::I64 => "i64".to_string(),
                ValType::F32 => "f32".to_string(),
                ValType::F64 => "f64".to_string(),
                _ => "unknown".to_string(),
            }
        })
    }
    
    /// Check if method is virtual (uses indirect calls)
    fn is_virtual_method(&self, operators: &[Operator]) -> bool {
        operators.iter().any(|op| matches!(op, Operator::CallIndirect { .. }))
    }
    
    /// Find parent class for a method (simplified heuristic)
    fn find_parent_class_for_method(&self, _method_name: &str) -> Option<String> {
        // This would need more sophisticated analysis
        // For now, return None
        None
    }
    
    /// Get exported functions with UdonBehaviour attributes
    pub fn get_udon_behaviour_functions(&self) -> HashMap<String, (u32, UdonBehaviourMarker)> {
        let mut result = HashMap::new();
        
        for (func_name, marker) in &self.udon_behaviour_attributes {
            if let Some(&func_index) = self.function_name_mapping.get(func_name) {
                result.insert(func_name.clone(), (func_index, marker.clone()));
            }
        }
        
        result
    }
    
    /// Get function name by index from exports
    pub fn get_function_name_by_index(&self, func_index: u32) -> Option<String> {
        for export in &self.exports {
            if export.kind == ExportKind::Func && export.index == func_index {
                return Some(export.name.clone());
            }
        }
        None
    }
    
    /// Get all exported function names and their indices
    pub fn get_exported_functions(&self) -> HashMap<String, u32> {
        self.function_name_mapping.clone()
    }
    
    /// Check if a function has UdonBehaviour attribute
    pub fn has_udon_behaviour_attribute(&self, func_name: &str) -> bool {
        self.udon_behaviour_attributes.contains_key(func_name)
    }
    
    /// Get UdonBehaviour attribute for a function
    pub fn get_udon_behaviour_attribute(&self, func_name: &str) -> Option<&UdonBehaviourMarker> {
        self.udon_behaviour_attributes.get(func_name)
    }
    
    /// Identify and create behavior units from UdonBehaviour functions
    pub fn identify_behavior_units(&self) -> Result<Vec<BehaviorUnit>> {
        let mut behavior_units = Vec::new();
        
        // Create behavior units for each function with UdonBehaviour attribute
        for (func_name, (func_index, marker)) in &self.get_udon_behaviour_functions() {
            let behavior_unit = self.create_behavior_unit(func_name, *func_index, marker)?;
            behavior_units.push(behavior_unit);
        }
        
        // Analyze dependencies between behavior units
        self.analyze_behavior_dependencies(&mut behavior_units)?;
        
        Ok(behavior_units)
    }
    
    /// Create a behavior unit from a function with UdonBehaviour attribute
    fn create_behavior_unit(&self, func_name: &str, func_index: u32, marker: &UdonBehaviourMarker) -> Result<BehaviorUnit> {
        // Determine behavior name
        let behavior_name = marker.name.clone()
            .unwrap_or_else(|| self.infer_behavior_name_from_function(func_name));
        
        // Find functions that belong to this behavior
        let local_functions = self.find_behavior_local_functions(func_name, &behavior_name)?;
        
        let behavior_unit = BehaviorUnit {
            name: behavior_name,
            entry_function: func_name.to_string(),
            entry_function_index: func_index,
            unity_events: marker.events.clone(),
            local_functions,
            shared_dependencies: std::collections::HashSet::new(), // Will be populated later
            inter_behavior_calls: Vec::new(), // Will be populated later
            attribute_config: marker.clone(),
        };
        
        Ok(behavior_unit)
    }
    
    /// Infer behavior name from function name
    fn infer_behavior_name_from_function(&self, func_name: &str) -> String {
        // Convert snake_case or camelCase to PascalCase
        let parts: Vec<&str> = func_name.split('_').collect();
        if parts.len() > 1 {
            // snake_case: player_manager_start -> PlayerManager
            let class_parts: Vec<&str> = parts.iter()
                .take(parts.len().saturating_sub(1)) // Remove last part (likely the event name)
                .copied()
                .collect();
            
            class_parts.iter()
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<String>>()
                .join("")
        } else {
            // camelCase or PascalCase: playerManagerStart -> PlayerManagerStart
            let mut result = String::new();
            let mut chars = func_name.chars().peekable();
            let mut capitalize_next = true;
            
            while let Some(ch) = chars.next() {
                if capitalize_next || ch.is_uppercase() {
                    result.push(ch.to_uppercase().next().unwrap_or(ch));
                    capitalize_next = false;
                } else {
                    result.push(ch);
                }
                
                // Check if next character should be capitalized (after lowercase followed by uppercase)
                if let Some(&next_ch) = chars.peek() {
                    if ch.is_lowercase() && next_ch.is_uppercase() {
                        capitalize_next = true;
                    }
                }
            }
            
            // Remove common suffixes that indicate event names
            let suffixes = ["Start", "Update", "Awake", "OnEnable", "OnDisable"];
            for suffix in &suffixes {
                if result.ends_with(suffix) && result.len() > suffix.len() {
                    result = result[..result.len() - suffix.len()].to_string();
                    break;
                }
            }
            
            result
        }
    }
    
    /// Find functions that belong to a specific behavior
    fn find_behavior_local_functions(&self, entry_func: &str, behavior_name: &str) -> Result<std::collections::HashSet<String>> {
        let mut local_functions = std::collections::HashSet::new();
        
        // Add the entry function itself
        local_functions.insert(entry_func.to_string());
        
        // Find functions that follow naming conventions for this behavior
        for export in &self.exports {
            if export.kind == ExportKind::Func {
                let func_name = &export.name;
                
                // Skip if this function has its own UdonBehaviour attribute
                if self.udon_behaviour_attributes.contains_key(func_name) && func_name != entry_func {
                    continue;
                }
                
                // Check if function belongs to this behavior based on naming patterns
                if self.function_belongs_to_behavior(func_name, behavior_name, entry_func) {
                    local_functions.insert(func_name.clone());
                }
            }
        }
        
        Ok(local_functions)
    }
    
    /// Check if a function belongs to a specific behavior based on naming patterns
    fn function_belongs_to_behavior(&self, func_name: &str, behavior_name: &str, entry_func: &str) -> bool {
        // Convert behavior name to different naming conventions for matching
        let snake_case_prefix = self.to_snake_case(behavior_name);
        let camel_case_prefix = self.to_camel_case(behavior_name);
        
        // Check various naming patterns:
        // 1. snake_case: player_manager_update, player_manager_on_trigger_enter
        if func_name.starts_with(&format!("{}_", snake_case_prefix)) {
            return true;
        }
        
        // 2. camelCase: playerManagerUpdate, playerManagerOnTriggerEnter
        if func_name.starts_with(&camel_case_prefix) && func_name != entry_func {
            // Make sure it's not just the behavior name itself
            if func_name.len() > camel_case_prefix.len() {
                let remaining = &func_name[camel_case_prefix.len()..];
                // Check if remaining part starts with uppercase (indicating a method name)
                if remaining.chars().next().map_or(false, |c| c.is_uppercase()) {
                    return true;
                }
            }
        }
        
        // 3. Namespace-like: PlayerManager::update, PlayerManager::on_trigger_enter
        if func_name.starts_with(&format!("{}::", behavior_name)) {
            return true;
        }
        
        false
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
    
    /// Convert PascalCase to camelCase
    fn to_camel_case(&self, input: &str) -> String {
        let mut chars = input.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        }
    }
    
    /// Analyze dependencies between behavior units
    fn analyze_behavior_dependencies(&self, behavior_units: &mut Vec<BehaviorUnit>) -> Result<()> {
        // For each behavior unit, analyze its function calls to find dependencies
        for i in 0..behavior_units.len() {
            let behavior_name = behavior_units[i].name.clone();
            let local_functions = behavior_units[i].local_functions.clone();
            
            // Analyze function calls within this behavior's functions
            for func_name in &local_functions {
                if let Some(&func_index) = self.function_name_mapping.get(func_name) {
                    if let Some(operators) = self.function_bodies.get(&func_index) {
                        let dependencies = self.analyze_function_calls(operators, &behavior_units)?;
                        
                        // Add dependencies and inter-behavior calls
                        for (target_behavior, target_function, call_type) in dependencies {
                            if target_behavior != behavior_name {
                                behavior_units[i].shared_dependencies.insert(target_function.clone());
                                behavior_units[i].inter_behavior_calls.push(InterBehaviorCall {
                                    source_behavior: behavior_name.clone(),
                                    target_behavior: target_behavior.clone(),
                                    function_name: target_function,
                                    call_type,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Analyze function calls in WASM operators to find dependencies
    fn analyze_function_calls(&self, operators: &[Operator], behavior_units: &[BehaviorUnit]) -> Result<Vec<(String, String, CallType)>> {
        let mut dependencies = Vec::new();
        
        for op in operators {
            match op {
                Operator::Call { function_index } => {
                    // Find which behavior this function belongs to
                    if let Some(func_name) = self.get_function_name_by_index(*function_index) {
                        if let Some((target_behavior, call_type)) = self.find_function_behavior(&func_name, behavior_units) {
                            dependencies.push((target_behavior, func_name, call_type));
                        }
                    }
                }
                Operator::CallIndirect { .. } => {
                    // Handle indirect calls (virtual method calls)
                    // This is more complex and would require additional analysis
                    // For now, we'll skip indirect calls
                }
                _ => {}
            }
        }
        
        Ok(dependencies)
    }
    
    /// Find which behavior a function belongs to
    fn find_function_behavior(&self, func_name: &str, behavior_units: &[BehaviorUnit]) -> Option<(String, CallType)> {
        for behavior in behavior_units {
            if behavior.local_functions.contains(func_name) {
                // Determine call type based on function name and behavior configuration
                let call_type = if behavior.attribute_config.auto_sync {
                    CallType::Network
                } else {
                    CallType::Direct
                };
                
                return Some((behavior.name.clone(), call_type));
            }
        }
        
        None
    }
    
    /// Build a comprehensive function call graph
    pub fn build_call_graph(&self) -> Result<CallGraph> {
        let mut call_graph = CallGraph {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            reverse_edges: HashMap::new(),
        };
        
        // Create nodes for all functions
        self.create_call_graph_nodes(&mut call_graph)?;
        
        // Analyze function calls to create edges
        self.create_call_graph_edges(&mut call_graph)?;
        
        // Build reverse edges for dependency tracking
        self.build_reverse_edges(&mut call_graph);
        
        Ok(call_graph)
    }
    
    /// Create nodes in the call graph for all functions
    fn create_call_graph_nodes(&self, call_graph: &mut CallGraph) -> Result<()> {
        // Add nodes for all functions (both imported and local)
        let total_functions = self.imports.iter()
            .filter(|imp| matches!(imp.kind, ImportKind::Func(_)))
            .count() + self.function_bodies.len();
        
        for func_index in 0..total_functions as u32 {
            let function_name = self.get_function_name_by_index(func_index);
            let behavior_name = self.get_behavior_name_for_function(func_index);
            let is_entry_point = function_name.as_ref()
                .map(|name| self.has_udon_behaviour_attribute(name))
                .unwrap_or(false);
            
            let unity_events = if let Some(ref name) = function_name {
                self.get_udon_behaviour_attribute(name)
                    .map(|attr| attr.events.clone())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            
            let node = CallGraphNode {
                function_index: func_index,
                function_name,
                behavior_name,
                is_entry_point,
                unity_events,
            };
            
            call_graph.nodes.insert(func_index, node);
        }
        
        Ok(())
    }
    
    /// Create edges in the call graph by analyzing function calls
    fn create_call_graph_edges(&self, call_graph: &mut CallGraph) -> Result<()> {
        for (func_index, operators) in &self.function_bodies {
            let mut edges = Vec::new();
            
            for (instruction_offset, op) in operators.iter().enumerate() {
                match op {
                    Operator::Call { function_index: target_func } => {
                        let call_type = self.determine_call_type(*func_index, *target_func);
                        let edge = CallGraphEdge {
                            target_function: *target_func,
                            call_type,
                            call_site: CallSite {
                                instruction_offset,
                                instruction_type: "call".to_string(),
                            },
                        };
                        edges.push(edge);
                    }
                    Operator::CallIndirect { type_index, .. } => {
                        // Handle indirect calls - these are more complex
                        // For now, we'll create a special edge type
                        let edge = CallGraphEdge {
                            target_function: u32::MAX, // Special marker for indirect calls
                            call_type: CallType::Event, // Indirect calls are often event-based
                            call_site: CallSite {
                                instruction_offset,
                                instruction_type: format!("call_indirect({})", type_index),
                            },
                        };
                        edges.push(edge);
                    }
                    _ => {}
                }
            }
            
            if !edges.is_empty() {
                call_graph.edges.insert(*func_index, edges);
            }
        }
        
        Ok(())
    }
    
    /// Build reverse edges for efficient dependency tracking
    fn build_reverse_edges(&self, call_graph: &mut CallGraph) {
        for (caller, edges) in &call_graph.edges {
            for edge in edges {
                if edge.target_function != u32::MAX { // Skip indirect calls
                    call_graph.reverse_edges
                        .entry(edge.target_function)
                        .or_insert_with(Vec::new)
                        .push(*caller);
                }
            }
        }
    }
    
    /// Determine the type of call between two functions
    fn determine_call_type(&self, caller_func: u32, target_func: u32) -> CallType {
        let caller_behavior = self.get_behavior_name_for_function(caller_func);
        let target_behavior = self.get_behavior_name_for_function(target_func);
        
        match (caller_behavior, target_behavior) {
            (Some(caller_beh), Some(target_beh)) if caller_beh == target_beh => {
                // Same behavior - direct call
                CallType::Direct
            }
            (Some(_), Some(_)) => {
                // Different behaviors - check if network sync is needed
                if let Some(target_name) = self.get_function_name_by_index(target_func) {
                    if let Some(attr) = self.get_udon_behaviour_attribute(&target_name) {
                        if attr.auto_sync {
                            return CallType::Network;
                        }
                    }
                }
                CallType::Event
            }
            _ => {
                // One or both functions don't belong to a behavior - direct call
                CallType::Direct
            }
        }
    }
    
    /// Get behavior name for a function
    fn get_behavior_name_for_function(&self, func_index: u32) -> Option<String> {
        if let Some(func_name) = self.get_function_name_by_index(func_index) {
            // Check if this function has a UdonBehaviour attribute
            if let Some(attr) = self.get_udon_behaviour_attribute(&func_name) {
                return attr.name.clone().or_else(|| {
                    Some(self.infer_behavior_name_from_function(&func_name))
                });
            }
            
            // Try to infer from naming patterns
            for (behavior_func, (_, marker)) in &self.get_udon_behaviour_functions() {
                let behavior_name = marker.name.clone()
                    .unwrap_or_else(|| self.infer_behavior_name_from_function(behavior_func));
                
                if self.function_belongs_to_behavior(&func_name, &behavior_name, behavior_func) {
                    return Some(behavior_name);
                }
            }
        }
        
        None
    }
    
    /// Analyze dependencies across behavior boundaries
    pub fn analyze_cross_behavior_dependencies(&self, call_graph: &CallGraph) -> Result<Vec<CrossBehaviorDependency>> {
        let mut dependencies = Vec::new();
        
        for (caller_func, edges) in &call_graph.edges {
            if let Some(caller_node) = call_graph.nodes.get(caller_func) {
                if let Some(ref caller_behavior) = caller_node.behavior_name {
                    for edge in edges {
                        if let Some(target_node) = call_graph.nodes.get(&edge.target_function) {
                            if let Some(ref target_behavior) = target_node.behavior_name {
                                if caller_behavior != target_behavior {
                                    dependencies.push(CrossBehaviorDependency {
                                        source_behavior: caller_behavior.clone(),
                                        target_behavior: target_behavior.clone(),
                                        source_function: caller_node.function_name.clone()
                                            .unwrap_or_else(|| format!("func_{}", caller_func)),
                                        target_function: target_node.function_name.clone()
                                            .unwrap_or_else(|| format!("func_{}", edge.target_function)),
                                        call_type: edge.call_type.clone(),
                                        call_sites: vec![edge.call_site.clone()],
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Merge duplicate dependencies
        self.merge_duplicate_dependencies(dependencies)
    }
    
    /// Merge duplicate cross-behavior dependencies
    fn merge_duplicate_dependencies(&self, dependencies: Vec<CrossBehaviorDependency>) -> Result<Vec<CrossBehaviorDependency>> {
        let mut merged: HashMap<(String, String, String, String), CrossBehaviorDependency> = HashMap::new();
        
        for dep in dependencies {
            let key = (
                dep.source_behavior.clone(),
                dep.target_behavior.clone(),
                dep.source_function.clone(),
                dep.target_function.clone(),
            );
            
            if let Some(existing) = merged.get_mut(&key) {
                existing.call_sites.extend(dep.call_sites);
            } else {
                merged.insert(key, dep);
            }
        }
        
        Ok(merged.into_values().collect())
    }
    
    /// Find strongly connected components (for circular dependency detection)
    pub fn find_strongly_connected_components(&self, call_graph: &CallGraph) -> Vec<Vec<u32>> {
        let mut scc_finder = StronglyConnectedComponents::new(call_graph);
        scc_finder.find_components()
    }
    
    /// Detect circular dependencies between behaviors
    pub fn detect_circular_dependencies(&self, call_graph: &CallGraph) -> Vec<CircularDependency> {
        let sccs = self.find_strongly_connected_components(call_graph);
        let mut circular_deps = Vec::new();
        
        for scc in sccs {
            if scc.len() > 1 {
                // This is a strongly connected component with multiple functions
                let behaviors: HashSet<String> = scc.iter()
                    .filter_map(|&func_idx| {
                        call_graph.nodes.get(&func_idx)
                            .and_then(|node| node.behavior_name.clone())
                    })
                    .collect();
                
                if behaviors.len() > 1 {
                    // Multiple behaviors involved in the cycle
                    circular_deps.push(CircularDependency {
                        behaviors: behaviors.into_iter().collect(),
                        functions: scc.iter()
                            .filter_map(|&func_idx| {
                                call_graph.nodes.get(&func_idx)
                                    .and_then(|node| node.function_name.clone())
                            })
                            .collect(),
                    });
                }
            }
        }
        
        circular_deps
    }
    
    /// Identify shared functions across behaviors
    fn identify_shared_functions(&self) -> Vec<String> {
        let mut shared_functions = Vec::new();
        
        // Get all behavior units
        let behavior_units = self.identify_behavior_units().unwrap_or_default();
        
        // Collect all functions that are used by multiple behaviors
        let mut function_usage = std::collections::HashMap::new();
        
        for behavior_unit in &behavior_units {
            for shared_dep in &behavior_unit.shared_dependencies {
                *function_usage.entry(shared_dep.clone()).or_insert(0) += 1;
            }
        }
        
        // Functions used by multiple behaviors are considered shared
        for (function_name, usage_count) in function_usage {
            if usage_count > 1 {
                shared_functions.push(function_name);
            }
        }
        
        // Also include functions that are not part of any specific behavior
        for export in &self.exports {
            if export.kind == ExportKind::Func {
                let function_name = &export.name;
                
                // Check if this function is not claimed by any behavior
                let is_behavior_function = behavior_units.iter().any(|bu| {
                    bu.entry_function == *function_name || 
                    bu.local_functions.contains(function_name)
                });
                
                if !is_behavior_function && !shared_functions.contains(function_name) {
                    shared_functions.push(function_name.clone());
                }
            }
        }
        
        shared_functions
    }
    
    /// Build the final analysis result
    fn build_analysis_result(&self) -> OopAnalysisResult {
        let behavior_units = self.identify_behavior_units().unwrap_or_default();
        
        // Build call graph and analyze dependencies
        let call_graph = self.build_call_graph().ok();
        let cross_behavior_dependencies = call_graph.as_ref()
            .and_then(|cg| self.analyze_cross_behavior_dependencies(cg).ok())
            .unwrap_or_default();
        let circular_dependencies = call_graph.as_ref()
            .map(|cg| self.detect_circular_dependencies(cg))
            .unwrap_or_default();
        
        OopAnalysisResult {
            classes: self.patterns.classes.clone(),
            methods: self.patterns.methods.clone(),
            inheritance_relationships: self.patterns.inheritance_relationships.clone(),
            interfaces: self.patterns.interfaces.clone(),
            vtables: self.patterns.vtables.clone(),
            udon_behaviour_functions: self.get_udon_behaviour_functions(),
            exported_functions: self.get_exported_functions(),
            behavior_units,
            call_graph,
            cross_behavior_dependencies,
            circular_dependencies,
            shared_functions: self.identify_shared_functions(),
        }
    }
}

/// Database of detected OOP patterns
#[derive(Debug, Default)]
struct PatternDatabase {
    classes: Vec<ClassInfo>,
    methods: Vec<MethodInfo>,
    inheritance_relationships: Vec<InheritanceInfo>,
    interfaces: Vec<InterfaceInfo>,
    vtables: Vec<VTableInfo>,
}

impl PatternDatabase {
    fn new() -> Self {
        Self::default()
    }
}

/// Information about imports
#[derive(Debug, Clone)]
struct ImportInfo {
    module: String,
    name: String,
    kind: ImportKind,
}

/// Information about exports
#[derive(Debug, Clone)]
struct ExportInfo {
    name: String,
    kind: ExportKind,
    index: u32,
}

/// Import kind enum
#[derive(Debug, Clone)]
enum ImportKind {
    Func(u32),
    Table,
    Memory,
    Global,
    Tag,
}

/// Export kind enum
#[derive(Debug, Clone, PartialEq)]
enum ExportKind {
    Func,
    Table,
    Memory,
    Global,
    Tag,
}

/// Result of OOP behavior analysis
#[derive(Debug, Default, Clone)]
pub struct OopAnalysisResult {
    pub classes: Vec<ClassInfo>,
    pub methods: Vec<MethodInfo>,
    pub inheritance_relationships: Vec<InheritanceInfo>,
    pub interfaces: Vec<InterfaceInfo>,
    pub vtables: Vec<VTableInfo>,
    /// Functions marked with UdonBehaviour attributes (function_name -> (function_index, marker))
    pub udon_behaviour_functions: HashMap<String, (u32, UdonBehaviourMarker)>,
    /// All exported functions (function_name -> function_index)
    pub exported_functions: HashMap<String, u32>,
    /// Identified behavior units
    pub behavior_units: Vec<BehaviorUnit>,
    /// Function call graph
    pub call_graph: Option<CallGraph>,
    /// Cross-behavior dependencies
    pub cross_behavior_dependencies: Vec<CrossBehaviorDependency>,
    /// Circular dependencies
    pub circular_dependencies: Vec<CircularDependency>,
    /// Shared functions across behaviors
    pub shared_functions: Vec<String>,
}

/// Information about a detected class
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub constructor_function: Option<u32>,
}

/// Information about a class field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub offset: usize,
    pub is_public: bool,
}

/// Information about a detected method
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub class: Option<String>,
    pub function_index: u32,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub is_static: bool,
    pub is_virtual: bool,
}

/// Information about inheritance relationships
#[derive(Debug, Clone)]
pub struct InheritanceInfo {
    pub child: String,
    pub parent: String,
    pub relationship_type: InheritanceType,
}

/// Type of inheritance relationship
#[derive(Debug, Clone)]
pub enum InheritanceType {
    Implementation,
    Interface,
    Composition,
}

/// Information about detected interfaces
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    pub name: String,
    pub methods: Vec<String>,
}

/// Information about virtual method tables
#[derive(Debug, Clone)]
pub struct VTableInfo {
    pub class_name: String,
    pub methods: Vec<VTableEntry>,
}

/// Entry in a virtual method table
#[derive(Debug, Clone)]
pub struct VTableEntry {
    pub method_name: String,
    pub function_index: u32,
}

/// Represents a logical UdonBehaviour unit with its functions and dependencies
#[derive(Debug, Clone)]
pub struct BehaviorUnit {
    /// Name of the behavior (from attribute or inferred)
    pub name: String,
    /// Entry function that defines this behavior
    pub entry_function: String,
    /// Function index of the entry function
    pub entry_function_index: u32,
    /// Unity events this behavior handles
    pub unity_events: Vec<String>,
    /// Functions that belong to this behavior unit
    pub local_functions: std::collections::HashSet<String>,
    /// Functions from other behaviors that this unit depends on
    pub shared_dependencies: std::collections::HashSet<String>,
    /// Inter-behavior calls this unit makes
    pub inter_behavior_calls: Vec<InterBehaviorCall>,
    /// UdonBehaviour attribute configuration
    pub attribute_config: UdonBehaviourMarker,
}

/// Represents a call from one behavior to another
#[derive(Debug, Clone)]
pub struct InterBehaviorCall {
    /// Source behavior name
    pub source_behavior: String,
    /// Target behavior name
    pub target_behavior: String,
    /// Function being called
    pub function_name: String,
    /// Type of call (direct, event, network)
    pub call_type: CallType,
}

/// Type of inter-behavior call
#[derive(Debug, Clone, PartialEq)]
pub enum CallType {
    /// Direct method call (same GameObject)
    Direct,
    /// Event-based call via SendCustomEvent
    Event,
    /// Network call via SendCustomNetworkEvent
    Network,
}

/// Function call graph for dependency analysis
#[derive(Debug, Clone)]
pub struct CallGraph {
    /// Nodes in the call graph (function_index -> function_info)
    pub nodes: HashMap<u32, CallGraphNode>,
    /// Edges representing function calls (caller -> callees)
    pub edges: HashMap<u32, Vec<CallGraphEdge>>,
    /// Reverse edges for dependency tracking (callee -> callers)
    pub reverse_edges: HashMap<u32, Vec<u32>>,
}

/// Node in the call graph representing a function
#[derive(Debug, Clone)]
pub struct CallGraphNode {
    /// Function index in WASM
    pub function_index: u32,
    /// Function name (if exported)
    pub function_name: Option<String>,
    /// Behavior this function belongs to (if any)
    pub behavior_name: Option<String>,
    /// Whether this function is an entry point (has UdonBehaviour attribute)
    pub is_entry_point: bool,
    /// Unity events this function handles
    pub unity_events: Vec<String>,
}

/// Edge in the call graph representing a function call
#[derive(Debug, Clone)]
pub struct CallGraphEdge {
    /// Target function index
    pub target_function: u32,
    /// Type of call
    pub call_type: CallType,
    /// Call site information (for debugging)
    pub call_site: CallSite,
}

/// Information about where a call occurs
#[derive(Debug, Clone)]
pub struct CallSite {
    /// Instruction offset in the function
    pub instruction_offset: usize,
    /// Type of WASM instruction
    pub instruction_type: String,
}

/// Cross-behavior dependency information
#[derive(Debug, Clone)]
pub struct CrossBehaviorDependency {
    /// Source behavior making the call
    pub source_behavior: String,
    /// Target behavior being called
    pub target_behavior: String,
    /// Source function making the call
    pub source_function: String,
    /// Target function being called
    pub target_function: String,
    /// Type of call
    pub call_type: CallType,
    /// All call sites where this dependency occurs
    pub call_sites: Vec<CallSite>,
}

/// Circular dependency information
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// Behaviors involved in the circular dependency
    pub behaviors: Vec<String>,
    /// Functions involved in the circular dependency
    pub functions: Vec<String>,
}

/// Strongly Connected Components finder using Tarjan's algorithm
struct StronglyConnectedComponents<'a> {
    call_graph: &'a CallGraph,
    index: u32,
    stack: Vec<u32>,
    indices: HashMap<u32, u32>,
    lowlinks: HashMap<u32, u32>,
    on_stack: HashSet<u32>,
    components: Vec<Vec<u32>>,
}

impl<'a> StronglyConnectedComponents<'a> {
    fn new(call_graph: &'a CallGraph) -> Self {
        Self {
            call_graph,
            index: 0,
            stack: Vec::new(),
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            components: Vec::new(),
        }
    }
    
    fn find_components(mut self) -> Vec<Vec<u32>> {
        // Run Tarjan's algorithm on all nodes
        for &node in self.call_graph.nodes.keys() {
            if !self.indices.contains_key(&node) {
                self.strongconnect(node);
            }
        }
        
        self.components
    }
    
    fn strongconnect(&mut self, v: u32) {
        // Set the depth index for v to the smallest unused index
        self.indices.insert(v, self.index);
        self.lowlinks.insert(v, self.index);
        self.index += 1;
        self.stack.push(v);
        self.on_stack.insert(v);
        
        // Consider successors of v
        if let Some(edges) = self.call_graph.edges.get(&v) {
            for edge in edges {
                let w = edge.target_function;
                if w == u32::MAX { // Skip indirect calls
                    continue;
                }
                
                if !self.indices.contains_key(&w) {
                    // Successor w has not yet been visited; recurse on it
                    self.strongconnect(w);
                    let v_lowlink = self.lowlinks[&v];
                    let w_lowlink = self.lowlinks[&w];
                    self.lowlinks.insert(v, v_lowlink.min(w_lowlink));
                } else if self.on_stack.contains(&w) {
                    // Successor w is in stack and hence in the current SCC
                    let v_lowlink = self.lowlinks[&v];
                    let w_index = self.indices[&w];
                    self.lowlinks.insert(v, v_lowlink.min(w_index));
                }
            }
        }
        
        // If v is a root node, pop the stack and print an SCC
        if self.lowlinks[&v] == self.indices[&v] {
            let mut component = Vec::new();
            loop {
                let w = self.stack.pop().unwrap();
                self.on_stack.remove(&w);
                component.push(w);
                if w == v {
                    break;
                }
            }
            self.components.push(component);
        }
    }
}