//! Procedural macros for UdonSharp code generation
//! 
//! This module contains macros for generating UdonSharp-compatible code.

/// Macro to mark a struct as a UdonBehaviour
/// 
/// This macro will generate the necessary boilerplate code for UdonSharp compatibility.
/// 
/// # Example
/// ```rust
/// use udonsharp_core::udon_behaviour;
/// 
/// #[udon_behaviour]
/// pub struct MyBehaviour {
///     #[udon_public]
///     pub my_field: i32,
/// }
/// ```
#[macro_export]
macro_rules! udon_behaviour {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name: $field_type,
            )*
        }
        
        // Implement UdonBehaviour trait with default implementations
        impl $crate::traits::UdonBehaviour for $name {}
        
        // Add metadata for code generation
        impl $name {
            pub const UDON_TYPE_NAME: &'static str = stringify!($name);
            
            pub fn get_udon_fields() -> Vec<(&'static str, &'static str)> {
                vec![
                    $(
                        (stringify!($field_name), stringify!($field_type)),
                    )*
                ]
            }
        }
    };
}

/// Macro to create a UdonSharp event handler method
/// 
/// # Example
/// ```rust
/// use udonsharp_core::udon_event;
/// 
/// impl MyBehaviour {
///     #[udon_event]
///     pub fn my_custom_event(&mut self) {
///         // Event handler code
///     }
/// }
/// ```
#[macro_export]
macro_rules! udon_event {
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident(&mut self $(, $param:ident: $param_type:ty)*) $body:block
    ) => {
        $(#[$meta])*
        $vis fn $name(&mut self $(, $param: $param_type)*) $body
        
        // Add event metadata
        paste::paste! {
            pub const [<$name:upper _EVENT_NAME>]: &'static str = stringify!($name);
        }
    };
}

/// Macro to create a UdonSharp network event handler method
/// 
/// # Example
/// ```rust
/// use udonsharp_core::udon_network_event;
/// 
/// impl MyBehaviour {
///     #[udon_network_event(target = "All")]
///     pub fn my_network_event(&mut self) {
///         // Network event handler code
///     }
/// }
/// ```
#[macro_export]
macro_rules! udon_network_event {
    (
        target = $target:literal,
        $(#[$meta:meta])*
        $vis:vis fn $name:ident(&mut self $(, $param:ident: $param_type:ty)*) $body:block
    ) => {
        $(#[$meta])*
        $vis fn $name(&mut self $(, $param: $param_type)*) $body
        
        // Add network event metadata
        paste::paste! {
            pub const [<$name:upper _NETWORK_EVENT_NAME>]: &'static str = stringify!($name);
            pub const [<$name:upper _NETWORK_EVENT_TARGET>]: &'static str = $target;
        }
    };
}

/// Macro to create UdonSharp-compatible logging
/// 
/// # Example
/// ```rust
/// use udonsharp_core::udon_log;
/// 
/// udon_log!("Hello, world!");
/// udon_log!("Value: {}", my_value);
/// ```
#[macro_export]
macro_rules! udon_log {
    ($($arg:tt)*) => {
        // In actual implementation, this would generate Debug.Log calls
        log::info!($($arg)*);
    };
}

/// Macro to create UdonSharp-compatible error logging
/// 
/// # Example
/// ```rust
/// use udonsharp_core::udon_error;
/// 
/// udon_error!("Something went wrong!");
/// ```
#[macro_export]
macro_rules! udon_error {
    ($($arg:tt)*) => {
        // In actual implementation, this would generate Debug.LogError calls
        log::error!($($arg)*);
    };
}

/// Macro to create UdonSharp-compatible warning logging
/// 
/// # Example
/// ```rust
/// use udonsharp_core::udon_warn;
/// 
/// udon_warn!("This is a warning!");
/// ```
#[macro_export]
macro_rules! udon_warn {
    ($($arg:tt)*) => {
        // In actual implementation, this would generate Debug.LogWarning calls
        log::warn!($($arg)*);
    };
}

/// Macro to send a custom event to a UdonBehaviour
/// 
/// # Example
/// ```rust
/// use udonsharp_core::send_custom_event;
/// 
/// send_custom_event!(target_object, "MyEvent");
/// ```
#[macro_export]
macro_rules! send_custom_event {
    ($target:expr, $event:expr) => {
        $crate::types::UdonSharpUtility::send_custom_event($target, $event);
    };
}

/// Macro to send a custom network event
/// 
/// # Example
/// ```rust
/// use udonsharp_core::send_custom_network_event;
/// 
/// send_custom_network_event!(NetworkEventTarget::All, "MyNetworkEvent");
/// ```
#[macro_export]
macro_rules! send_custom_network_event {
    ($target:expr, $event:expr) => {
        $crate::types::UdonSharpUtility::send_custom_network_event($target, $event);
    };
}