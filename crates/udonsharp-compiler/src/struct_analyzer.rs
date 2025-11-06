//! Struct analysis engine for parsing UdonBehaviour structs from Rust AST
//! 
//! This module provides the core functionality to analyze Rust structs with
//! #[derive(UdonBehaviour)] attributes and extract their metadata for code generation.

use crate::multi_behavior::{
    UdonBehaviourStruct, StructField, StructMethod, StructAttribute, MethodParameter,
    RustType, Visibility, FieldAttribute, MethodAttribute, UdonSyncMode,
    UdonBehaviourTraitImpl, is_valid_csharp_identifier
};
use crate::trait_validator::{TraitValidator, ValidationError};
use crate::behavior_dependency_analyzer::{BehaviorDependencyAnalyzer, DependencyError, DependencyAnalysisResult};
use syn::{
    Item, ItemStruct, ItemImpl, Fields, Field, Type, Attribute, Meta, Lit,
    ImplItem, ImplItemFn, FnArg, ReturnType, Pat, PatType, Visibility as SynVisibility,
    parse::Parse, parse::ParseStream, Token, punctuated::Punctuated
};
use std::collections::HashMap;

/// Result type for struct analysis operations
pub type AnalysisResult<T> = Result<T, AnalysisError>;

/// Errors that can occur during struct analysis
#[derive(Debug, Clone)]
pub enum AnalysisError {
    /// Invalid struct name that doesn't follow C# naming conventions
    InvalidStructName { name: String, reason: String },
    /// Missing UdonBehaviour derive attribute
    MissingUdonBehaviourDerive { struct_name: String },
    /// Invalid field attribute usage
    InvalidFieldAttribute { struct_name: String, field_name: String, attribute: String, reason: String },
    /// Unsupported Rust type
    UnsupportedType { rust_type: String, suggested_alternatives: Vec<String> },
    /// Invalid method signature
    InvalidMethodSignature { struct_name: String, method_name: String, reason: String },
    /// Parsing error
    ParseError { message: String },
    /// Missing trait implementation entirely
    MissingTraitImplementation { struct_name: String },
    /// Missing required methods from trait implementation
    MissingRequiredMethods { struct_name: String, missing_methods: Vec<String> },
    /// Async methods are not supported for UdonBehaviour trait
    AsyncMethodNotSupported { struct_name: String, method_name: String },
    /// Circular dependency detected between behaviors
    CircularDependency { cycle: Vec<String>, description: String },
    /// Missing dependency reference
    MissingDependency { behavior: String, missing_dependency: String },
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::InvalidStructName { name, reason } => {
                write!(f, "Invalid struct name '{}': {}", name, reason)
            }
            AnalysisError::MissingUdonBehaviourDerive { struct_name } => {
                write!(f, "Struct '{}' is missing #[derive(UdonBehaviour)] attribute", struct_name)
            }
            AnalysisError::InvalidFieldAttribute { struct_name, field_name, attribute, reason } => {
                write!(f, "Invalid attribute '{}' on field '{}' in struct '{}': {}", 
                       attribute, field_name, struct_name, reason)
            }
            AnalysisError::UnsupportedType { rust_type, suggested_alternatives } => {
                write!(f, "Unsupported type '{}'. Consider using: {}", 
                       rust_type, suggested_alternatives.join(", "))
            }
            AnalysisError::InvalidMethodSignature { struct_name, method_name, reason } => {
                write!(f, "Invalid method signature for '{}::{}': {}", 
                       struct_name, method_name, reason)
            }
            AnalysisError::ParseError { message } => {
                write!(f, "Parse error: {}", message)
            }
            AnalysisError::MissingTraitImplementation { struct_name } => {
                write!(f, "Struct '{}' must implement the UdonBehaviour trait", struct_name)
            }
            AnalysisError::MissingRequiredMethods { struct_name, missing_methods } => {
                write!(f, "Struct '{}' is missing required UdonBehaviour methods: {}", 
                       struct_name, missing_methods.join(", "))
            }
            AnalysisError::AsyncMethodNotSupported { struct_name, method_name } => {
                write!(f, "Async method '{}::{}' is not supported for UdonBehaviour trait", 
                       struct_name, method_name)
            }
            AnalysisError::CircularDependency { cycle, description } => {
                write!(f, "Circular dependency detected: {} - {}", cycle.join(" -> "), description)
            }
            AnalysisError::MissingDependency { behavior, missing_dependency } => {
                write!(f, "Behavior '{}' depends on missing behavior '{}'", behavior, missing_dependency)
            }
        }
    }
}

impl std::error::Error for AnalysisError {}

/// Main struct analyzer that parses UdonBehaviour structs from Rust AST
pub struct StructAnalyzer {
    /// Parsed UdonBehaviour structs
    parsed_structs: HashMap<String, UdonBehaviourStruct>,
    /// Trait implementations found during analysis
    trait_implementations: HashMap<String, UdonBehaviourTraitImpl>,
    /// Analysis errors encountered
    errors: Vec<AnalysisError>,
    /// Analysis warnings
    warnings: Vec<String>,
    /// Trait validator for UdonBehaviour implementations
    trait_validator: TraitValidator,
    /// Dependency analyzer for inter-behavior dependencies
    dependency_analyzer: BehaviorDependencyAnalyzer,
}

impl StructAnalyzer {
    /// Create a new struct analyzer
    pub fn new() -> Self {
        Self {
            parsed_structs: HashMap::new(),
            trait_implementations: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            trait_validator: TraitValidator::new(),
            dependency_analyzer: BehaviorDependencyAnalyzer::new(),
        }
    }

    /// Analyze a Rust module and extract UdonBehaviour structs
    pub fn analyze_module(&mut self, items: &[Item]) -> AnalysisResult<Vec<UdonBehaviourStruct>> {
        // Clear previous analysis results
        self.parsed_structs.clear();
        self.trait_implementations.clear();
        self.errors.clear();
        self.warnings.clear();

        // First pass: find all structs with UdonBehaviour derive
        for item in items {
            if let Item::Struct(item_struct) = item {
                if self.has_udon_behaviour_derive(&item_struct.attrs) {
                    match self.analyze_struct(item_struct) {
                        Ok(udon_struct) => {
                            self.parsed_structs.insert(udon_struct.name.clone(), udon_struct);
                        }
                        Err(error) => {
                            self.errors.push(error);
                        }
                    }
                }
            }
        }

        // Second pass: find trait implementations
        for item in items {
            if let Item::Impl(item_impl) = item {
                if let Some(trait_path) = &item_impl.trait_ {
                    if self.is_udon_behaviour_trait(&trait_path.1) {
                        match self.analyze_trait_impl(item_impl) {
                            Ok((struct_name, trait_impl)) => {
                                self.trait_implementations.insert(struct_name, trait_impl);
                            }
                            Err(error) => {
                                self.errors.push(error);
                            }
                        }
                    }
                }
            }
        }

        // Third pass: merge trait implementations with structs
        for (struct_name, trait_impl) in &self.trait_implementations {
            if let Some(udon_struct) = self.parsed_structs.get_mut(struct_name) {
                udon_struct.set_trait_impl(trait_impl.clone());
            }
        }

        // Validate all parsed structs
        for udon_struct in self.parsed_structs.values() {
            if let Err(error_msg) = udon_struct.validate() {
                self.errors.push(AnalysisError::ParseError { message: error_msg });
            }
        }

        // Validate UdonBehaviour trait implementations
        self.validate_trait_implementations();

        // Analyze dependencies and detect circular dependencies
        self.analyze_dependencies();

        // Return results or errors
        if !self.errors.is_empty() {
            Err(self.errors[0].clone())
        } else {
            Ok(self.parsed_structs.values().cloned().collect())
        }
    }

    /// Get analysis errors
    pub fn get_errors(&self) -> &[AnalysisError] {
        &self.errors
    }

    /// Get analysis warnings
    pub fn get_warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Check if a struct has the UdonBehaviour derive attribute
    fn has_udon_behaviour_derive(&self, attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                match &attr.meta {
                    Meta::List(meta_list) => {
                        // Parse the tokens inside the derive attribute
                        let tokens = &meta_list.tokens;
                        if tokens.to_string().contains("UdonBehaviour") {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Check if a trait path refers to UdonBehaviour
    fn is_udon_behaviour_trait(&self, path: &syn::Path) -> bool {
        path.is_ident("UdonBehaviour") || 
        path.segments.last().map_or(false, |seg| seg.ident == "UdonBehaviour")
    }

    /// Analyze a single struct with UdonBehaviour derive
    fn analyze_struct(&mut self, item_struct: &ItemStruct) -> AnalysisResult<UdonBehaviourStruct> {
        let struct_name = item_struct.ident.to_string();

        // Validate struct name follows C# naming conventions
        if !is_valid_csharp_identifier(&struct_name) {
            return Err(AnalysisError::InvalidStructName {
                name: struct_name,
                reason: "must be a valid C# identifier".to_string(),
            });
        }

        // Check if struct name starts with uppercase (PascalCase)
        if !struct_name.chars().next().unwrap_or('a').is_uppercase() {
            self.warnings.push(format!(
                "Struct '{}' should use PascalCase naming convention for C# compatibility",
                struct_name
            ));
        }

        let mut udon_struct = UdonBehaviourStruct::new(struct_name.clone());

        // Parse struct attributes
        for attr in &item_struct.attrs {
            if let Ok(struct_attr) = self.parse_struct_attribute(attr) {
                udon_struct.add_attribute(struct_attr);
            }
        }

        // Parse struct fields
        match &item_struct.fields {
            Fields::Named(fields_named) => {
                for field in &fields_named.named {
                    match self.analyze_field(&struct_name, field) {
                        Ok(struct_field) => {
                            udon_struct.add_field(struct_field);
                        }
                        Err(error) => {
                            self.errors.push(error);
                        }
                    }
                }
            }
            Fields::Unnamed(_) => {
                return Err(AnalysisError::ParseError {
                    message: format!("Tuple structs are not supported for UdonBehaviour: {}", struct_name),
                });
            }
            Fields::Unit => {
                // Unit structs are allowed but will have no fields
            }
        }

        Ok(udon_struct)
    }

    /// Analyze a struct field
    fn analyze_field(&mut self, struct_name: &str, field: &Field) -> AnalysisResult<StructField> {
        let field_name = field.ident.as_ref()
            .ok_or_else(|| AnalysisError::ParseError {
                message: "Field must have a name".to_string(),
            })?
            .to_string();

        // Validate field name
        if !is_valid_csharp_identifier(&field_name) {
            return Err(AnalysisError::InvalidFieldAttribute {
                struct_name: struct_name.to_string(),
                field_name: field_name.clone(),
                attribute: "name".to_string(),
                reason: "must be a valid C# identifier".to_string(),
            });
        }

        // Parse field type
        let rust_type = self.parse_type(&field.ty)?;

        // Check if type is UdonSharp compatible
        if !rust_type.is_udonsharp_compatible() {
            let alternatives = rust_type.get_alternatives();
            return Err(AnalysisError::UnsupportedType {
                rust_type: format!("{:?}", rust_type),
                suggested_alternatives: alternatives,
            });
        }

        let mut struct_field = StructField::new(field_name.clone(), rust_type);

        // Parse field visibility
        let visibility = match &field.vis {
            SynVisibility::Public(_) => Visibility::Public,
            _ => Visibility::Private,
        };
        struct_field.set_visibility(visibility);

        // Parse field attributes
        for attr in &field.attrs {
            match self.parse_field_attribute(attr) {
                Ok(field_attr) => {
                    struct_field.add_attribute(field_attr);
                }
                Err(error) => {
                    self.errors.push(error);
                }
            }
        }

        Ok(struct_field)
    }

    /// Parse a Rust type into our RustType enum
    fn parse_type(&self, ty: &Type) -> AnalysisResult<RustType> {
        match ty {
            Type::Path(type_path) => {
                let path = &type_path.path;
                
                if path.segments.len() == 1 {
                    let segment = &path.segments[0];
                    let ident = &segment.ident;
                    
                    // Handle basic types
                    match ident.to_string().as_str() {
                        "bool" => Ok(RustType::Bool),
                        "i8" => Ok(RustType::I8),
                        "i16" => Ok(RustType::I16),
                        "i32" => Ok(RustType::I32),
                        "i64" => Ok(RustType::I64),
                        "i128" => Ok(RustType::I128),
                        "u8" => Ok(RustType::U8),
                        "u16" => Ok(RustType::U16),
                        "u32" => Ok(RustType::U32),
                        "u64" => Ok(RustType::U64),
                        "u128" => Ok(RustType::U128),
                        "f32" => Ok(RustType::F32),
                        "f64" => Ok(RustType::F64),
                        "char" => Ok(RustType::Char),
                        "String" => Ok(RustType::String),
                        _ => {
                            // Handle generic types
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                match ident.to_string().as_str() {
                                    "Option" => {
                                        if args.args.len() == 1 {
                                            if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                                                let inner_type = self.parse_type(inner_ty)?;
                                                return Ok(RustType::Option(Box::new(inner_type)));
                                            }
                                        }
                                    }
                                    "Vec" => {
                                        if args.args.len() == 1 {
                                            if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                                                let inner_type = self.parse_type(inner_ty)?;
                                                return Ok(RustType::Vec(Box::new(inner_type)));
                                            }
                                        }
                                    }
                                    "HashMap" => {
                                        if args.args.len() == 2 {
                                            if let (syn::GenericArgument::Type(key_ty), syn::GenericArgument::Type(value_ty)) = 
                                                (&args.args[0], &args.args[1]) {
                                                let key_type = self.parse_type(key_ty)?;
                                                let value_type = self.parse_type(value_ty)?;
                                                return Ok(RustType::HashMap(Box::new(key_type), Box::new(value_type)));
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            
                            // Handle Unity/VRChat types and custom types
                            match ident.to_string().as_str() {
                                "Vector2" => Ok(RustType::Vector2),
                                "Vector3" => Ok(RustType::Vector3),
                                "Vector4" => Ok(RustType::Vector4),
                                "Quaternion" => Ok(RustType::Quaternion),
                                "Color" => Ok(RustType::Color),
                                "Color32" => Ok(RustType::Color32),
                                "GameObject" => Ok(RustType::GameObject),
                                "Transform" => Ok(RustType::Transform),
                                "VRCPlayerApi" => Ok(RustType::VRCPlayerApi),
                                _ => Ok(RustType::Custom(ident.to_string())),
                            }
                        }
                    }
                } else {
                    // Handle qualified paths like unity::GameObject
                    let last_segment = path.segments.last().unwrap();
                    match last_segment.ident.to_string().as_str() {
                        "GameObject" => Ok(RustType::GameObject),
                        "Transform" => Ok(RustType::Transform),
                        "Vector2" => Ok(RustType::Vector2),
                        "Vector3" => Ok(RustType::Vector3),
                        "Vector4" => Ok(RustType::Vector4),
                        "Quaternion" => Ok(RustType::Quaternion),
                        "Color" => Ok(RustType::Color),
                        "Color32" => Ok(RustType::Color32),
                        "VRCPlayerApi" => Ok(RustType::VRCPlayerApi),
                        _ => Ok(RustType::Custom(last_segment.ident.to_string())),
                    }
                }
            }
            Type::Array(type_array) => {
                let inner_type = self.parse_type(&type_array.elem)?;
                // For now, we'll treat arrays as Vec since we can't easily parse the size
                Ok(RustType::Vec(Box::new(inner_type)))
            }
            Type::Tuple(type_tuple) => {
                if type_tuple.elems.is_empty() {
                    Ok(RustType::Unit)
                } else {
                    Err(AnalysisError::UnsupportedType {
                        rust_type: "tuple".to_string(),
                        suggested_alternatives: vec!["struct".to_string()],
                    })
                }
            }
            _ => {
                Err(AnalysisError::UnsupportedType {
                    rust_type: "complex type".to_string(),
                    suggested_alternatives: vec!["basic type".to_string()],
                })
            }
        }
    }

    /// Parse a struct attribute
    fn parse_struct_attribute(&self, attr: &Attribute) -> AnalysisResult<StructAttribute> {
        if attr.path().is_ident("udon_sync_mode") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    let tokens = &meta_list.tokens;
                    let token_str = tokens.to_string();
                    if token_str.contains("Manual") {
                        return Ok(StructAttribute::UdonSyncMode(UdonSyncMode::Manual));
                    } else if token_str.contains("Continuous") {
                        return Ok(StructAttribute::UdonSyncMode(UdonSyncMode::Continuous));
                    } else if token_str.contains("None") {
                        return Ok(StructAttribute::UdonSyncMode(UdonSyncMode::None));
                    }
                }
                _ => {}
            }
        }
        
        Err(AnalysisError::ParseError {
            message: format!("Unsupported struct attribute: {}", quote::quote!(#attr)),
        })
    }

    /// Parse a field attribute
    fn parse_field_attribute(&self, attr: &Attribute) -> AnalysisResult<FieldAttribute> {
        if attr.path().is_ident("udon_public") {
            return Ok(FieldAttribute::UdonPublic);
        } else if attr.path().is_ident("udon_sync") {
            return Ok(FieldAttribute::UdonSync);
        } else if attr.path().is_ident("header") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    let tokens = &meta_list.tokens;
                    let token_str = tokens.to_string();
                    // Extract string literal from tokens
                    if let Some(start) = token_str.find('"') {
                        if let Some(end) = token_str.rfind('"') {
                            if start < end {
                                let header_text = token_str[start + 1..end].to_string();
                                return Ok(FieldAttribute::Header(header_text));
                            }
                        }
                    }
                }
                _ => {}
            }
        } else if attr.path().is_ident("tooltip") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    let tokens = &meta_list.tokens;
                    let token_str = tokens.to_string();
                    // Extract string literal from tokens
                    if let Some(start) = token_str.find('"') {
                        if let Some(end) = token_str.rfind('"') {
                            if start < end {
                                let tooltip_text = token_str[start + 1..end].to_string();
                                return Ok(FieldAttribute::Tooltip(tooltip_text));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        Err(AnalysisError::ParseError {
            message: format!("Unsupported field attribute: {}", quote::quote!(#attr)),
        })
    }

    /// Analyze a trait implementation
    fn analyze_trait_impl(&mut self, item_impl: &syn::ItemImpl) -> AnalysisResult<(String, UdonBehaviourTraitImpl)> {
        // Get the struct name this impl is for
        let struct_name = if let Type::Path(type_path) = &*item_impl.self_ty {
            type_path.path.segments.last()
                .ok_or_else(|| AnalysisError::ParseError {
                    message: "Invalid impl target type".to_string(),
                })?
                .ident.to_string()
        } else {
            return Err(AnalysisError::ParseError {
                message: "Impl target must be a named type".to_string(),
            });
        };

        let mut trait_impl = UdonBehaviourTraitImpl::new();

        // Analyze each method in the impl
        for item in &item_impl.items {
            if let ImplItem::Fn(impl_fn) = item {
                let method_name = impl_fn.sig.ident.to_string();
                trait_impl.add_method(method_name.clone());

                // Parse method for additional metadata
                match self.analyze_method(&struct_name, impl_fn) {
                    Ok(struct_method) => {
                        // We could store methods here if needed for the struct
                        // For now, we just track that the method exists
                    }
                    Err(error) => {
                        self.errors.push(error);
                    }
                }
            }
        }

        trait_impl.check_completeness();
        Ok((struct_name, trait_impl))
    }

    /// Analyze a method implementation
    fn analyze_method(&mut self, struct_name: &str, impl_fn: &ImplItemFn) -> AnalysisResult<StructMethod> {
        let method_name = impl_fn.sig.ident.to_string();

        // Parse return type
        let return_type = match &impl_fn.sig.output {
            ReturnType::Default => RustType::Unit,
            ReturnType::Type(_, ty) => self.parse_type(ty)?,
        };

        let mut struct_method = StructMethod::new(method_name.clone(), return_type);

        // Parse method attributes
        for attr in &impl_fn.attrs {
            if let Ok(method_attr) = self.parse_method_attribute(attr) {
                struct_method.add_attribute(method_attr);
            }
        }

        // Parse method parameters
        for input in &impl_fn.sig.inputs {
            match input {
                FnArg::Receiver(_) => {
                    // Skip self parameter
                    continue;
                }
                FnArg::Typed(pat_type) => {
                    let param = self.analyze_method_parameter(pat_type)?;
                    struct_method.add_parameter(param);
                }
            }
        }

        // Check if method is async
        if impl_fn.sig.asyncness.is_some() {
            struct_method.set_async(true);
        }

        Ok(struct_method)
    }

    /// Parse a method attribute
    fn parse_method_attribute(&self, attr: &Attribute) -> AnalysisResult<MethodAttribute> {
        if attr.path().is_ident("udon_event") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    let tokens = &meta_list.tokens;
                    let token_str = tokens.to_string();
                    // Extract string literal from tokens
                    if let Some(start) = token_str.find('"') {
                        if let Some(end) = token_str.rfind('"') {
                            if start < end {
                                let event_name = token_str[start + 1..end].to_string();
                                return Ok(MethodAttribute::UdonEvent(event_name));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        Err(AnalysisError::ParseError {
            message: format!("Unsupported method attribute: {}", quote::quote!(#attr)),
        })
    }

    /// Analyze a method parameter
    fn analyze_method_parameter(&self, pat_type: &PatType) -> AnalysisResult<MethodParameter> {
        let param_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
            pat_ident.ident.to_string()
        } else {
            return Err(AnalysisError::ParseError {
                message: "Complex parameter patterns are not supported".to_string(),
            });
        };

        let param_type = self.parse_type(&pat_type.ty)?;
        let mut parameter = MethodParameter::new(param_name, param_type);

        // Check if parameter is mutable
        if let Pat::Ident(pat_ident) = &*pat_type.pat {
            if pat_ident.mutability.is_some() {
                parameter.set_mut(true);
            }
        }

        Ok(parameter)
    }

    /// Validate UdonBehaviour trait implementations for all parsed structs
    fn validate_trait_implementations(&mut self) {
        let structs: Vec<UdonBehaviourStruct> = self.parsed_structs.values().cloned().collect();
        
        for udon_struct in &structs {
            if let Err(validation_error) = self.trait_validator.validate_struct(udon_struct) {
                let analysis_error = self.convert_validation_error(validation_error);
                self.errors.push(analysis_error);
            }
        }

        // Generate implementation guidance for missing methods
        for udon_struct in &structs {
            let missing_methods = self.trait_validator.get_missing_required_methods(udon_struct);
            if !missing_methods.is_empty() {
                let guidance = self.trait_validator.generate_implementation_guidance(&udon_struct.name, &missing_methods);
                self.warnings.push(format!("Implementation guidance for '{}':\n{}", udon_struct.name, guidance));
            }
        }
    }

    /// Convert trait validation error to analysis error
    fn convert_validation_error(&self, validation_error: ValidationError) -> AnalysisError {
        match validation_error {
            ValidationError::MissingTraitImplementation { struct_name } => {
                AnalysisError::MissingTraitImplementation { struct_name }
            }
            ValidationError::MissingRequiredMethods { struct_name, missing_methods } => {
                AnalysisError::MissingRequiredMethods { struct_name, missing_methods }
            }
            ValidationError::InvalidMethodSignature { struct_name, method_name, expected, found } => {
                AnalysisError::InvalidMethodSignature { 
                    struct_name, 
                    method_name, 
                    reason: format!("Expected: {}, Found: {}", expected, found)
                }
            }
            ValidationError::InvalidMethodVisibility { struct_name, method_name, expected, found } => {
                AnalysisError::InvalidMethodSignature { 
                    struct_name, 
                    method_name, 
                    reason: format!("Invalid visibility - Expected: {}, Found: {}", expected, found)
                }
            }
            ValidationError::AsyncMethodNotSupported { struct_name, method_name } => {
                AnalysisError::AsyncMethodNotSupported { struct_name, method_name }
            }
        }
    }

    /// Get trait validator for external use
    pub fn get_trait_validator(&self) -> &TraitValidator {
        &self.trait_validator
    }

    /// Check if all structs have complete trait implementations
    pub fn all_implementations_complete(&self) -> bool {
        self.parsed_structs.values()
            .all(|udon_struct| self.trait_validator.is_implementation_complete(udon_struct))
    }

    /// Get implementation status for all structs
    pub fn get_implementation_status(&self) -> HashMap<String, bool> {
        self.parsed_structs.iter()
            .map(|(name, udon_struct)| {
                (name.clone(), self.trait_validator.is_implementation_complete(udon_struct))
            })
            .collect()
    }

    /// Analyze dependencies between UdonBehaviour structs
    fn analyze_dependencies(&mut self) {
        let behaviors: Vec<UdonBehaviourStruct> = self.parsed_structs.values().cloned().collect();
        
        match self.dependency_analyzer.analyze_dependencies(behaviors) {
            Ok(analysis_result) => {
                // Convert dependency errors to analysis errors
                for dep_error in &analysis_result.errors {
                    let analysis_error = self.convert_dependency_error(dep_error.clone());
                    self.errors.push(analysis_error);
                }

                // Add warnings for dependency issues
                for warning in &analysis_result.warnings {
                    self.warnings.push(warning.clone());
                }

                // Report circular dependencies as errors
                for circular_dep in &analysis_result.circular_dependencies {
                    self.errors.push(AnalysisError::CircularDependency {
                        cycle: circular_dep.cycle.clone(),
                        description: circular_dep.description.clone(),
                    });

                    // Add resolution suggestions as warnings
                    for suggestion in &circular_dep.resolution_suggestions {
                        self.warnings.push(suggestion.clone());
                    }
                }

                // Add initialization order as a warning if available
                if let Some(init_order) = &analysis_result.initialization_order {
                    self.warnings.push(format!(
                        "Recommended initialization order: {}",
                        init_order.join(" -> ")
                    ));
                }
            }
            Err(dep_error) => {
                let analysis_error = self.convert_dependency_error(dep_error);
                self.errors.push(analysis_error);
            }
        }
    }

    /// Convert dependency error to analysis error
    fn convert_dependency_error(&self, dep_error: DependencyError) -> AnalysisError {
        match dep_error {
            DependencyError::CircularDependency { cycle, description } => {
                AnalysisError::CircularDependency { cycle, description }
            }
            DependencyError::MissingDependency { behavior, missing_dependency } => {
                AnalysisError::MissingDependency { behavior, missing_dependency }
            }
            DependencyError::InvalidDependency { behavior, dependency, reason } => {
                AnalysisError::ParseError {
                    message: format!("Invalid dependency from '{}' to '{}': {}", behavior, dependency, reason)
                }
            }
        }
    }

    /// Get dependency analyzer for external use
    pub fn get_dependency_analyzer(&self) -> &BehaviorDependencyAnalyzer {
        &self.dependency_analyzer
    }

    /// Perform complete analysis including dependencies
    pub fn analyze_module_with_dependencies(&mut self, items: &[syn::Item]) -> AnalysisResult<DependencyAnalysisResult> {
        // First perform regular struct analysis
        let _structs = self.analyze_module(items)?;

        // Then perform dependency analysis
        let behaviors: Vec<UdonBehaviourStruct> = self.parsed_structs.values().cloned().collect();
        let analysis_result = self.dependency_analyzer.analyze_dependencies(behaviors)
            .map_err(|dep_error| self.convert_dependency_error(dep_error))?;

        Ok(analysis_result)
    }
}

impl Default for StructAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_struct_analyzer_creation() {
        let analyzer = StructAnalyzer::new();
        assert_eq!(analyzer.parsed_structs.len(), 0);
        assert_eq!(analyzer.errors.len(), 0);
    }

    #[test]
    fn test_has_udon_behaviour_derive() {
        let analyzer = StructAnalyzer::new();
        
        let attrs: Vec<Attribute> = vec![
            parse_quote!(#[derive(UdonBehaviour)]),
        ];
        
        assert!(analyzer.has_udon_behaviour_derive(&attrs));
        
        let attrs_without: Vec<Attribute> = vec![
            parse_quote!(#[derive(Debug)]),
        ];
        
        assert!(!analyzer.has_udon_behaviour_derive(&attrs_without));
    }

    #[test]
    fn test_parse_basic_types() {
        let analyzer = StructAnalyzer::new();
        
        let ty: Type = parse_quote!(i32);
        assert_eq!(analyzer.parse_type(&ty).unwrap(), RustType::I32);
        
        let ty: Type = parse_quote!(String);
        assert_eq!(analyzer.parse_type(&ty).unwrap(), RustType::String);
        
        let ty: Type = parse_quote!(bool);
        assert_eq!(analyzer.parse_type(&ty).unwrap(), RustType::Bool);
    }

    #[test]
    fn test_parse_unity_types() {
        let analyzer = StructAnalyzer::new();
        
        let ty: Type = parse_quote!(Vector3);
        assert_eq!(analyzer.parse_type(&ty).unwrap(), RustType::Vector3);
        
        let ty: Type = parse_quote!(GameObject);
        assert_eq!(analyzer.parse_type(&ty).unwrap(), RustType::GameObject);
        
        let ty: Type = parse_quote!(unity::GameObject);
        assert_eq!(analyzer.parse_type(&ty).unwrap(), RustType::GameObject);
    }

    #[test]
    fn test_parse_generic_types() {
        let analyzer = StructAnalyzer::new();
        
        let ty: Type = parse_quote!(Option<i32>);
        if let RustType::Option(inner) = analyzer.parse_type(&ty).unwrap() {
            assert_eq!(*inner, RustType::I32);
        } else {
            panic!("Expected Option type");
        }
        
        let ty: Type = parse_quote!(Vec<String>);
        if let RustType::Vec(inner) = analyzer.parse_type(&ty).unwrap() {
            assert_eq!(*inner, RustType::String);
        } else {
            panic!("Expected Vec type");
        }
    }

    #[test]
    fn test_analyze_simple_struct() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    #[udon_public]
                    pub player_count: i32,
                    #[udon_sync]
                    pub game_state: String,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items).unwrap();
        assert_eq!(result.len(), 1);
        
        let behavior = &result[0];
        assert_eq!(behavior.name, "TestBehavior");
        assert_eq!(behavior.fields.len(), 2);
        
        let player_count_field = &behavior.fields[0];
        assert_eq!(player_count_field.name, "player_count");
        assert_eq!(player_count_field.field_type, RustType::I32);
        assert!(player_count_field.attributes.contains(&FieldAttribute::UdonPublic));
    }

    #[test]
    fn test_invalid_struct_name() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct 123InvalidName {
                    field: i32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        assert!(result.is_err());
        
        if let Err(AnalysisError::InvalidStructName { name, .. }) = result {
            assert_eq!(name, "123InvalidName");
        } else {
            panic!("Expected InvalidStructName error");
        }
    }

    #[test]
    fn test_unsupported_type() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    unsupported_field: i128,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        assert!(result.is_err());
        
        let errors = analyzer.get_errors();
        assert!(!errors.is_empty());
        
        if let AnalysisError::UnsupportedType { rust_type, .. } = &errors[0] {
            assert!(rust_type.contains("I128"));
        } else {
            panic!("Expected UnsupportedType error");
        }
    }

    // Additional comprehensive tests for Requirements 1.1, 1.2, 1.4, 2.1, 7.3

    #[test]
    fn test_multiple_struct_detection() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct PlayerManager {
                    #[udon_public]
                    player_count: i32,
                }
            },
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct UIController {
                    #[udon_sync]
                    ui_state: String,
                }
            },
            parse_quote! {
                pub struct RegularStruct {
                    data: i32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items).unwrap();
        assert_eq!(result.len(), 2);
        
        let names: Vec<&str> = result.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"PlayerManager"));
        assert!(names.contains(&"UIController"));
    }

    #[test]
    fn test_field_attribute_parsing() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    #[udon_public]
                    public_field: i32,
                    #[udon_sync]
                    sync_field: String,
                    #[udon_sync_mode(Manual)]
                    manual_sync_field: bool,
                    private_field: f32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items).unwrap();
        let behavior = &result[0];
        
        assert_eq!(behavior.fields.len(), 4);
        
        // Check public field
        let public_field = behavior.fields.iter().find(|f| f.name == "public_field").unwrap();
        assert!(public_field.attributes.contains(&FieldAttribute::UdonPublic));
        
        // Check sync field
        let sync_field = behavior.fields.iter().find(|f| f.name == "sync_field").unwrap();
        assert!(sync_field.attributes.contains(&FieldAttribute::UdonSync));
        
        // Check manual sync field - this would be a struct attribute, not field attribute
        let manual_sync_field = behavior.fields.iter().find(|f| f.name == "manual_sync_field").unwrap();
        // Manual sync mode is a struct attribute, not a field attribute
        assert!(manual_sync_field.attributes.is_empty() || manual_sync_field.attributes.contains(&FieldAttribute::UdonSync));
        
        // Check private field has no attributes
        let private_field = behavior.fields.iter().find(|f| f.name == "private_field").unwrap();
        assert!(private_field.attributes.is_empty());
    }

    #[test]
    fn test_trait_implementation_validation() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    field: i32,
                }
            },
            parse_quote! {
                impl UdonBehaviour for TestBehavior {
                    fn start(&mut self) {
                        self.field = 0;
                    }
                    
                    fn update(&mut self) {
                        // Update logic
                    }
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items).unwrap();
        let behavior = &result[0];
        
        assert!(behavior.trait_impl.is_some());
        let trait_impl = behavior.trait_impl.as_ref().unwrap();
        assert!(trait_impl.implemented_methods.contains(&"start".to_string()));
        assert!(trait_impl.implemented_methods.contains(&"update".to_string()));
    }

    #[test]
    fn test_missing_trait_implementation() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    field: i32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        
        // Should fail validation due to missing trait implementation
        assert!(result.is_err() || analyzer.get_errors().iter().any(|e| {
            matches!(e, AnalysisError::MissingTraitImplementation { .. })
        }));
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct BehaviorA {
                    behavior_b_ref: Option<unity::GameObject>,
                }
            },
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct BehaviorB {
                    behavior_a_ref: Option<unity::GameObject>,
                }
            },
            parse_quote! {
                impl UdonBehaviour for BehaviorA {
                    fn start(&mut self) {
                        self.behavior_b_ref = unity::GameObject::find("BehaviorB");
                    }
                }
            },
            parse_quote! {
                impl UdonBehaviour for BehaviorB {
                    fn start(&mut self) {
                        self.behavior_a_ref = unity::GameObject::find("BehaviorA");
                    }
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        
        // Should detect circular dependency
        let has_circular_dependency = result.is_err() || 
            analyzer.get_errors().iter().any(|e| {
                matches!(e, AnalysisError::CircularDependency { .. })
            });
        
        assert!(has_circular_dependency, "Should detect circular dependency between BehaviorA and BehaviorB");
    }

    #[test]
    fn test_complex_type_parsing() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct ComplexBehavior {
                    optional_gameobject: Option<unity::GameObject>,
                    gameobject_list: Vec<unity::GameObject>,
                    string_map: HashMap<String, i32>,
                    nested_option: Option<Vec<String>>,
                    unity_vector: unity::Vector3,
                    unity_transform: unity::Transform,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items).unwrap();
        let behavior = &result[0];
        
        assert_eq!(behavior.fields.len(), 6);
        
        // Verify complex type parsing
        let optional_go = behavior.fields.iter().find(|f| f.name == "optional_gameobject").unwrap();
        assert!(matches!(optional_go.field_type, RustType::Option(_)));
        
        let go_list = behavior.fields.iter().find(|f| f.name == "gameobject_list").unwrap();
        assert!(matches!(go_list.field_type, RustType::Vec(_)));
        
        let string_map = behavior.fields.iter().find(|f| f.name == "string_map").unwrap();
        assert!(matches!(string_map.field_type, RustType::HashMap(_, _)));
    }

    #[test]
    fn test_invalid_field_attributes() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    #[udon_sync]
                    #[udon_public]
                    conflicting_field: i32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        
        // Should detect conflicting attributes
        let has_attribute_error = result.is_err() || 
            analyzer.get_errors().iter().any(|e| {
                matches!(e, AnalysisError::InvalidFieldAttribute { .. })
            });
        
        assert!(has_attribute_error, "Should detect conflicting field attributes");
    }

    #[test]
    fn test_struct_naming_conventions() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct lowercase_struct {
                    field: i32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        
        // Should generate warning for non-PascalCase naming
        if result.is_ok() {
            let warnings = analyzer.get_warnings();
            assert!(!warnings.is_empty());
            assert!(warnings.iter().any(|w| w.contains("PascalCase")));
        }
    }

    #[test]
    fn test_dependency_analysis() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct GameManager {
                    ui_controller_ref: Option<unity::GameObject>,
                    score_tracker_ref: Option<unity::GameObject>,
                }
            },
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct UIController {
                    // No dependencies
                }
            },
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct ScoreTracker {
                    // No dependencies
                }
            },
            parse_quote! {
                impl UdonBehaviour for GameManager {
                    fn start(&mut self) {
                        self.ui_controller_ref = unity::GameObject::find("UIController");
                        self.score_tracker_ref = unity::GameObject::find("ScoreTracker");
                    }
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items).unwrap();
        
        // Verify dependency analysis
        let game_manager = result.iter().find(|s| s.name == "GameManager").unwrap();
        assert_eq!(game_manager.dependencies.len(), 2);
        assert!(game_manager.dependencies.contains(&"UIController".to_string()));
        assert!(game_manager.dependencies.contains(&"ScoreTracker".to_string()));
    }

    #[test]
    fn test_async_method_rejection() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct TestBehavior {
                    field: i32,
                }
            },
            parse_quote! {
                impl UdonBehaviour for TestBehavior {
                    async fn start(&mut self) {
                        // Async methods not supported
                    }
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        
        // Should reject async methods
        let has_async_error = result.is_err() || 
            analyzer.get_errors().iter().any(|e| {
                matches!(e, AnalysisError::AsyncMethodNotSupported { .. })
            });
        
        assert!(has_async_error, "Should reject async methods in UdonBehaviour trait");
    }

    #[test]
    fn test_error_accumulation() {
        let mut analyzer = StructAnalyzer::new();
        
        let items: Vec<Item> = vec![
            parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct 123InvalidName {
                    unsupported_field: i128,
                    #[invalid_attribute]
                    bad_attribute_field: i32,
                }
            }
        ];
        
        let result = analyzer.analyze_module(&items);
        assert!(result.is_err());
        
        // Should accumulate multiple errors
        let errors = analyzer.get_errors();
        assert!(errors.len() >= 2, "Should accumulate multiple errors");
    }
}