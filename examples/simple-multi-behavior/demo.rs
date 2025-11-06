#!/usr/bin/env rust-script

//! Demo script for the simple multi-behavior example
//! 
//! This script demonstrates how the Counter and Display behaviors work together
//! in the Standard Multi-Behavior Pattern.

use std::collections::HashMap;

// Mock types for demonstration
#[derive(Debug, Clone)]
struct GameObject {
    name: String,
    components: HashMap<String, String>,
}

impl GameObject {
    fn find(name: &str) -> Option<Self> {
        Some(GameObject {
            name: name.to_string(),
            components: HashMap::new(),
        })
    }
    
    fn send_custom_event(&self, event_name: &str) {
        println!("🔔 {} received event: {}", self.name, event_name);
    }
}

#[derive(Debug, Clone)]
struct Text {
    content: String,
}

impl Text {
    fn set_text(&mut self, text: &str) {
        self.content = text.to_string();
        println!("📝 Text updated: {}", text);
    }
}

// Simple multi-behavior structs (simplified for demo)
#[derive(Debug)]
struct Counter {
    current_count: i32,
    max_count: i32,
    display: Option<GameObject>,
}

#[derive(Debug)]
struct Display {
    count_text: Option<Text>,
    status_text: Option<Text>,
    counter: Option<GameObject>,
}

impl Counter {
    fn new() -> Self {
        Self {
            current_count: 0,
            max_count: 100,
            display: None,
        }
    }
    
    fn start(&mut self) {
        println!("🚀 Counter starting...");
        self.current_count = 0;
        self.max_count = 100;
        self.display = GameObject::find("Display");
        self.update_display();
        println!("✅ Counter initialized");
    }
    
    fn increment(&mut self) {
        if self.current_count < self.max_count {
            self.current_count += 1;
            self.update_display();
            println!("⬆️  Counter incremented to {}", self.current_count);
        } else {
            println!("⚠️  Counter is at maximum value");
        }
    }
    
    fn decrement(&mut self) {
        if self.current_count > 0 {
            self.current_count -= 1;
            self.update_display();
            println!("⬇️  Counter decremented to {}", self.current_count);
        } else {
            println!("⚠️  Counter is at minimum value");
        }
    }
    
    fn reset(&mut self) {
        self.current_count = 0;
        self.update_display();
        println!("🔄 Counter reset to 0");
    }
    
    fn get_count(&self) -> i32 {
        self.current_count
    }
    
    fn update_display(&self) {
        if let Some(display_obj) = &self.display {
            display_obj.send_custom_event("UpdateDisplay");
        }
    }
}

impl Display {
    fn new() -> Self {
        Self {
            count_text: Some(Text { content: String::new() }),
            status_text: Some(Text { content: String::new() }),
            counter: None,
        }
    }
    
    fn start(&mut self) {
        println!("🚀 Display starting...");
        self.counter = GameObject::find("Counter");
        println!("✅ Display initialized");
    }
    
    fn update_display(&mut self, counter: &Counter) {
        let count = counter.get_count();
        
        // Update count text
        if let Some(mut text) = self.count_text.clone() {
            text.set_text(&format!("Count: {}", count));
            self.count_text = Some(text);
        }
        
        // Update status text with additional info
        if let Some(mut status) = self.status_text.clone() {
            let status_message = get_count_status(count, counter.max_count);
            status.set_text(&status_message);
            self.status_text = Some(status);
        }
    }
    
    fn handle_increment_button(&self) {
        println!("🔘 Increment button clicked");
    }
    
    fn handle_decrement_button(&self) {
        println!("🔘 Decrement button clicked");
    }
    
    fn handle_reset_button(&self) {
        println!("🔘 Reset button clicked");
    }
}

// Shared utility functions (would be in SharedRuntime.cs)
fn get_count_status(count: i32, max_count: i32) -> String {
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

fn format_number(number: i32) -> String {
    if number >= 1000 {
        format!("{},000+", number / 1000)
    } else {
        number.to_string()
    }
}

fn main() {
    println!("🎯 Simple Multi-Behavior Pattern Demo");
    println!("=====================================\n");
    
    // Create behaviors
    let mut counter = Counter::new();
    let mut display = Display::new();
    
    // Initialize behaviors (like Unity Start())
    counter.start();
    display.start();
    
    println!("\n📊 Demonstrating Counter Operations:");
    println!("-----------------------------------");
    
    // Simulate user interactions
    display.update_display(&counter);
    
    println!("\n🔢 Incrementing counter...");
    for i in 1..=5 {
        counter.increment();
        display.update_display(&counter);
        if i == 3 {
            println!("   Status: {}", get_count_status(counter.get_count(), counter.max_count));
        }
    }
    
    println!("\n🔢 Decrementing counter...");
    for _ in 1..=2 {
        counter.decrement();
        display.update_display(&counter);
    }
    
    println!("\n🔄 Resetting counter...");
    counter.reset();
    display.update_display(&counter);
    
    println!("\n🧪 Testing boundary conditions...");
    counter.max_count = 3;
    
    // Test max boundary
    for _ in 1..=5 {
        counter.increment();
        display.update_display(&counter);
    }
    
    // Test min boundary
    for _ in 1..=5 {
        counter.decrement();
        display.update_display(&counter);
    }
    
    println!("\n✨ Shared Functions Demo:");
    println!("------------------------");
    println!("Format number 500: {}", format_number(500));
    println!("Format number 1500: {}", format_number(1500));
    
    println!("\n🎉 Demo completed!");
    println!("\nThis demonstrates how the Standard Multi-Behavior Pattern works:");
    println!("• Counter and Display are separate behaviors");
    println!("• They communicate through events and GameObject references");
    println!("• Shared functions (get_count_status, format_number) would be in SharedRuntime.cs");
    println!("• Each behavior has its own responsibilities and state");
    println!("• The pattern scales well for complex VRChat world logic");
}