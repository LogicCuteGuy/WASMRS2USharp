//! Networking-focused UdonSharp project written in Rust
//! 
//! This demonstrates networking features of the Rust UdonSharp framework.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

/// Network message types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum NetworkMessage {
    PlayerUpdate { player_id: String, position: Vector3 },
    GameEvent { event_type: String, data: String },
    ChatMessage { sender: String, message: String },
}

/// Networking-focused UdonSharp behaviour
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct NetworkingController {
    #[udon_sync]
    pub network_data: String,
    
    #[udon_sync]
    pub player_positions: String, // JSON serialized player positions
    
    #[udon_sync]
    pub game_state: i32,
    
    // Local state
    message_queue: Vec<NetworkMessage>,
    last_sync_time: f32,
    sync_interval: f32,
}

impl UdonBehaviour for NetworkingController {
    fn start(&mut self) {
        self.sync_interval = 0.1; // Sync every 100ms
        self.last_sync_time = 0.0;
        debug_log("Networking controller started");
    }
    
    fn update(&mut self) {
        let current_time = Time::time();
        
        // Process message queue
        self.process_message_queue();
        
        // Sync data at regular intervals
        if current_time - self.last_sync_time > self.sync_interval {
            self.sync_network_data();
            self.last_sync_time = current_time;
        }
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        let message = NetworkMessage::GameEvent {
            event_type: "player_joined".to_string(),
            data: player.get_display_name(),
        };
        self.queue_message(message);
        
        debug_log(&format!("Player joined: {}", player.get_display_name()));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        let message = NetworkMessage::GameEvent {
            event_type: "player_left".to_string(),
            data: player.get_display_name(),
        };
        self.queue_message(message);
        
        debug_log(&format!("Player left: {}", player.get_display_name()));
    }
}

impl NetworkingController {
    pub fn new() -> Self {
        Self {
            network_data: String::new(),
            player_positions: String::new(),
            game_state: 0,
            message_queue: Vec::new(),
            last_sync_time: 0.0,
            sync_interval: 0.1,
        }
    }
    
    pub fn queue_message(&mut self, message: NetworkMessage) {
        self.message_queue.push(message);
    }
    
    fn process_message_queue(&mut self) {
        for message in &self.message_queue {
            match message {
                NetworkMessage::PlayerUpdate { player_id, position } => {
                    debug_log(&format!("Player {} moved to {:?}", player_id, position));
                }
                NetworkMessage::GameEvent { event_type, data } => {
                    debug_log(&format!("Game event {}: {}", event_type, data));
                }
                NetworkMessage::ChatMessage { sender, message } => {
                    debug_log(&format!("{}: {}", sender, message));
                }
            }
        }
        
        // Clear processed messages
        self.message_queue.clear();
    }
    
    fn sync_network_data(&mut self) {
        if Networking::is_master() {
            // Master updates network data
            self.update_player_positions();
            self.game_state += 1;
        }
    }
    
    fn update_player_positions(&mut self) {
        let players = Networking::get_players();
        let mut positions = Vec::new();
        
        for player in players {
            if let Some(player_obj) = player.get_player_object() {
                let position = player_obj.transform().position();
                let message = NetworkMessage::PlayerUpdate {
                    player_id: player.get_display_name(),
                    position,
                };
                positions.push(message);
            }
        }
        
        // Serialize positions to JSON
        if let Ok(json) = serde_json::to_string(&positions) {
            self.player_positions = json;
        }
    }
    
    pub fn send_chat_message(&mut self, message: &str) {
        let local_player = Networking::local_player();
        let chat_message = NetworkMessage::ChatMessage {
            sender: local_player.get_display_name(),
            message: message.to_string(),
        };
        
        self.queue_message(chat_message);
        
        // Broadcast to other players
        self.broadcast_message(&chat_message);
    }
    
    fn broadcast_message(&mut self, message: &NetworkMessage) {
        if let Ok(json) = serde_json::to_string(message) {
            self.network_data = json;
            // Trigger network sync
            self.request_serialization();
        }
    }
    
    #[udon_event]
    pub fn on_network_data_received(&mut self) {
        debug_log("Network data received");
        
        // Parse received network data
        if !self.network_data.is_empty() {
            if let Ok(message) = serde_json::from_str::<NetworkMessage>(&self.network_data) {
                self.queue_message(message);
            }
        }
    }
    
    #[udon_event]
    pub fn on_send_chat(&mut self) {
        // Example event for sending chat messages
        self.send_chat_message("Hello from Rust UdonSharp!");
    }
    
    #[udon_event]
    pub fn on_game_event(&mut self) {
        // Example event for triggering game events
        let message = NetworkMessage::GameEvent {
            event_type: "custom_event".to_string(),
            data: "Event triggered by player".to_string(),
        };
        self.broadcast_message(&message);
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> NetworkingController {
    NetworkingController::new()
}