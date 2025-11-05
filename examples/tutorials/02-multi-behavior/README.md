# Tutorial 02: Multi-Behavior Basics

This tutorial introduces the multi-behavior compilation feature, showing how to split a single Rust project into multiple UdonBehaviour classes.

## What You'll Learn

- How to use `#[udon_behaviour]` attributes
- Inter-behavior communication
- Shared functionality
- Configuration options

## Prerequisites

- Completed Tutorial 01 (Basics)
- Understanding of Rust basics
- Familiarity with VRChat/UdonSharp concepts

## Step 1: Project Setup

Create a new project with multi-behavior support:

```toml
# Cargo.toml
[package]
name = "multi-behavior-tutorial"
version = "0.1.0"
edition = "2021"

[dependencies]
udonsharp-core = { path = "../../../crates/udonsharp-core" }

[lib]
crate-type = ["cdylib"]
```

```toml
# udonsharp.toml
[project]
name = "MultiBehaviorTutorial"
namespace = "Tutorial.MultiBehavior"

[multi_behavior]
enabled = true
generate_shared_runtime = true
naming_convention = "PascalCase"
```

## Step 2: Define Your First Behavior

```rust
// src/lib.rs
use udonsharp_core::prelude::*;

// This function becomes a separate UdonBehaviour class
#[udon_behaviour(name = "Counter", events = ["Start", "Update"])]
pub fn counter_start() {
    log_info("Counter behavior started!");
    set_counter_value(0);
}

#[udon_behaviour_event(behavior = "Counter")]
pub fn counter_update() {
    // Increment counter every second
    static mut TIMER: f32 = 0.0;
    
    unsafe {
        TIMER += get_delta_time();
        if TIMER >= 1.0 {
            TIMER = 0.0;
            let current = get_counter_value();
            set_counter_value(current + 1);
            
            // Notify the display behavior
            notify_display_updated();
        }
    }
}
```

## Step 3: Add a Second Behavior

```rust
// Display behavior that shows the counter value
#[udon_behaviour(name = "Display", events = ["Start"])]
pub fn display_start() {
    log_info("Display behavior started!");
    update_display_text();
}

// This method can be called from other behaviors
#[udon_behaviour_method(behavior = "Display")]
pub fn on_counter_updated() {
    update_display_text();
    log_info(&format!("Display updated: {}", get_counter_value()));
}
```

## Step 4: Shared Functionality

```rust
// These functions will be moved to SharedRuntime automatically
static mut COUNTER_VALUE: i32 = 0;

pub fn get_counter_value() -> i32 {
    unsafe { COUNTER_VALUE }
}

pub fn set_counter_value(value: i32) {
    unsafe { COUNTER_VALUE = value; }
}

pub fn log_info(message: &str) {
    println!("[INFO] {}", message);
}

// Inter-behavior communication
pub fn notify_display_updated() {
    send_custom_event("Display", "OnCounterUpdated");
}

// Placeholder implementations
pub fn send_custom_event(behavior: &str, event: &str) {
    log_info(&format!("SendCustomEvent: {} -> {}", behavior, event));
}

pub fn get_delta_time() -> f32 {
    0.016 // 60 FPS
}

fn update_display_text() {
    log_info(&format!("Counter value: {}", get_counter_value()));
}
```

## Step 5: Build and Test

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Generated Output

The compiler will generate:

```
Counter.cs          - Counter behavior class
Display.cs          - Display behavior class  
SharedRuntime.cs    - Shared functions and data
Counter.prefab      - Counter behavior prefab
Display.prefab      - Display behavior prefab
System.prefab       - Combined system prefab
```

## Key Concepts

### Behavior Attributes

- `#[udon_behaviour]` - Marks a function as a behavior entry point
- `name` - Sets the generated class name
- `events` - Specifies Unity events to handle

### Event Methods

- `#[udon_behaviour_event]` - Handles Unity events (Start, Update, etc.)
- `behavior` - Specifies which behavior this event belongs to

### Custom Methods

- `#[udon_behaviour_method]` - Creates methods callable from other behaviors
- Called via `SendCustomEvent` in UdonSharp

### Shared Code

- Functions without behavior attributes become shared
- Automatically moved to `SharedRuntime` class
- Available to all behaviors

## Communication Patterns

### Event-Based Communication

```rust
// Sender
pub fn notify_other_behavior() {
    send_custom_event("TargetBehavior", "MethodName");
}

// Receiver
#[udon_behaviour_method(behavior = "TargetBehavior")]
pub fn method_name() {
    // Handle the event
}
```

### Shared State

```rust
// Shared data accessible by all behaviors
static mut SHARED_DATA: i32 = 0;

pub fn get_shared_data() -> i32 {
    unsafe { SHARED_DATA }
}

pub fn set_shared_data(value: i32) {
    unsafe { SHARED_DATA = value; }
}
```

## Best Practices

1. **Single Responsibility**: Each behavior should have one clear purpose
2. **Loose Coupling**: Use events for communication, not direct calls
3. **Shared State**: Keep shared data in global functions
4. **Clear Naming**: Use descriptive behavior and method names
5. **Error Handling**: Handle edge cases gracefully

## Next Steps

- Try the comprehensive [Multi-Behavior Demo](../multi-behavior-demo/)
- Learn about [Performance Optimization](../03-performance/)
- Explore [Advanced Patterns](../04-advanced-patterns/)

## Troubleshooting

### Common Issues

**Behavior not generated**: Check that function has `#[udon_behaviour]` attribute

**Communication not working**: Verify event names match exactly

**Compilation errors**: Ensure all dependencies are properly configured

**Missing shared functions**: Functions without attributes are automatically shared