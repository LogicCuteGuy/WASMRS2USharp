use crate::types::*;

/// Core trait that all UdonSharp behaviors must implement
/// This mirrors the UdonSharp lifecycle methods and provides the foundation
/// for Rust-to-UdonSharp compilation.
pub trait UdonBehaviour {
    /// Called when the UdonBehaviour is first created
    /// This is equivalent to UdonSharp's Start() method
    fn start(&mut self) {}
    
    /// Called once per frame
    /// This is equivalent to UdonSharp's Update() method
    fn update(&mut self) {}
    
    /// Called at fixed intervals for physics
    /// This is equivalent to UdonSharp's FixedUpdate() method
    fn fixed_update(&mut self) {}
    
    /// Called once per frame after Update
    /// This is equivalent to UdonSharp's LateUpdate() method
    fn late_update(&mut self) {}
    
    /// Called when a player joins the world
    /// This is equivalent to UdonSharp's OnPlayerJoined() method
    fn on_player_joined(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when a player leaves the world
    /// This is equivalent to UdonSharp's OnPlayerLeft() method
    fn on_player_left(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when ownership of this object changes
    /// This is equivalent to UdonSharp's OnOwnershipTransferred() method
    fn on_ownership_transferred(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when a custom event is triggered
    /// This is equivalent to receiving a SendCustomEvent() call
    fn on_custom_event(&mut self, _event_name: &str) {}
    
    /// Called when a network event is received
    /// This is equivalent to receiving a SendCustomNetworkEvent() call
    fn on_custom_network_event(&mut self, _event_name: &str) {}
    
    /// Called when the object is enabled
    /// This is equivalent to UdonSharp's OnEnable() method
    fn on_enable(&mut self) {}
    
    /// Called when the object is disabled
    /// This is equivalent to UdonSharp's OnDisable() method
    fn on_disable(&mut self) {}
    
    /// Called when the object is destroyed
    /// This is equivalent to UdonSharp's OnDestroy() method
    fn on_destroy(&mut self) {}
    
    /// Called before the first frame update
    /// This is equivalent to UdonSharp's Awake() method
    fn awake(&mut self) {}
    
    /// Called when the object becomes visible to any camera
    /// This is equivalent to UdonSharp's OnBecameVisible() method
    fn on_became_visible(&mut self) {}
    
    /// Called when the object is no longer visible to any camera
    /// This is equivalent to UdonSharp's OnBecameInvisible() method
    fn on_became_invisible(&mut self) {}
    
    /// Called when the application gains focus
    /// This is equivalent to UdonSharp's OnApplicationFocus() method
    fn on_application_focus(&mut self, _focus: bool) {}
    
    /// Called when the application is paused
    /// This is equivalent to UdonSharp's OnApplicationPause() method
    fn on_application_pause(&mut self, _pause: bool) {}
    
    /// Get the UdonSharp type name for this behaviour
    fn get_udon_type_name(&self) -> &'static str {
        "UdonBehaviour"
    }
    
    /// Get the sync mode for this behaviour
    fn get_sync_mode(&self) -> UdonSyncMode {
        UdonSyncMode::None
    }
    
    /// Get field information for code generation
    fn get_field_info(&self) -> Vec<UdonFieldInfo> {
        Vec::new()
    }
}

/// Trait for objects that can be synchronized over the network
pub trait UdonSyncable {
    /// Called when synchronization data is received
    fn on_deserialization(&mut self) {}
    
    /// Called before synchronization data is sent
    fn on_pre_serialization(&mut self) {}
    
    /// Called after synchronization data is sent
    fn on_post_serialization(&mut self) {}
    
    /// Returns true if this object should be synchronized
    fn should_sync(&self) -> bool {
        true
    }
}

/// Trait for objects that can handle VRChat events
pub trait VRCEventHandler {
    /// Called when a player triggers an interact event
    fn interact(&mut self) {}
    
    /// Called when a player enters a trigger
    fn on_trigger_enter(&mut self, _other: Collider) {}
    
    /// Called when a player exits a trigger
    fn on_trigger_exit(&mut self, _other: Collider) {}
    
    /// Called when a player stays in a trigger
    fn on_trigger_stay(&mut self, _other: Collider) {}
    
    /// Called when a collision occurs
    fn on_collision_enter(&mut self, _collision: Collision) {}
    
    /// Called when a collision ends
    fn on_collision_exit(&mut self, _collision: Collision) {}
    
    /// Called while a collision is ongoing
    fn on_collision_stay(&mut self, _collision: Collision) {}
}

/// Trait for objects that can handle Unity animation events
pub trait AnimationEventHandler {
    /// Called when an animation event is triggered
    fn on_animation_event(&mut self, _event_name: &str) {}
}

/// Trait for objects that can handle Unity UI events
pub trait UIEventHandler {
    /// Called when a UI button is clicked
    fn on_click(&mut self) {}
    
    /// Called when a UI input field value changes
    fn on_value_changed(&mut self, _value: &str) {}
    
    /// Called when a UI slider value changes
    fn on_slider_changed(&mut self, _value: f32) {}
    
    /// Called when a UI toggle value changes
    fn on_toggle_changed(&mut self, _value: bool) {}
}

/// Trait for objects that can handle VRChat station events
pub trait StationEventHandler {
    /// Called when a player enters the station
    fn on_station_entered(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when a player exits the station
    fn on_station_exited(&mut self, _player: VRCPlayerApi) {}
}

/// Trait for objects that can handle VRChat pickup events
pub trait PickupEventHandler {
    /// Called when the pickup is grabbed
    fn on_pickup(&mut self) {}
    
    /// Called when the pickup is dropped
    fn on_drop(&mut self) {}
    
    /// Called when the pickup is used (trigger pressed while holding)
    fn on_pickup_use_down(&mut self) {}
    
    /// Called when the pickup use is released
    fn on_pickup_use_up(&mut self) {}
}

/// Trait for objects that can handle VRChat video player events
pub trait VideoPlayerEventHandler {
    /// Called when video starts playing
    fn on_video_start(&mut self) {}
    
    /// Called when video ends
    fn on_video_end(&mut self) {}
    
    /// Called when video is paused
    fn on_video_pause(&mut self) {}
    
    /// Called when video playback encounters an error
    fn on_video_error(&mut self, _error: VideoError) {}
    
    /// Called when video is ready to play
    fn on_video_ready(&mut self) {}
}

/// Trait for objects that can handle VRChat mirror events
pub trait MirrorEventHandler {
    /// Called when mirror reflection changes
    fn on_mirror_reflection_change(&mut self, _enabled: bool) {}
}

/// Trait for objects that can handle VRChat world events
pub trait WorldEventHandler {
    /// Called when the world is loaded
    fn on_world_loaded(&mut self) {}
    
    /// Called when the world is about to be unloaded
    fn on_world_unload(&mut self) {}
    
    /// Called when world settings change
    fn on_world_settings_changed(&mut self) {}
}

/// Trait for objects that can handle VRChat avatar events
pub trait AvatarEventHandler {
    /// Called when an avatar is loaded
    fn on_avatar_loaded(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when an avatar is changed
    fn on_avatar_changed(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when avatar scaling changes
    fn on_avatar_scale_changed(&mut self, _player: VRCPlayerApi, _scale: f32) {}
}

/// Trait for objects that can be serialized for UdonSharp
pub trait UdonSerializable {
    /// Serialize the object to a byte array
    fn serialize(&self) -> Vec<u8>;
    
    /// Deserialize the object from a byte array
    fn deserialize(&mut self, data: &[u8]) -> Result<(), crate::error::UdonSharpError>;
}

/// Trait for objects that can handle UdonSharp lifecycle events
pub trait UdonLifecycle {
    /// Called when the object is first initialized
    fn on_init(&mut self) {}
    
    /// Called when the object is about to be destroyed
    fn on_cleanup(&mut self) {}
    
    /// Called when the object is reset to default state
    fn on_reset(&mut self) {}
}

/// Trait for objects that can handle input events
pub trait InputEventHandler {
    /// Called when a key is pressed
    fn on_key_down(&mut self, _key: KeyCode) {}
    
    /// Called when a key is released
    fn on_key_up(&mut self, _key: KeyCode) {}
    
    /// Called when mouse button is pressed
    fn on_mouse_down(&mut self, _button: MouseButton) {}
    
    /// Called when mouse button is released
    fn on_mouse_up(&mut self, _button: MouseButton) {}
    
    /// Called when mouse moves
    fn on_mouse_move(&mut self, _delta: Vector2) {}
}

/// Trait for objects that can handle VRChat world loading events
pub trait WorldLoadEventHandler {
    /// Called when the world finishes loading
    fn on_world_load_complete(&mut self) {}
    
    /// Called when world data is being loaded
    fn on_world_data_loading(&mut self) {}
    
    /// Called when world assets are being loaded
    fn on_world_assets_loading(&mut self) {}
}

/// Trait for objects that can handle VRChat instance events
pub trait InstanceEventHandler {
    /// Called when the instance master changes
    fn on_master_client_switched(&mut self, _new_master: VRCPlayerApi) {}
    
    /// Called when instance settings change
    fn on_instance_settings_changed(&mut self) {}
    
    /// Called when the instance is about to close
    fn on_instance_closing(&mut self) {}
}

/// Trait for objects that can handle VRChat avatar pedestal events
pub trait AvatarPedestalEventHandler {
    /// Called when an avatar is placed on the pedestal
    fn on_avatar_changed(&mut self) {}
    
    /// Called when someone tries on the avatar
    fn on_avatar_equipped(&mut self, _player: VRCPlayerApi) {}
}

/// Trait for objects that can handle VRChat portal events
pub trait PortalEventHandler {
    /// Called when a player enters the portal
    fn on_portal_enter(&mut self, _player: VRCPlayerApi) {}
    
    /// Called when a player exits the portal
    fn on_portal_exit(&mut self, _player: VRCPlayerApi) {}
}

/// Trait for objects that can be configured via UdonSharp attributes
pub trait UdonConfigurable {
    /// Apply configuration from UdonSharp attributes
    fn apply_udon_config(&mut self) {}
    
    /// Validate UdonSharp configuration
    fn validate_udon_config(&self) -> Result<(), crate::error::UdonSharpError> {
        Ok(())
    }
}

/// Video error types for VideoPlayerEventHandler
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoError {
    /// Unknown error occurred
    Unknown,
    /// Network error
    NetworkError,
    /// Invalid URL
    InvalidUrl,
    /// Access denied
    AccessDenied,
    /// Player error
    PlayerError,
}

/// Key codes for input handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Alpha0, Alpha1, Alpha2, Alpha3, Alpha4, Alpha5, Alpha6, Alpha7, Alpha8, Alpha9,
    Space, Return, Escape, Tab, Backspace, Delete,
    LeftArrow, RightArrow, UpArrow, DownArrow,
    LeftShift, RightShift, LeftControl, RightControl, LeftAlt, RightAlt,
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}