use udonsharp_bindings::{vrchat::VRCPlayerApi, unity};
use serde::{Deserialize, Serialize};

/// Comprehensive data structure for tracking individual players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// The VRChat player API reference
    pub player_api: VRCPlayerApi,
    
    /// Player's display name
    pub display_name: String,
    
    /// Player's unique ID
    pub player_id: i32,
    
    /// Time when the player joined (in seconds since world start)
    pub join_time: f32,
    
    /// Whether the player is using VR
    pub is_vr_user: bool,
    
    /// Current session duration
    pub session_duration: f32,
    
    /// Whether this is the local player
    pub is_local: bool,
    
    /// Player's position when last updated
    pub last_position: unity::Vector3,
    
    /// Number of times the player has moved significantly
    pub movement_count: i32,
    
    /// Last time the player moved
    pub last_movement_time: f32,
}

impl PlayerData {
    /// Create new player data from a VRCPlayerApi
    pub fn new(player: VRCPlayerApi, join_time: f32) -> Self {
        let display_name = player.get_display_name();
        let player_id = player.player_id();
        let is_vr_user = player.is_user_in_vr();
        let is_local = player.is_local();
        let last_position = player.get_position();
        
        Self {
            player_api: player,
            display_name,
            player_id,
            join_time,
            is_vr_user,
            session_duration: 0.0,
            is_local,
            last_position,
            movement_count: 0,
            last_movement_time: join_time,
        }
    }
    
    /// Update the session duration
    pub fn update_session_duration(&mut self, current_time: f32) {
        self.session_duration = current_time - self.join_time;
    }
    
    /// Update player position and track movement
    pub fn update_position(&mut self, current_time: f32) {
        let new_position = self.player_api.get_position();
        let distance = self.calculate_distance(&self.last_position, &new_position);
        
        // Consider it movement if they moved more than 0.5 units
        if distance > 0.5 {
            self.movement_count += 1;
            self.last_movement_time = current_time;
            self.last_position = new_position;
        }
    }
    
    /// Calculate distance between two positions
    fn calculate_distance(&self, pos1: &unity::Vector3, pos2: &unity::Vector3) -> f32 {
        let dx = pos2.x - pos1.x;
        let dy = pos2.y - pos1.y;
        let dz = pos2.z - pos1.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
    
    /// Check if the player has been idle for a certain duration
    pub fn is_idle(&self, current_time: f32, idle_threshold: f32) -> bool {
        current_time - self.last_movement_time > idle_threshold
    }
    
    /// Get a formatted string representation of the player
    pub fn format_display(&self) -> String {
        let platform = if self.is_vr_user { "VR" } else { "Desktop" };
        let local_indicator = if self.is_local { " (You)" } else { "" };
        
        format!("{} [{}] - {:.1}s{}", 
            self.display_name, 
            platform, 
            self.session_duration,
            local_indicator
        )
    }
    
    /// Get player statistics
    pub fn get_stats(&self) -> PlayerStatistics {
        PlayerStatistics {
            display_name: self.display_name.clone(),
            player_id: self.player_id,
            session_duration: self.session_duration,
            is_vr_user: self.is_vr_user,
            is_local: self.is_local,
            movement_count: self.movement_count,
            last_movement_time: self.last_movement_time,
            current_position: self.last_position,
        }
    }
}

/// Notification data for UI display
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub message: String,
    pub timestamp: f32,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    PlayerJoined,
    PlayerLeft,
    MasterChanged,
    SystemMessage,
}

impl NotificationData {
    /// Get the color for this notification type
    pub fn get_color(&self) -> unity::Color {
        match self.notification_type {
            NotificationType::PlayerJoined => unity::Color::new(0.0, 1.0, 0.0, 1.0), // Green
            NotificationType::PlayerLeft => unity::Color::new(1.0, 0.5, 0.0, 1.0),   // Orange
            NotificationType::MasterChanged => unity::Color::new(1.0, 1.0, 0.0, 1.0), // Yellow
            NotificationType::SystemMessage => unity::Color::new(0.5, 0.5, 1.0, 1.0), // Light Blue
        }
    }
    
    /// Get the icon for this notification type
    pub fn get_icon(&self) -> &str {
        match self.notification_type {
            NotificationType::PlayerJoined => "👋",
            NotificationType::PlayerLeft => "👋",
            NotificationType::MasterChanged => "👑",
            NotificationType::SystemMessage => "ℹ️",
        }
    }
    
    /// Format the notification for display
    pub fn format_for_display(&self) -> String {
        let elapsed = unity::Time::time() - self.timestamp;
        let time_str = if elapsed < 60.0 {
            format!("{}s ago", elapsed as i32)
        } else {
            format!("{}m ago", (elapsed / 60.0) as i32)
        };
        
        format!("{} {} ({})", self.get_icon(), self.message, time_str)
    }
}

/// Detailed player statistics
#[derive(Debug, Clone)]
pub struct PlayerStatistics {
    pub display_name: String,
    pub player_id: i32,
    pub session_duration: f32,
    pub is_vr_user: bool,
    pub is_local: bool,
    pub movement_count: i32,
    pub last_movement_time: f32,
    pub current_position: unity::Vector3,
}