# Compilation Pipeline

This document explains the Rust UdonSharp compilation pipeline that transforms Rust code into UdonSharp-compatible C# code.

## Overview

The compilation pipeline consists of several stages that transform your Rust code through WebAssembly into optimized UdonSharp C# code:

```
Rust Source → WASM → Optimized WASM → Enhanced C# → UdonSharp Files
```

## Pipeline Stages

### 1. Rust Compilation

**Input**: Rust source code with UdonSharp traits and attributes  
**Output**: WebAssembly binary  
**Tool**: `rustc` with `wasm32-unknown-unknown` target

The Rust compiler transforms your code into WebAssembly with UdonSharp-specific configurations:

```rust
// Your Rust code
#[derive(UdonBehaviour)]
pub struct MyBehaviour {
    #[udon_public]
    pub message: String,
}

impl UdonBehaviour for MyBehaviour {
    fn start(&mut self) {
        self.message = "Hello World".to_string();
    }
}
```

**Compilation Configuration**:
```toml
[profile.wasm-release]
opt-level = "s"        # Optimize for size
codegen-units = 1      # Single codegen unit for better optimization
panic = "abort"        # Abort on panic (UdonSharp compatible)
strip = true          # Strip debug symbols
```

### 2. WASM Optimization

**Input**: Raw WebAssembly binary  
**Output**: Optimized WebAssembly binary  
**Tool**: `wasm-opt` from Binaryen

The optimizer applies UdonSharp-specific optimizations:

```bash
wasm-opt input.wasm -o output.wasm \
  --optimize-level=3 \
  --shrink-level=2 \
  --enable-bulk-memory \
  --enable-sign-ext \
  --enable-mutable-globals \
  --disable-threads \
  --disable-simd
```

**Optimization Passes**:
- Dead code elimination
- Function inlining
- Constant propagation
- Memory layout optimization
- Size reduction for VRChat constraints

### 3. OOP Behavior Analysis

**Input**: Optimized WebAssembly binary  
**Output**: OOP pattern metadata  
**Tool**: Enhanced wasm2usharp analyzer

The analyzer detects object-oriented patterns in the WASM:

```rust
pub struct OopBehaviorAnalyzer {
    detected_classes: Vec<ClassPattern>,
    method_mappings: HashMap<String, MethodInfo>,
    inheritance_chains: Vec<InheritancePattern>,
}

pub struct ClassPattern {
    pub name: String,
    pub methods: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub constructor_signature: Option<String>,
    pub base_class: Option<String>,
}
```

**Detection Patterns**:
- Class structure identification
- Method grouping and organization
- Field access pattern analysis
- Inheritance relationship mapping
- Interface implementation detection

### 4. Enhanced WASM to C# Conversion

**Input**: Optimized WASM + OOP metadata  
**Output**: Raw C# code with UdonSharp patterns  
**Tool**: Enhanced wasm2usharp converter

The converter transforms WASM instructions into C# while applying OOP patterns:

```csharp
// Generated C# from Rust UdonBehaviour
using UdonSharp;
using UnityEngine;
using VRC.SDKBase;

[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]
public class MyBehaviour : UdonSharpBehaviour
{
    [UdonSynced, FieldChangeCallback(nameof(MessageChanged))]
    public string message = "";
    
    private bool initialized = false;
    
    void Start()
    {
        // Generated from Rust start() method
        this.initialized = true;
        this.message = "Hello World";
    }
    
    public void MessageChanged()
    {
        // Generated sync callback
        Debug.Log($"Message changed to: {message}");
    }
}
```

### 5. File Splitting and Organization

**Input**: Raw C# code  
**Output**: Organized C# files  
**Tool**: File splitter with multiple strategies

The splitter organizes code into multiple files based on configurable strategies:

```rust
pub enum SplittingStrategy {
    ByClass,           // One file per class
    ByNamespace,       // One file per namespace  
    BySize(usize),     // Split when file exceeds size
    ByFunctionality,   // Group related functionality
}
```

**Output Structure**:
```
output/
├── MyBehaviour.cs              # Main behavior class
├── MyBehaviour_Generated.cs    # Generated helper methods
├── bindings/
│   ├── VRChatBindings.cs      # VRChat API bindings
│   ├── UnityBindings.cs       # Unity API bindings
│   └── SystemBindings.cs      # C# system bindings
└── utilities/
    ├── Collections.cs         # Collection helpers
    └── Extensions.cs          # Extension methods
```

### 6. Main Class Generation

**Input**: Split C# files  
**Output**: Complete UdonSharp project  
**Tool**: Main class generator with templates

The generator creates entry point classes and integration code:

```csharp
// Generated main class
using UdonSharp;
using UnityEngine;

public class MyBehaviour_Main : UdonSharpBehaviour
{
    private MyBehaviour_Core core;
    private MyBehaviour_Generated generated;
    
    void Start()
    {
        // Initialize components
        core = new MyBehaviour_Core();
        generated = new MyBehaviour_Generated();
        
        // Call Rust-generated start method
        core.w2us_start();
    }
    
    void Update()
    {
        if (core != null)
        {
            core.w2us_update();
        }
    }
}
```

## Configuration

### Pipeline Configuration

Configure the compilation pipeline through `udonsharp.toml`:

```toml
[compilation]
# Rust compilation settings
optimize_for_performance = true
generate_debug_info = false
target_udonsharp_version = "1.1"

# WASM optimization settings
wasm_opt_level = 3
wasm_shrink_level = 2
enable_bulk_memory = true
enable_sign_ext = true
enable_mutable_globals = true

# OOP analysis settings
detect_inheritance = true
group_related_methods = true
generate_property_accessors = true

# File splitting settings
splitting_strategy = "ByClass"
max_file_size = 50000  # bytes
generate_separate_bindings = true

# Code generation settings
use_udonsharp_attributes = true
generate_sync_callbacks = true
include_debug_comments = true
```

### Build Profiles

Different build profiles for different use cases:

```toml
[profile.dev]
# Development profile - fast compilation, debug info
optimize_for_performance = false
generate_debug_info = true
wasm_opt_level = 1
include_debug_comments = true

[profile.release]
# Release profile - optimized for production
optimize_for_performance = true
generate_debug_info = false
wasm_opt_level = 3
wasm_shrink_level = 2
strip_debug_symbols = true

[profile.test]
# Testing profile - includes test utilities
generate_test_helpers = true
include_mock_objects = true
enable_runtime_checks = true
```

## Performance Considerations

### WASM Size Optimization

The pipeline optimizes for VRChat's size constraints:

- **Dead code elimination**: Removes unused functions and data
- **Function inlining**: Reduces call overhead for small functions
- **Constant folding**: Evaluates constants at compile time
- **Memory layout optimization**: Minimizes memory usage

### UdonSharp Compatibility

Ensures generated code works within UdonSharp limitations:

- **No unsupported C# features**: Avoids generics, reflection, etc.
- **Proper networking**: Generates correct sync attributes and callbacks
- **Event handling**: Maps Rust events to UdonSharp events
- **Type safety**: Maintains type safety across the pipeline

### Build Performance

Optimizations for faster development iteration:

- **Incremental compilation**: Only recompiles changed modules
- **Parallel processing**: Uses multiple CPU cores when possible
- **Caching**: Caches intermediate results between builds
- **Smart rebuilding**: Detects when full rebuild is needed

## Debugging the Pipeline

### Verbose Output

Enable detailed logging to debug pipeline issues:

```bash
cargo udonsharp build --verbose
```

### Intermediate Files

Save intermediate files for inspection:

```bash
cargo udonsharp build --save-intermediates
```

This creates:
```
build/
├── intermediate.wasm      # Raw WASM output
├── optimized.wasm        # After wasm-opt
├── analysis.json         # OOP analysis results
├── raw_csharp.cs        # Before file splitting
└── splitting_plan.json   # File organization plan
```

### Pipeline Profiling

Profile pipeline performance:

```bash
cargo udonsharp build --profile
```

Output:
```
Pipeline Performance Report:
├── Rust compilation: 2.3s
├── WASM optimization: 0.8s  
├── OOP analysis: 0.4s
├── C# conversion: 1.2s
├── File splitting: 0.2s
└── Total: 4.9s
```

## Error Handling

### Common Pipeline Errors

**Rust compilation errors**:
- Syntax errors in Rust code
- Missing dependencies
- UdonSharp trait implementation issues

**WASM optimization errors**:
- Invalid WASM output from Rust
- Unsupported WASM features
- Memory layout issues

**C# generation errors**:
- OOP pattern detection failures
- UdonSharp compatibility issues
- File splitting conflicts

### Error Recovery

The pipeline includes error recovery mechanisms:

- **Graceful degradation**: Falls back to simpler strategies on errors
- **Partial compilation**: Generates what it can when some parts fail
- **Detailed diagnostics**: Provides actionable error messages
- **Rollback capability**: Can revert to last known good state

## Advanced Usage

### Custom Optimization Passes

Add custom WASM optimization passes:

```rust
use udonsharp_compiler::optimizer::OptimizationPass;

pub struct CustomPass;

impl OptimizationPass for CustomPass {
    fn run(&self, module: &mut WasmModule) -> Result<(), OptimizationError> {
        // Custom optimization logic
        Ok(())
    }
}

// Register in build.rs
fn main() {
    udonsharp_build::Builder::new()
        .add_optimization_pass(Box::new(CustomPass))
        .build();
}
```

### Custom File Splitting

Implement custom file splitting strategies:

```rust
use udonsharp_compiler::splitter::SplittingStrategy;

pub struct CustomSplitter;

impl SplittingStrategy for CustomSplitter {
    fn split(&self, code: &str) -> Vec<FileOutput> {
        // Custom splitting logic
        vec![]
    }
}
```

### Pipeline Extensions

Extend the pipeline with custom stages:

```rust
use udonsharp_compiler::pipeline::{PipelineStage, PipelineContext};

pub struct CustomStage;

impl PipelineStage for CustomStage {
    fn execute(&self, context: &mut PipelineContext) -> Result<(), PipelineError> {
        // Custom pipeline stage logic
        Ok(())
    }
}
```

## See Also

- [API Reference](api-reference.md) - Complete API documentation
- [Best Practices](best-practices.md) - Development guidelines
- [Performance Guide](performance.md) - Optimization strategies
- [Troubleshooting](troubleshooting.md) - Common issues and solutions