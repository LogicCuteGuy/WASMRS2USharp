# Multi-UdonBehaviour Support Design

## Overview

This design implements support for generating multiple UdonBehaviour classes from a single WASM module, enabling better code organization and separation of concerns in VRChat world development. The system introduces a behavior splitting mechanism that analyzes Rust code attributes and WASM exports to create distinct UdonSharp behaviors while maintaining efficient inter-behavior communication.

## Architecture

### Core Components

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   Rust Source   │───▶│  WASM Compiler   │───▶│  Behaviour Splitter │
│  with Attributes│    │                  │    │                     │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
                                                           │
                                                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Generated Output                                 │
├─────────────────┬─────────────────┬─────────────────┬──────────────┤
│ UdonBehaviour1  │ UdonBehaviour2  │ UdonBehaviour3  │ SharedRuntime│
│     .cs         │     .cs         │     .cs         │     .cs      │
└─────────────────┴─────────────────┴─────────────────┴──────────────┘
```

### Attribute System

The system uses Rust attributes to mark functions for UdonBehaviour generation:

```rust
#[udon_behaviour(name = "PlayerManager")]
pub fn player_manager_start() {
    // This becomes PlayerManager.cs with Start() method
}

#[udon_behaviour(name = "UIController", events = ["Update", "OnTriggerEnter"])]
pub fn ui_controller_update() {
    // This becomes UIController.cs with Update() and OnTriggerEnter() methods
}

// Shared functions without attributes go to SharedRuntime
pub fn calculate_distance(a: Vector3, b: Vector3) -> f32 {
    // Available to all UdonBehaviour classes
}
```

## Components and Interfaces

### 1. Behaviour Splitter (`BehaviourSplitter`)

**Location**: `crates/wasm2usharp-enhanced/src/splitter.rs`

**Responsibilities**:
- Parse WASM module to identify exported functions
- Analyze function call graphs and dependencies
- Split WASM into logical UdonBehaviour units
- Generate inter-behavior communication code

**Key Methods**:
```rust
pub struct BehaviourSplitter {
    wasm_module: WasmModule,
    attribute_parser: AttributeParser,
    dependency_analyzer: DependencyAnalyzer,
}

impl BehaviourSplitter {
    pub fn split_behaviors(&self) -> Result<Vec<BehaviorUnit>, SplitterError>;
    pub fn analyze_dependencies(&self) -> DependencyGraph;
    pub fn generate_shared_runtime(&self) -> SharedRuntimeCode;
}
```

### 2. Attribute Parser (`AttributeParser`)

**Location**: `crates/udonsharp-core/src/attributes.rs`

**Responsibilities**:
- Parse `#[udon_behaviour]` attributes from Rust source
- Extract behavior configuration (name, events, dependencies)
- Validate attribute parameters

**Key Methods**:
```rust
pub struct UdonBehaviourAttribute {
    pub name: String,
    pub events: Vec<UnityEvent>,
    pub dependencies: Vec<String>,
    pub auto_sync: bool,
}

impl AttributeParser {
    pub fn parse_attributes(&self, source: &str) -> Vec<UdonBehaviourAttribute>;
    pub fn validate_attribute(&self, attr: &UdonBehaviourAttribute) -> Result<(), ValidationError>;
}
```

### 3. Dependency Analyzer (`DependencyAnalyzer`)

**Location**: `crates/wasm2usharp-enhanced/src/analyzer.rs`

**Responsibilities**:
- Build function call graphs
- Detect circular dependencies
- Determine optimal code sharing strategies
- Generate initialization order

**Key Methods**:
```rust
pub struct DependencyGraph {
    pub nodes: HashMap<String, BehaviorNode>,
    pub edges: Vec<DependencyEdge>,
    pub shared_functions: HashSet<String>,
}

impl DependencyAnalyzer {
    pub fn build_call_graph(&self, wasm: &WasmModule) -> CallGraph;
    pub fn detect_cycles(&self, graph: &CallGraph) -> Vec<CyclicDependency>;
    pub fn optimize_sharing(&self, behaviors: &[BehaviorUnit]) -> SharingStrategy;
}
```

### 4. Code Generator Extensions

**Location**: `crates/wasm2usharp-enhanced/src/file_generator.rs`

**Enhanced Responsibilities**:
- Generate multiple UdonBehaviour class files
- Create SharedRuntime utility class
- Generate inter-behavior communication methods
- Create Unity prefab configurations

## Data Models

### BehaviorUnit

```rust
pub struct BehaviorUnit {
    pub name: String,
    pub entry_function: String,
    pub unity_events: Vec<UnityEvent>,
    pub local_functions: HashSet<String>,
    pub shared_dependencies: HashSet<String>,
    pub inter_behavior_calls: Vec<InterBehaviorCall>,
}
```

### InterBehaviorCall

```rust
pub struct InterBehaviorCall {
    pub source_behavior: String,
    pub target_behavior: String,
    pub function_name: String,
    pub call_type: CallType, // Direct, Event, Network
}

pub enum CallType {
    Direct,           // Same GameObject
    Event,            // Cross-GameObject via SendCustomEvent
    Network,          // Networked call
}
```

### SharedRuntimeCode

```rust
pub struct SharedRuntimeCode {
    pub utility_functions: Vec<Function>,
    pub shared_data: Vec<SharedVariable>,
    pub initialization_code: String,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, Error)]
pub enum MultiBehaviorError {
    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },
    
    #[error("Invalid attribute configuration: {message}")]
    InvalidAttribute { message: String },
    
    #[error("Function {function} not found in behavior {behavior}")]
    MissingFunction { function: String, behavior: String },
    
    #[error("Conflicting behavior names: {name}")]
    ConflictingNames { name: String },
}
```

### Error Recovery

- **Circular Dependencies**: Suggest refactoring or provide automatic cycle breaking
- **Missing Functions**: Generate stub implementations with warnings
- **Name Conflicts**: Auto-generate unique names with user notification
- **Invalid Attributes**: Provide detailed validation messages with suggestions

## Testing Strategy

### Unit Tests

1. **Attribute Parsing Tests**
   - Valid attribute configurations
   - Invalid attribute handling
   - Edge cases (empty names, invalid events)

2. **Dependency Analysis Tests**
   - Simple linear dependencies
   - Complex dependency graphs
   - Circular dependency detection
   - Optimization algorithm validation

3. **Code Generation Tests**
   - Single behavior generation
   - Multi-behavior generation
   - SharedRuntime generation
   - Inter-behavior communication code

### Integration Tests

1. **End-to-End Pipeline Tests**
   - Complete Rust → WASM → Multi-UdonBehaviour flow
   - Real-world example projects
   - Performance benchmarking

2. **Unity Integration Tests**
   - Generated prefab validation
   - Runtime behavior verification
   - Inter-behavior communication testing

### Performance Tests

1. **Compilation Performance**
   - Large projects with many behaviors
   - Complex dependency graphs
   - Memory usage during compilation

2. **Runtime Performance**
   - Inter-behavior call overhead
   - SharedRuntime efficiency
   - Memory footprint of generated code

## Implementation Phases

### Phase 1: Core Infrastructure
- Implement BehaviourSplitter basic functionality
- Add AttributeParser for `#[udon_behaviour]`
- Extend WASM analysis capabilities

### Phase 2: Dependency Management
- Implement DependencyAnalyzer
- Add circular dependency detection
- Create optimization algorithms

### Phase 3: Code Generation
- Extend FileGenerator for multi-behavior output
- Implement SharedRuntime generation
- Add inter-behavior communication

### Phase 4: Unity Integration
- Generate Unity prefab configurations
- Add component dependency management
- Implement initialization ordering

### Phase 5: Optimization & Polish
- Performance optimizations
- Advanced sharing strategies
- Developer experience improvements