# Design Document

## Overview

The Standard Multi-Behavior Pattern provides a clean, type-safe approach for creating complex VRChat worlds with multiple interacting UdonBehaviour systems. This design enables developers to define multiple `#[derive(UdonBehaviour)]` structs in a single Rust WASM module, with automatic code generation that produces proper UdonSharp C# classes following Unity conventions.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Rust WASM Module                             │
├─────────────────┬─────────────────┬─────────────────────────────┤
│  Behavior A     │  Behavior B     │       Behavior C            │
│                 │                 │                             │
│ #[derive(       │ #[derive(       │ #[derive(UdonBehaviour)]    │
│  UdonBehaviour)]│  UdonBehaviour)]│ #[udon_sync_mode(Manual)]   │
│                 │                 │                             │
│ impl            │ impl            │ impl UdonBehaviour          │
│ UdonBehaviour   │ UdonBehaviour   │                             │
└─────────────────┴─────────────────┴─────────────────────────────┘
                                │
                   ┌─────────────────────────┐
                   │   Compilation Pipeline  │
                   │                         │
                   │ - Struct Analysis       │
                   │ - Trait Validation      │
                   │ - Dependency Resolution │
                   │ - Code Generation       │
                   └─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
│  BehaviorA.cs  │    │  BehaviorB.cs   │    │  BehaviorC.cs   │
│                │    │                 │    │                 │
│ - Unity Events │    │ - Unity Events  │    │ - Unity Events  │
│ - Custom Events│    │ - Custom Events │    │ - Custom Events │
│ - GameObject   │    │ - GameObject    │    │ - GameObject    │
│   References   │    │   References    │    │   References    │
└────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                   ┌─────────────────────────┐
                   │   SharedRuntime.cs      │
                   │                         │
                   │ - Shared Functions      │
                   │ - Common Data Types     │
                   │ - Utility Methods       │
                   │ - Static State          │
                   └─────────────────────────┘
```

### Component Architecture

#### 1. Struct Analysis Engine

**Purpose**: Analyzes Rust structs and validates UdonBehaviour pattern compliance.

**Components**:
- **Struct Parser**: Extracts struct definitions and attributes
- **Trait Validator**: Ensures UdonBehaviour trait implementation
- **Attribute Processor**: Handles `#[udon_public]`, `#[udon_sync]`, etc.
- **Dependency Analyzer**: Maps inter-behavior relationships

**Key Algorithms**:
```rust
struct StructAnalyzer {
    parsed_structs: Vec<UdonBehaviourStruct>,
    trait_implementations: HashMap<String, TraitImpl>,
    field_attributes: HashMap<String, Vec<FieldAttribute>>,
    dependencies: DependencyGraph,
}

impl StructAnalyzer {
    fn analyze_module(&mut self, module: &syn::ItemMod) -> AnalysisResult {
        // 1. Extract all structs with #[derive(UdonBehaviour)]
        // 2. Validate UdonBehaviour trait implementations
        // 3. Process field attributes
        // 4. Build dependency graph
        // 5. Detect circular dependencies
    }
}
```

#### 2. Code Generation Engine

**Purpose**: Generates UdonSharp C# classes from analyzed Rust structs.

**Components**:
- **Class Generator**: Creates C# class structure
- **Method Generator**: Converts trait methods to C# methods
- **Field Generator**: Handles field attributes and types
- **Event Generator**: Creates custom event handlers
- **SharedRuntime Generator**: Extracts shared functionality

**Key Algorithms**:
```rust
struct CodeGenerator {
    template_engine: TemplateEngine,
    type_mapper: RustToCSharpTypeMapper,
    attribute_mapper: AttributeMapper,
    shared_extractor: SharedFunctionExtractor,
}

impl CodeGenerator {
    fn generate_behavior_class(&self, struct_info: &UdonBehaviourStruct) -> String {
        // 1. Generate class header with attributes
        // 2. Generate fields with proper C# attributes
        // 3. Generate Unity event methods
        // 4. Generate custom event handlers
        // 5. Generate GameObject reference management
    }
    
    fn generate_shared_runtime(&self, shared_items: &SharedItems) -> String {
        // 1. Extract shared functions
        // 2. Generate shared data types
        // 3. Create utility methods
        // 4. Handle static state management
    }
}
```

#### 3. Inter-Behavior Communication System

**Purpose**: Manages communication between UdonBehaviour instances.

**Design Pattern**: Observer pattern with GameObject-based event routing

**Components**:
- **Event Router**: Routes custom events between behaviors
- **Reference Manager**: Manages GameObject references
- **Parameter Handler**: Handles event parameter passing
- **Validation Layer**: Ensures safe inter-behavior communication

**Communication Flow**:
```
Behavior A → GameObject.Find("BehaviorB") → Reference Storage
         ↓
Behavior A → SendCustomEvent("EventName") → Event Router
         ↓
Event Router → Parameter Validation → Behavior B.EventHandler()
```

#### 4. Network Synchronization Layer

**Purpose**: Handles VRChat networking for synchronized behaviors.

**Components**:
- **Sync Field Manager**: Manages `#[udon_sync]` fields
- **Master Client Validator**: Ensures only master can modify sync data
- **Serialization Controller**: Handles RequestSerialization() calls
- **Deserialization Handler**: Processes incoming network data

**Synchronization Pattern**:
```rust
// Generated C# pattern
public class PlayerManager : UdonSharpBehaviour {
    [UdonSynced] public int totalPlayers;
    
    public void UpdatePlayerCount(int newCount) {
        if (Networking.IsMaster) {
            totalPlayers = newCount;
            RequestSerialization();
        }
    }
    
    public override void OnDeserialization() {
        // Update UI or notify other behaviors
        NotifyPlayerCountChanged();
    }
}
```

## Components and Interfaces

### Core Interfaces

#### IUdonBehaviourStruct
```rust
trait IUdonBehaviourStruct {
    fn get_struct_name(&self) -> &str;
    fn get_fields(&self) -> &[StructField];
    fn get_methods(&self) -> &[StructMethod];
    fn get_attributes(&self) -> &[StructAttribute];
    fn has_networking(&self) -> bool;
}
```

#### ICodeGenerator
```rust
trait ICodeGenerator {
    fn generate_class(&self, struct_info: &dyn IUdonBehaviourStruct) -> GeneratedClass;
    fn generate_shared_runtime(&self, shared_items: &SharedItems) -> GeneratedClass;
    fn validate_generated_code(&self, code: &str) -> ValidationResult;
}
```

#### IInterBehaviorCommunication
```rust
trait IInterBehaviorCommunication {
    fn register_behavior(&mut self, name: &str, behavior: &dyn IUdonBehaviourStruct);
    fn resolve_dependencies(&self) -> DependencyGraph;
    fn generate_communication_code(&self) -> Vec<CommunicationMethod>;
}
```

### Data Models

#### UdonBehaviourStruct
```rust
#[derive(Debug, Clone)]
pub struct UdonBehaviourStruct {
    pub name: String,
    pub fields: Vec<StructField>,
    pub methods: Vec<StructMethod>,
    pub attributes: Vec<StructAttribute>,
    pub trait_impl: Option<UdonBehaviourTraitImpl>,
    pub dependencies: Vec<String>,
}
```

#### StructField
```rust
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub field_type: RustType,
    pub visibility: Visibility,
    pub attributes: Vec<FieldAttribute>,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FieldAttribute {
    UdonPublic,
    UdonSync,
    UdonSyncMode(SyncMode),
}
```

#### GeneratedClass
```rust
#[derive(Debug, Clone)]
pub struct GeneratedClass {
    pub class_name: String,
    pub namespace: Option<String>,
    pub using_statements: Vec<String>,
    pub class_attributes: Vec<String>,
    pub fields: Vec<GeneratedField>,
    pub methods: Vec<GeneratedMethod>,
    pub custom_events: Vec<CustomEventHandler>,
}
```

## Data Models

### Type Mapping System

#### Rust to C# Type Mapping
```rust
pub struct TypeMapper {
    mappings: HashMap<String, CSharpType>,
}

impl TypeMapper {
    fn new() -> Self {
        let mut mappings = HashMap::new();
        mappings.insert("i32".to_string(), CSharpType::Int);
        mappings.insert("f32".to_string(), CSharpType::Float);
        mappings.insert("String".to_string(), CSharpType::String);
        mappings.insert("bool".to_string(), CSharpType::Bool);
        mappings.insert("Option<unity::GameObject>".to_string(), CSharpType::GameObject);
        mappings.insert("Vec<T>".to_string(), CSharpType::Array);
        mappings.insert("HashMap<K,V>".to_string(), CSharpType::Dictionary);
        Self { mappings }
    }
}
```

#### Attribute Mapping
```rust
pub struct AttributeMapper {
    field_mappings: HashMap<FieldAttribute, Vec<String>>,
    class_mappings: HashMap<ClassAttribute, Vec<String>>,
}

impl AttributeMapper {
    fn map_field_attribute(&self, attr: &FieldAttribute) -> Vec<String> {
        match attr {
            FieldAttribute::UdonPublic => vec!["[SerializeField]".to_string()],
            FieldAttribute::UdonSync => vec!["[UdonSynced]".to_string()],
            FieldAttribute::UdonSyncMode(mode) => vec![format!("[UdonSyncMode({})]", mode)],
        }
    }
}
```

### Shared Runtime Extraction

#### Shared Function Detection
```rust
pub struct SharedFunctionExtractor {
    function_usage: HashMap<String, Vec<String>>, // function -> behaviors using it
    shared_threshold: usize, // minimum usage count to be shared
}

impl SharedFunctionExtractor {
    fn extract_shared_functions(&self, behaviors: &[UdonBehaviourStruct]) -> SharedItems {
        let mut shared_functions = Vec::new();
        let mut shared_types = Vec::new();
        
        // Find functions used by multiple behaviors
        for (func_name, users) in &self.function_usage {
            if users.len() >= self.shared_threshold {
                shared_functions.push(func_name.clone());
            }
        }
        
        // Extract shared data types
        shared_types.extend(self.extract_shared_types(behaviors));
        
        SharedItems {
            functions: shared_functions,
            types: shared_types,
            constants: self.extract_shared_constants(behaviors),
        }
    }
}
```

## Error Handling

### Error Categories

#### Compilation Errors
```rust
#[derive(Debug, Clone)]
pub enum CompilationError {
    MissingTraitImplementation {
        struct_name: String,
        missing_methods: Vec<String>,
    },
    InvalidFieldAttribute {
        struct_name: String,
        field_name: String,
        attribute: String,
        reason: String,
    },
    CircularDependency {
        cycle: Vec<String>,
    },
    UnsupportedType {
        rust_type: String,
        suggested_alternatives: Vec<String>,
    },
    InvalidStructName {
        name: String,
        reason: String,
    },
}
```

#### Runtime Validation
```rust
pub struct RuntimeValidator {
    validation_rules: Vec<Box<dyn ValidationRule>>,
}

trait ValidationRule {
    fn validate(&self, generated_code: &GeneratedClass) -> ValidationResult;
    fn get_error_message(&self) -> String;
}

impl RuntimeValidator {
    fn validate_generated_code(&self, code: &GeneratedClass) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        for rule in &self.validation_rules {
            match rule.validate(code) {
                ValidationResult::Error(msg) => errors.push(msg),
                ValidationResult::Warning(msg) => warnings.push(msg),
                ValidationResult::Ok => {}
            }
        }
        
        if errors.is_empty() {
            ValidationResult::Ok
        } else {
            ValidationResult::Error(errors.join("; "))
        }
    }
}
```

## Testing Strategy

### Unit Testing Approach

#### Struct Analysis Testing
```rust
#[cfg(test)]
mod struct_analysis_tests {
    use super::*;
    
    #[test]
    fn test_multiple_struct_detection() {
        let rust_code = r#"
            #[derive(UdonBehaviour)]
            pub struct PlayerManager { /* ... */ }
            
            #[derive(UdonBehaviour)]
            pub struct UIController { /* ... */ }
        "#;
        
        let analyzer = StructAnalyzer::new();
        let result = analyzer.analyze_code(rust_code);
        
        assert_eq!(result.structs.len(), 2);
        assert!(result.should_generate_shared_runtime());
    }
    
    #[test]
    fn test_trait_implementation_validation() {
        let rust_code = r#"
            #[derive(UdonBehaviour)]
            pub struct TestBehavior {
                field: i32,
            }
            // Missing UdonBehaviour trait implementation
        "#;
        
        let analyzer = StructAnalyzer::new();
        let result = analyzer.analyze_code(rust_code);
        
        assert!(result.has_errors());
        assert!(result.errors.iter().any(|e| matches!(e, CompilationError::MissingTraitImplementation { .. })));
    }
}
```

#### Code Generation Testing
```rust
#[cfg(test)]
mod code_generation_tests {
    use super::*;
    
    #[test]
    fn test_field_attribute_generation() {
        let field = StructField {
            name: "playerCount".to_string(),
            field_type: RustType::I32,
            attributes: vec![FieldAttribute::UdonSync],
            visibility: Visibility::Private,
            default_value: Some("0".to_string()),
        };
        
        let generator = CodeGenerator::new();
        let generated = generator.generate_field(&field);
        
        assert!(generated.contains("[UdonSynced]"));
        assert!(generated.contains("public int playerCount"));
    }
    
    #[test]
    fn test_custom_event_generation() {
        let method = StructMethod {
            name: "on_player_count_changed".to_string(),
            attributes: vec![MethodAttribute::UdonEvent("OnPlayerCountChanged".to_string())],
            parameters: vec![],
            return_type: RustType::Unit,
        };
        
        let generator = CodeGenerator::new();
        let generated = generator.generate_method(&method);
        
        assert!(generated.contains("public void OnPlayerCountChanged()"));
    }
}
```

### Integration Testing

#### End-to-End Compilation Testing
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_full_multi_behavior_compilation() {
        let test_project = create_test_project_with_multi_behaviors();
        let compiler = StandardMultiBehaviorCompiler::new();
        
        let result = compiler.compile_project(&test_project);
        
        assert!(result.is_ok());
        let output = result.unwrap();
        
        // Verify correct number of generated files
        assert_eq!(output.behavior_classes.len(), 3);
        assert!(output.shared_runtime.is_some());
        
        // Verify inter-behavior communication
        assert!(output.has_custom_events());
        assert!(output.has_gameobject_references());
        
        // Verify network synchronization
        assert!(output.has_synchronized_fields());
    }
}
```

This design provides a comprehensive foundation for implementing the standard multi-behavior pattern with proper separation of concerns, robust error handling, and extensive testing coverage.