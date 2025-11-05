use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Attribute, Meta, ItemFn, ReturnType, Type};

/// Derive macro for UdonBehaviour trait
/// 
/// This macro generates the necessary boilerplate for UdonSharp compatibility
/// and processes UdonSharp-specific attributes.
#[proc_macro_derive(UdonBehaviour, attributes(udon_sync_mode, udon_public, udon_sync, udon_event, udon_header, udon_tooltip, udon_range, udon_text_area))]
pub fn derive_udon_behaviour(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    
    // Extract sync mode from attributes
    let sync_mode = extract_sync_mode(&input.attrs);
    
    // Process fields for UdonSharp attributes
    let field_metadata = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            process_fields(&fields.named)
        } else {
            quote! {}
        }
    } else {
        quote! {}
    };
    
    let expanded = quote! {
        impl udonsharp_core::traits::UdonBehaviour for #name {}
        
        impl #name {
            pub const UDON_TYPE_NAME: &'static str = stringify!(#name);
            pub const UDON_SYNC_MODE: udonsharp_core::types::UdonSyncMode = #sync_mode;
            
            #field_metadata
        }
    };
    
    TokenStream::from(expanded)
}

/// Attribute macro for marking fields as UdonSharp public
#[proc_macro_attribute]
pub fn udon_public(_args: TokenStream, input: TokenStream) -> TokenStream {
    // For now, just pass through the input
    // The actual processing happens in the derive macro
    input
}

/// Attribute macro for marking fields as UdonSharp synchronized
#[proc_macro_attribute]
pub fn udon_sync(_args: TokenStream, input: TokenStream) -> TokenStream {
    // For now, just pass through the input
    // The actual processing happens in the derive macro
    input
}

/// Attribute macro for marking methods as UdonSharp events
#[proc_macro_attribute]
pub fn udon_event(_args: TokenStream, input: TokenStream) -> TokenStream {
    // For now, just pass through the input
    // The actual processing happens in the derive macro
    input
}

/// Attribute macro for setting UdonSharp sync mode
#[proc_macro_attribute]
pub fn udon_sync_mode(_args: TokenStream, input: TokenStream) -> TokenStream {
    // For now, just pass through the input
    // The actual processing happens in the derive macro
    input
}

/// Attribute macro for marking functions as UdonSharp tests
/// 
/// This macro transforms regular Rust test functions into UdonSharp-compatible
/// test functions that can run in mock VRChat/Unity environments.
#[proc_macro_attribute]
pub fn udon_test(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as syn::ItemFn);
    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    
    // Generate test wrapper that includes UdonSharp test environment setup
    let expanded = quote! {
        #(#fn_attrs)*
        #[test]
        #fn_vis fn #fn_name() {
            // Initialize UdonSharp test environment
            let _test_env = udonsharp_core::testing::UdonTestEnvironment::new();
            
            // Set up mock VRChat and Unity systems
            udonsharp_core::testing::setup_mock_environment();
            
            // Run the actual test
            let test_result = std::panic::catch_unwind(|| {
                #fn_block
            });
            
            // Clean up test environment
            udonsharp_core::testing::cleanup_mock_environment();
            
            // Handle test result
            match test_result {
                Ok(_) => {
                    println!("UdonSharp test '{}' passed", stringify!(#fn_name));
                }
                Err(e) => {
                    println!("UdonSharp test '{}' failed", stringify!(#fn_name));
                    std::panic::resume_unwind(e);
                }
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Attribute macro for marking functions as UdonBehaviour entry points
/// 
/// This macro marks functions that should become separate UdonBehaviour classes
/// when the WASM is split during compilation.
/// 
/// # Usage
/// 
/// ```rust
/// use udonsharp_macros::udon_behaviour;
/// 
/// #[udon_behaviour]
/// pub fn player_manager() {
///     // This becomes PlayerManager.cs
/// }
/// 
/// #[udon_behaviour(name = "UIController")]
/// pub fn ui_controller() {
///     // This becomes UIController.cs
/// }
/// 
/// #[udon_behaviour(name = "GameLogic", events = "Update,OnTriggerEnter")]
/// pub fn game_logic() {
///     // This becomes GameLogic.cs with Update and OnTriggerEnter methods
/// }
/// 
/// #[udon_behaviour(name = "NetworkManager", dependencies = "PlayerManager")]
/// pub fn network_manager() {
///     // This becomes NetworkManager.cs with dependency on PlayerManager
/// }
/// ```
#[proc_macro_attribute]
pub fn udon_behaviour(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as syn::ItemFn);
    
    // Parse attribute arguments - simplified for syn 2.0
    let behaviour_config = match parse_udon_behaviour_args_simple(&args.to_string()) {
        Ok(config) => config,
        Err(err) => {
            return syn::Error::new_spanned(&input_fn, err)
                .to_compile_error()
                .into();
        }
    };
    
    // Validate the function signature
    if let Err(err) = validate_udon_behaviour_function(&input_fn) {
        return syn::Error::new_spanned(&input_fn, err)
            .to_compile_error()
            .into();
    }
    
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    
    // Generate metadata for the behaviour splitter
    let behaviour_name = behaviour_config.name.unwrap_or_else(|| {
        // Convert snake_case function name to PascalCase
        snake_to_pascal_case(&fn_name.to_string())
    });
    
    let events_str = behaviour_config.events.join(",");
    let deps_str = behaviour_config.dependencies.join(",");
    let auto_sync = behaviour_config.auto_sync;
    
    // Generate the function with metadata
    let expanded = quote! {
        #(#fn_attrs)*
        #[doc = concat!("UdonBehaviour entry point: ", #behaviour_name)]
        #[allow(non_snake_case)]
        #fn_vis fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }
        
        // Generate metadata that can be extracted during compilation
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        const _: () = {
            // This metadata will be extracted by the WASM analyzer
            #[export_name = concat!("__udon_behaviour_", stringify!(#fn_name))]
            static UDON_BEHAVIOUR_METADATA: &str = concat!(
                "name:", #behaviour_name, ";",
                "events:", #events_str, ";",
                "dependencies:", #deps_str, ";",
                "auto_sync:", #auto_sync, ";"
            );
        };
    };
    
    TokenStream::from(expanded)
}

fn extract_sync_mode(attrs: &[Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("udon_sync_mode") {
            if let Meta::List(meta_list) = &attr.meta {
                let tokens_str = meta_list.tokens.to_string();
                if tokens_str.contains("Manual") {
                    return quote! { udonsharp_core::types::UdonSyncMode::Manual };
                } else if tokens_str.contains("Continuous") {
                    return quote! { udonsharp_core::types::UdonSyncMode::Continuous };
                }
            }
        }
    }
    quote! { udonsharp_core::types::UdonSyncMode::None }
}

fn process_fields(fields: &syn::punctuated::Punctuated<Field, syn::Token![,]>) -> proc_macro2::TokenStream {
    let mut field_info = Vec::new();
    
    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        
        let mut is_public = false;
        let mut is_sync = false;
        let mut sync_mode = quote! { udonsharp_core::types::UdonSyncMode::None };
        let mut header_text = None;
        let mut tooltip_text = None;
        let _range_info: Option<(f32, f32)> = None;
        let _text_area_info: Option<(u32, u32)> = None;
        
        // Process field attributes
        for attr in &field.attrs {
            if attr.path().is_ident("udon_public") {
                is_public = true;
            } else if attr.path().is_ident("udon_sync") {
                is_sync = true;
                // Extract sync mode if specified - simplified for syn 2.0
                if let Meta::List(meta_list) = &attr.meta {
                    // Simple string parsing for now
                    let tokens_str = meta_list.tokens.to_string();
                    if tokens_str.contains("Manual") {
                        sync_mode = quote! { udonsharp_core::types::UdonSyncMode::Manual };
                    } else if tokens_str.contains("Continuous") {
                        sync_mode = quote! { udonsharp_core::types::UdonSyncMode::Continuous };
                    }
                }
            } else if attr.path().is_ident("udon_header") {
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens_str = meta_list.tokens.to_string();
                    // Extract string literal from tokens
                    if let Some(start) = tokens_str.find('"') {
                        if let Some(end) = tokens_str.rfind('"') {
                            if start < end {
                                header_text = Some(tokens_str[start + 1..end].to_string());
                            }
                        }
                    }
                }
            } else if attr.path().is_ident("udon_tooltip") {
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens_str = meta_list.tokens.to_string();
                    // Extract string literal from tokens
                    if let Some(start) = tokens_str.find('"') {
                        if let Some(end) = tokens_str.rfind('"') {
                            if start < end {
                                tooltip_text = Some(tokens_str[start + 1..end].to_string());
                            }
                        }
                    }
                }
            }
        }
        
        let header_text_opt = match header_text {
            Some(text) => quote! { Some(#text.to_string()) },
            None => quote! { None },
        };
        let tooltip_text_opt = match tooltip_text {
            Some(text) => quote! { Some(#text.to_string()) },
            None => quote! { None },
        };
        
        field_info.push(quote! {
            udonsharp_core::types::UdonFieldInfo {
                name: stringify!(#field_name),
                type_name: stringify!(#field_type),
                is_public: #is_public,
                is_sync: #is_sync,
                sync_mode: #sync_mode,
                header_text: #header_text_opt,
                tooltip_text: #tooltip_text_opt,
            }
        });
    }
    
    quote! {
        pub fn get_udon_field_info() -> Vec<udonsharp_core::types::UdonFieldInfo> {
            vec![
                #(#field_info),*
            ]
        }
    }
}

/// Configuration for udon_behaviour attribute
#[derive(Debug, Default)]
struct UdonBehaviourConfig {
    name: Option<String>,
    events: Vec<String>,
    dependencies: Vec<String>,
    auto_sync: bool,
}

/// Parse arguments for udon_behaviour attribute (simplified for syn 2.0)
fn parse_udon_behaviour_args_simple(args_str: &str) -> Result<UdonBehaviourConfig, String> {
    let mut config = UdonBehaviourConfig::default();
    
    if args_str.trim().is_empty() {
        // No arguments, use defaults
        config.events.push("Start".to_string());
        return Ok(config);
    }
    
    // Very simple parsing - look for specific patterns
    let args_clean = args_str.replace(" ", "");
    
    // Extract name
    if let Some(start) = args_clean.find("name=\"") {
        let start = start + 6; // length of "name=\""
        if let Some(end) = args_clean[start..].find("\"") {
            config.name = Some(args_clean[start..start + end].to_string());
        }
    }
    
    // Extract events
    if let Some(start) = args_clean.find("events=\"") {
        let start = start + 8; // length of "events=\""
        if let Some(end) = args_clean[start..].find("\"") {
            let events_str = &args_clean[start..start + end];
            config.events = events_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    
    // Extract dependencies
    if let Some(start) = args_clean.find("dependencies=\"") {
        let start = start + 14; // length of "dependencies=\""
        if let Some(end) = args_clean[start..].find("\"") {
            let deps_str = &args_clean[start..start + end];
            config.dependencies = deps_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }
    
    // Check for auto_sync
    if args_clean.contains("auto_sync=true") || args_clean.contains("auto_sync") {
        config.auto_sync = true;
    }
    
    // Set default events if none specified
    if config.events.is_empty() {
        config.events.push("Start".to_string());
    }
    
    Ok(config)
}

/// Validate that a function is suitable for udon_behaviour attribute
fn validate_udon_behaviour_function(func: &ItemFn) -> Result<(), String> {
    // Check function visibility
    if !matches!(func.vis, syn::Visibility::Public(_)) {
        return Err("udon_behaviour functions must be public".to_string());
    }
    
    // Check function parameters - should be empty or only &mut self
    if !func.sig.inputs.is_empty() {
        let first_param = func.sig.inputs.first().unwrap();
        match first_param {
            syn::FnArg::Receiver(receiver) => {
                if !receiver.mutability.is_some() {
                    return Err("udon_behaviour functions with self parameter must take &mut self".to_string());
                }
            }
            syn::FnArg::Typed(_) => {
                return Err("udon_behaviour functions should not take parameters other than &mut self".to_string());
            }
        }
        
        if func.sig.inputs.len() > 1 {
            return Err("udon_behaviour functions should only take &mut self as parameter".to_string());
        }
    }
    
    // Check return type - should be () or Result<(), E>
    match &func.sig.output {
        ReturnType::Default => {}, // () is fine
        ReturnType::Type(_, ty) => {
            // Allow Result<(), E> types
            if let Type::Path(type_path) = ty.as_ref() {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident != "Result" {
                        return Err("udon_behaviour functions should return () or Result<(), E>".to_string());
                    }
                }
            } else {
                return Err("udon_behaviour functions should return () or Result<(), E>".to_string());
            }
        }
    }
    
    Ok(())
}

/// Convert snake_case to PascalCase
fn snake_to_pascal_case(snake_str: &str) -> String {
    snake_str
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}