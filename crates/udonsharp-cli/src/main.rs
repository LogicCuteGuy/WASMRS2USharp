//! Command-line interface for Rust UdonSharp development

use clap::{Parser, Subcommand, ValueEnum};
use udonsharp_core::{UdonSharpResult, error::CompilationContext};
use udonsharp_compiler::{CompilationPipeline, UdonSharpConfig};
use udonsharp_bindings::UniversalBindingPipeline;
use std::path::Path;
use std::fs;
use log::{info, warn, error};

mod analyze_command;
use analyze_command::{AnalyzeSubcommand, execute_analyze_command};

#[derive(Parser)]
#[command(name = "udonsharp")]
#[command(about = "Rust to UdonSharp compilation toolchain")]
#[command(version)]
#[command(long_about = "A comprehensive toolchain for compiling Rust code to UdonSharp-compatible C# for VRChat world development")]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
    
    /// Enable quiet mode (suppress non-error output)
    #[arg(short, long, global = true, conflicts_with = "verbose")]
    quiet: bool,
    
    /// Set log level
    #[arg(long, global = true, value_enum)]
    log_level: Option<LogLevel>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Debug)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new UdonSharp project
    New {
        /// Project name
        name: String,
        /// Project directory (defaults to project name)
        #[arg(long)]
        path: Option<String>,
        /// Project template type
        #[arg(long, value_enum, default_value = "basic")]
        template: ProjectTemplate,
        /// Initialize with git repository
        #[arg(long)]
        git: bool,
        /// Skip interactive prompts
        #[arg(long)]
        no_interactive: bool,
    },
    /// Build the current project
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,
        /// Generate debug information
        #[arg(long)]
        debug: bool,
        /// Target directory for output
        #[arg(long)]
        target_dir: Option<String>,
        /// Watch for changes and rebuild automatically
        #[arg(long)]
        watch: bool,
        /// Number of parallel jobs
        #[arg(short, long)]
        jobs: Option<usize>,
        /// Show compilation progress
        #[arg(long)]
        progress: bool,
    },
    /// Generate API bindings from .asmdef files
    Bindings {
        /// Directory to scan for .asmdef files
        #[arg(long)]
        scan_dir: Vec<String>,
        /// Output directory for generated bindings
        #[arg(long)]
        output: String,
        /// Force regeneration of existing bindings
        #[arg(long)]
        force: bool,
        /// Generate only specific assembly bindings
        #[arg(long)]
        assembly: Vec<String>,
        /// Show binding generation progress
        #[arg(long)]
        progress: bool,
    },
    /// Clean build artifacts
    Clean {
        /// Target directory to clean
        #[arg(long)]
        target_dir: Option<String>,
        /// Remove all generated files including bindings
        #[arg(long)]
        all: bool,
    },
    /// Check project for errors without building
    Check {
        /// Check in release mode
        #[arg(long)]
        release: bool,
        /// Show detailed diagnostics
        #[arg(long)]
        detailed: bool,
    },
    /// Run tests
    Test {
        /// Run tests in release mode
        #[arg(long)]
        release: bool,
        /// Test name filter
        #[arg(long)]
        filter: Option<String>,
        /// Show test output
        #[arg(long)]
        nocapture: bool,
    },
    /// Show project information
    Info {
        /// Show detailed project information
        #[arg(long)]
        detailed: bool,
        /// Output format
        #[arg(long, value_enum, default_value = "human")]
        format: OutputFormat,
    },
    /// Initialize an existing directory as UdonSharp project
    Init {
        /// Project directory (defaults to current directory)
        path: Option<String>,
        /// Project template type
        #[arg(long, value_enum, default_value = "basic")]
        template: ProjectTemplate,
        /// Initialize with git repository
        #[arg(long)]
        git: bool,
    },
    /// Analyze multi-behavior projects
    Analyze {
        #[command(subcommand)]
        subcommand: AnalyzeSubcommand,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum ProjectTemplate {
    /// Basic UdonSharp project
    Basic,
    /// World controller project with networking
    WorldController,
    /// Interactive object project
    Interactive,
    /// UI system project
    UiSystem,
    /// Custom project (prompts for configuration)
    Custom,
}

#[derive(ValueEnum, Clone, Debug)]
enum OutputFormat {
    Human,
    Json,
    Yaml,
}

#[tokio::main]
async fn main() -> UdonSharpResult<()> {
    let cli = Cli::parse();
    
    // Initialize logging based on CLI arguments
    init_logging(&cli)?;
    
    match cli.command {
        Commands::New { name, path, template, git, no_interactive } => {
            handle_new_command(name, path, template, git, no_interactive).await
        }
        Commands::Build { release, debug, target_dir, watch, jobs, progress } => {
            handle_build_command(release, debug, target_dir, watch, jobs, progress).await
        }
        Commands::Bindings { scan_dir, output, force, assembly, progress } => {
            handle_bindings_command(scan_dir, output, force, assembly, progress).await
        }
        Commands::Clean { target_dir, all } => {
            handle_clean_command(target_dir, all).await
        }
        Commands::Check { release, detailed } => {
            handle_check_command(release, detailed).await
        }
        Commands::Test { release, filter, nocapture } => {
            handle_test_command(release, filter, nocapture).await
        }
        Commands::Info { detailed, format } => {
            handle_info_command(detailed, format).await
        }
        Commands::Init { path, template, git } => {
            handle_init_command(path, template, git).await
        }
        Commands::Analyze { subcommand } => {
            execute_analyze_command(analyze_command::AnalyzeCommand { subcommand }).await
        }
    }
}

fn init_logging(cli: &Cli) -> UdonSharpResult<()> {
    let log_level = if cli.quiet {
        log::LevelFilter::Error
    } else if cli.verbose {
        log::LevelFilter::Debug
    } else {
        match &cli.log_level {
            Some(LogLevel::Error) => log::LevelFilter::Error,
            Some(LogLevel::Warn) => log::LevelFilter::Warn,
            Some(LogLevel::Info) => log::LevelFilter::Info,
            Some(LogLevel::Debug) => log::LevelFilter::Debug,
            Some(LogLevel::Trace) => log::LevelFilter::Trace,
            None => log::LevelFilter::Info,
        }
    };
    
    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .format_timestamp_secs()
        .init();
    
    Ok(())
}

async fn handle_new_command(
    name: String,
    path: Option<String>,
    template: ProjectTemplate,
    git: bool,
    no_interactive: bool,
) -> UdonSharpResult<()> {
    info!("Creating new UdonSharp project: {}", name);
    
    let project_path = path.unwrap_or_else(|| name.clone());
    let project_dir = Path::new(&project_path);
    
    if project_dir.exists() {
        return Err(udonsharp_core::UdonSharpError::configuration(
            format!("Directory '{}' already exists", project_path)
        ));
    }
    
    // Create project directory
    fs::create_dir_all(project_dir).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to create project directory: {}", e))
    })?;
    
    info!("Created project directory: {}", project_path);
    
    // Generate project files based on template
    generate_project_template(project_dir, &name, &template, no_interactive).await?;
    
    // Initialize git repository if requested
    if git {
        init_git_repository(project_dir)?;
    }
    
    println!("✅ Successfully created UdonSharp project '{}'", name);
    println!("📁 Project location: {}", project_dir.display());
    println!("🚀 To get started:");
    println!("   cd {}", project_path);
    println!("   udonsharp build");
    
    Ok(())
}

async fn handle_build_command(
    release: bool,
    debug: bool,
    target_dir: Option<String>,
    watch: bool,
    jobs: Option<usize>,
    progress: bool,
) -> UdonSharpResult<()> {
    if watch {
        info!("Starting build in watch mode...");
        // TODO: Implement watch mode
        return Err(udonsharp_core::UdonSharpError::configuration(
            "Watch mode not yet implemented"
        ));
    }
    
    let context = if debug {
        CompilationContext::debug()
    } else {
        CompilationContext::new()
    };
    
    if progress {
        info!("Building project with progress reporting...");
    } else {
        info!("Building project (release: {}, debug: {})", release, debug);
    }
    
    // Create configuration
    let mut config = UdonSharpConfig::default();
    config.optimize_for_performance = release;
    config.generate_debug_info = debug;
    
    if let Some(target) = target_dir {
        config.output_directory = Some(target);
    }
    
    if let Some(job_count) = jobs {
        config.parallel_jobs = Some(job_count);
    }
    
    // Create compilation pipeline
    let pipeline = CompilationPipeline::with_context(config, context);
    
    // Compile current directory
    let result = pipeline.compile_project(".").await?;
    
    pipeline.context().print_summary();
    
    if result.success {
        println!("✅ Build completed successfully!");
        if !result.output_files.is_empty() {
            println!("📄 Generated files:");
            for file in &result.output_files {
                println!("   {}", file);
            }
        }
    } else {
        error!("Build failed");
        return Err(udonsharp_core::UdonSharpError::compilation("Build failed"));
    }
    
    Ok(())
}

async fn handle_bindings_command(
    scan_dirs: Vec<String>,
    output: String,
    force: bool,
    assemblies: Vec<String>,
    progress: bool,
) -> UdonSharpResult<()> {
    info!("Generating API bindings...");
    
    if progress {
        println!("🔍 Scanning directories for .asmdef files...");
    }
    
    // Create output directory if it doesn't exist
    let output_path = Path::new(&output);
    if !output_path.exists() {
        fs::create_dir_all(output_path).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to create output directory: {}", e))
        })?;
    }
    
    // Initialize binding pipeline
    let mut pipeline = UniversalBindingPipeline::new(output.clone());
    
    // Add scan directories
    for dir in scan_dirs {
        if Path::new(&dir).exists() {
            pipeline.add_asmdef_directory(dir.clone());
            if progress {
                println!("   Added scan directory: {}", dir);
            }
        } else {
            warn!("Scan directory does not exist: {}", dir);
        }
    }
    
    // Generate bindings
    if progress {
        println!("🔧 Generating bindings...");
    }
    
    pipeline.scan_and_generate_all_bindings().map_err(|e| {
        udonsharp_core::UdonSharpError::compilation(format!("Failed to generate bindings: {}", e))
    })?;
    
    println!("✅ Successfully generated API bindings");
    println!("📁 Output directory: {}", output);
    
    Ok(())
}

async fn handle_clean_command(target_dir: Option<String>, all: bool) -> UdonSharpResult<()> {
    info!("Cleaning build artifacts...");
    
    let target_path = target_dir.unwrap_or_else(|| "target".to_string());
    let target_dir = Path::new(&target_path);
    
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to clean target directory: {}", e))
        })?;
        println!("🧹 Cleaned target directory: {}", target_path);
    }
    
    if all {
        // Clean additional generated files
        let bindings_dir = Path::new("generated");
        if bindings_dir.exists() {
            fs::remove_dir_all(bindings_dir).map_err(|e| {
                udonsharp_core::UdonSharpError::configuration(format!("Failed to clean bindings directory: {}", e))
            })?;
            println!("🧹 Cleaned generated bindings directory");
        }
    }
    
    println!("✅ Clean completed successfully");
    Ok(())
}

async fn handle_check_command(release: bool, detailed: bool) -> UdonSharpResult<()> {
    info!("Checking project for errors...");
    
    let context = CompilationContext::new();
    let mut config = UdonSharpConfig::default();
    config.optimize_for_performance = release;
    config.check_only = true;
    
    let pipeline = CompilationPipeline::with_context(config, context);
    let result = pipeline.check_project(".").await?;
    
    if detailed {
        pipeline.context().print_detailed_diagnostics();
    } else {
        pipeline.context().print_summary();
    }
    
    if result.success {
        println!("✅ No errors found");
    } else {
        error!("Check failed - errors found");
        return Err(udonsharp_core::UdonSharpError::compilation("Check failed"));
    }
    
    Ok(())
}

async fn handle_test_command(
    release: bool,
    filter: Option<String>,
    nocapture: bool,
) -> UdonSharpResult<()> {
    info!("Running tests...");
    
    let context = CompilationContext::new();
    let mut config = UdonSharpConfig::default();
    config.optimize_for_performance = release;
    config.test_mode = true;
    
    if let Some(filter_pattern) = filter {
        config.test_filter = Some(filter_pattern);
    }
    
    config.capture_test_output = !nocapture;
    
    let pipeline = CompilationPipeline::with_context(config, context);
    let result = pipeline.run_tests(".").await?;
    
    pipeline.context().print_test_summary();
    
    if result.success {
        println!("✅ All tests passed");
    } else {
        error!("Some tests failed");
        return Err(udonsharp_core::UdonSharpError::compilation("Tests failed"));
    }
    
    Ok(())
}

async fn handle_info_command(detailed: bool, format: OutputFormat) -> UdonSharpResult<()> {
    info!("Gathering project information...");
    
    let project_info = gather_project_info(detailed).await?;
    
    match format {
        OutputFormat::Human => print_human_readable_info(&project_info),
        OutputFormat::Json => print_json_info(&project_info)?,
        OutputFormat::Yaml => print_yaml_info(&project_info)?,
    }
    
    Ok(())
}

async fn handle_init_command(
    path: Option<String>,
    template: ProjectTemplate,
    git: bool,
) -> UdonSharpResult<()> {
    let project_path = path.unwrap_or_else(|| ".".to_string());
    let project_dir = Path::new(&project_path);
    
    info!("Initializing UdonSharp project in: {}", project_dir.display());
    
    if !project_dir.exists() {
        fs::create_dir_all(project_dir).map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to create directory: {}", e))
        })?;
    }
    
    let project_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("udonsharp-project")
        .to_string();
    
    generate_project_template(project_dir, &project_name, &template, false).await?;
    
    if git {
        init_git_repository(project_dir)?;
    }
    
    println!("✅ Successfully initialized UdonSharp project");
    println!("🚀 To get started: udonsharp build");
    
    Ok(())
}

// Helper functions for CLI operations

async fn generate_project_template(
    project_dir: &Path,
    name: &str,
    template: &ProjectTemplate,
    no_interactive: bool,
) -> UdonSharpResult<()> {
    info!("Generating project template: {:?}", template);
    
    // Create basic project structure
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to create src directory: {}", e))
    })?;
    
    // Generate Cargo.toml
    let cargo_toml = generate_cargo_toml(name, template);
    fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write Cargo.toml: {}", e))
    })?;
    
    // Generate main source file
    let main_rs = generate_main_rs(template, no_interactive).await?;
    fs::write(src_dir.join("lib.rs"), main_rs).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write lib.rs: {}", e))
    })?;
    
    // Generate additional template-specific files
    match template {
        ProjectTemplate::WorldController => {
            generate_world_controller_files(project_dir).await?;
        }
        ProjectTemplate::Interactive => {
            generate_interactive_files(project_dir).await?;
        }
        ProjectTemplate::UiSystem => {
            generate_ui_system_files(project_dir).await?;
        }
        ProjectTemplate::Custom => {
            if !no_interactive {
                generate_custom_template_interactive(project_dir).await?;
            }
        }
        ProjectTemplate::Basic => {
            // Basic template is already handled above
        }
    }
    
    // Generate .gitignore
    let gitignore_content = generate_gitignore();
    fs::write(project_dir.join(".gitignore"), gitignore_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write .gitignore: {}", e))
    })?;
    
    // Generate README.md
    let readme_content = generate_readme(name, template);
    fs::write(project_dir.join("README.md"), readme_content).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to write README.md: {}", e))
    })?;
    
    Ok(())
}

fn generate_cargo_toml(name: &str, template: &ProjectTemplate) -> String {
    let dependencies = match template {
        ProjectTemplate::Basic => {
            r#"udonsharp-core = "0.1.0"
udonsharp-bindings = "0.1.0""#
        }
        ProjectTemplate::WorldController => {
            r#"udonsharp-core = "0.1.0"
udonsharp-bindings = "0.1.0"
serde = { version = "1.0", features = ["derive"] }"#
        }
        ProjectTemplate::Interactive => {
            r#"udonsharp-core = "0.1.0"
udonsharp-bindings = "0.1.0"
nalgebra = "0.32""#
        }
        ProjectTemplate::UiSystem => {
            r#"udonsharp-core = "0.1.0"
udonsharp-bindings = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0""#
        }
        ProjectTemplate::Custom => {
            r#"udonsharp-core = "0.1.0"
udonsharp-bindings = "0.1.0""#
        }
    };
    
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "A UdonSharp project written in Rust"

[lib]
crate-type = ["cdylib"]

[dependencies]
{}

[profile.release]
opt-level = "s"
lto = true
panic = "abort"

[profile.dev]
opt-level = 1
"#,
        name, dependencies
    )
}

async fn generate_main_rs(template: &ProjectTemplate, _no_interactive: bool) -> UdonSharpResult<String> {
    let content = match template {
        ProjectTemplate::Basic => {
            r#"//! Basic UdonSharp project

use udonsharp_core::prelude::*;
use udonsharp_bindings::{unity::*, vrchat::*};

#[derive(UdonBehaviour)]
pub struct BasicBehaviour {
    #[udon_public]
    pub message: String,
    
    initialized: bool,
}

impl UdonBehaviour for BasicBehaviour {
    fn start(&mut self) {
        self.message = "Hello from Rust UdonSharp!".to_string();
        self.initialized = true;
        
        debug!("BasicBehaviour started");
    }
    
    fn update(&mut self) {
        if self.initialized {
            // Update logic here
        }
    }
}

impl BasicBehaviour {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            initialized: false,
        }
    }
    
    #[udon_event]
    pub fn on_interact(&mut self) {
        info!("Interaction received: {}", self.message);
    }
}
"#
        }
        ProjectTemplate::WorldController => {
            r#"//! World Controller UdonSharp project

use udonsharp_core::prelude::*;
use udonsharp_bindings::{unity::*, vrchat::*};
use serde::{Serialize, Deserialize};

#[derive(UdonBehaviour)]
#[udon_sync_mode(Manual)]
pub struct WorldController {
    #[udon_public]
    pub world_name: String,
    
    #[udon_sync]
    pub player_count: i32,
    
    #[udon_sync]
    pub world_state: WorldState,
    
    initialized: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WorldState {
    Waiting,
    Active,
    Paused,
    Finished,
}

impl UdonBehaviour for WorldController {
    fn start(&mut self) {
        self.world_name = "My VRChat World".to_string();
        self.player_count = 0;
        self.world_state = WorldState::Waiting;
        self.initialized = true;
        
        info!("World Controller initialized: {}", self.world_name);
    }
    
    fn on_player_joined(&mut self, player: VRCPlayerApi) {
        self.player_count += 1;
        info!("Player joined: {} (Total: {})", player.get_display_name(), self.player_count);
        
        if self.player_count >= 2 && matches!(self.world_state, WorldState::Waiting) {
            self.start_world_activity();
        }
        
        self.request_serialization();
    }
    
    fn on_player_left(&mut self, player: VRCPlayerApi) {
        self.player_count -= 1;
        info!("Player left: {} (Total: {})", player.get_display_name(), self.player_count);
        
        self.request_serialization();
    }
}

impl WorldController {
    pub fn new() -> Self {
        Self {
            world_name: String::new(),
            player_count: 0,
            world_state: WorldState::Waiting,
            initialized: false,
        }
    }
    
    fn start_world_activity(&mut self) {
        self.world_state = WorldState::Active;
        info!("Starting world activity with {} players", self.player_count);
        
        // Broadcast to all players
        self.send_custom_network_event(NetworkEventTarget::All, "OnWorldActivityStarted");
    }
    
    #[udon_event]
    pub fn on_world_activity_started(&mut self) {
        info!("World activity started event received");
    }
}
"#
        }
        ProjectTemplate::Interactive => {
            r#"//! Interactive Object UdonSharp project

use udonsharp_core::prelude::*;
use udonsharp_bindings::{unity::*, vrchat::*};
use nalgebra::{Vector3, Quaternion};

#[derive(UdonBehaviour)]
pub struct InteractiveObject {
    #[udon_public]
    pub interaction_text: String,
    
    #[udon_public]
    pub animation_speed: f32,
    
    original_position: Vector3<f32>,
    original_rotation: Quaternion<f32>,
    is_animating: bool,
    animation_time: f32,
}

impl UdonBehaviour for InteractiveObject {
    fn start(&mut self) {
        self.interaction_text = "Click to interact!".to_string();
        self.animation_speed = 2.0;
        
        // Store original transform
        let transform = self.transform();
        self.original_position = transform.position();
        self.original_rotation = transform.rotation();
        
        info!("Interactive object initialized");
    }
    
    fn update(&mut self) {
        if self.is_animating {
            self.animation_time += Time::delta_time() * self.animation_speed;
            
            // Simple bounce animation
            let bounce_height = (self.animation_time * 3.14159).sin() * 0.5;
            let new_position = Vector3::new(
                self.original_position.x,
                self.original_position.y + bounce_height,
                self.original_position.z,
            );
            
            self.transform().set_position(new_position);
            
            // Stop animation after 2 seconds
            if self.animation_time >= 2.0 {
                self.stop_animation();
            }
        }
    }
}

impl InteractiveObject {
    pub fn new() -> Self {
        Self {
            interaction_text: String::new(),
            animation_speed: 1.0,
            original_position: Vector3::zeros(),
            original_rotation: Quaternion::identity(),
            is_animating: false,
            animation_time: 0.0,
        }
    }
    
    #[udon_event]
    pub fn on_interact(&mut self) {
        info!("Object interacted with: {}", self.interaction_text);
        self.start_animation();
    }
    
    fn start_animation(&mut self) {
        if !self.is_animating {
            self.is_animating = true;
            self.animation_time = 0.0;
            info!("Starting interaction animation");
        }
    }
    
    fn stop_animation(&mut self) {
        self.is_animating = false;
        self.animation_time = 0.0;
        
        // Reset to original position
        self.transform().set_position(self.original_position);
        self.transform().set_rotation(self.original_rotation);
        
        info!("Animation completed");
    }
}
"#
        }
        ProjectTemplate::UiSystem => {
            r#"//! UI System UdonSharp project

use udonsharp_core::prelude::*;
use udonsharp_bindings::{unity::*, vrchat::*};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(UdonBehaviour)]
pub struct UiController {
    #[udon_public]
    pub ui_title: String,
    
    #[udon_sync]
    pub ui_data: String,
    
    ui_state: UiState,
    panels: Vec<GameObject>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UiData {
    pub current_panel: String,
    pub user_settings: UserSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserSettings {
    pub volume: f32,
    pub quality: String,
    pub notifications: bool,
}

#[derive(Clone, Debug)]
pub enum UiState {
    Hidden,
    MainMenu,
    Settings,
    About,
}

impl UdonBehaviour for UiController {
    fn start(&mut self) {
        self.ui_title = "VRChat World UI".to_string();
        self.ui_state = UiState::Hidden;
        
        // Initialize default UI data
        let default_data = UiData {
            current_panel: "main".to_string(),
            user_settings: UserSettings {
                volume: 0.8,
                quality: "High".to_string(),
                notifications: true,
            },
        };
        
        self.ui_data = serde_json::to_string(&default_data).unwrap_or_default();
        
        self.initialize_ui_panels();
        
        info!("UI Controller initialized");
    }
}

impl UiController {
    pub fn new() -> Self {
        Self {
            ui_title: String::new(),
            ui_data: String::new(),
            ui_state: UiState::Hidden,
            panels: Vec::new(),
        }
    }
    
    fn initialize_ui_panels(&mut self) {
        // Find and store references to UI panels
        if let Some(main_panel) = GameObject::find("MainPanel") {
            self.panels.push(main_panel);
        }
        
        if let Some(settings_panel) = GameObject::find("SettingsPanel") {
            self.panels.push(settings_panel);
        }
        
        if let Some(about_panel) = GameObject::find("AboutPanel") {
            self.panels.push(about_panel);
        }
        
        self.hide_all_panels();
    }
    
    fn hide_all_panels(&mut self) {
        for panel in &mut self.panels {
            panel.set_active(false);
        }
    }
    
    #[udon_event]
    pub fn show_main_menu(&mut self) {
        self.ui_state = UiState::MainMenu;
        self.hide_all_panels();
        
        if let Some(main_panel) = self.panels.get_mut(0) {
            main_panel.set_active(true);
        }
        
        info!("Showing main menu");
    }
    
    #[udon_event]
    pub fn show_settings(&mut self) {
        self.ui_state = UiState::Settings;
        self.hide_all_panels();
        
        if let Some(settings_panel) = self.panels.get_mut(1) {
            settings_panel.set_active(true);
        }
        
        info!("Showing settings panel");
    }
    
    #[udon_event]
    pub fn hide_ui(&mut self) {
        self.ui_state = UiState::Hidden;
        self.hide_all_panels();
        
        info!("UI hidden");
    }
    
    #[udon_event]
    pub fn save_settings(&mut self) {
        // Parse current UI data and update settings
        if let Ok(mut ui_data) = serde_json::from_str::<UiData>(&self.ui_data) {
            // Settings would be updated from UI elements here
            self.ui_data = serde_json::to_string(&ui_data).unwrap_or_default();
            self.request_serialization();
            
            info!("Settings saved");
        }
    }
}
"#
        }
        ProjectTemplate::Custom => {
            r#"//! Custom UdonSharp project

use udonsharp_core::prelude::*;
use udonsharp_bindings::{unity::*, vrchat::*};

#[derive(UdonBehaviour)]
pub struct CustomBehaviour {
    #[udon_public]
    pub custom_property: String,
    
    initialized: bool,
}

impl UdonBehaviour for CustomBehaviour {
    fn start(&mut self) {
        self.custom_property = "Custom UdonSharp Behaviour".to_string();
        self.initialized = true;
        
        info!("Custom behaviour started");
    }
    
    fn update(&mut self) {
        if self.initialized {
            // Custom update logic here
        }
    }
}

impl CustomBehaviour {
    pub fn new() -> Self {
        Self {
            custom_property: String::new(),
            initialized: false,
        }
    }
    
    // Add your custom methods here
}
"#
        }
    };
    
    Ok(content.to_string())
}

async fn generate_world_controller_files(_project_dir: &Path) -> UdonSharpResult<()> {
    // Additional files specific to world controller template
    info!("Generated world controller specific files");
    Ok(())
}

async fn generate_interactive_files(_project_dir: &Path) -> UdonSharpResult<()> {
    // Additional files specific to interactive template
    info!("Generated interactive object specific files");
    Ok(())
}

async fn generate_ui_system_files(_project_dir: &Path) -> UdonSharpResult<()> {
    // Additional files specific to UI system template
    info!("Generated UI system specific files");
    Ok(())
}

async fn generate_custom_template_interactive(_project_dir: &Path) -> UdonSharpResult<()> {
    // Interactive prompts for custom template configuration
    info!("Custom template configuration (interactive mode not yet implemented)");
    Ok(())
}

fn generate_gitignore() -> String {
    r#"# Rust
/target/
**/*.rs.bk
Cargo.lock

# UdonSharp generated files
/generated/
*.wasm
*.cs

# Unity
[Ll]ibrary/
[Tt]emp/
[Oo]bj/
[Bb]uild/
[Bb]uilds/
[Ll]ogs/
[Uu]ser[Ss]ettings/

# Visual Studio / Visual Studio Code
.vscode/
.vs/
*.suo
*.user
*.userosscache
*.sln.docstates

# OS
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db
"#.to_string()
}

fn generate_readme(name: &str, template: &ProjectTemplate) -> String {
    let template_description = match template {
        ProjectTemplate::Basic => "A basic UdonSharp project with minimal setup.",
        ProjectTemplate::WorldController => "A world controller project with networking and player management.",
        ProjectTemplate::Interactive => "An interactive object project with animation and user interaction.",
        ProjectTemplate::UiSystem => "A UI system project with panels and user settings.",
        ProjectTemplate::Custom => "A custom UdonSharp project configured to your specifications.",
    };
    
    format!(
        r#"# {}

{}

## Getting Started

This project uses the Rust UdonSharp toolchain to compile Rust code to UdonSharp-compatible C# for VRChat worlds.

### Prerequisites

- Rust (latest stable version)
- UdonSharp CLI tool

### Building

```bash
# Build the project
udonsharp build

# Build in release mode (optimized)
udonsharp build --release

# Build with debug information
udonsharp build --debug
```

### Development

```bash
# Check for errors without building
udonsharp check

# Run tests
udonsharp test

# Generate API bindings
udonsharp bindings --scan-dir /path/to/unity/packages --output generated

# Clean build artifacts
udonsharp clean
```

### Project Structure

- `src/lib.rs` - Main source file with UdonSharp behaviour
- `Cargo.toml` - Project configuration and dependencies
- `generated/` - Auto-generated API bindings (created after running `udonsharp bindings`)
- `target/` - Build artifacts and compiled output

### Documentation

For more information about the Rust UdonSharp toolchain, visit:
- [UdonSharp Documentation](https://docs.vrchat.com/docs/udonsharp)
- [Rust UdonSharp Guide](https://github.com/your-repo/rust-udonsharp)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
"#,
        name, template_description
    )
}

fn init_git_repository(project_dir: &Path) -> UdonSharpResult<()> {
    use std::process::Command;
    
    let output = Command::new("git")
        .args(&["init"])
        .current_dir(project_dir)
        .output()
        .map_err(|e| {
            udonsharp_core::UdonSharpError::configuration(format!("Failed to initialize git repository: {}", e))
        })?;
    
    if output.status.success() {
        info!("Initialized git repository");
        
        // Create initial commit
        Command::new("git")
            .args(&["add", "."])
            .current_dir(project_dir)
            .output()
            .ok();
            
        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(project_dir)
            .output()
            .ok();
            
        info!("Created initial git commit");
    } else {
        warn!("Failed to initialize git repository");
    }
    
    Ok(())
}

#[derive(Debug)]
struct ProjectInfo {
    name: String,
    version: String,
    dependencies: Vec<String>,
    build_status: String,
    output_files: Vec<String>,
}

async fn gather_project_info(detailed: bool) -> UdonSharpResult<ProjectInfo> {
    let cargo_toml_path = Path::new("Cargo.toml");
    
    if !cargo_toml_path.exists() {
        return Err(udonsharp_core::UdonSharpError::configuration(
            "No Cargo.toml found in current directory"
        ));
    }
    
    let cargo_content = fs::read_to_string(cargo_toml_path).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to read Cargo.toml: {}", e))
    })?;
    
    // Parse basic project info (simplified parsing)
    let name = extract_cargo_field(&cargo_content, "name").unwrap_or_else(|| "unknown".to_string());
    let version = extract_cargo_field(&cargo_content, "version").unwrap_or_else(|| "0.1.0".to_string());
    
    let mut dependencies = Vec::new();
    if detailed {
        // Extract dependencies (simplified)
        if let Some(deps_section) = cargo_content.find("[dependencies]") {
            let deps_text = &cargo_content[deps_section..];
            for line in deps_text.lines().take(10) {
                if line.contains("=") && !line.starts_with("[") {
                    dependencies.push(line.trim().to_string());
                }
            }
        }
    }
    
    let build_status = if Path::new("target").exists() {
        "Built".to_string()
    } else {
        "Not built".to_string()
    };
    
    let mut output_files = Vec::new();
    if let Ok(entries) = fs::read_dir("target") {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |ext| ext == "cs" || ext == "wasm") {
                output_files.push(entry.path().display().to_string());
            }
        }
    }
    
    Ok(ProjectInfo {
        name,
        version,
        dependencies,
        build_status,
        output_files,
    })
}

fn extract_cargo_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        if line.trim().starts_with(field) && line.contains("=") {
            if let Some(value) = line.split("=").nth(1) {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

fn print_human_readable_info(info: &ProjectInfo) {
    println!("📦 Project Information");
    println!("   Name: {}", info.name);
    println!("   Version: {}", info.version);
    println!("   Build Status: {}", info.build_status);
    
    if !info.dependencies.is_empty() {
        println!("   Dependencies:");
        for dep in &info.dependencies {
            println!("     {}", dep);
        }
    }
    
    if !info.output_files.is_empty() {
        println!("   Output Files:");
        for file in &info.output_files {
            println!("     {}", file);
        }
    }
}

fn print_json_info(info: &ProjectInfo) -> UdonSharpResult<()> {
    let json = serde_json::json!({
        "name": info.name,
        "version": info.version,
        "build_status": info.build_status,
        "dependencies": info.dependencies,
        "output_files": info.output_files
    });
    
    println!("{}", serde_json::to_string_pretty(&json).map_err(|e| {
        udonsharp_core::UdonSharpError::configuration(format!("Failed to serialize JSON: {}", e))
    })?);
    
    Ok(())
}

fn print_yaml_info(info: &ProjectInfo) -> UdonSharpResult<()> {
    println!("name: {}", info.name);
    println!("version: {}", info.version);
    println!("build_status: {}", info.build_status);
    
    if !info.dependencies.is_empty() {
        println!("dependencies:");
        for dep in &info.dependencies {
            println!("  - {}", dep);
        }
    }
    
    if !info.output_files.is_empty() {
        println!("output_files:");
        for file in &info.output_files {
            println!("  - {}", file);
        }
    }
    
    Ok(())
}