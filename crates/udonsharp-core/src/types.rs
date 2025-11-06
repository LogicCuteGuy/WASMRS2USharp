use serde::{Deserialize, Serialize};

// Unity Math Types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };
    pub const ONE: Vector2 = Vector2 { x: 1.0, y: 1.0 };
    pub const UP: Vector2 = Vector2 { x: 0.0, y: 1.0 };
    pub const DOWN: Vector2 = Vector2 { x: 0.0, y: -1.0 };
    pub const LEFT: Vector2 = Vector2 { x: -1.0, y: 0.0 };
    pub const RIGHT: Vector2 = Vector2 { x: 1.0, y: 0.0 };
    
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self { x: self.x / mag, y: self.y / mag }
        } else {
            Self::ZERO
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Vector3 = Vector3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Vector3 = Vector3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const DOWN: Vector3 = Vector3 { x: 0.0, y: -1.0, z: 0.0 };
    pub const LEFT: Vector3 = Vector3 { x: -1.0, y: 0.0, z: 0.0 };
    pub const RIGHT: Vector3 = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const FORWARD: Vector3 = Vector3 { x: 0.0, y: 0.0, z: 1.0 };
    pub const BACK: Vector3 = Vector3 { x: 0.0, y: 0.0, z: -1.0 };
    
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    pub fn zero() -> Self {
        Self::ZERO
    }
    
    pub fn one() -> Self {
        Self::ONE
    }
    
    pub fn up() -> Self {
        Self::UP
    }
    
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self { x: self.x / mag, y: self.y / mag, z: self.z / mag }
        } else {
            Self::ZERO
        }
    }
    
    pub fn dot(&self, other: &Vector3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    pub fn cross(&self, other: &Vector3) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl std::ops::Add for Vector3 {
    type Output = Vector3;
    
    fn add(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl std::ops::Sub for Vector3 {
    type Output = Vector3;
    
    fn sub(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl std::ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    
    fn mul(self, scalar: f32) -> Vector3 {
        Vector3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4 {
    pub const ZERO: Vector4 = Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE: Vector4 = Vector4 { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
    
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub const IDENTITY: Quaternion = Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };
    
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
    
    pub fn identity() -> Self {
        Self::IDENTITY
    }
    
    pub fn from_euler(x: f32, y: f32, z: f32) -> Self {
        let (sx, cx) = (x * 0.5).sin_cos();
        let (sy, cy) = (y * 0.5).sin_cos();
        let (sz, cz) = (z * 0.5).sin_cos();
        
        Self {
            x: sx * cy * cz - cx * sy * sz,
            y: cx * sy * cz + sx * cy * sz,
            z: cx * cy * sz - sx * sy * cz,
            w: cx * cy * cz + sx * sy * sz,
        }
    }
    
    pub fn look_rotation(forward: Vector3, up: Vector3) -> Self {
        // Simplified look rotation for testing
        // In a real implementation, this would be more complex
        let right = up.cross(&forward).normalized();
        let up = forward.cross(&right).normalized();
        
        // Create rotation matrix and convert to quaternion
        // This is a simplified version for testing purposes
        let trace = right.x + up.y + forward.z;
        if trace > 0.0 {
            let s = (trace + 1.0).sqrt() * 2.0;
            Self {
                w: 0.25 * s,
                x: (up.z - forward.y) / s,
                y: (forward.x - right.z) / s,
                z: (right.y - up.x) / s,
            }
        } else {
            Self::IDENTITY
        }
    }
}

impl std::ops::Mul for Quaternion {
    type Output = Quaternion;
    
    fn mul(self, other: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w * other.w - self.x * other.x - self.y * other.y - self.z * other.z,
            x: self.w * other.x + self.x * other.w + self.y * other.z - self.z * other.y,
            y: self.w * other.y - self.x * other.z + self.y * other.w + self.z * other.x,
            z: self.w * other.z + self.x * other.y - self.y * other.x + self.z * other.w,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color32 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

// VRChat Types
#[derive(Debug, Clone)]
pub struct VRCPlayerApi {
    // Internal handle - will be mapped to actual VRCPlayerApi in generated code
    pub(crate) handle: ObjectHandle,
}

impl VRCPlayerApi {
    pub fn get_display_name(&self) -> String {
        // This will be replaced with actual binding in generated code
        String::from("Player")
    }
    
    pub fn is_local(&self) -> bool {
        // This will be replaced with actual binding in generated code
        false
    }
    
    pub fn is_master(&self) -> bool {
        // This will be replaced with actual binding in generated code
        false
    }
    
    pub fn respawn(&self) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn teleport_to(&self, _position: Vector3, _rotation: Quaternion) {
        // This will be replaced with actual binding in generated code
    }
}

// Unity Component Types
#[derive(Debug, Clone)]
pub struct GameObject {
    pub(crate) handle: ObjectHandle,
}

impl GameObject {
    pub fn find(_name: &str) -> Option<Self> {
        // This will be replaced with actual binding in generated code
        None
    }
    
    pub fn new(_name: &str) -> Self {
        // This will be replaced with actual binding in generated code
        Self { handle: ObjectHandle::default() }
    }
    
    pub fn get_component<T>(&self) -> Option<T> {
        // This will be replaced with actual binding in generated code
        None
    }
    
    pub fn set_active(&mut self, _active: bool) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn transform(&self) -> Transform {
        // This will be replaced with actual binding in generated code
        Transform { handle: ObjectHandle::default() }
    }
    
    pub fn send_custom_event(&self, _event_name: &str) {
        // This will be replaced with actual binding in generated code
    }
}

#[derive(Debug, Clone)]
pub struct Transform {
    pub(crate) handle: ObjectHandle,
}

impl Transform {
    pub fn position(&self) -> Vector3 {
        // This will be replaced with actual binding in generated code
        Vector3::ZERO
    }
    
    pub fn set_position(&mut self, _position: Vector3) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn rotation(&self) -> Quaternion {
        // This will be replaced with actual binding in generated code
        Quaternion::IDENTITY
    }
    
    pub fn set_rotation(&mut self, _rotation: Quaternion) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn translate(&mut self, _translation: Vector3) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn rotate(&mut self, _rotation: Vector3) {
        // This will be replaced with actual binding in generated code
    }
}

#[derive(Debug, Clone)]
pub struct Collider {
    pub(crate) handle: ObjectHandle,
}

#[derive(Debug, Clone)]
pub struct Collision {
    pub(crate) handle: ObjectHandle,
}

// Internal handle type for Unity objects
#[derive(Debug, Clone, Default)]
pub(crate) struct ObjectHandle {
    // This will be replaced with actual object reference in generated code
    pub(crate) id: u32,
}

// UdonSharp Sync Modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UdonSyncMode {
    None,
    Manual,
    Continuous,
}

impl Default for UdonSyncMode {
    fn default() -> Self {
        UdonSyncMode::None
    }
}

// Network Event Targets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkEventTarget {
    All,
    Others,
    Owner,
}

// UdonSharp attribute data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonPublicAttribute {
    pub field_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonSyncAttribute {
    pub field_name: String,
    pub sync_mode: UdonSyncMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonEventAttribute {
    pub method_name: String,
    pub event_name: String,
}

/// Information about a field in a UdonSharp class
#[derive(Debug, Clone)]
pub struct UdonFieldInfo {
    pub name: &'static str,
    pub type_name: &'static str,
    pub is_public: bool,
    pub is_sync: bool,
    pub sync_mode: UdonSyncMode,
    pub header_text: Option<String>,
    pub tooltip_text: Option<String>,
}

/// Unity event types that can be handled by UdonBehaviour
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnityEvent {
    Start,
    Update,
    FixedUpdate,
    LateUpdate,
    OnEnable,
    OnDisable,
    OnDestroy,
    OnTriggerEnter,
    OnTriggerExit,
    OnTriggerStay,
    OnCollisionEnter,
    OnCollisionExit,
    OnCollisionStay,
    OnPlayerJoined,
    OnPlayerLeft,
    OnPlayerTriggerEnter,
    OnPlayerTriggerExit,
    OnPlayerTriggerStay,
    OnPlayerCollisionEnter,
    OnPlayerCollisionExit,
    OnPlayerCollisionStay,
    OnPickup,
    OnDrop,
    OnPickupUseDown,
    OnPickupUseUp,
    OnStationEntered,
    OnStationExited,
    Custom(String),
}

impl UnityEvent {
    /// Parse a string into a UnityEvent
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Start" => Ok(UnityEvent::Start),
            "Update" => Ok(UnityEvent::Update),
            "FixedUpdate" => Ok(UnityEvent::FixedUpdate),
            "LateUpdate" => Ok(UnityEvent::LateUpdate),
            "OnEnable" => Ok(UnityEvent::OnEnable),
            "OnDisable" => Ok(UnityEvent::OnDisable),
            "OnDestroy" => Ok(UnityEvent::OnDestroy),
            "OnTriggerEnter" => Ok(UnityEvent::OnTriggerEnter),
            "OnTriggerExit" => Ok(UnityEvent::OnTriggerExit),
            "OnTriggerStay" => Ok(UnityEvent::OnTriggerStay),
            "OnCollisionEnter" => Ok(UnityEvent::OnCollisionEnter),
            "OnCollisionExit" => Ok(UnityEvent::OnCollisionExit),
            "OnCollisionStay" => Ok(UnityEvent::OnCollisionStay),
            "OnPlayerJoined" => Ok(UnityEvent::OnPlayerJoined),
            "OnPlayerLeft" => Ok(UnityEvent::OnPlayerLeft),
            "OnPlayerTriggerEnter" => Ok(UnityEvent::OnPlayerTriggerEnter),
            "OnPlayerTriggerExit" => Ok(UnityEvent::OnPlayerTriggerExit),
            "OnPlayerTriggerStay" => Ok(UnityEvent::OnPlayerTriggerStay),
            "OnPlayerCollisionEnter" => Ok(UnityEvent::OnPlayerCollisionEnter),
            "OnPlayerCollisionExit" => Ok(UnityEvent::OnPlayerCollisionExit),
            "OnPlayerCollisionStay" => Ok(UnityEvent::OnPlayerCollisionStay),
            "OnPickup" => Ok(UnityEvent::OnPickup),
            "OnDrop" => Ok(UnityEvent::OnDrop),
            "OnPickupUseDown" => Ok(UnityEvent::OnPickupUseDown),
            "OnPickupUseUp" => Ok(UnityEvent::OnPickupUseUp),
            "OnStationEntered" => Ok(UnityEvent::OnStationEntered),
            "OnStationExited" => Ok(UnityEvent::OnStationExited),
            _ => Ok(UnityEvent::Custom(s.to_string())),
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            UnityEvent::Start => "Start".to_string(),
            UnityEvent::Update => "Update".to_string(),
            UnityEvent::FixedUpdate => "FixedUpdate".to_string(),
            UnityEvent::LateUpdate => "LateUpdate".to_string(),
            UnityEvent::OnEnable => "OnEnable".to_string(),
            UnityEvent::OnDisable => "OnDisable".to_string(),
            UnityEvent::OnDestroy => "OnDestroy".to_string(),
            UnityEvent::OnTriggerEnter => "OnTriggerEnter".to_string(),
            UnityEvent::OnTriggerExit => "OnTriggerExit".to_string(),
            UnityEvent::OnTriggerStay => "OnTriggerStay".to_string(),
            UnityEvent::OnCollisionEnter => "OnCollisionEnter".to_string(),
            UnityEvent::OnCollisionExit => "OnCollisionExit".to_string(),
            UnityEvent::OnCollisionStay => "OnCollisionStay".to_string(),
            UnityEvent::OnPlayerJoined => "OnPlayerJoined".to_string(),
            UnityEvent::OnPlayerLeft => "OnPlayerLeft".to_string(),
            UnityEvent::OnPlayerTriggerEnter => "OnPlayerTriggerEnter".to_string(),
            UnityEvent::OnPlayerTriggerExit => "OnPlayerTriggerExit".to_string(),
            UnityEvent::OnPlayerTriggerStay => "OnPlayerTriggerStay".to_string(),
            UnityEvent::OnPlayerCollisionEnter => "OnPlayerCollisionEnter".to_string(),
            UnityEvent::OnPlayerCollisionExit => "OnPlayerCollisionExit".to_string(),
            UnityEvent::OnPlayerCollisionStay => "OnPlayerCollisionStay".to_string(),
            UnityEvent::OnPickup => "OnPickup".to_string(),
            UnityEvent::OnDrop => "OnDrop".to_string(),
            UnityEvent::OnPickupUseDown => "OnPickupUseDown".to_string(),
            UnityEvent::OnPickupUseUp => "OnPickupUseUp".to_string(),
            UnityEvent::OnStationEntered => "OnStationEntered".to_string(),
            UnityEvent::OnStationExited => "OnStationExited".to_string(),
            UnityEvent::Custom(name) => name.clone(),
        }
    }
}

/// Configuration for a UdonBehaviour attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdonBehaviourAttribute {
    /// Name of the generated UdonBehaviour class
    pub name: String,
    /// Unity events this behaviour should handle
    pub events: Vec<UnityEvent>,
    /// Dependencies on other UdonBehaviour classes
    pub dependencies: Vec<String>,
    /// Whether to automatically sync this behaviour
    pub auto_sync: bool,
}

impl UdonBehaviourAttribute {
    /// Create a new UdonBehaviourAttribute with default values
    pub fn new(name: String) -> Self {
        Self {
            name,
            events: vec![UnityEvent::Start],
            dependencies: Vec::new(),
            auto_sync: false,
        }
    }

    /// Add an event to handle
    pub fn with_event(mut self, event: UnityEvent) -> Self {
        self.events.push(event);
        self
    }

    /// Add multiple events to handle
    pub fn with_events(mut self, events: Vec<UnityEvent>) -> Self {
        self.events.extend(events);
        self
    }

    /// Add a dependency
    pub fn with_dependency(mut self, dependency: String) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Add multiple dependencies
    pub fn with_dependencies(mut self, dependencies: Vec<String>) -> Self {
        self.dependencies.extend(dependencies);
        self
    }

    /// Enable auto sync
    pub fn with_auto_sync(mut self) -> Self {
        self.auto_sync = true;
        self
    }
}

// Additional Unity Engine Types

/// Unity Rigidbody component
#[derive(Debug, Clone)]
pub struct Rigidbody {
    pub(crate) handle: ObjectHandle,
}

impl Rigidbody {
    pub fn velocity(&self) -> Vector3 {
        Vector3::ZERO
    }
    
    pub fn set_velocity(&mut self, _velocity: Vector3) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn add_force(&mut self, _force: Vector3) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn add_torque(&mut self, _torque: Vector3) {
        // This will be replaced with actual binding in generated code
    }
}

/// Unity Animator component
#[derive(Debug, Clone)]
pub struct Animator {
    pub(crate) handle: ObjectHandle,
}

impl Animator {
    pub fn play(&mut self, _state_name: &str) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_bool(&mut self, _name: &str, _value: bool) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_float(&mut self, _name: &str, _value: f32) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_integer(&mut self, _name: &str, _value: i32) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_trigger(&mut self, _name: &str) {
        // This will be replaced with actual binding in generated code
    }
}

/// Unity AudioSource component
#[derive(Debug, Clone)]
pub struct AudioSource {
    pub(crate) handle: ObjectHandle,
}

impl AudioSource {
    pub fn play(&mut self) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn stop(&mut self) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn pause(&mut self) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_volume(&mut self, _volume: f32) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_pitch(&mut self, _pitch: f32) {
        // This will be replaced with actual binding in generated code
    }
}

/// Unity Light component
#[derive(Debug, Clone)]
pub struct Light {
    pub(crate) handle: ObjectHandle,
}

impl Light {
    pub fn color(&self) -> Color {
        Color::WHITE
    }
    
    pub fn set_color(&mut self, _color: Color) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn intensity(&self) -> f32 {
        1.0
    }
    
    pub fn set_intensity(&mut self, _intensity: f32) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_enabled(&mut self, _enabled: bool) {
        // This will be replaced with actual binding in generated code
    }
}

// VRChat Specific Types

/// VRChat Station component
#[derive(Debug, Clone)]
pub struct VRCStation {
    pub(crate) handle: ObjectHandle,
}

impl VRCStation {
    pub fn use_station(&mut self, _player: VRCPlayerApi) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn exit_station(&mut self, _player: VRCPlayerApi) {
        // This will be replaced with actual binding in generated code
    }
}

/// VRChat Pickup component
#[derive(Debug, Clone)]
pub struct VRCPickup {
    pub(crate) handle: ObjectHandle,
}

impl VRCPickup {
    pub fn drop(&mut self) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn set_kinematic(&mut self, _kinematic: bool) {
        // This will be replaced with actual binding in generated code
    }
}

/// VRChat Object Pool
#[derive(Debug, Clone)]
pub struct VRCObjectPool {
    pub(crate) handle: ObjectHandle,
}

impl VRCObjectPool {
    pub fn try_to_spawn(&mut self) -> Option<GameObject> {
        // This will be replaced with actual binding in generated code
        None
    }
    
    pub fn return_object(&mut self, _obj: GameObject) {
        // This will be replaced with actual binding in generated code
    }
}

// Unity UI Types

/// Unity UI Text component
#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) handle: ObjectHandle,
}

impl Text {
    pub fn text(&self) -> String {
        String::new()
    }
    
    pub fn set_text(&mut self, _text: &str) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn color(&self) -> Color {
        Color::BLACK
    }
    
    pub fn set_color(&mut self, _color: Color) {
        // This will be replaced with actual binding in generated code
    }
}

/// Unity UI Button component
#[derive(Debug, Clone)]
pub struct Button {
    pub(crate) handle: ObjectHandle,
}

impl Button {
    pub fn set_interactable(&mut self, _interactable: bool) {
        // This will be replaced with actual binding in generated code
    }
}

/// Unity UI Slider component
#[derive(Debug, Clone)]
pub struct Slider {
    pub(crate) handle: ObjectHandle,
}

impl Slider {
    pub fn value(&self) -> f32 {
        0.0
    }
    
    pub fn set_value(&mut self, _value: f32) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn min_value(&self) -> f32 {
        0.0
    }
    
    pub fn set_min_value(&mut self, _min: f32) {
        // This will be replaced with actual binding in generated code
    }
    
    pub fn max_value(&self) -> f32 {
        1.0
    }
    
    pub fn set_max_value(&mut self, _max: f32) {
        // This will be replaced with actual binding in generated code
    }
}

/// Unity UI Toggle component
#[derive(Debug, Clone)]
pub struct Toggle {
    pub(crate) handle: ObjectHandle,
}

impl Toggle {
    pub fn is_on(&self) -> bool {
        false
    }
    
    pub fn set_is_on(&mut self, _is_on: bool) {
        // This will be replaced with actual binding in generated code
    }
}

// Utility Types and Functions

/// UdonSharp utility functions
pub struct UdonSharpUtility;

impl UdonSharpUtility {
    /// Send a custom event to a UdonBehaviour
    pub fn send_custom_event(_target: &GameObject, _event_name: &str) {
        // This will be replaced with actual binding in generated code
    }
    
    /// Send a custom network event
    pub fn send_custom_network_event(_target: NetworkEventTarget, _event_name: &str) {
        // This will be replaced with actual binding in generated code
    }
    
    /// Get the local player
    pub fn get_local_player() -> VRCPlayerApi {
        VRCPlayerApi { handle: ObjectHandle::default() }
    }
    
    /// Get all players in the world
    pub fn get_players() -> Vec<VRCPlayerApi> {
        Vec::new()
    }
}

/// VRChat Networking utilities
pub struct Networking;

impl Networking {
    /// Check if the local player is the owner of an object
    pub fn is_owner(_player: &VRCPlayerApi, _target: &GameObject) -> bool {
        false
    }
    
    /// Set the owner of an object
    pub fn set_owner(_player: &VRCPlayerApi, _target: &GameObject) {
        // This will be replaced with actual binding in generated code
    }
    
    /// Get the owner of an object
    pub fn get_owner(_target: &GameObject) -> VRCPlayerApi {
        VRCPlayerApi { handle: ObjectHandle::default() }
    }
    
    /// Check if the local player is the master
    pub fn is_master(_player: &VRCPlayerApi) -> bool {
        false
    }
    
    /// Get the master player
    pub fn get_master() -> VRCPlayerApi {
        VRCPlayerApi { handle: ObjectHandle::default() }
    }
}

// Math Utilities

/// Unity Mathf utilities
pub struct Mathf;

impl Mathf {
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;
    
    pub fn abs(value: f32) -> f32 {
        value.abs()
    }
    
    pub fn sin(value: f32) -> f32 {
        value.sin()
    }
    
    pub fn cos(value: f32) -> f32 {
        value.cos()
    }
    
    pub fn tan(value: f32) -> f32 {
        value.tan()
    }
    
    pub fn sqrt(value: f32) -> f32 {
        value.sqrt()
    }
    
    pub fn pow(base: f32, exp: f32) -> f32 {
        base.powf(exp)
    }
    
    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t.clamp(0.0, 1.0)
    }
    
    pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
        value.clamp(min, max)
    }
    
    pub fn clamp01(value: f32) -> f32 {
        value.clamp(0.0, 1.0)
    }
    
    pub fn min(a: f32, b: f32) -> f32 {
        a.min(b)
    }
    
    pub fn max(a: f32, b: f32) -> f32 {
        a.max(b)
    }
    
    pub fn floor(value: f32) -> f32 {
        value.floor()
    }
    
    pub fn ceil(value: f32) -> f32 {
        value.ceil()
    }
    
    pub fn round(value: f32) -> f32 {
        value.round()
    }
}

/// Unity Random utilities
pub struct Random;

impl Random {
    pub fn value() -> f32 {
        // This will be replaced with actual binding in generated code
        0.5
    }
    
    pub fn range_float(min: f32, max: f32) -> f32 {
        // This will be replaced with actual binding in generated code
        min + (max - min) * 0.5
    }
    
    pub fn range_int(min: i32, max: i32) -> i32 {
        // This will be replaced with actual binding in generated code
        (min + max) / 2
    }
}

/// Unity Time utilities
pub struct Time;

impl Time {
    pub fn time() -> f32 {
        // This will be replaced with actual binding in generated code
        0.0
    }
    
    pub fn delta_time() -> f32 {
        // This will be replaced with actual binding in generated code
        0.016
    }
    
    pub fn fixed_time() -> f32 {
        // This will be replaced with actual binding in generated code
        0.0
    }
    
    pub fn time_scale() -> f32 {
        // This will be replaced with actual binding in generated code
        1.0
    }
    
    pub fn set_time_scale(_scale: f32) {
        // This will be replaced with actual binding in generated code
    }
}