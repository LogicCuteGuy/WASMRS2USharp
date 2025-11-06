# Simple Multi-Behavior Pattern Example

This is a beginner-friendly example that demonstrates the basics of the Standard Multi-Behavior Pattern in UdonSharp-Rust. It shows how two simple behaviors can work together to create interactive functionality.

## What This Example Shows

This example implements a simple counter system with two behaviors:

- **Counter**: Manages a numeric counter with increment, decrement, and reset functionality
- **Display**: Shows the counter value and status messages in the UI

## Key Concepts Demonstrated

### 1. Multiple Behaviors in One File
```rust
#[derive(UdonBehaviour)]
pub struct Counter {
    // Counter logic
}

#[derive(UdonBehaviour)]
pub struct Display {
    // Display logic
}
```

### 2. Inter-Behavior Communication
The behaviors communicate through GameObject references and custom events:
```rust
// Counter finds Display
self.display = unity::GameObject::find("Display");

// Counter notifies Display of changes
display_obj.send_custom_event("UpdateDisplay");

// Display handles the event
#[udon_event("UpdateDisplay")]
pub fn update_display(&mut self) {
    // Update UI elements
}
```

### 3. Unity Inspector Integration
Fields marked with `#[udon_public]` appear in the Unity Inspector:
```rust
#[udon_public]
#[header("Counter Settings")]
#[tooltip("The current count value")]
current_count: i32,
```

### 4. Shared Functions
Functions used by multiple behaviors are automatically moved to SharedRuntime:
```rust
// This function will be in SharedRuntime.cs
pub fn get_count_status(count: i32, max_count: i32) -> String {
    // Status logic
}
```

## Building the Example

1. Navigate to the example directory:
```bash
cd examples/simple-multi-behavior
```

2. Build the project:
```bash
cargo udonsharp build
```

This generates:
- `Counter.cs` - The counter behavior
- `Display.cs` - The display behavior  
- `SharedRuntime.cs` - Shared utility functions

## Setting Up in Unity

### 1. Create GameObjects
Create two GameObjects in your scene:
- `Counter` (with Counter.cs component)
- `Display` (with Display.cs component)

### 2. Set Up UI Elements
For the Display behavior, create UI elements and assign them:
- **Count Text**: A UI Text component to show the counter value
- **Status Text**: A UI Text component to show status messages

### 3. Add Buttons (Optional)
Create UI buttons and set up their OnClick events to call:
- `Display.OnIncrementButton` - Increment the counter
- `Display.OnDecrementButton` - Decrement the counter  
- `Display.OnResetButton` - Reset the counter

### 4. Configure the Counter
In the Counter component inspector, set:
- **Current Count**: Starting value (usually 0)
- **Max Count**: Maximum allowed value (e.g., 100)

## How It Works

### Initialization
1. Both behaviors start and find each other using `GameObject::find()`
2. Counter initializes its values and updates the display
3. Display finds the counter and shows initial values

### User Interaction
1. User clicks a button (increment/decrement/reset)
2. Display receives the button event
3. Display calls the appropriate method on Counter
4. Counter updates its value and notifies Display
5. Display updates the UI to show new values

### Generated C# Code

The Rust code generates clean, efficient C# code:

**Counter.cs**:
```csharp
using UnityEngine;
using UdonSharp;

public class Counter : UdonSharpBehaviour
{
    [Header("Counter Settings")]
    [Tooltip("The current count value")]
    [SerializeField] public int currentCount = 0;
    
    [Tooltip("Maximum value the counter can reach")]
    [SerializeField] public int maxCount = 100;
    
    private GameObject display;
    
    void Start()
    {
        display = GameObject.Find("Display");
        UpdateDisplay();
        SharedRuntime.LogMessage("Counter initialized");
    }
    
    public void Increment()
    {
        if (currentCount < maxCount)
        {
            currentCount++;
            UpdateDisplay();
            SharedRuntime.LogMessage($"Counter incremented to {currentCount}");
        }
    }
    
    // ... other methods
}
```

**SharedRuntime.cs**:
```csharp
public class SharedRuntime : UdonSharpBehaviour
{
    public static string GetCountStatus(int count, int maxCount)
    {
        float percentage = (count / (float)maxCount) * 100f;
        
        if (count == 0)
            return "Ready to start counting!";
        else if (count == maxCount)
            return "Maximum count reached!";
        // ... other conditions
    }
    
    public static void LogMessage(string message)
    {
        Debug.Log($"[LOG] {message}");
    }
}
```

## Extending the Example

### Add New Features
- **Multiplier**: Add a multiplier that increases increment amount
- **History**: Track the history of counter changes
- **Persistence**: Save counter value between sessions
- **Animation**: Animate the counter changes

### Add New Behaviors
- **Settings**: Manage counter configuration
- **Statistics**: Track usage statistics
- **Network Sync**: Synchronize counter across players

### Example Extension
```rust
#[derive(UdonBehaviour)]
pub struct CounterSettings {
    #[udon_public]
    increment_amount: i32,
    
    #[udon_public]
    auto_increment: bool,
    
    counter: Option<unity::GameObject>,
}

impl UdonBehaviour for CounterSettings {
    fn start(&mut self) {
        self.increment_amount = 1;
        self.auto_increment = false;
        self.counter = unity::GameObject::find("Counter");
    }
    
    fn update(&mut self) {
        if self.auto_increment {
            // Auto-increment logic
        }
    }
}
```

## Best Practices Shown

1. **Clear Naming**: Descriptive names for behaviors and methods
2. **Single Responsibility**: Each behavior has one clear purpose
3. **Loose Coupling**: Behaviors communicate through events, not direct dependencies
4. **Error Handling**: Null checks for GameObject references
5. **Documentation**: Comments and tooltips for Unity Inspector
6. **Shared Code**: Common functionality extracted to SharedRuntime

## Next Steps

After understanding this example, try:
1. The [Standard Multi-Behavior Example](../standard-multi-behavior/) for a more complex system
2. Creating your own multi-behavior project
3. Reading the [Standard Multi-Behavior Pattern Documentation](../../docs/standard-multi-behavior-pattern.md)

## Troubleshooting

**Counter not updating Display**:
- Check that GameObject names match exactly ("Counter", "Display")
- Ensure both GameObjects are active in the scene

**UI not showing values**:
- Verify UI Text components are assigned in Display inspector
- Check that UI elements are active and visible

**Buttons not working**:
- Ensure button OnClick events are set up correctly
- Check that the correct method names are used (case-sensitive)