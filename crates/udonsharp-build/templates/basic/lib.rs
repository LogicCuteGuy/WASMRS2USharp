//! Basic UdonSharp project written in Rust
//! 
//! This demonstrates the basic structure of a UdonSharp project using Rust.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

/// Main UdonSharp behaviour for this project
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct MyUdonBehaviour {
    #[udon_public]
    pub message: String,
    
    #[udon_sync]
    pub counter: i32,
    
    initialized: bool,
}

impl UdonBehaviour for MyUdonBehaviour {
    fn start(&mut self) {
        self.initialized = true;
        self.message = "Hello from Rust UdonSharp!".to_string();
        self.counter = 0;
        
        // Log a message to the VRChat console
        debug_log(&format!("UdonSharp behaviour started: {}", self.message));
    }
    
    fn update(&mut self) {
        // Update logic here
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.counter += 1;
        debug_log(&format!("Player joined: {}. Total players: {}", 
                          player.get_display_name(), self.counter));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.counter -= 1;
        debug_log(&format!("Player left: {}. Total players: {}", 
                          player.get_display_name(), self.counter));
    }
}

impl MyUdonBehaviour {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            counter: 0,
            initialized: false,
        }
    }
    
    #[udon_event]
    pub fn on_interact(&mut self) {
        if self.initialized {
            debug_log("Interact event triggered!");
            self.counter += 1;
        }
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> MyUdonBehaviour {
    MyUdonBehaviour::new()
}