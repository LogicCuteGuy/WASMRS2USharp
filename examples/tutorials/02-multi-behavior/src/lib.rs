//! Multi-Behavior Tutorial
//! 
//! This tutorial demonstrates the basics of multi-behavior compilation.
//! It creates two simple behaviors that communicate with each other:
//! - Counter: Increments a value every second
//! - Display: Shows the current counter value

use udonsharp_core::prelude::*;

// ============================================================================
// COUNTER BEHAVIOR
// ============================================================================

/// Counter behavior increments a value every second
#[udon_behaviour(name = "Counter", events = ["Start", "Update"])]
pub fn counter_start() {
    log_info("Counter behavior started!");
    set_counter_value(0);
}

/// Update method called every frame
#[udon_behaviour_event(behavior = "Counter")]
pub fn counter_update() {
    // Increment counter every second
    static mut TIMER: f32 = 0.0;
    
    unsafe {
        TIMER += get_delta_time();
        if TIMER >= 1.0 {
            TIMER = 0.0;
            let current = get_counter_value();
            set_counter_value(current + 1);
            
            // Notify the display behavior
            notify_display_updated();
        }
    }
}

// ============================================================================
// DISPLAY BEHAVIOR
// ============================================================================

/// Display behavior shows the current counter value
#[udon_behaviour(name = "Display", events = ["Start"])]
pub fn display_start() {
    log_info("Display behavior started!");
    update_display_text();
}

/// Method called when counter is updated
#[udon_behaviour_method(behavior = "Display")]
pub fn on_counter_updated() {
    update_display_text();
    log_info(&format!("Display updated: {}", get_counter_value()));
}

// ============================================================================
// SHARED FUNCTIONALITY
// ============================================================================
// These functions will be moved to SharedRuntime by the compiler

/// Shared counter value
static mut COUNTER_VALUE: i32 = 0;

/// Get the current counter value
pub fn get_counter_value() -> i32 {
    unsafe { COUNTER_VALUE }
}

/// Set the counter value
pub fn set_counter_value(value: i32) {
    unsafe { COUNTER_VALUE = value; }
}

/// Logging utility
pub fn log_info(message: &str) {
    println!("[INFO] {}", message);
}

/// Notify display behavior that counter was updated
pub fn notify_display_updated() {
    send_custom_event("Display", "OnCounterUpdated");
}

/// Update the display text (placeholder implementation)
fn update_display_text() {
    log_info(&format!("Counter value: {}", get_counter_value()));
}

// ============================================================================
// PLACEHOLDER IMPLEMENTATIONS
// ============================================================================
// These would be replaced by actual UdonSharp bindings

/// Send a custom event to another behavior
pub fn send_custom_event(behavior: &str, event: &str) {
    log_info(&format!("SendCustomEvent: {} -> {}", behavior, event));
}

/// Get delta time (time since last frame)
pub fn get_delta_time() -> f32 {
    0.016 // Placeholder: 60 FPS
}