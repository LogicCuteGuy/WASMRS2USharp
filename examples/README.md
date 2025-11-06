# UdonSharp-Rust Examples

This directory contains example projects demonstrating various features and patterns of UdonSharp-Rust development. Each example is a complete, buildable project with documentation and explanations.

## Getting Started

Each example is a self-contained Rust project that can be built independently:

```bash
cd examples/[example-name]
cargo udonsharp build
```

The generated C# files can then be imported into your Unity VRChat project.

## Available Examples

### Simple Multi-Behavior
**Path**: `examples/simple-multi-behavior/`
**Description**: Beginner-friendly introduction to the Standard Multi-Behavior Pattern.
**Difficulty**: Beginner
**Features**:
- Two simple behaviors (Counter and Display)
- Inter-behavior communication
- SharedRuntime generation
- Unity Inspector integration
- Custom event handlers

**What you'll learn**:
- How to define multiple UdonBehaviour structs in one file
- Basic inter-behavior communication using GameObject references
- Custom event handlers for UI interactions
- Automatic SharedRuntime generation for shared functions

### Standard Multi-Behavior
**Path**: `examples/standard-multi-behavior/`
**Description**: Complete example of the Standard Multi-Behavior Pattern with a multiplayer game system.
**Difficulty**: Intermediate to Advanced
**Features**:
- Four interconnected behaviors (GameManager, PlayerTracker, UIController, ScoreSystem)
- Network synchronization
- Master client validation
- Comprehensive UI management
- Shared utility functions
- Event-driven architecture

**What you'll learn**:
- Complex multi-behavior system design
- Network synchronization patterns
- Master client authority handling
- Event-driven architecture
- UI management across multiple behaviors
- Performance optimization techniques

## Learning Path

We recommend following this order when learning UdonSharp-Rust:

1. **Start with Simple Multi-Behavior** - Learn the basics of the multi-behavior pattern
2. **Study Standard Multi-Behavior** - See how to build complex, real-world systems
3. **Create Your Own Project** - Apply what you've learned to your own VRChat world

## Example Structure

Each example follows a consistent structure:

```
example-name/
├── Cargo.toml              # Rust project configuration
├── udonsharp.toml          # UdonSharp-Rust specific configuration
├── build.rs                # Build script (if needed)
├── src/
│   └── lib.rs              # Main source file with behaviors
└── README.md               # Detailed documentation and setup instructions
```

## Building Examples

### Prerequisites

1. Install UdonSharp-Rust:
```bash
cargo install cargo-udonsharp
```

2. Ensure you have the required Rust target:
```bash
rustup target add wasm32-unknown-unknown
```

### Build Process

1. Navigate to an example directory:
```bash
cd examples/simple-multi-behavior
```

2. Build the project:
```bash
cargo udonsharp build
```

3. The generated C# files will be created in the current directory or specified output directory.

### Generated Files

After building, you'll typically see these generated files:
- `[BehaviorName].cs` - Individual behavior classes
- `SharedRuntime.cs` - Shared utility functions (if applicable)
- `[CoordinatorName].cs` - Initialization coordinator (if enabled)
- Unity prefab files (if prefab generation is enabled)

## Using Generated Code in Unity

1. **Import Files**: Copy the generated `.cs` files to your Unity project's `Assets` folder.

2. **Set Up GameObjects**: Create GameObjects for each behavior and attach the corresponding scripts.

3. **Configure References**: Set up GameObject references and UI component assignments in the Unity Inspector.

4. **Test in Play Mode**: Test the functionality in Unity's Play Mode before uploading to VRChat.

## Configuration Options

Each example includes a `udonsharp.toml` configuration file that controls compilation behavior:

```toml
[multi_behavior]
enabled = true                          # Enable multi-behavior compilation
generate_shared_runtime = true         # Generate SharedRuntime class
naming_convention = "PascalCase"        # C# class naming convention
min_behaviors_threshold = 2             # Minimum behaviors for multi-behavior mode

[multi_behavior.prefab_settings]
generate_individual_prefabs = true      # Generate prefabs for each behavior
generate_master_prefab = true           # Generate master prefab with all behaviors
auto_setup_references = true            # Automatically set up GameObject references

[build]
generate_debug_info = true              # Include debug information
optimize_for_performance = true         # Enable performance optimizations
```

## Common Patterns Demonstrated

### Inter-Behavior Communication
- GameObject references for direct communication
- Custom events for loose coupling
- Shared data through synchronized fields

### Network Synchronization
- Master client validation
- Synchronized field patterns
- Proper serialization handling

### UI Management
- Component reference patterns
- Event-driven UI updates
- Cross-behavior UI coordination

### Code Organization
- Single responsibility principle
- Shared utility extraction
- Clean separation of concerns

## Troubleshooting

### Build Issues

**"No UdonBehaviour structs found"**:
- Ensure structs have `#[derive(UdonBehaviour)]`
- Check that `udonsharp-core` is properly imported

**"Circular dependency detected"**:
- Review behavior dependencies
- Consider using events instead of direct references

### Runtime Issues

**"GameObject not found"**:
- Verify GameObject names match `find()` calls exactly
- Ensure GameObjects are active in the scene

**"Component not found"**:
- Check that behavior scripts are attached to GameObjects
- Verify component types match expected behaviors

### Unity Integration

**Generated code doesn't compile in Unity**:
- Ensure all required UdonSharp packages are installed
- Check that generated code follows UdonSharp conventions
- Verify Unity project settings are correct for VRChat development

## Best Practices

1. **Start Simple**: Begin with the simple examples before attempting complex systems
2. **Read Documentation**: Each example includes detailed README files
3. **Experiment**: Modify examples to understand how changes affect the generated code
4. **Test Thoroughly**: Always test in Unity Play Mode before uploading to VRChat
5. **Follow Conventions**: Use the patterns demonstrated in examples for consistency

## Contributing

If you'd like to contribute additional examples:

1. Follow the existing example structure
2. Include comprehensive documentation
3. Test thoroughly in Unity and VRChat
4. Submit a pull request with your example

## Further Reading

- [Standard Multi-Behavior Pattern Documentation](../docs/standard-multi-behavior-pattern.md)
- [UdonSharp-Rust API Reference](../docs/api-reference.md)
- [Best Practices Guide](../docs/best-practices.md)
- [Getting Started Guide](../docs/getting-started.md)