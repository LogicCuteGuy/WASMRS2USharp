//! Simple Multi-Behavior Pattern Example
//! 
//! This is a beginner-friendly example showing the basics of the Standard Multi-Behavior Pattern.
//! It demonstrates two simple behaviors that work together: a Counter and a Display.

use udonsharp_core::prelude::*;
use udonsharp_macros::{UdonBehaviour, udon_public, udon_event};

#[cfg(test)]
mod test;

/// A simple counter that tracks a number and can increment/decrement it
#[derive(UdonBehaviour)]
pub struct Counter {
    /// The current count value (visible in Unity Inspector)
    #[udon_public]
    current_count: i32,
    
    /// Maximum allowed count
    #[udon_public]
    max_count: i32,
    
    /// Reference to the display behavior
    display: Option<GameObject>,
}

impl Counter {
    fn start(&mut self) {
        // Initialize the counter
        self.current_count = 0;
        self.max_count = 100;
        
        // Find the display behavior
        self.display = GameObject::find("Display");
        
        // Update the display initially
        self.update_display();
        
        log_message("Counter initialized");
    }
}

impl Counter {
    /// Increment the counter by 1
    pub fn increment(&mut self) {
        if self.current_count < self.max_count {
            self.current_count += 1;
            self.update_display();
            log_message(&format!("Counter incremented to {}", self.current_count));
        } else {
            log_message("Counter is at maximum value");
        }
    }
    
    /// Decrement the counter by 1
    pub fn decrement(&mut self) {
        if self.current_count > 0 {
            self.current_count -= 1;
            self.update_display();
            log_message(&format!("Counter decremented to {}", self.current_count));
        } else {
            log_message("Counter is at minimum value");
        }
    }
    
    /// Reset the counter to 0
    #[udon_event("ResetCounter")]
    pub fn reset(&mut self) {
        self.current_count = 0;
        self.update_display();
        log_message("Counter reset to 0");
    }
    
    /// Get the current count value
    pub fn get_count(&self) -> i32 {
        self.current_count
    }
    
    /// Update the display with current count
    fn update_display(&self) {
        if let Some(display_obj) = &self.display {
            display_obj.send_custom_event("UpdateDisplay");
        }
    }
}

/// A display that shows information from the counter
#[derive(UdonBehaviour)]
pub struct Display {
    /// Text component to show the count
    #[udon_public]
    count_text: Option<Text>,
    
    /// Text component to show status messages
    #[udon_public]
    status_text: Option<Text>,
    
    /// Reference to the counter behavior
    counter: Option<GameObject>,
}

impl Display {
    fn start(&mut self) {
        // Find the counter behavior
        self.counter = GameObject::find("Counter");
        
        // Initial display update
        self.update_display();
        
        log_message("Display initialized");
    }
}

impl Display {
    /// Update the display with current counter information
    #[udon_event("UpdateDisplay")]
    pub fn update_display(&mut self) {
        if let Some(counter_obj) = &self.counter {
            if let Some(counter_component) = counter_obj.get_component::<Counter>() {
                let count = counter_component.get_count();
                
                // Update count text
                if let Some(mut text) = self.count_text.clone() {
                    text.set_text(&format!("Count: {}", count));
                }
                
                // Update status text with additional info
                if let Some(mut status) = self.status_text.clone() {
                    let status_message = get_count_status(count, counter_component.max_count);
                    status.set_text(&status_message);
                }
            }
        }
    }
    
    /// Handle button clicks to increment counter
    #[udon_event("OnIncrementButton")]
    pub fn handle_increment_button(&self) {
        if let Some(counter_obj) = &self.counter {
            if let Some(mut counter_component) = counter_obj.get_component::<Counter>() {
                counter_component.increment();
            }
        }
    }
    
    /// Handle button clicks to decrement counter
    #[udon_event("OnDecrementButton")]
    pub fn handle_decrement_button(&self) {
        if let Some(counter_obj) = &self.counter {
            if let Some(mut counter_component) = counter_obj.get_component::<Counter>() {
                counter_component.decrement();
            }
        }
    }
    
    /// Handle button clicks to reset counter
    #[udon_event("OnResetButton")]
    pub fn handle_reset_button(&self) {
        if let Some(counter_obj) = &self.counter {
            counter_obj.send_custom_event("ResetCounter");
        }
    }
}

// Shared utility functions (automatically moved to SharedRuntime)

/// Get a status message based on the current count
pub fn get_count_status(count: i32, max_count: i32) -> String {
    let percentage = (count as f32 / max_count as f32) * 100.0;
    
    if count == 0 {
        "Ready to start counting!".to_string()
    } else if count == max_count {
        "Maximum count reached!".to_string()
    } else if percentage >= 75.0 {
        "Getting close to maximum!".to_string()
    } else if percentage >= 50.0 {
        "Halfway there!".to_string()
    } else if percentage >= 25.0 {
        "Making progress!".to_string()
    } else {
        "Just getting started!".to_string()
    }
}

/// Format a number with appropriate styling
pub fn format_number(number: i32) -> String {
    if number >= 1000 {
        format!("{},000+", number / 1000)
    } else {
        number.to_string()
    }
}

/// Log a message (shared logging utility)
pub fn log_message(message: &str) {
    // In real implementation, this would use UdonSharp's Debug.Log
    println!("[LOG] {}", message);
}