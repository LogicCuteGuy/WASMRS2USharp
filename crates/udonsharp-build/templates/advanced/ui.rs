//! UI utilities for advanced UdonSharp projects

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

/// UI manager for handling Unity UI components
#[derive(UdonBehaviour)]
pub struct UIManager {
    #[udon_public]
    pub main_canvas: Option<GameObject>,
    
    #[udon_public]
    pub player_count_text: Option<GameObject>,
    
    #[udon_public]
    pub status_text: Option<GameObject>,
    
    initialized: bool,
}

impl UdonBehaviour for UIManager {
    fn start(&mut self) {
        self.initialize_ui();
        self.initialized = true;
        debug_log("UI manager initialized");
    }
    
    fn update(&mut self) {
        if self.initialized {
            self.update_ui();
        }
    }
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            main_canvas: None,
            player_count_text: None,
            status_text: None,
            initialized: false,
        }
    }
    
    fn initialize_ui(&mut self) {
        // Find UI components if not already assigned
        if self.main_canvas.is_none() {
            self.main_canvas = GameObject::find("Main Canvas");
        }
        
        if self.player_count_text.is_none() {
            self.player_count_text = GameObject::find("PlayerCountText");
        }
        
        if self.status_text.is_none() {
            self.status_text = GameObject::find("StatusText");
        }
        
        debug_log("UI components initialized");
    }
    
    fn update_ui(&mut self) {
        // Update player count display
        if let Some(ref text_obj) = self.player_count_text {
            let player_count = Networking::get_players().len();
            self.set_text(text_obj, &format!("Players: {}", player_count));
        }
        
        // Update status display
        if let Some(ref text_obj) = self.status_text {
            let status = if Networking::is_master() {
                "Master"
            } else {
                "Client"
            };
            self.set_text(text_obj, &format!("Status: {}", status));
        }
    }
    
    fn set_text(&self, text_obj: &GameObject, text: &str) {
        if let Some(text_component) = text_obj.get_component::<UnityEngine::UI::Text>() {
            text_component.set_text(text);
        }
    }
    
    pub fn show_message(&mut self, message: &str) {
        if let Some(ref text_obj) = self.status_text {
            self.set_text(text_obj, message);
        }
        debug_log(&format!("UI message: {}", message));
    }
    
    #[udon_event]
    pub fn on_ui_button_clicked(&mut self) {
        debug_log("UI button clicked");
        self.show_message("Button clicked!");
    }
}