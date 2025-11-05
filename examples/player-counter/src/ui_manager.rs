use udonsharp_bindings::unity;
use crate::{PlayerData, NotificationData};
use std::collections::VecDeque;

/// Manages all UI elements for the player counter system
pub struct UIManager {
    // UI element references
    player_count_text: Option<unity::UI::Text>,
    player_list_text: Option<unity::UI::Text>,
    master_info_text: Option<unity::UI::Text>,
    world_stats_text: Option<unity::UI::Text>,
    notification_text: Option<unity::UI::Text>,
    
    // Animation and display state
    notification_queue: VecDeque<NotificationData>,
    current_notification: Option<NotificationData>,
    notification_display_time: f32,
    notification_fade_time: f32,
    last_notification_update: f32,
    
    // Configuration
    max_player_list_entries: usize,
    notification_duration: f32,
    fade_duration: f32,
    
    initialized: bool,
}

impl UIManager {
    pub fn new() -> Self {
        Self {
            player_count_text: None,
            player_list_text: None,
            master_info_text: None,
            world_stats_text: None,
            notification_text: None,
            notification_queue: VecDeque::new(),
            current_notification: None,
            notification_display_time: 0.0,
            notification_fade_time: 0.0,
            last_notification_update: 0.0,
            max_player_list_entries: 20,
            notification_duration: 3.0,
            fade_duration: 0.5,
            initialized: false,
        }
    }
    
    /// Initialize the UI system by finding UI elements
    pub fn initialize(&mut self) {
        log::info!("🎨 Initializing UI Manager...");
        
        // Find UI elements
        self.player_count_text = self.find_ui_text("PlayerCountText");
        self.player_list_text = self.find_ui_text("PlayerListText");
        self.master_info_text = self.find_ui_text("MasterInfoText");
        self.world_stats_text = self.find_ui_text("WorldStatsText");
        self.notification_text = self.find_ui_text("NotificationText");
        
        // Log which UI elements were found
        self.log_ui_element_status("PlayerCountText", &self.player_count_text);
        self.log_ui_element_status("PlayerListText", &self.player_list_text);
        self.log_ui_element_status("MasterInfoText", &self.master_info_text);
        self.log_ui_element_status("WorldStatsText", &self.world_stats_text);
        self.log_ui_element_status("NotificationText", &self.notification_text);
        
        // Initialize notification text if available
        if let Some(text) = &self.notification_text {
            text.set_text("");
            text.set_color(unity::Color::new(1.0, 1.0, 1.0, 0.0)); // Transparent initially
        }
        
        self.initialized = true;
        log::info!("✅ UI Manager initialized");
    }
    
    /// Find a UI Text component by GameObject name
    fn find_ui_text(&self, name: &str) -> Option<unity::UI::Text> {
        if let Some(game_object) = unity::GameObject::find(name) {
            game_object.get_component::<unity::UI::Text>()
        } else {
            None
        }
    }
    
    /// Log the status of a UI element
    fn log_ui_element_status(&self, name: &str, element: &Option<unity::UI::Text>) {
        if element.is_some() {
            log::info!("✅ Found UI element: {}", name);
        } else {
            log::warn!("❌ UI element not found: {}", name);
        }
    }
    
    /// Update the player count display
    pub fn update_player_count(&mut self, count: i32) {
        if let Some(text) = &self.player_count_text {
            let display_text = format!("Players: {}", count);
            text.set_text(&display_text);
            
            // Color code based on player count
            let color = if count == 0 {
                unity::Color::new(0.7, 0.7, 0.7, 1.0) // Gray for empty
            } else if count < 10 {
                unity::Color::new(0.0, 1.0, 0.0, 1.0) // Green for low
            } else if count < 50 {
                unity::Color::new(1.0, 1.0, 0.0, 1.0) // Yellow for medium
            } else {
                unity::Color::new(1.0, 0.5, 0.0, 1.0) // Orange for high
            };
            
            text.set_color(color);
        }
    }
    
    /// Update the player list display
    pub fn update_player_list(&mut self, players: &[&PlayerData]) {
        if let Some(text) = &self.player_list_text {
            let mut display_lines = Vec::new();
            
            // Sort players by join time (oldest first)
            let mut sorted_players = players.to_vec();
            sorted_players.sort_by(|a, b| a.join_time.partial_cmp(&b.join_time).unwrap());
            
            // Take only the configured maximum number of entries
            let display_count = std::cmp::min(sorted_players.len(), self.max_player_list_entries);
            
            for player in sorted_players.iter().take(display_count) {
                display_lines.push(player.format_display());
            }
            
            // Add "and X more..." if there are more players
            if sorted_players.len() > self.max_player_list_entries {
                let remaining = sorted_players.len() - self.max_player_list_entries;
                display_lines.push(format!("... and {} more", remaining));
            }
            
            let display_text = if display_lines.is_empty() {
                "No players".to_string()
            } else {
                display_lines.join("\n")
            };
            
            text.set_text(&display_text);
        }
    }
    
    /// Update master client information
    pub fn update_master_info(&mut self, master_name: &str) {
        if let Some(text) = &self.master_info_text {
            let display_text = format!("👑 Master: {}", master_name);
            text.set_text(&display_text);
            text.set_color(unity::Color::new(1.0, 1.0, 0.0, 1.0)); // Yellow for master
        }
    }
    
    /// Update world statistics display
    pub fn update_world_stats(&mut self, uptime: f32, player_count: i32) {
        if let Some(text) = &self.world_stats_text {
            let uptime_minutes = (uptime / 60.0) as i32;
            let uptime_seconds = (uptime % 60.0) as i32;
            
            let display_text = format!(
                "🌍 Uptime: {}:{:02}\n📊 Players: {}",
                uptime_minutes, uptime_seconds, player_count
            );
            
            text.set_text(&display_text);
        }
    }
    
    /// Show a notification
    pub fn show_notification(&mut self, notification: &NotificationData) {
        self.notification_queue.push_back(notification.clone());
    }
    
    /// Update notification display (call from update loop)
    pub fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        let current_time = unity::Time::time();
        
        // Handle notification display
        self.update_notifications(current_time);
    }
    
    /// Update notification system
    fn update_notifications(&mut self, current_time: f32) {
        if let Some(text) = &self.notification_text {
            // Check if we need to start a new notification
            if self.current_notification.is_none() && !self.notification_queue.is_empty() {
                if let Some(notification) = self.notification_queue.pop_front() {
                    self.current_notification = Some(notification.clone());
                    self.notification_display_time = current_time;
                    self.notification_fade_time = current_time + self.notification_duration;
                    
                    // Set notification text and color
                    text.set_text(&notification.format_for_display());
                    text.set_color(notification.get_color());
                    
                    self.last_notification_update = current_time;
                }
            }
            
            // Handle current notification
            if let Some(notification) = &self.current_notification {
                let elapsed = current_time - self.notification_display_time;
                
                if elapsed > self.notification_duration + self.fade_duration {
                    // Notification finished, clear it
                    self.current_notification = None;
                    text.set_text("");
                    text.set_color(unity::Color::new(1.0, 1.0, 1.0, 0.0));
                } else if elapsed > self.notification_duration {
                    // Fade out phase
                    let fade_progress = (elapsed - self.notification_duration) / self.fade_duration;
                    let alpha = 1.0 - fade_progress;
                    
                    let mut color = notification.get_color();
                    color.a = alpha;
                    text.set_color(color);
                } else {
                    // Update notification text periodically (for relative timestamps)
                    if current_time - self.last_notification_update > 1.0 {
                        text.set_text(&notification.format_for_display());
                        self.last_notification_update = current_time;
                    }
                }
            }
        }
    }
    
    /// Set the maximum number of players to show in the list
    pub fn set_max_player_list_entries(&mut self, max_entries: usize) {
        self.max_player_list_entries = max_entries;
    }
    
    /// Set notification display duration
    pub fn set_notification_duration(&mut self, duration: f32) {
        self.notification_duration = duration;
    }
    
    /// Set notification fade duration
    pub fn set_fade_duration(&mut self, duration: f32) {
        self.fade_duration = duration;
    }
    
    /// Clear all notifications
    pub fn clear_notifications(&mut self) {
        self.notification_queue.clear();
        self.current_notification = None;
        
        if let Some(text) = &self.notification_text {
            text.set_text("");
            text.set_color(unity::Color::new(1.0, 1.0, 1.0, 0.0));
        }
    }
    
    /// Get the number of queued notifications
    pub fn get_notification_queue_size(&self) -> usize {
        self.notification_queue.len()
    }
    
    /// Check if UI is properly initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Get a status report of available UI elements
    pub fn get_ui_status(&self) -> UIStatus {
        UIStatus {
            player_count_available: self.player_count_text.is_some(),
            player_list_available: self.player_list_text.is_some(),
            master_info_available: self.master_info_text.is_some(),
            world_stats_available: self.world_stats_text.is_some(),
            notification_available: self.notification_text.is_some(),
            initialized: self.initialized,
        }
    }
}

/// Status of UI elements
#[derive(Debug, Clone)]
pub struct UIStatus {
    pub player_count_available: bool,
    pub player_list_available: bool,
    pub master_info_available: bool,
    pub world_stats_available: bool,
    pub notification_available: bool,
    pub initialized: bool,
}

impl UIStatus {
    /// Check if all UI elements are available
    pub fn all_available(&self) -> bool {
        self.player_count_available
            && self.player_list_available
            && self.master_info_available
            && self.world_stats_available
            && self.notification_available
    }
    
    /// Get a list of missing UI elements
    pub fn get_missing_elements(&self) -> Vec<&str> {
        let mut missing = Vec::new();
        
        if !self.player_count_available {
            missing.push("PlayerCountText");
        }
        if !self.player_list_available {
            missing.push("PlayerListText");
        }
        if !self.master_info_available {
            missing.push("MasterInfoText");
        }
        if !self.world_stats_available {
            missing.push("WorldStatsText");
        }
        if !self.notification_available {
            missing.push("NotificationText");
        }
        
        missing
    }
}