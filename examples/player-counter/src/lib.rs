use udonsharp_core::prelude::*;
use udonsharp_bindings::{vrchat, unity};
use std::collections::HashMap;

mod player_data;
mod ui_manager;

pub use player_data::*;
pub use ui_manager::*;

/// Main player tracking behavior - handles core player management and networking.
/// 
/// This behavior demonstrates:
/// - Network synchronization with manual sync mode
/// - Core player data tracking
/// - Master client responsibilities
/// - Inter-behavior communication
#[udon_behaviour(name = "PlayerTracker")]
pub fn player_tracker_start() {
    // Entry point for PlayerTracker UdonBehaviour
}

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct PlayerTracker {
    // Public configuration fields
    #[udon_public]
    pub update_interval: f32,
    
    #[udon_public]
    pub show_join_notifications: bool,
    
    #[udon_public]
    pub show_leave_notifications: bool,
    
    #[udon_public]
    pub max_notification_history: i32,
    
    // Synchronized fields
    #[udon_sync]
    pub total_players: i32,
    
    #[udon_sync]
    pub master_player_name: String,
    
    #[udon_sync]
    pub world_start_time: f32,
    
    // Private state
    players: HashMap<i32, PlayerData>,
    ui_manager: UIManager,
    last_update_time: f32,
    notification_history: Vec<NotificationData>,
    initialized: bool,
    is_master_client: bool,
}

impl PlayerTracker {
    pub fn new() -> Self {
        Self {
            update_interval: 1.0,
            show_join_notifications: true,
            show_leave_notifications: true,
            max_notification_history: 50,
            total_players: 0,
            master_player_name: String::new(),
            world_start_time: 0.0,
            players: HashMap::new(),
            ui_manager: UIManager::new(),
            last_update_time: 0.0,
            notification_history: Vec::new(),
            initialized: false,
            is_master_client: false,
        }
    }
    
    /// Initialize the player counter system
    fn initialize(&mut self) {
        self.world_start_time = unity::Time::time();
        self.is_master_client = vrchat::Networking::is_master();
        
        if self.is_master_client {
            self.master_player_name = vrchat::Networking::local_player().get_display_name();
        }
        
        // Initialize UI system
        self.ui_manager.initialize();
        
        // Get initial player list
        let current_players = vrchat::Networking::get_players();
        for player in current_players {
            self.add_player_internal(player);
        }
        
        self.update_synchronized_data();
        self.initialized = true;
        
        log::info!("🎯 PlayerCounter initialized - {} players, Master: {}", 
            self.total_players, self.master_player_name);
    }
    
    /// Add a player to the tracking system
    fn add_player_internal(&mut self, player: VRCPlayerApi) {
        let player_data = PlayerData::new(player.clone(), unity::Time::time());
        let player_id = player.player_id();
        
        self.players.insert(player_id, player_data.clone());
        
        // Add notification
        if self.show_join_notifications {
            let notification = NotificationData {
                message: format!("👋 {} joined", player_data.display_name),
                timestamp: unity::Time::time(),
                notification_type: NotificationType::PlayerJoined,
            };
            self.add_notification(notification);
        }
        
        log::info!("➕ Player added: {} (ID: {})", player_data.display_name, player_id);
    }
    
    /// Remove a player from the tracking system
    fn remove_player_internal(&mut self, player: VRCPlayerApi) {
        let player_id = player.player_id();
        
        if let Some(player_data) = self.players.remove(&player_id) {
            let session_duration = unity::Time::time() - player_data.join_time;
            
            // Add notification
            if self.show_leave_notifications {
                let notification = NotificationData {
                    message: format!("👋 {} left ({}s)", 
                        player_data.display_name, 
                        session_duration as i32
                    ),
                    timestamp: unity::Time::time(),
                    notification_type: NotificationType::PlayerLeft,
                };
                self.add_notification(notification);
            }
            
            log::info!("➖ Player removed: {} (Session: {:.1}s)", 
                player_data.display_name, session_duration);
        }
    }
    
    /// Update synchronized data (master client only)
    fn update_synchronized_data(&mut self) {
        if !self.is_master_client {
            return;
        }
        
        let new_total = self.players.len() as i32;
        let needs_sync = new_total != self.total_players;
        
        if needs_sync {
            self.total_players = new_total;
            self.request_serialization();
            
            log::debug!("🔄 Synchronized player count: {}", self.total_players);
        }
    }
    
    /// Add a notification to the history
    fn add_notification(&mut self, notification: NotificationData) {
        self.notification_history.push(notification.clone());
        
        // Trim history if too long
        if self.notification_history.len() > self.max_notification_history as usize {
            self.notification_history.remove(0);
        }
        
        // Update UI with new notification
        self.ui_manager.show_notification(&notification);
    }
    
    /// Update all UI elements
    fn update_ui(&mut self) {
        // Update player count display
        self.ui_manager.update_player_count(self.total_players);
        
        // Update player list
        let player_list: Vec<&PlayerData> = self.players.values().collect();
        self.ui_manager.update_player_list(&player_list);
        
        // Update master client info
        self.ui_manager.update_master_info(&self.master_player_name);
        
        // Update world stats
        let uptime = unity::Time::time() - self.world_start_time;
        self.ui_manager.update_world_stats(uptime, self.total_players);
    }
    
    /// Handle master client change
    fn handle_master_change(&mut self) {
        let new_master = vrchat::Networking::get_master();
        let local_player = vrchat::Networking::local_player();
        
        self.is_master_client = new_master.player_id() == local_player.player_id();
        
        if self.is_master_client {
            self.master_player_name = local_player.get_display_name();
            self.request_serialization();
            
            log::info!("👑 Became master client: {}", self.master_player_name);
            
            // Add notification
            let notification = NotificationData {
                message: format!("👑 {} is now master", self.master_player_name),
                timestamp: unity::Time::time(),
                notification_type: NotificationType::MasterChanged,
            };
            self.add_notification(notification);
        }
    }
    
    /// Get comprehensive player statistics
    pub fn get_player_stats(&self) -> PlayerStats {
        let mut vr_users = 0;
        let mut desktop_users = 0;
        let mut total_session_time = 0.0;
        let current_time = unity::Time::time();
        
        for player_data in self.players.values() {
            if player_data.is_vr_user {
                vr_users += 1;
            } else {
                desktop_users += 1;
            }
            total_session_time += current_time - player_data.join_time;
        }
        
        PlayerStats {
            total_players: self.total_players,
            vr_users,
            desktop_users,
            average_session_time: if self.total_players > 0 {
                total_session_time / self.total_players as f32
            } else {
                0.0
            },
            world_uptime: current_time - self.world_start_time,
            master_player: self.master_player_name.clone(),
        }
    }
}

impl UdonBehaviour for PlayerTracker {
    fn start(&mut self) {
        log::info!("🚀 PlayerCounter starting...");
        
        // Validate configuration
        if self.update_interval <= 0.0 {
            self.update_interval = 1.0;
            log::warn!("⚠️ Invalid update_interval, using default: 1.0s");
        }
        
        if self.max_notification_history <= 0 {
            self.max_notification_history = 50;
            log::warn!("⚠️ Invalid max_notification_history, using default: 50");
        }
        
        self.initialize();
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        let current_time = unity::Time::time();
        
        // Periodic updates based on update_interval
        if current_time - self.last_update_time >= self.update_interval {
            self.last_update_time = current_time;
            
            // Update UI
            self.update_ui();
            
            // Update player session times
            for player_data in self.players.values_mut() {
                player_data.update_session_duration(current_time);
            }
            
            // Check for master client changes
            let current_master = vrchat::Networking::get_master();
            if current_master.get_display_name() != self.master_player_name {
                self.handle_master_change();
            }
        }
        
        // Update UI manager (for animations, etc.)
        self.ui_manager.update();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.add_player_internal(player);
        self.update_synchronized_data();
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.remove_player_internal(player);
        self.update_synchronized_data();
    }
    
    fn on_ownership_transferred(&mut self, player: VRCPlayerApi) {
        log::info!("🔄 Ownership transferred to: {}", player.get_display_name());
    }
    
    fn on_post_deserialization(&mut self) {
        // Update UI when we receive synchronized data
        self.update_ui();
        
        log::debug!("📥 Received sync data - Players: {}, Master: {}", 
            self.total_players, self.master_player_name);
    }
}

/// Statistics about players in the world
#[derive(Debug, Clone)]
pub struct PlayerStats {
    pub total_players: i32,
    pub vr_users: i32,
    pub desktop_users: i32,
    pub average_session_time: f32,
    pub world_uptime: f32,
    pub master_player: String,
}

impl Default for PlayerTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// UI management behavior - handles all user interface updates and notifications.
#[udon_behaviour(name = "UIController")]
pub fn ui_controller_start() {
    // Entry point for UIController UdonBehaviour
}

#[derive(UdonBehaviour)]
pub struct UIController {
    ui_manager: UIManager,
    player_tracker_ref: Option<unity::GameObject>,
    initialized: bool,
}

impl UIController {
    pub fn new() -> Self {
        Self {
            ui_manager: UIManager::new(),
            player_tracker_ref: None,
            initialized: false,
        }
    }
    
    fn find_player_tracker(&mut self) {
        if let Some(tracker_obj) = unity::GameObject::find("PlayerTracker") {
            self.player_tracker_ref = Some(tracker_obj);
            log::info!("✅ Found PlayerTracker reference");
        } else {
            log::warn!("❌ PlayerTracker not found");
        }
    }
    
    pub fn update_display(&mut self, stats: &PlayerStats) {
        self.ui_manager.update_player_count(stats.total_players);
        self.ui_manager.update_master_info(&stats.master_player);
        self.ui_manager.update_world_stats(stats.world_uptime, stats.total_players);
    }
    
    pub fn show_notification(&mut self, notification: &NotificationData) {
        self.ui_manager.show_notification(notification);
    }
}

impl UdonBehaviour for UIController {
    fn start(&mut self) {
        log::info!("🎨 UIController starting...");
        self.ui_manager.initialize();
        self.find_player_tracker();
        self.initialized = true;
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        self.ui_manager.update();
    }
}

impl Default for UIController {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics behavior - handles data analysis and performance monitoring.
#[udon_behaviour(name = "StatsManager")]
pub fn stats_manager_start() {
    // Entry point for StatsManager UdonBehaviour
}

#[derive(UdonBehaviour)]
pub struct StatsManager {
    performance_metrics: Vec<f32>,
    last_stats_update: f32,
    stats_update_interval: f32,
    player_tracker_ref: Option<unity::GameObject>,
    ui_controller_ref: Option<unity::GameObject>,
    initialized: bool,
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            performance_metrics: Vec::new(),
            last_stats_update: 0.0,
            stats_update_interval: 2.0,
            player_tracker_ref: None,
            ui_controller_ref: None,
            initialized: false,
        }
    }
    
    fn find_behavior_references(&mut self) {
        if let Some(tracker_obj) = unity::GameObject::find("PlayerTracker") {
            self.player_tracker_ref = Some(tracker_obj);
        }
        if let Some(ui_obj) = unity::GameObject::find("UIController") {
            self.ui_controller_ref = Some(ui_obj);
        }
    }
    
    fn collect_performance_metrics(&mut self) {
        let frame_time = unity::Time::delta_time();
        self.performance_metrics.push(frame_time);
        
        // Keep only last 60 frames
        if self.performance_metrics.len() > 60 {
            self.performance_metrics.remove(0);
        }
    }
    
    fn calculate_average_frame_time(&self) -> f32 {
        if self.performance_metrics.is_empty() {
            return 0.0;
        }
        self.performance_metrics.iter().sum::<f32>() / self.performance_metrics.len() as f32
    }
}

impl UdonBehaviour for StatsManager {
    fn start(&mut self) {
        log::info!("📊 StatsManager starting...");
        self.find_behavior_references();
        self.initialized = true;
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        let current_time = unity::Time::time();
        
        // Collect performance data
        self.collect_performance_metrics();
        
        // Periodic stats updates
        if current_time - self.last_stats_update >= self.stats_update_interval {
            let avg_frame_time = self.calculate_average_frame_time();
            if avg_frame_time > 0.02 { // 20ms threshold
                log::warn!("⚠️ Performance issue: {:.3}ms avg frame time", avg_frame_time * 1000.0);
            }
            
            self.last_stats_update = current_time;
        }
    }
}

impl Default for StatsManager {
    fn default() -> Self {
        Self::new()
    }
}

// Shared utility functions available to all behaviors
pub fn calculate_distance(a: unity::Vector3, b: unity::Vector3) -> f32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn format_time_duration(seconds: f32) -> String {
    let minutes = (seconds / 60.0) as i32;
    let secs = (seconds % 60.0) as i32;
    format!("{}:{:02}", minutes, secs)
}