//! Networking utilities for advanced UdonSharp projects

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

/// Network event types
#[derive(Clone, Debug)]
pub enum NetworkEvent {
    PlayerJoined(VRCPlayerApi),
    PlayerLeft(VRCPlayerApi),
    DataSync(String),
    CustomEvent(String, String),
}

/// Network manager for handling VRChat networking
#[derive(UdonBehaviour)]
pub struct NetworkManager {
    #[udon_sync]
    pub network_data: String,
    
    event_queue: Vec<NetworkEvent>,
}

impl UdonBehaviour for NetworkManager {
    fn start(&mut self) {
        debug_log("Network manager started");
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.event_queue.push(NetworkEvent::PlayerJoined(player));
        self.process_network_events();
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.event_queue.push(NetworkEvent::PlayerLeft(player));
        self.process_network_events();
    }
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            network_data: String::new(),
            event_queue: Vec::new(),
        }
    }
    
    pub fn send_network_event(&mut self, event_type: &str, data: &str) {
        let event = NetworkEvent::CustomEvent(event_type.to_string(), data.to_string());
        self.event_queue.push(event);
        self.process_network_events();
    }
    
    fn process_network_events(&mut self) {
        for event in &self.event_queue {
            match event {
                NetworkEvent::PlayerJoined(player) => {
                    debug_log(&format!("Processing player joined: {}", player.get_display_name()));
                }
                NetworkEvent::PlayerLeft(player) => {
                    debug_log(&format!("Processing player left: {}", player.get_display_name()));
                }
                NetworkEvent::DataSync(data) => {
                    debug_log(&format!("Processing data sync: {}", data));
                }
                NetworkEvent::CustomEvent(event_type, data) => {
                    debug_log(&format!("Processing custom event {}: {}", event_type, data));
                }
            }
        }
        
        // Clear processed events
        self.event_queue.clear();
    }
    
    #[udon_event]
    pub fn on_network_data_received(&mut self) {
        debug_log("Network data received");
        let event = NetworkEvent::DataSync(self.network_data.clone());
        self.event_queue.push(event);
        self.process_network_events();
    }
}