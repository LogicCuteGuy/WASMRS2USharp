//! Build script integration for UdonSharp projects
//! 
//! This crate provides utilities for integrating UdonSharp compilation
//! into Cargo build scripts (build.rs files).

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use udonsharp_bindings::UniversalBindingPipeline;

/// Configuration for UdonSharp build integration
#[derive(Debug, Clone)]
pub struct UdonSharpBuild {
    /// Directories to scan for .asmdef files
    pub asmdef_scan_dirs: Vec<PathBuf>,
    /// Output directory for generated bindings
    pub bindings_output_dir: PathBuf,
    /// Whether to automatically generate bindings during build
    pub auto_generate_bindings: bool,
    /// Whether to rerun build script when .asmdef files change
    pub watch_asmdef_files: bool,
    /// Whether this is a workspace member
    pub is_workspace_member: bool,
    /// Workspace root directory (if applicable)
    pub workspace_root: Option<PathBuf>,
}

impl Default for UdonSharpBuild {
    fn default() -> Self {
        Self {
            asmdef_scan_dirs: Vec::new(),
            bindings_output_dir: PathBuf::from("generated"),
            auto_generate_bindings: true,
            watch_asmdef_files: true,
            is_workspace_member: false,
            workspace_root: None,
        }
    }
}

impl UdonSharpBuild {
    /// Create a new UdonSharp build configuration
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a directory to scan for .asmdef files
    pub fn add_asmdef_scan_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.asmdef_scan_dirs.push(dir.into());
        self
    }
    
    /// Set the output directory for generated bindings
    pub fn bindings_output_dir<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.bindings_output_dir = dir.into();
        self
    }
    
    /// Enable or disable automatic binding generation
    pub fn auto_generate_bindings(mut self, enabled: bool) -> Self {
        self.auto_generate_bindings = enabled;
        self
    }
    
    /// Enable or disable watching .asmdef files for changes
    pub fn watch_asmdef_files(mut self, enabled: bool) -> Self {
        self.watch_asmdef_files = enabled;
        self
    }
    
    /// Detect Unity project paths automatically
    pub fn with_unity_project_detection(mut self) -> Self {
        let unity_paths = detect_unity_paths();
        for path in unity_paths {
            self.asmdef_scan_dirs.push(path);
        }
        self
    }
    
    /// Configure for workspace member
    pub fn as_workspace_member(mut self) -> Self {
        self.is_workspace_member = true;
        self.workspace_root = find_workspace_root();
        
        // If we're in a workspace, adjust paths relative to workspace root
        if let Some(ref workspace_root) = self.workspace_root {
            self.bindings_output_dir = workspace_root.join("generated");
        }
        
        self
    }
    
    /// Set workspace root manually
    pub fn with_workspace_root<P: Into<PathBuf>>(mut self, root: P) -> Self {
        self.workspace_root = Some(root.into());
        self.is_workspace_member = true;
        self
    }
    
    /// Run the UdonSharp build process
    pub fn run(&self) -> Result<()> {
        println!("cargo:rerun-if-changed=build.rs");
        
        // Set up output directory
        if !self.bindings_output_dir.exists() {
            fs::create_dir_all(&self.bindings_output_dir)
                .with_context(|| format!("Failed to create bindings output directory: {:?}", self.bindings_output_dir))?;
        }
        
        // Watch .asmdef files for changes if enabled
        if self.watch_asmdef_files {
            self.setup_asmdef_watching()?;
        }
        
        // Generate bindings if enabled
        if self.auto_generate_bindings {
            self.generate_bindings()?;
        }
        
        // Set environment variables for the main build
        println!("cargo:rustc-env=UDONSHARP_BINDINGS_DIR={}", self.bindings_output_dir.display());
        
        Ok(())
    }
    
    /// Generate API bindings from discovered .asmdef files
    fn generate_bindings(&self) -> Result<()> {
        if self.asmdef_scan_dirs.is_empty() {
            println!("cargo:warning=No .asmdef scan directories configured. Skipping binding generation.");
            return Ok(());
        }
        
        let mut pipeline = UniversalBindingPipeline::new(
            self.bindings_output_dir.to_string_lossy().to_string()
        );
        
        // Add all scan directories
        for dir in &self.asmdef_scan_dirs {
            if dir.exists() {
                pipeline.add_asmdef_directory(dir.to_string_lossy().to_string());
                println!("cargo:warning=Added .asmdef scan directory: {:?}", dir);
            } else {
                println!("cargo:warning=.asmdef scan directory does not exist: {:?}", dir);
            }
        }
        
        // Generate bindings
        match pipeline.scan_and_generate_all_bindings() {
            Ok(_) => {
                println!("cargo:warning=Successfully generated UdonSharp API bindings");
            }
            Err(e) => {
                println!("cargo:warning=Failed to generate UdonSharp API bindings: {}", e);
                // Don't fail the build, just warn
            }
        }
        
        Ok(())
    }
    
    /// Set up watching for .asmdef file changes
    fn setup_asmdef_watching(&self) -> Result<()> {
        for dir in &self.asmdef_scan_dirs {
            if dir.exists() {
                // Tell Cargo to rerun if any .asmdef files change in this directory
                println!("cargo:rerun-if-changed={}", dir.display());
                
                // Recursively watch for .asmdef files
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "asmdef") {
                            println!("cargo:rerun-if-changed={}", path.display());
                        } else if path.is_dir() {
                            // Recursively watch subdirectories
                            self.watch_directory_recursive(&path)?;
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Recursively watch a directory for .asmdef files
    fn watch_directory_recursive(&self, dir: &Path) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "asmdef") {
                    println!("cargo:rerun-if-changed={}", path.display());
                } else if path.is_dir() {
                    self.watch_directory_recursive(&path)?;
                }
            }
        }
        Ok(())
    }
}

/// Detect common Unity project paths
fn detect_unity_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    // Common Unity project paths relative to the Rust project
    let potential_paths = [
        "Library/PackageCache",
        "Assets",
        "Packages",
        "../Library/PackageCache",
        "../Assets", 
        "../Packages",
        "../../Library/PackageCache",
        "../../Assets",
        "../../Packages",
        "../../../Library/PackageCache",
        "../../../Assets",
        "../../../Packages",
    ];
    
    for path in &potential_paths {
        let path_buf = PathBuf::from(path);
        if path_buf.exists() {
            paths.push(path_buf);
        }
    }
    
    paths
}

/// Convenience function to set up UdonSharp build with Unity project detection
pub fn setup_udonsharp_build() -> Result<()> {
    UdonSharpBuild::new()
        .with_unity_project_detection()
        .run()
}

/// Convenience function to set up UdonSharp build with custom configuration
pub fn setup_udonsharp_build_with_config<F>(config_fn: F) -> Result<()>
where
    F: FnOnce(UdonSharpBuild) -> UdonSharpBuild,
{
    let config = config_fn(UdonSharpBuild::new());
    config.run()
}

/// Check if we're in a build script environment
pub fn is_build_script() -> bool {
    env::var("CARGO_MANIFEST_DIR").is_ok()
}

/// Get the manifest directory (only available in build scripts)
pub fn get_manifest_dir() -> Option<PathBuf> {
    env::var("CARGO_MANIFEST_DIR").ok().map(PathBuf::from)
}

/// Get the output directory (only available in build scripts)
pub fn get_out_dir() -> Option<PathBuf> {
    env::var("OUT_DIR").ok().map(PathBuf::from)
}

/// Find the workspace root by looking for a Cargo.toml with [workspace]
fn find_workspace_root() -> Option<PathBuf> {
    let manifest_dir = get_manifest_dir()?;
    let mut current_dir = manifest_dir.as_path();
    
    loop {
        let cargo_toml = current_dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return Some(current_dir.to_path_buf());
                }
            }
        }
        
        match current_dir.parent() {
            Some(parent) => current_dir = parent,
            None => break,
        }
    }
    
    None
}

/// Check if current project is a workspace member
pub fn is_workspace_member() -> bool {
    find_workspace_root().is_some()
}

/// Create Cargo.toml content for a project
fn create_project_cargo_toml(project_name: &str, template: &ProjectTemplate) -> Result<String> {
    let base_dependencies = r#"udonsharp-core = { git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }
udonsharp-macros = { git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }
udonsharp-bindings = { git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
log = "0.4""#;
    
    let additional_deps = match template {
        ProjectTemplate::Advanced | ProjectTemplate::Networking | ProjectTemplate::GameLogic => {
            "\nserde = { version = \"1.0\", features = [\"derive\"] }\nserde_json = \"1.0\""
        }
        ProjectTemplate::UI => {
            "\nserde = { version = \"1.0\", features = [\"derive\"] }\nserde_json = \"1.0\""
        }
        ProjectTemplate::Physics => {
            "\nrand = \"0.8\""
        }
        ProjectTemplate::Audio => {
            "\nserde = { version = \"1.0\", features = [\"derive\"] }"
        }
        _ => ""
    };
    
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "{}"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
{}{}

[build-dependencies]
udonsharp-build = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}

[dependencies.web-sys]
version = "0.3"
features = [
  "console",
]

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
debug = false
panic = "abort"
"#,
        project_name,
        template.description(),
        base_dependencies,
        additional_deps
    );
    
    Ok(cargo_toml)
}

/// Create workspace Cargo.toml content
fn create_workspace_cargo_toml(project_name: &str, template: &ProjectTemplate) -> Result<String> {
    let cargo_toml = format!(
        r#"[workspace]
members = [
    "src"
]
resolver = "2"

[workspace.dependencies]
udonsharp-core = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-macros = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
udonsharp-bindings = {{ git = "https://github.com/vrchat-community/rust-udonsharp", branch = "main" }}
wasm-bindgen = "0.2"
console_error_panic_hook = "0.1"
log = "0.4"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
rand = "0.8"

[profile.release]
opt-level = "s"
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
opt-level = 1
debug = false
panic = "abort"
"#
    );
    
    Ok(cargo_toml)
}

/// Create build.rs content for a template
fn create_build_rs_content(
    template: &ProjectTemplate,
    vrc_sdk_path: &Option<String>,
    unity_project_path: &Option<String>,
) -> Result<String> {
    let mut content = String::from(
        r#"//! Build script for UdonSharp project
//! 
//! This build script automatically generates API bindings from Unity and VRChat
//! .asmdef files and sets up the build environment for UdonSharp compilation.

use udonsharp_build::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up UdonSharp build with automatic Unity project detection
    setup_udonsharp_build_with_config(|mut config| {
        config = config
            .with_unity_project_detection()
            .bindings_output_dir("generated")
            .auto_generate_bindings(true)
            .watch_asmdef_files(true);
"#,
    );
    
    if let Some(vrc_path) = vrc_sdk_path {
        content.push_str(&format!(
            r#"        
        // Add custom VRChat SDK path
        config = config.add_asmdef_scan_dir("{}");
"#,
            vrc_path
        ));
    }
    
    if let Some(unity_path) = unity_project_path {
        content.push_str(&format!(
            r#"        
        // Add custom Unity project path
        config = config.add_asmdef_scan_dir("{}/Library/PackageCache");
        config = config.add_asmdef_scan_dir("{}/Assets");
        config = config.add_asmdef_scan_dir("{}/Packages");
"#,
            unity_path, unity_path, unity_path
        ));
    }
    
    content.push_str(
        r#"        
        config
    })?;
    
    Ok(())
}
"#,
    );
    
    Ok(content)
}

/// Set up UdonSharp build with automatic workspace detection
pub fn setup_udonsharp_build_auto() -> Result<()> {
    let mut config = UdonSharpBuild::new();
    
    if is_workspace_member() {
        config = config.as_workspace_member();
    }
    
    config
        .with_unity_project_detection()
        .run()
}

/// Available project templates
#[derive(Debug, Clone)]
pub enum ProjectTemplate {
    Basic,
    Advanced,
    Networking,
    GameLogic,
    UI,
    Physics,
    Audio,
    Custom(String),
}

impl ProjectTemplate {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "advanced" => Ok(Self::Advanced),
            "networking" => Ok(Self::Networking),
            "game-logic" | "gamelogic" => Ok(Self::GameLogic),
            "ui" => Ok(Self::UI),
            "physics" => Ok(Self::Physics),
            "audio" => Ok(Self::Audio),
            custom => Ok(Self::Custom(custom.to_string())),
        }
    }
    
    pub fn name(&self) -> &str {
        match self {
            Self::Basic => "basic",
            Self::Advanced => "advanced",
            Self::Networking => "networking",
            Self::GameLogic => "game-logic",
            Self::UI => "ui",
            Self::Physics => "physics",
            Self::Audio => "audio",
            Self::Custom(name) => name,
        }
    }
    
    pub fn description(&self) -> &str {
        match self {
            Self::Basic => "Basic UdonSharp project with simple player interaction",
            Self::Advanced => "Advanced project with networking, UI, and complex game logic",
            Self::Networking => "Networking-focused project with multiplayer features",
            Self::GameLogic => "Game logic template with state management and events",
            Self::UI => "UI-focused project with canvas management and interactions",
            Self::Physics => "Physics-based project with rigidbodies and collisions",
            Self::Audio => "Audio-focused project with sound management and music",
            Self::Custom(name) => "Custom template",
        }
    }
    
    pub fn features(&self) -> Vec<&str> {
        match self {
            Self::Basic => vec!["Player events", "Basic interactions", "Debug logging"],
            Self::Advanced => vec!["Networking", "UI management", "Configuration", "State management"],
            Self::Networking => vec!["Network sync", "Player positions", "Chat system", "Game events"],
            Self::GameLogic => vec!["State machines", "Event system", "Game rules", "Score tracking"],
            Self::UI => vec!["Canvas management", "Button interactions", "Text updates", "Animations"],
            Self::Physics => vec!["Rigidbody interactions", "Collision detection", "Force application", "Joints"],
            Self::Audio => vec!["Audio sources", "Music playback", "Sound effects", "Volume control"],
            Self::Custom(_) => vec!["Custom features"],
        }
    }
}

/// Project scaffolding configuration
#[derive(Debug, Clone)]
pub struct ScaffoldingConfig {
    pub template: ProjectTemplate,
    pub project_name: String,
    pub workspace: bool,
    pub include_examples: bool,
    pub include_tests: bool,
    pub include_documentation: bool,
    pub vrc_sdk_path: Option<String>,
    pub unity_project_path: Option<String>,
    pub custom_features: Vec<String>,
}

impl Default for ScaffoldingConfig {
    fn default() -> Self {
        Self {
            template: ProjectTemplate::Basic,
            project_name: "my-udonsharp-project".to_string(),
            workspace: false,
            include_examples: true,
            include_tests: false,
            include_documentation: true,
            vrc_sdk_path: None,
            unity_project_path: None,
            custom_features: Vec::new(),
        }
    }
}

/// Create a new UdonSharp project template
pub fn create_project_template(
    project_dir: &Path,
    template_name: &str,
    workspace: bool,
) -> Result<()> {
    let template = ProjectTemplate::from_str(template_name)?;
    let config = ScaffoldingConfig {
        template,
        project_name: project_dir.file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("udonsharp-project"))
            .to_string_lossy()
            .to_string(),
        workspace,
        ..Default::default()
    };
    
    create_project_with_config(project_dir, &config)
}

/// Create a new UdonSharp project with detailed configuration
pub fn create_project_with_config(
    project_dir: &Path,
    config: &ScaffoldingConfig,
) -> Result<()> {
    if !project_dir.exists() {
        fs::create_dir_all(project_dir)
            .with_context(|| format!("Failed to create project directory: {:?}", project_dir))?;
    }
    
    match &config.template {
        ProjectTemplate::Basic => create_basic_template(project_dir, config)?,
        ProjectTemplate::Advanced => create_advanced_template(project_dir, config)?,
        ProjectTemplate::Networking => create_networking_template(project_dir, config)?,
        ProjectTemplate::GameLogic => create_game_logic_template(project_dir, config)?,
        ProjectTemplate::UI => create_ui_template(project_dir, config)?,
        ProjectTemplate::Physics => create_physics_template(project_dir, config)?,
        ProjectTemplate::Audio => create_audio_template(project_dir, config)?,
        ProjectTemplate::Custom(name) => create_custom_template(project_dir, config, name)?,
    }
    
    // Create common files
    create_common_files(project_dir, config)?;
    
    // Create examples if requested
    if config.include_examples {
        create_example_files(project_dir, config)?;
    }
    
    // Create tests if requested
    if config.include_tests {
        create_test_files(project_dir, config)?;
    }
    
    // Create documentation if requested
    if config.include_documentation {
        create_documentation_files(project_dir, config)?;
    }
    
    Ok(())
}

fn create_basic_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    // Create basic project structure
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create lib.rs
    let lib_rs = include_str!("../templates/basic/lib.rs");
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    Ok(())
}

fn create_advanced_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    // Create advanced project structure
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create lib.rs
    let lib_rs = include_str!("../templates/advanced/lib.rs");
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create additional modules
    let networking_rs = include_str!("../templates/advanced/networking.rs");
    fs::write(src_dir.join("networking.rs"), networking_rs)?;
    
    let ui_rs = include_str!("../templates/advanced/ui.rs");
    fs::write(src_dir.join("ui.rs"), ui_rs)?;
    
    Ok(())
}

fn create_networking_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    // Create networking-focused project structure
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create lib.rs
    let lib_rs = include_str!("../templates/networking/lib.rs");
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    Ok(())
}

fn create_game_logic_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create game logic template files
    create_game_logic_files(&src_dir)?;
    
    // Create additional module files
    create_events_file(&src_dir)?;
    create_scoring_file(&src_dir)?;
    
    Ok(())
}

fn create_ui_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create UI template files
    create_ui_files(&src_dir)?;
    
    Ok(())
}

fn create_physics_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create physics template files
    create_physics_files(&src_dir)?;
    
    Ok(())
}

fn create_audio_template(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create audio template files
    create_audio_files(&src_dir)?;
    
    Ok(())
}

fn create_custom_template(project_dir: &Path, config: &ScaffoldingConfig, template_name: &str) -> Result<()> {
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir)?;
    
    // Create Cargo.toml
    let cargo_toml = if config.workspace {
        create_workspace_cargo_toml(&config.project_name, &config.template)?
    } else {
        create_project_cargo_toml(&config.project_name, &config.template)?
    };
    
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Create build.rs
    let build_rs = create_build_rs_content(&config.template, &config.vrc_sdk_path, &config.unity_project_path)?;
    fs::write(project_dir.join("build.rs"), build_rs)?;
    
    // Create basic custom template
    create_custom_files(&src_dir, template_name, &config.custom_features)?;
    
    Ok(())
}

/// Create game logic template files
fn create_game_logic_files(src_dir: &Path) -> Result<()> {
    let lib_rs = r#"//! Game Logic UdonSharp project written in Rust
//! 
//! This demonstrates game logic patterns and state management.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

pub mod game_state;
pub mod events;
pub mod scoring;

use game_state::*;
use events::*;
use scoring::*;

/// Game logic controller with state management
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct GameLogicController {
    #[udon_sync]
    pub current_state: i32, // GameState as i32
    
    #[udon_sync]
    pub game_time: f32,
    
    #[udon_sync]
    pub player_scores: String, // JSON serialized scores
    
    // Local state
    state_machine: GameStateMachine,
    event_system: GameEventSystem,
    score_manager: ScoreManager,
    initialized: bool,
}

impl UdonBehaviour for GameLogicController {
    fn start(&mut self) {
        self.state_machine = GameStateMachine::new();
        self.event_system = GameEventSystem::new();
        self.score_manager = ScoreManager::new();
        
        self.current_state = GameState::Waiting as i32;
        self.game_time = 0.0;
        
        self.initialized = true;
        debug_log("Game logic controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update game time
        self.game_time += Time::delta_time();
        
        // Update state machine
        self.state_machine.update(Time::delta_time());
        
        // Process events
        self.event_system.process_events();
        
        // Update scores
        self.score_manager.update();
        
        // Sync state changes
        if self.state_machine.state_changed() {
            self.current_state = self.state_machine.current_state() as i32;
        }
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.score_manager.add_player(&player.get_display_name());
        self.event_system.emit_event(GameEvent::PlayerJoined {
            player_name: player.get_display_name(),
        });
        
        debug_log(&format!("Player joined game: {}", player.get_display_name()));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.event_system.emit_event(GameEvent::PlayerLeft {
            player_name: player.get_display_name(),
        });
        
        debug_log(&format!("Player left game: {}", player.get_display_name()));
    }
}

impl GameLogicController {
    pub fn new() -> Self {
        Self {
            current_state: GameState::Waiting as i32,
            game_time: 0.0,
            player_scores: String::new(),
            state_machine: GameStateMachine::new(),
            event_system: GameEventSystem::new(),
            score_manager: ScoreManager::new(),
            initialized: false,
        }
    }
    
    #[udon_event]
    pub fn on_start_game(&mut self) {
        if self.state_machine.can_transition_to(GameState::Playing) {
            self.state_machine.transition_to(GameState::Playing);
            self.game_time = 0.0;
            self.score_manager.reset_scores();
            
            self.event_system.emit_event(GameEvent::GameStarted);
            debug_log("Game started!");
        }
    }
    
    #[udon_event]
    pub fn on_end_game(&mut self) {
        if self.state_machine.can_transition_to(GameState::Finished) {
            self.state_machine.transition_to(GameState::Finished);
            
            let winner = self.score_manager.get_winner();
            self.event_system.emit_event(GameEvent::GameEnded { winner });
            debug_log("Game ended!");
        }
    }
    
    #[udon_event]
    pub fn on_player_scored(&mut self) {
        let local_player = Networking::local_player();
        let player_name = local_player.get_display_name();
        
        self.score_manager.add_score(&player_name, 1);
        self.event_system.emit_event(GameEvent::PlayerScored {
            player_name: player_name.clone(),
            score: self.score_manager.get_score(&player_name),
        });
        
        debug_log(&format!("Player {} scored!", player_name));
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> GameLogicController {
    GameLogicController::new()
}
"#;
    
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create game_state.rs
    let game_state_rs = r#"//! Game state management

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Waiting = 0,
    Starting = 1,
    Playing = 2,
    Paused = 3,
    Finished = 4,
}

pub struct GameStateMachine {
    current_state: GameState,
    previous_state: GameState,
    state_changed: bool,
    state_time: f32,
}

impl GameStateMachine {
    pub fn new() -> Self {
        Self {
            current_state: GameState::Waiting,
            previous_state: GameState::Waiting,
            state_changed: false,
            state_time: 0.0,
        }
    }
    
    pub fn update(&mut self, delta_time: f32) {
        self.state_time += delta_time;
        self.state_changed = false;
    }
    
    pub fn transition_to(&mut self, new_state: GameState) {
        if self.current_state != new_state {
            self.previous_state = self.current_state;
            self.current_state = new_state;
            self.state_changed = true;
            self.state_time = 0.0;
        }
    }
    
    pub fn can_transition_to(&self, target_state: GameState) -> bool {
        match (self.current_state, target_state) {
            (GameState::Waiting, GameState::Starting) => true,
            (GameState::Starting, GameState::Playing) => true,
            (GameState::Playing, GameState::Paused) => true,
            (GameState::Playing, GameState::Finished) => true,
            (GameState::Paused, GameState::Playing) => true,
            (GameState::Finished, GameState::Waiting) => true,
            _ => false,
        }
    }
    
    pub fn current_state(&self) -> GameState {
        self.current_state
    }
    
    pub fn state_changed(&self) -> bool {
        self.state_changed
    }
    
    pub fn state_time(&self) -> f32 {
        self.state_time
    }
}
"#;
    
    fs::write(src_dir.join("game_state.rs"), game_state_rs)?;
    
    Ok(())
}

/// Create events.rs for game logic template
fn create_events_file(src_dir: &Path) -> Result<()> {
    let events_rs = r#"//! Game event system

use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerJoined { player_name: String },
    PlayerLeft { player_name: String },
    PlayerScored { player_name: String, score: i32 },
    GameStarted,
    GameEnded { winner: Option<String> },
    PowerUpCollected { player_name: String, power_up_type: String },
}

pub struct GameEventSystem {
    event_queue: VecDeque<GameEvent>,
    event_handlers: Vec<Box<dyn Fn(&GameEvent)>>,
}

impl GameEventSystem {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            event_handlers: Vec::new(),
        }
    }
    
    pub fn emit_event(&mut self, event: GameEvent) {
        self.event_queue.push_back(event);
    }
    
    pub fn process_events(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            for handler in &self.event_handlers {
                handler(&event);
            }
        }
    }
    
    pub fn add_handler<F>(&mut self, handler: F)
    where
        F: Fn(&GameEvent) + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }
}
"#;
    
    fs::write(src_dir.join("events.rs"), events_rs)?;
    Ok(())
}

/// Create scoring.rs for game logic template
fn create_scoring_file(src_dir: &Path) -> Result<()> {
    let scoring_rs = r#"//! Score management system

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    pub name: String,
    pub score: i32,
    pub kills: i32,
    pub deaths: i32,
}

pub struct ScoreManager {
    scores: HashMap<String, PlayerScore>,
}

impl ScoreManager {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
        }
    }
    
    pub fn add_player(&mut self, player_name: &str) {
        self.scores.insert(
            player_name.to_string(),
            PlayerScore {
                name: player_name.to_string(),
                score: 0,
                kills: 0,
                deaths: 0,
            },
        );
    }
    
    pub fn add_score(&mut self, player_name: &str, points: i32) {
        if let Some(player_score) = self.scores.get_mut(player_name) {
            player_score.score += points;
        }
    }
    
    pub fn get_score(&self, player_name: &str) -> i32 {
        self.scores
            .get(player_name)
            .map(|score| score.score)
            .unwrap_or(0)
    }
    
    pub fn get_winner(&self) -> Option<String> {
        self.scores
            .values()
            .max_by_key(|score| score.score)
            .map(|score| score.name.clone())
    }
    
    pub fn reset_scores(&mut self) {
        for score in self.scores.values_mut() {
            score.score = 0;
            score.kills = 0;
            score.deaths = 0;
        }
    }
    
    pub fn update(&mut self) {
        // Update logic for score system
    }
}
"#;
    
    fs::write(src_dir.join("scoring.rs"), scoring_rs)?;
    Ok(())
}

/// Create UI template files
fn create_ui_files(src_dir: &Path) -> Result<()> {
    let lib_rs = r#"//! UI-focused UdonSharp project written in Rust
//! 
//! This demonstrates UI management and canvas interactions.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

pub mod canvas_manager;
pub mod ui_components;
pub mod animations;

use canvas_manager::*;
use ui_components::*;
use animations::*;

/// UI controller for managing canvas and interactions
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct UIController {
    #[udon_public]
    pub main_canvas: Option<GameObject>,
    
    #[udon_public]
    pub hud_canvas: Option<GameObject>,
    
    #[udon_sync]
    pub ui_state: String, // JSON serialized UI state
    
    // Local state
    canvas_manager: CanvasManager,
    component_registry: UIComponentRegistry,
    animation_system: UIAnimationSystem,
    initialized: bool,
}

impl UdonBehaviour for UIController {
    fn start(&mut self) {
        self.canvas_manager = CanvasManager::new();
        self.component_registry = UIComponentRegistry::new();
        self.animation_system = UIAnimationSystem::new();
        
        self.setup_canvases();
        self.register_components();
        
        self.initialized = true;
        debug_log("UI controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update animations
        self.animation_system.update(Time::delta_time());
        
        // Update UI components
        self.component_registry.update();
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.show_welcome_message(&player.get_display_name());
        debug_log(&format!("Showing welcome UI for: {}", player.get_display_name()));
    }
}

impl UIController {
    pub fn new() -> Self {
        Self {
            main_canvas: None,
            hud_canvas: None,
            ui_state: String::new(),
            canvas_manager: CanvasManager::new(),
            component_registry: UIComponentRegistry::new(),
            animation_system: UIAnimationSystem::new(),
            initialized: false,
        }
    }
    
    fn setup_canvases(&mut self) {
        // Find and setup main canvas
        if let Some(main_canvas) = GameObject::find("MainCanvas") {
            self.main_canvas = Some(main_canvas.clone());
            self.canvas_manager.register_canvas("main", main_canvas);
        }
        
        // Find and setup HUD canvas
        if let Some(hud_canvas) = GameObject::find("HUDCanvas") {
            self.hud_canvas = Some(hud_canvas.clone());
            self.canvas_manager.register_canvas("hud", hud_canvas);
        }
    }
    
    fn register_components(&mut self) {
        // Register UI components for management
        self.component_registry.register_button("start_button", "StartGame");
        self.component_registry.register_button("settings_button", "OpenSettings");
        self.component_registry.register_text("player_count", "PlayerCount");
        self.component_registry.register_text("game_status", "GameStatus");
    }
    
    fn show_welcome_message(&mut self, player_name: &str) {
        let message = format!("Welcome, {}!", player_name);
        self.component_registry.update_text("welcome_text", &message);
        
        // Animate welcome message
        self.animation_system.fade_in("welcome_panel", 1.0);
    }
    
    #[udon_event]
    pub fn on_button_click(&mut self) {
        debug_log("UI button clicked");
        
        // Handle button interactions
        self.animation_system.scale_bounce("clicked_button", 0.2);
    }
    
    #[udon_event]
    pub fn on_show_menu(&mut self) {
        self.canvas_manager.show_canvas("main");
        self.animation_system.slide_in("main_menu", 0.5);
        debug_log("Main menu shown");
    }
    
    #[udon_event]
    pub fn on_hide_menu(&mut self) {
        self.animation_system.slide_out("main_menu", 0.5);
        // Hide canvas after animation completes
        debug_log("Main menu hidden");
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> UIController {
    UIController::new()
}
"#;
    
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    
    // Create canvas_manager.rs
    let canvas_manager_rs = r#"//! Canvas management system

use std::collections::HashMap;

pub struct CanvasManager {
    canvases: HashMap<String, GameObject>,
    active_canvas: Option<String>,
}

impl CanvasManager {
    pub fn new() -> Self {
        Self {
            canvases: HashMap::new(),
            active_canvas: None,
        }
    }
    
    pub fn register_canvas(&mut self, name: &str, canvas: GameObject) {
        self.canvases.insert(name.to_string(), canvas);
    }
    
    pub fn show_canvas(&mut self, name: &str) {
        // Hide current canvas
        if let Some(ref current) = self.active_canvas {
            if let Some(canvas) = self.canvases.get(current) {
                canvas.set_active(false);
            }
        }
        
        // Show new canvas
        if let Some(canvas) = self.canvases.get(name) {
            canvas.set_active(true);
            self.active_canvas = Some(name.to_string());
        }
    }
    
    pub fn hide_canvas(&mut self, name: &str) {
        if let Some(canvas) = self.canvases.get(name) {
            canvas.set_active(false);
            
            if self.active_canvas.as_ref() == Some(&name.to_string()) {
                self.active_canvas = None;
            }
        }
    }
    
    pub fn get_canvas(&self, name: &str) -> Option<&GameObject> {
        self.canvases.get(name)
    }
}
"#;
    
    fs::write(src_dir.join("canvas_manager.rs"), canvas_manager_rs)?;
    
    Ok(())
}

/// Create physics template files
fn create_physics_files(src_dir: &Path) -> Result<()> {
    let lib_rs = r#"//! Physics-based UdonSharp project written in Rust
//! 
//! This demonstrates physics interactions and rigidbody management.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use rand::Rng;

pub mod physics_manager;
pub mod collision_handler;
pub mod force_controller;

use physics_manager::*;
use collision_handler::*;
use force_controller::*;

/// Physics controller for managing rigidbodies and forces
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct PhysicsController {
    #[udon_public]
    pub physics_objects: Vec<GameObject>,
    
    #[udon_sync]
    pub gravity_multiplier: f32,
    
    #[udon_sync]
    pub wind_force: Vector3,
    
    // Local state
    physics_manager: PhysicsManager,
    collision_handler: CollisionHandler,
    force_controller: ForceController,
    initialized: bool,
}

impl UdonBehaviour for PhysicsController {
    fn start(&mut self) {
        self.physics_manager = PhysicsManager::new();
        self.collision_handler = CollisionHandler::new();
        self.force_controller = ForceController::new();
        
        self.gravity_multiplier = 1.0;
        self.wind_force = Vector3::zero();
        
        self.setup_physics_objects();
        
        self.initialized = true;
        debug_log("Physics controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update physics simulation
        self.physics_manager.update(Time::delta_time());
        
        // Apply forces
        self.force_controller.apply_forces(&self.physics_objects, Time::delta_time());
        
        // Handle collisions
        self.collision_handler.process_collisions();
    }
}

impl PhysicsController {
    pub fn new() -> Self {
        Self {
            physics_objects: Vec::new(),
            gravity_multiplier: 1.0,
            wind_force: Vector3::zero(),
            physics_manager: PhysicsManager::new(),
            collision_handler: CollisionHandler::new(),
            force_controller: ForceController::new(),
            initialized: false,
        }
    }
    
    fn setup_physics_objects(&mut self) {
        // Find all physics objects in the scene
        for i in 0..20 {
            let obj_name = format!("PhysicsObject_{}", i);
            if let Some(obj) = GameObject::find(&obj_name) {
                self.physics_objects.push(obj.clone());
                self.physics_manager.register_object(obj);
            }
        }
        
        debug_log(&format!("Found {} physics objects", self.physics_objects.len()));
    }
    
    #[udon_event]
    pub fn on_apply_explosion(&mut self) {
        let explosion_center = Vector3::new(0.0, 1.0, 0.0);
        let explosion_force = 500.0;
        let explosion_radius = 10.0;
        
        self.force_controller.apply_explosion(
            explosion_center,
            explosion_force,
            explosion_radius,
            &self.physics_objects,
        );
        
        debug_log("Explosion applied to physics objects");
    }
    
    #[udon_event]
    pub fn on_toggle_gravity(&mut self) {
        self.gravity_multiplier = if self.gravity_multiplier > 0.0 { 0.0 } else { 1.0 };
        self.physics_manager.set_gravity_multiplier(self.gravity_multiplier);
        
        debug_log(&format!("Gravity multiplier set to: {}", self.gravity_multiplier));
    }
    
    #[udon_event]
    pub fn on_random_wind(&mut self) {
        let mut rng = rand::thread_rng();
        self.wind_force = Vector3::new(
            rng.gen_range(-10.0..10.0),
            rng.gen_range(-5.0..5.0),
            rng.gen_range(-10.0..10.0),
        );
        
        self.force_controller.set_wind_force(self.wind_force);
        debug_log(&format!("Wind force set to: {:?}", self.wind_force));
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> PhysicsController {
    PhysicsController::new()
}
"#;
    
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

/// Create audio template files
fn create_audio_files(src_dir: &Path) -> Result<()> {
    let lib_rs = r#"//! Audio-focused UdonSharp project written in Rust
//! 
//! This demonstrates audio management and sound systems.

use udonsharp_core::prelude::*;
use udonsharp_macros::*;
use serde::{Serialize, Deserialize};

pub mod audio_manager;
pub mod music_system;
pub mod sound_effects;

use audio_manager::*;
use music_system::*;
use sound_effects::*;

/// Audio controller for managing sounds and music
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct AudioController {
    #[udon_public]
    pub audio_sources: Vec<AudioSource>,
    
    #[udon_sync]
    pub master_volume: f32,
    
    #[udon_sync]
    pub music_volume: f32,
    
    #[udon_sync]
    pub sfx_volume: f32,
    
    // Local state
    audio_manager: AudioManager,
    music_system: MusicSystem,
    sound_effects: SoundEffectSystem,
    initialized: bool,
}

impl UdonBehaviour for AudioController {
    fn start(&mut self) {
        self.audio_manager = AudioManager::new();
        self.music_system = MusicSystem::new();
        self.sound_effects = SoundEffectSystem::new();
        
        self.master_volume = 1.0;
        self.music_volume = 0.7;
        self.sfx_volume = 0.8;
        
        self.setup_audio_sources();
        
        self.initialized = true;
        debug_log("Audio controller initialized");
    }
    
    fn update(&mut self) {
        if !self.initialized {
            return;
        }
        
        // Update audio systems
        self.music_system.update(Time::delta_time());
        self.sound_effects.update(Time::delta_time());
        
        // Update volume levels
        self.audio_manager.set_master_volume(self.master_volume);
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.sound_effects.play_sound("player_join");
        debug_log(&format!("Played join sound for: {}", player.get_display_name()));
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.sound_effects.play_sound("player_leave");
        debug_log(&format!("Played leave sound for: {}", player.get_display_name()));
    }
}

impl AudioController {
    pub fn new() -> Self {
        Self {
            audio_sources: Vec::new(),
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            audio_manager: AudioManager::new(),
            music_system: MusicSystem::new(),
            sound_effects: SoundEffectSystem::new(),
            initialized: false,
        }
    }
    
    fn setup_audio_sources(&mut self) {
        // Find audio sources in the scene
        for i in 0..10 {
            let source_name = format!("AudioSource_{}", i);
            if let Some(obj) = GameObject::find(&source_name) {
                if let Some(audio_source) = obj.get_component::<AudioSource>() {
                    self.audio_sources.push(audio_source.clone());
                    self.audio_manager.register_source(&source_name, audio_source);
                }
            }
        }
        
        debug_log(&format!("Found {} audio sources", self.audio_sources.len()));
    }
    
    #[udon_event]
    pub fn on_play_music(&mut self) {
        self.music_system.play_track("background_music");
        debug_log("Started playing background music");
    }
    
    #[udon_event]
    pub fn on_stop_music(&mut self) {
        self.music_system.stop_current_track();
        debug_log("Stopped background music");
    }
    
    #[udon_event]
    pub fn on_play_sound_effect(&mut self) {
        self.sound_effects.play_sound("button_click");
        debug_log("Played button click sound effect");
    }
    
    #[udon_event]
    pub fn on_volume_up(&mut self) {
        self.master_volume = (self.master_volume + 0.1).min(1.0);
        debug_log(&format!("Master volume: {}", self.master_volume));
    }
    
    #[udon_event]
    pub fn on_volume_down(&mut self) {
        self.master_volume = (self.master_volume - 0.1).max(0.0);
        debug_log(&format!("Master volume: {}", self.master_volume));
    }
}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> AudioController {
    AudioController::new()
}
"#;
    
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

/// Create custom template files
fn create_custom_files(src_dir: &Path, template_name: &str, features: &[String]) -> Result<()> {
    let lib_rs = format!(
        r#"//! Custom UdonSharp project: {}
//! 
//! This is a custom template with the following features:
{}

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

/// Custom UdonSharp behaviour
#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct CustomBehaviour {{
    #[udon_public]
    pub custom_data: String,
    
    #[udon_sync]
    pub sync_value: i32,
    
    initialized: bool,
}}

impl UdonBehaviour for CustomBehaviour {{
    fn start(&mut self) {{
        self.initialized = true;
        self.custom_data = "Custom UdonSharp Behaviour".to_string();
        self.sync_value = 0;
        
        debug_log("Custom behaviour initialized");
    }}
    
    fn update(&mut self) {{
        // Custom update logic here
    }}
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {{
        debug_log(&format!("Player joined: {{}}", player.get_display_name()));
    }}
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {{
        debug_log(&format!("Player left: {{}}", player.get_display_name()));
    }}
}}

impl CustomBehaviour {{
    pub fn new() -> Self {{
        Self {{
            custom_data: String::new(),
            sync_value: 0,
            initialized: false,
        }}
    }}
    
    #[udon_event]
    pub fn on_custom_event(&mut self) {{
        debug_log("Custom event triggered!");
        self.sync_value += 1;
    }}
}}

// Export the main behaviour for UdonSharp compilation
#[no_mangle]
pub extern "C" fn create_behaviour() -> CustomBehaviour {{
    CustomBehaviour::new()
}}
"#,
        template_name,
        features
            .iter()
            .map(|f| format!("//! - {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    fs::write(src_dir.join("lib.rs"), lib_rs)?;
    Ok(())
}

/// Create common files for all templates
fn create_common_files(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    // Create .gitignore
    let gitignore = r#"/target
/generated
Cargo.lock
*.wasm
*.cs
.DS_Store
*.tmp
*.log
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    
    // Create README.md
    let readme = format!(
        r#"# {}

A {} UdonSharp project written in Rust.

## Description

{}

## Features

{}

## Building

```bash
cargo udonsharp build
```

## Development

This project uses the Rust UdonSharp framework to compile Rust code to UdonSharp-compatible C# for VRChat world development.

### Project Structure

- `src/` - Main source code
- `generated/` - Auto-generated API bindings (created during build)
- `target/` - Build artifacts
- `build.rs` - Build script for generating bindings

### Commands

- `cargo udonsharp build` - Build the project
- `cargo udonsharp check` - Check for errors without building
- `cargo udonsharp clean` - Clean build artifacts
- `cargo udonsharp bindings` - Generate API bindings

## Documentation

For more information about the Rust UdonSharp framework, see:
- [Rust UdonSharp Documentation](https://github.com/vrchat-community/rust-udonsharp)
- [VRChat Creator Documentation](https://creators.vrchat.com/)
- [UdonSharp Documentation](https://udonsharp.docs.vrchat.com/)

## License

This project is licensed under the MIT License.
"#,
        config.project_name,
        config.template.name(),
        config.template.description(),
        config
            .template
            .features()
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n")
    );
    
    fs::write(project_dir.join("README.md"), readme)?;
    
    // Create rust-toolchain.toml
    let rust_toolchain = r#"[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
"#;
    fs::write(project_dir.join("rust-toolchain.toml"), rust_toolchain)?;
    
    Ok(())
}

/// Create example files if requested
fn create_example_files(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let examples_dir = project_dir.join("examples");
    fs::create_dir_all(&examples_dir)?;
    
    match &config.template {
        ProjectTemplate::Basic => create_basic_examples(&examples_dir)?,
        ProjectTemplate::Advanced => create_advanced_examples(&examples_dir)?,
        ProjectTemplate::Networking => create_networking_examples(&examples_dir)?,
        ProjectTemplate::GameLogic => create_game_logic_examples(&examples_dir)?,
        ProjectTemplate::UI => create_ui_examples(&examples_dir)?,
        ProjectTemplate::Physics => create_physics_examples(&examples_dir)?,
        ProjectTemplate::Audio => create_audio_examples(&examples_dir)?,
        ProjectTemplate::Custom(_) => create_custom_examples(&examples_dir)?,
    }
    
    Ok(())
}

/// Create test files if requested
fn create_test_files(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let tests_dir = project_dir.join("tests");
    fs::create_dir_all(&tests_dir)?;
    
    let integration_test = r#"//! Integration tests for UdonSharp project

use udonsharp_core::testing::*;

#[udon_test]
fn test_basic_functionality() {
    // Test basic UdonSharp functionality
    assert!(true, "Basic test should pass");
}

#[udon_test]
fn test_player_interaction() {
    // Test player interaction logic
    let mock_player = create_mock_player("TestPlayer");
    assert_eq!(mock_player.get_display_name(), "TestPlayer");
}

#[udon_test]
fn test_networking() {
    // Test networking functionality
    let is_master = Networking::is_master();
    assert!(is_master || !is_master, "Networking state should be deterministic");
}
"#;
    
    fs::write(tests_dir.join("integration_tests.rs"), integration_test)?;
    
    Ok(())
}

/// Create documentation files if requested
fn create_documentation_files(project_dir: &Path, config: &ScaffoldingConfig) -> Result<()> {
    let docs_dir = project_dir.join("docs");
    fs::create_dir_all(&docs_dir)?;
    
    // Create API documentation
    let api_docs = format!(
        r#"# {} API Documentation

## Overview

This document describes the API for the {} UdonSharp project.

## Main Components

### Primary Behaviour

The main behaviour class provides the following functionality:

- Player event handling
- State management
- Network synchronization
- Custom event processing

### Methods

#### Public Methods

- `start()` - Initialize the behaviour
- `update()` - Update loop called every frame
- `on_player_joined(player)` - Called when a player joins
- `on_player_left(player)` - Called when a player leaves

#### Event Methods

- `on_interact()` - Called when the object is interacted with
- Custom events as defined in the implementation

### Properties

#### Synchronized Properties

Properties marked with `#[udon_sync]` are automatically synchronized across all clients.

#### Public Properties

Properties marked with `#[udon_public]` are exposed in the Unity inspector.

## Usage Examples

See the `examples/` directory for detailed usage examples.

## Building and Deployment

1. Build the project: `cargo udonsharp build`
2. Copy the generated C# files to your Unity project
3. Attach the main behaviour to a GameObject in your scene
4. Configure public properties in the Unity inspector
5. Build and upload your VRChat world

## Troubleshooting

### Common Issues

- **Build errors**: Check that all dependencies are properly configured
- **Runtime errors**: Enable debug logging to see detailed error messages
- **Networking issues**: Ensure proper synchronization of networked properties

### Debug Logging

Use `debug_log()` function to output messages to the VRChat console for debugging.
"#,
        config.project_name,
        config.template.description()
    );
    
    fs::write(docs_dir.join("API.md"), api_docs)?;
    
    // Create development guide
    let dev_guide = r#"# Development Guide

## Getting Started

1. Install Rust and the required toolchain
2. Clone or create your UdonSharp project
3. Run `cargo udonsharp build` to build the project
4. Import the generated C# files into Unity

## Project Structure

- `src/lib.rs` - Main entry point
- `src/` - Additional modules
- `build.rs` - Build configuration
- `Cargo.toml` - Project dependencies
- `generated/` - Auto-generated bindings

## Best Practices

### Code Organization

- Keep related functionality in separate modules
- Use clear, descriptive names for functions and variables
- Document public APIs with doc comments

### Performance

- Minimize allocations in update loops
- Use appropriate data structures for your use case
- Profile your code to identify bottlenecks

### Networking

- Only synchronize data that needs to be shared
- Use appropriate sync modes for your use case
- Handle network events gracefully

## Testing

Run tests with:
```bash
cargo test
```

## Debugging

Enable verbose logging during development:
```bash
cargo udonsharp build --verbose
```
"#;
    
    fs::write(docs_dir.join("DEVELOPMENT.md"), dev_guide)?;
    
    Ok(())
}

/// Create basic examples
fn create_basic_examples(examples_dir: &Path) -> Result<()> {
    let simple_interaction = r#"//! Simple interaction example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
pub struct SimpleInteraction {
    #[udon_public]
    pub message: String,
    
    click_count: i32,
}

impl UdonBehaviour for SimpleInteraction {
    fn start(&mut self) {
        self.message = "Click me!".to_string();
        self.click_count = 0;
    }
}

impl SimpleInteraction {
    #[udon_event]
    pub fn on_interact(&mut self) {
        self.click_count += 1;
        debug_log(&format!("Clicked {} times!", self.click_count));
    }
}
"#;
    
    fs::write(examples_dir.join("simple_interaction.rs"), simple_interaction)?;
    Ok(())
}

/// Create advanced examples
fn create_advanced_examples(examples_dir: &Path) -> Result<()> {
    let complex_system = r#"//! Complex system example with multiple components

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
pub struct ComplexSystem {
    #[udon_sync]
    pub system_state: i32,
    
    components: Vec<SystemComponent>,
}

pub struct SystemComponent {
    pub id: String,
    pub active: bool,
    pub data: String,
}

impl UdonBehaviour for ComplexSystem {
    fn start(&mut self) {
        self.initialize_components();
    }
    
    fn update(&mut self) {
        self.update_components();
    }
}

impl ComplexSystem {
    fn initialize_components(&mut self) {
        for i in 0..5 {
            self.components.push(SystemComponent {
                id: format!("component_{}", i),
                active: true,
                data: String::new(),
            });
        }
    }
    
    fn update_components(&mut self) {
        for component in &mut self.components {
            if component.active {
                // Update component logic
            }
        }
    }
}
"#;
    
    fs::write(examples_dir.join("complex_system.rs"), complex_system)?;
    Ok(())
}

/// Create networking examples
fn create_networking_examples(examples_dir: &Path) -> Result<()> {
    let sync_example = r#"//! Network synchronization example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct NetworkSync {
    #[udon_sync]
    pub shared_counter: i32,
    
    #[udon_sync]
    pub shared_message: String,
}

impl UdonBehaviour for NetworkSync {
    fn start(&mut self) {
        if Networking::is_master() {
            self.shared_counter = 0;
            self.shared_message = "Hello Network!".to_string();
        }
    }
}

impl NetworkSync {
    #[udon_event]
    pub fn on_increment_counter(&mut self) {
        if Networking::is_master() {
            self.shared_counter += 1;
            self.request_serialization();
        }
    }
}
"#;
    
    fs::write(examples_dir.join("network_sync.rs"), sync_example)?;
    Ok(())
}

/// Create game logic examples
fn create_game_logic_examples(examples_dir: &Path) -> Result<()> {
    let state_machine = r#"//! State machine example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(UdonBehaviour)]
pub struct StateMachine {
    current_state: GameState,
    state_time: f32,
}

impl UdonBehaviour for StateMachine {
    fn start(&mut self) {
        self.current_state = GameState::Menu;
        self.state_time = 0.0;
    }
    
    fn update(&mut self) {
        self.state_time += Time::delta_time();
        
        match self.current_state {
            GameState::Menu => self.update_menu(),
            GameState::Playing => self.update_playing(),
            GameState::Paused => self.update_paused(),
            GameState::GameOver => self.update_game_over(),
        }
    }
}

impl StateMachine {
    fn update_menu(&mut self) {
        // Menu logic
    }
    
    fn update_playing(&mut self) {
        // Game logic
    }
    
    fn update_paused(&mut self) {
        // Pause logic
    }
    
    fn update_game_over(&mut self) {
        // Game over logic
    }
    
    fn transition_to(&mut self, new_state: GameState) {
        self.current_state = new_state;
        self.state_time = 0.0;
    }
}
"#;
    
    fs::write(examples_dir.join("state_machine.rs"), state_machine)?;
    Ok(())
}

/// Create UI examples
fn create_ui_examples(examples_dir: &Path) -> Result<()> {
    let button_handler = r#"//! UI button handler example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
pub struct ButtonHandler {
    #[udon_public]
    pub button_text: Option<GameObject>,
    
    #[udon_public]
    pub counter_display: Option<GameObject>,
    
    button_count: i32,
}

impl UdonBehaviour for ButtonHandler {
    fn start(&mut self) {
        self.button_count = 0;
        self.update_display();
    }
}

impl ButtonHandler {
    #[udon_event]
    pub fn on_button_click(&mut self) {
        self.button_count += 1;
        self.update_display();
        debug_log(&format!("Button clicked {} times", self.button_count));
    }
    
    fn update_display(&mut self) {
        if let Some(ref display) = self.counter_display {
            if let Some(text_component) = display.get_component::<UnityEngine::UI::Text>() {
                text_component.set_text(&format!("Count: {}", self.button_count));
            }
        }
    }
}
"#;
    
    fs::write(examples_dir.join("button_handler.rs"), button_handler)?;
    Ok(())
}

/// Create physics examples
fn create_physics_examples(examples_dir: &Path) -> Result<()> {
    let force_example = r#"//! Physics force application example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
pub struct ForceApplicator {
    #[udon_public]
    pub target_rigidbody: Option<Rigidbody>,
    
    #[udon_public]
    pub force_strength: f32,
}

impl UdonBehaviour for ForceApplicator {
    fn start(&mut self) {
        self.force_strength = 10.0;
    }
}

impl ForceApplicator {
    #[udon_event]
    pub fn on_apply_force(&mut self) {
        if let Some(ref rb) = self.target_rigidbody {
            let force = Vector3::up() * self.force_strength;
            rb.add_force(force, ForceMode::Impulse);
            debug_log("Applied upward force to rigidbody");
        }
    }
    
    #[udon_event]
    pub fn on_apply_random_force(&mut self) {
        if let Some(ref rb) = self.target_rigidbody {
            let random_force = Vector3::new(
                Random::range(-self.force_strength, self.force_strength),
                Random::range(0.0, self.force_strength),
                Random::range(-self.force_strength, self.force_strength),
            );
            rb.add_force(random_force, ForceMode::Impulse);
            debug_log("Applied random force to rigidbody");
        }
    }
}
"#;
    
    fs::write(examples_dir.join("force_applicator.rs"), force_example)?;
    Ok(())
}

/// Create audio examples
fn create_audio_examples(examples_dir: &Path) -> Result<()> {
    let sound_player = r#"//! Audio playback example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
pub struct SoundPlayer {
    #[udon_public]
    pub audio_source: Option<AudioSource>,
    
    #[udon_public]
    pub sound_clips: Vec<AudioClip>,
    
    current_clip_index: usize,
}

impl UdonBehaviour for SoundPlayer {
    fn start(&mut self) {
        self.current_clip_index = 0;
    }
}

impl SoundPlayer {
    #[udon_event]
    pub fn on_play_sound(&mut self) {
        if let Some(ref audio_source) = self.audio_source {
            if !self.sound_clips.is_empty() {
                let clip = &self.sound_clips[self.current_clip_index];
                audio_source.set_clip(clip.clone());
                audio_source.play();
                debug_log("Playing sound clip");
            }
        }
    }
    
    #[udon_event]
    pub fn on_next_clip(&mut self) {
        if !self.sound_clips.is_empty() {
            self.current_clip_index = (self.current_clip_index + 1) % self.sound_clips.len();
            debug_log(&format!("Selected clip {}", self.current_clip_index));
        }
    }
    
    #[udon_event]
    pub fn on_stop_sound(&mut self) {
        if let Some(ref audio_source) = self.audio_source {
            audio_source.stop();
            debug_log("Stopped audio playback");
        }
    }
}
"#;
    
    fs::write(examples_dir.join("sound_player.rs"), sound_player)?;
    Ok(())
}

/// Create custom examples
fn create_custom_examples(examples_dir: &Path) -> Result<()> {
    let custom_example = r#"//! Custom template example

use udonsharp_core::prelude::*;
use udonsharp_macros::*;

#[derive(UdonBehaviour)]
pub struct CustomExample {
    #[udon_public]
    pub example_data: String,
    
    example_counter: i32,
}

impl UdonBehaviour for CustomExample {
    fn start(&mut self) {
        self.example_data = "Custom Example".to_string();
        self.example_counter = 0;
    }
    
    fn update(&mut self) {
        // Custom update logic
    }
}

impl CustomExample {
    #[udon_event]
    pub fn on_custom_action(&mut self) {
        self.example_counter += 1;
        debug_log(&format!("Custom action performed {} times", self.example_counter));
    }
}
"#;
    
    fs::write(examples_dir.join("custom_example.rs"), custom_example)?;
    Ok(())
}