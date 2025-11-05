//! Game Logic UdonSharp project written in Rust
//! 
//! This demonstrates game logic patterns and state management.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

pub mod game_state;
pub mod events;
pub mod scoring;

use game_state::*;
use events::*;
use scoring::*;

/// Game logic controller with state management
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct GameLogicController {
    #[udon_sync]
    pub current_state: i32, // GameState as i32
    
    #[udon_sync]
    pub game_time: f32,
    
    #[udon_sync]
    pub player_scores: String, // JSON serialized scores
    
    // Local state
    state_machine: GameStateMachine,
    event_system: GameEventSystem,
    score_manager: ScoreManager,
    initialized: bool,
}

impl UdonBehaviour for GameLogicController {
    fn start(&mut self) {
        self.state_machine = GameStateMachine::new();
        self.event_system = GameEventSystem::new();
        self.score_manager = ScoreManager::new();
        
        self.current_state = GameState::Waiting as i32;
        self.game_time = 0.0;
        
        self.initialized = true;
        debug_log("Game logic controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update game time
        self.game_time += Time::delta_time();
        
        // Update state machine
        self.state_machine.update(Time::delta_time());
        
        // Process events
        self.event_system.process_events();
        
        // Update scores
        self.score_manager.update();
        
        // Sync state changes
        if self.state_machine.state_changed() {
            self.current_state = self.state_machine.current_state() as i32;
        }
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.score_manager.add_player(&player.get_display_name());
        self.event_system.emit_event(GameEvent::PlayerJoined {
            player_name: player.get_display_name(),
        });
        
        debug_log(&format!("Player joined game: {}", player.get_display_name()));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.event_system.emit_event(GameEvent::PlayerLeft {
            player_name: player.get_display_name(),
        });
        
        debug_log(&format!("Player left game: {}", player.get_display_name()));
    }
}

impl GameLogicController {
    pub fn new() -> Self {
        Self {
            current_state: GameState::Waiting as i32,
            game_time: 0.0,
            player_scores: String::new(),
            state_machine: GameStateMachine::new(),
            event_system: GameEventSystem::new(),
            score_manager: ScoreManager::new(),
            initialized: false,
        }
    }
    
    #[udon_event]
    pub fn on_start_game(&mut self) {
        if self.state_machine.can_transition_to(GameState::Playing) {
            self.state_machine.transition_to(GameState::Playing);
            self.game_time = 0.0;
            self.score_manager.reset_scores();
            
            self.event_system.emit_event(GameEvent::GameStarted);
            debug_log("Game started!");
        }
    }
    
    #[udon_event]
    pub fn on_end_game(&mut self) {
        if self.state_machine.can_transition_to(GameState::Finished) {
            self.state_machine.transition_to(GameState::Finished);
            
            let winner = self.score_manager.get_winner();
            self.event_system.emit_event(GameEvent::GameEnded { winner });
            debug_log("Game ended!");
        }
    }
    
    #[udon_event]
    pub fn on_player_scored(&mut self) {
        let local_player = Networking::local_player();
        let player_name = local_player.get_display_name();
        
        self.score_manager.add_score(&player_name, 1);
        self.event_system.emit_event(GameEvent::PlayerScored {
            player_name: player_name.clone(),
            score: self.score_manager.get_score(&player_name),
        });
        
        debug_log(&format!("Player {} scored!", player_name));
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> GameLogicController {
    GameLogicController::new()
}