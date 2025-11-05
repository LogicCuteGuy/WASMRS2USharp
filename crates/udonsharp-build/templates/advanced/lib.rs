//! Advanced UdonSharp project written in Rust
//! 
//! This demonstrates advanced features of the Rust UdonSharp framework.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

pub mod networking;
pub mod ui;

/// Configuration data structure
#[derive(Serialize, Deserialize, Clone)]
pub struct WorldConfig {
    pub world_name: String,
    pub max_players: i32,
    pub enable_networking: bool,
}

/// Advanced UdonSharp behaviour with networking and Unity integration
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct AdvancedWorldController {
    #[udon_public]
    pub config: String, // JSON serialized WorldConfig
    
    #[udon_sync]
    pub world_state: i32,
    
    #[udon_sync]
    pub active_players: i32,
    
    // Internal state
    initialized: bool,
    config_data: Option<WorldConfig>,
}

impl UdonBehaviour for AdvancedWorldController {
    fn start(&mut self) {
        self.initialize_world();
        self.initialized = true;
        debug_log("Advanced world controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update world state
        self.update_world_state();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.active_players += 1;
        debug_log(&format!("Player {} joined. Active players: {}", 
                          player.get_display_name(), self.active_players));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.active_players -= 1;
        debug_log(&format!("Player {} left. Active players: {}", 
                          player.get_display_name(), self.active_players));
    }
}

impl AdvancedWorldController {
    pub fn new() -> Self {
        Self {
            config: String::new(),
            world_state: 0,
            active_players: 0,
            initialized: false,
            config_data: None,
        }
    }
    
    fn initialize_world(&mut self) {
        // Parse configuration
        if !self.config.is_empty() {
            match serde_json::from_str::<WorldConfig>(&self.config) {
                Ok(config) => {
                    self.config_data = Some(config);
                    debug_log("World configuration loaded successfully");
                }
                Err(e) => {
                    debug_log(&format!("Failed to parse world config: {}", e));
                    // Use default configuration
                    self.config_data = Some(WorldConfig {
                        world_name: "Advanced Rust UdonSharp World".to_string(),
                        max_players: 20,
                        enable_networking: true,
                    });
                }
            }
        }
    }
    
    fn update_world_state(&mut self) {
        // Update world state based on player count and time
        let time = Time::time();
        self.world_state = (time as i32) % 1000;
    }
    
    #[udon_event]
    pub fn on_world_reset(&mut self) {
        debug_log("World reset triggered");
        self.world_state = 0;
    }
    
    #[udon_event]
    pub fn on_config_update(&mut self) {
        debug_log("Configuration update triggered");
        self.initialize_world();
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> AdvancedWorldController {
    AdvancedWorldController::new()
}