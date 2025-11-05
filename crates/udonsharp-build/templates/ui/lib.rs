//! UI-focused UdonSharp project written in Rust
//! 
//! This demonstrates UI management and canvas interactions.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

pub mod canvas_manager;
pub mod ui_components;
pub mod animations;

use canvas_manager::*;
use ui_components::*;
use animations::*;

/// UI controller for managing canvas and interactions
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct UIController {
    #[udon_public]
    pub main_canvas: Option<GameObject>,
    
    #[udon_public]
    pub hud_canvas: Option<GameObject>,
    
    #[udon_sync]
    pub ui_state: String, // JSON serialized UI state
    
    // Local state
    canvas_manager: CanvasManager,
    component_registry: UIComponentRegistry,
    animation_system: UIAnimationSystem,
    initialized: bool,
}

impl UdonBehaviour for UIController {
    fn start(&mut self) {
        self.canvas_manager = CanvasManager::new();
        self.component_registry = UIComponentRegistry::new();
        self.animation_system = UIAnimationSystem::new();
        
        self.setup_canvases();
        self.register_components();
        
        self.initialized = true;
        debug_log("UI controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update animations
        self.animation_system.update(Time::delta_time());
        
        // Update UI components
        self.component_registry.update();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.show_welcome_message(&player.get_display_name());
        debug_log(&format!("Showing welcome UI for: {}", player.get_display_name()));
    }
}

impl UIController {
    pub fn new() -> Self {
        Self {
            main_canvas: None,
            hud_canvas: None,
            ui_state: String::new(),
            canvas_manager: CanvasManager::new(),
            component_registry: UIComponentRegistry::new(),
            animation_system: UIAnimationSystem::new(),
            initialized: false,
        }
    }
    
    fn setup_canvases(&mut self) {
        // Find and setup main canvas
        if let Some(main_canvas) = GameObject::find("MainCanvas") {
            self.main_canvas = Some(main_canvas.clone());
            self.canvas_manager.register_canvas("main", main_canvas);
        }
        
        // Find and setup HUD canvas
        if let Some(hud_canvas) = GameObject::find("HUDCanvas") {
            self.hud_canvas = Some(hud_canvas.clone());
            self.canvas_manager.register_canvas("hud", hud_canvas);
        }
    }
    
    fn register_components(&mut self) {
        // Register UI components for management
        self.component_registry.register_button("start_button", "StartGame");
        self.component_registry.register_button("settings_button", "OpenSettings");
        self.component_registry.register_text("player_count", "PlayerCount");
        self.component_registry.register_text("game_status", "GameStatus");
    }
    
    fn show_welcome_message(&mut self, player_name: &str) {
        let message = format!("Welcome, {}!", player_name);
        self.component_registry.update_text("welcome_text", &message);
        
        // Animate welcome message
        self.animation_system.fade_in("welcome_panel", 1.0);
    }
    
    #[udon_event]
    pub fn on_button_click(&mut self) {
        debug_log("UI button clicked");
        
        // Handle button interactions
        self.animation_system.scale_bounce("clicked_button", 0.2);
    }
    
    #[udon_event]
    pub fn on_show_menu(&mut self) {
        self.canvas_manager.show_canvas("main");
        self.animation_system.slide_in("main_menu", 0.5);
        debug_log("Main menu shown");
    }
    
    #[udon_event]
    pub fn on_hide_menu(&mut self) {
        self.animation_system.slide_out("main_menu", 0.5);
        // Hide canvas after animation completes
        debug_log("Main menu hidden");
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> UIController {
    UIController::new()
}