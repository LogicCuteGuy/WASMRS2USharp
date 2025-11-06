//! CLI command for analyzing multi-behavior dependencies and generating reports
//! 
//! This module provides the `analyze` command that can be used to analyze
//! multi-behavior projects and generate detailed reports about dependencies,
//! performance, and code quality.

use clap::{Args, Subcommand};
use udonsharp_compiler::{
    DependencyAnalyzerTool, 
    CompilationReporter,
    StandardMultiBehaviorIntegration,
    UdonSharpConfig,
};
use udonsharp_core::{UdonSharpResult, error::CompilationContext};
use std::path::PathBuf;
use std::fs;

/// Analyze command and subcommands
#[derive(Debug, Args)]
pub struct AnalyzeCommand {
    #[command(subcommand)]
    pub subcommand: AnalyzeSubcommand,
}

/// Analyze subcommands
#[derive(Debug, Subcommand)]
pub enum AnalyzeSubcommand {
    /// Analyze dependencies between behaviors
    Dependencies(DependenciesArgs),
    /// Generate compilation report
    Report(ReportArgs),
    /// Analyze code quality metrics
    Quality(QualityArgs),
    /// Generate dependency graph visualization
    Graph(GraphArgs),
}

/// Arguments for dependency analysis
#[derive(Debug, Args)]
pub struct DependenciesArgs {
    /// Path to the Rust source file or project directory
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
    
    /// Output format (text, json, dot)
    #[arg(short, long, default_value = "text")]
    pub format: String,
    
    /// Output file (if not specified, prints to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Include detailed analysis
    #[arg(long)]
    pub detailed: bool,
    
    /// Check for circular dependencies only
    #[arg(long)]
    pub check_cycles: bool,
}

/// Arguments for compilation report
#[derive(Debug, Args)]
pub struct ReportArgs {
    /// Path to the Rust source file or project directory
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
    
    /// Output format (text, json, html)
    #[arg(short, long, default_value = "text")]
    pub format: String,
    
    /// Output file (if not specified, prints to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Include performance metrics
    #[arg(long)]
    pub include_performance: bool,
    
    /// Include quality metrics
    #[arg(long)]
    pub include_quality: bool,
}

/// Arguments for quality analysis
#[derive(Debug, Args)]
pub struct QualityArgs {
    /// Path to the Rust source file or project directory
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
    
    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    pub format: String,
    
    /// Output file (if not specified, prints to stdout)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
    
    /// Minimum quality threshold (0.0 to 1.0)
    #[arg(long, default_value = "0.7")]
    pub threshold: f32,
    
    /// Fail if quality is below threshold
    #[arg(long)]
    pub fail_on_low_quality: bool,
}

/// Arguments for graph generation
#[derive(Debug, Args)]
pub struct GraphArgs {
    /// Path to the Rust source file or project directory
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,
    
    /// Output file for the graph (DOT format)
    #[arg(short, long, default_value = "dependency_graph.dot")]
    pub output: PathBuf,
    
    /// Generate SVG output (requires graphviz)
    #[arg(long)]
    pub svg: bool,
    
    /// Generate PNG output (requires graphviz)
    #[arg(long)]
    pub png: bool,
    
    /// Include isolated behaviors in graph
    #[arg(long)]
    pub include_isolated: bool,
}

/// Execute the analyze command
pub async fn execute_analyze_command(args: AnalyzeCommand) -> UdonSharpResult<()> {
    match args.subcommand {
        AnalyzeSubcommand::Dependencies(deps_args) => {
            execute_dependencies_analysis(deps_args).await
        }
        AnalyzeSubcommand::Report(report_args) => {
            execute_report_generation(report_args).await
        }
        AnalyzeSubcommand::Quality(quality_args) => {
            execute_quality_analysis(quality_args).await
        }
        AnalyzeSubcommand::Graph(graph_args) => {
            execute_graph_generation(graph_args).await
        }
    }
}

/// Execute dependency analysis
async fn execute_dependencies_analysis(args: DependenciesArgs) -> UdonSharpResult<()> {
    println!("Analyzing dependencies for project: {:?}", args.path);
    
    // Read source code
    let source_code = read_source_code(&args.path)?;
    
    // Set up analysis
    let config = UdonSharpConfig::default();
    let context = CompilationContext::new();
    let integration = StandardMultiBehaviorIntegration::new(config, context);
    
    // Check if this is a multi-behavior project
    if !integration.should_use_multi_behavior(&source_code)? {
        println!("Project does not use multi-behavior pattern. No dependency analysis needed.");
        return Ok(());
    }
    
    // Perform analysis (simplified - in real implementation would use full compilation)
    let mut analyzer = DependencyAnalyzerTool::new();
    
    // TODO: Parse behaviors from source and add to analyzer
    // For now, create a placeholder analysis
    
    let analysis_report = analyzer.analyze();
    
    // Generate output based on format
    let output_content = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&analysis_report)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("JSON serialization failed: {}", e)))?,
        "dot" => analyzer.generate_dependency_graph_dot(),
        "text" | _ => {
            if args.check_cycles {
                generate_cycle_check_report(&analysis_report)
            } else if args.detailed {
                analyzer.generate_text_report()
            } else {
                generate_summary_report(&analysis_report)
            }
        }
    };
    
    // Write output
    write_output(&output_content, args.output)?;
    
    // Print summary to stderr so it doesn't interfere with piped output
    if !args.check_cycles {
        eprintln!("Analysis complete:");
        eprintln!("  Total behaviors: {}", analysis_report.total_behaviors);
        eprintln!("  Total dependencies: {}", analysis_report.total_dependencies);
        eprintln!("  Circular dependencies: {}", analysis_report.circular_dependencies.len());
        eprintln!("  Max depth: {}", analysis_report.complexity_metrics.max_depth);
    }
    
    Ok(())
}

/// Execute report generation
async fn execute_report_generation(args: ReportArgs) -> UdonSharpResult<()> {
    println!("Generating compilation report for project: {:?}", args.path);
    
    // Read source code
    let source_code = read_source_code(&args.path)?;
    
    // Set up compilation
    let config = UdonSharpConfig::default();
    let context = CompilationContext::new();
    let integration = StandardMultiBehaviorIntegration::new(config, context);
    
    // Check if this is a multi-behavior project
    if !integration.should_use_multi_behavior(&source_code)? {
        println!("Project does not use multi-behavior pattern. Generating basic report.");
    }
    
    // Perform compilation to get detailed metrics
    let compilation_result = integration.compile_multi_behavior(&source_code).await?;
    
    // Set up reporter
    let mut reporter = CompilationReporter::new();
    
    // Generate dependency analysis
    let mut dependency_analyzer = DependencyAnalyzerTool::new();
    let dependency_analysis = dependency_analyzer.analyze();
    
    // Generate comprehensive report
    let behaviors = vec![]; // TODO: Extract behaviors from compilation result
    let report = reporter.generate_report(&compilation_result, &behaviors, &dependency_analysis);
    
    // Generate output based on format
    let output_content = match args.format.as_str() {
        "json" => reporter.generate_json_report(&report)?,
        "html" => generate_html_report(&report),
        "text" | _ => reporter.generate_text_report(&report),
    };
    
    // Write output
    write_output(&output_content, args.output)?;
    
    println!("Report generated successfully");
    
    Ok(())
}

/// Execute quality analysis
async fn execute_quality_analysis(args: QualityArgs) -> UdonSharpResult<()> {
    println!("Analyzing code quality for project: {:?}", args.path);
    
    // Read source code
    let source_code = read_source_code(&args.path)?;
    
    // Set up compilation
    let config = UdonSharpConfig::default();
    let context = CompilationContext::new();
    let integration = StandardMultiBehaviorIntegration::new(config, context);
    
    // Perform compilation
    let compilation_result = integration.compile_multi_behavior(&source_code).await?;
    
    // Generate quality report
    let mut reporter = CompilationReporter::new();
    let behaviors = vec![]; // TODO: Extract behaviors
    let dependency_analysis = DependencyAnalyzerTool::new().analyze();
    let report = reporter.generate_report(&compilation_result, &behaviors, &dependency_analysis);
    
    // Check quality threshold
    let quality_score = report.quality_metrics.maintainability_index / 100.0;
    let meets_threshold = quality_score >= args.threshold;
    
    // Generate output
    let output_content = match args.format.as_str() {
        "json" => {
            let quality_report = serde_json::json!({
                "quality_score": quality_score,
                "threshold": args.threshold,
                "meets_threshold": meets_threshold,
                "metrics": report.quality_metrics,
                "warnings": report.warnings.iter()
                    .filter(|w| matches!(w.category, udonsharp_compiler::WarningCategory::Performance | 
                                                   udonsharp_compiler::WarningCategory::Maintainability))
                    .collect::<Vec<_>>()
            });
            serde_json::to_string_pretty(&quality_report)
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(format!("JSON serialization failed: {}", e)))?
        }
        "text" | _ => {
            generate_quality_text_report(&report.quality_metrics, quality_score, args.threshold, meets_threshold)
        }
    };
    
    // Write output
    write_output(&output_content, args.output)?;
    
    // Print result
    if meets_threshold {
        println!("✅ Quality analysis passed (score: {:.1}%, threshold: {:.1}%)", 
            quality_score * 100.0, args.threshold * 100.0);
    } else {
        println!("❌ Quality analysis failed (score: {:.1}%, threshold: {:.1}%)", 
            quality_score * 100.0, args.threshold * 100.0);
        
        if args.fail_on_low_quality {
            return Err(udonsharp_core::UdonSharpError::compilation(
                format!("Quality score {:.1}% is below threshold {:.1}%", 
                    quality_score * 100.0, args.threshold * 100.0)
            ));
        }
    }
    
    Ok(())
}

/// Execute graph generation
async fn execute_graph_generation(args: GraphArgs) -> UdonSharpResult<()> {
    println!("Generating dependency graph for project: {:?}", args.path);
    
    // Read source code
    let source_code = read_source_code(&args.path)?;
    
    // Set up analysis
    let config = UdonSharpConfig::default();
    let context = CompilationContext::new();
    let integration = StandardMultiBehaviorIntegration::new(config, context);
    
    // Check if this is a multi-behavior project
    if !integration.should_use_multi_behavior(&source_code)? {
        println!("Project does not use multi-behavior pattern. No dependency graph needed.");
        return Ok(());
    }
    
    // Generate dependency graph
    let analyzer = DependencyAnalyzerTool::new();
    let dot_content = analyzer.generate_dependency_graph_dot();
    
    // Write DOT file
    fs::write(&args.output, &dot_content)
        .map_err(|e| udonsharp_core::UdonSharpError::compilation(
            format!("Failed to write DOT file: {}", e)
        ))?;
    
    println!("DOT file generated: {:?}", args.output);
    
    // Generate additional formats if requested
    if args.svg || args.png {
        generate_graphviz_output(&args.output, args.svg, args.png)?;
    }
    
    Ok(())
}

/// Read source code from path
fn read_source_code(path: &PathBuf) -> UdonSharpResult<String> {
    if path.is_file() {
        fs::read_to_string(path)
            .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                format!("Failed to read source file {:?}: {}", path, e)
            ))
    } else if path.is_dir() {
        // Try to find main source file
        let possible_files = [
            path.join("src/lib.rs"),
            path.join("src/main.rs"),
            path.join("lib.rs"),
            path.join("main.rs"),
        ];
        
        for file_path in &possible_files {
            if file_path.exists() {
                return fs::read_to_string(file_path)
                    .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                        format!("Failed to read source file {:?}: {}", file_path, e)
                    ));
            }
        }
        
        Err(udonsharp_core::UdonSharpError::compilation(
            format!("Could not find Rust source file in directory: {:?}", path)
        ))
    } else {
        Err(udonsharp_core::UdonSharpError::compilation(
            format!("Path does not exist: {:?}", path)
        ))
    }
}

/// Write output to file or stdout
fn write_output(content: &str, output_path: Option<PathBuf>) -> UdonSharpResult<()> {
    match output_path {
        Some(path) => {
            fs::write(&path, content)
                .map_err(|e| udonsharp_core::UdonSharpError::compilation(
                    format!("Failed to write output file {:?}: {}", path, e)
                ))?;
            println!("Output written to: {:?}", path);
        }
        None => {
            print!("{}", content);
        }
    }
    Ok(())
}

/// Generate cycle check report
fn generate_cycle_check_report(analysis: &udonsharp_compiler::DependencyAnalysisReport) -> String {
    if analysis.circular_dependencies.is_empty() {
        "✅ No circular dependencies detected".to_string()
    } else {
        let mut report = format!("❌ {} circular dependencies detected:\n\n", 
            analysis.circular_dependencies.len());
        
        for (i, cycle) in analysis.circular_dependencies.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, cycle.join(" -> ")));
        }
        
        report
    }
}

/// Generate summary report
fn generate_summary_report(analysis: &udonsharp_compiler::DependencyAnalysisReport) -> String {
    format!(
        "Multi-Behavior Dependency Summary\n\
         ================================\n\
         Total Behaviors: {}\n\
         Total Dependencies: {}\n\
         Root Behaviors: {}\n\
         Leaf Behaviors: {}\n\
         Circular Dependencies: {}\n\
         Max Dependency Depth: {}\n\
         Coupling Factor: {:.2}\n\
         Cohesion Score: {:.2}\n",
        analysis.total_behaviors,
        analysis.total_dependencies,
        analysis.root_behaviors.len(),
        analysis.leaf_behaviors.len(),
        analysis.circular_dependencies.len(),
        analysis.complexity_metrics.max_depth,
        analysis.complexity_metrics.coupling_factor,
        analysis.complexity_metrics.cohesion_score
    )
}

/// Generate quality text report
fn generate_quality_text_report(
    metrics: &udonsharp_compiler::QualityMetrics,
    quality_score: f32,
    threshold: f32,
    meets_threshold: bool,
) -> String {
    format!(
        "Code Quality Analysis\n\
         ====================\n\
         Overall Quality Score: {:.1}% {}\n\
         Threshold: {:.1}%\n\n\
         Detailed Metrics:\n\
         - UdonSharp Compatibility: {:.1}%\n\
         - Performance Score: {:.1}%\n\
         - Maintainability Index: {:.1}\n\
         - Code Duplication: {:.1}%\n\
         - Avg Cyclomatic Complexity: {:.1}\n\n\
         Result: {}\n",
        quality_score * 100.0,
        if meets_threshold { "✅" } else { "❌" },
        threshold * 100.0,
        metrics.udonsharp_compatibility_score * 100.0,
        metrics.performance_score * 100.0,
        metrics.maintainability_index,
        metrics.code_duplication_percentage,
        metrics.avg_cyclomatic_complexity,
        if meets_threshold { "PASSED" } else { "FAILED" }
    )
}

/// Generate HTML report
fn generate_html_report(report: &udonsharp_compiler::CompilationReport) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>UdonSharp-Rust Compilation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 20px; border-radius: 5px; }}
        .section {{ margin: 20px 0; }}
        .metric {{ display: inline-block; margin: 10px; padding: 10px; background-color: #e8f4f8; border-radius: 3px; }}
        .success {{ color: green; }}
        .warning {{ color: orange; }}
        .error {{ color: red; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>UdonSharp-Rust Compilation Report</h1>
        <p>Generated: {}</p>
        <p>Compiler Version: {}</p>
    </div>
    
    <div class="section">
        <h2>Success Indicators</h2>
        <div class="metric {}">Compilation: {}</div>
        <div class="metric {}">Quality Gates: {}</div>
        <div class="metric {}">Performance: {}</div>
    </div>
    
    <div class="section">
        <h2>Code Generation Statistics</h2>
        <table>
            <tr><th>Metric</th><th>Value</th></tr>
            <tr><td>Rust Source Lines</td><td>{}</td></tr>
            <tr><td>Generated C# Lines</td><td>{}</td></tr>
            <tr><td>Code Expansion Ratio</td><td>{:.2}x</td></tr>
        </table>
    </div>
    
    <div class="section">
        <h2>Quality Metrics</h2>
        <table>
            <tr><th>Metric</th><th>Score</th></tr>
            <tr><td>UdonSharp Compatibility</td><td>{:.1}%</td></tr>
            <tr><td>Performance Score</td><td>{:.1}%</td></tr>
            <tr><td>Maintainability Index</td><td>{:.1}</td></tr>
        </table>
    </div>
</body>
</html>"#,
        report.compilation_info.timestamp,
        report.compilation_info.compiler_version,
        if report.success_indicators.compilation_successful { "success" } else { "error" },
        if report.success_indicators.compilation_successful { "SUCCESS" } else { "FAILED" },
        if report.success_indicators.quality_gates_passed { "success" } else { "warning" },
        if report.success_indicators.quality_gates_passed { "PASSED" } else { "FAILED" },
        if report.success_indicators.performance_targets_met { "success" } else { "warning" },
        if report.success_indicators.performance_targets_met { "MET" } else { "NOT MET" },
        report.code_generation_stats.rust_source_lines,
        report.code_generation_stats.generated_csharp_lines,
        report.code_generation_stats.code_expansion_ratio,
        report.quality_metrics.udonsharp_compatibility_score * 100.0,
        report.quality_metrics.performance_score * 100.0,
        report.quality_metrics.maintainability_index
    )
}

/// Generate graphviz output formats
fn generate_graphviz_output(dot_file: &PathBuf, svg: bool, png: bool) -> UdonSharpResult<()> {
    use std::process::Command;
    
    if svg {
        let svg_file = dot_file.with_extension("svg");
        let output = Command::new("dot")
            .args(["-Tsvg", "-o"])
            .arg(&svg_file)
            .arg(dot_file)
            .output();
        
        match output {
            Ok(result) if result.status.success() => {
                println!("SVG file generated: {:?}", svg_file);
            }
            Ok(result) => {
                eprintln!("Warning: Failed to generate SVG: {}", 
                    String::from_utf8_lossy(&result.stderr));
            }
            Err(e) => {
                eprintln!("Warning: graphviz not found, could not generate SVG: {}", e);
            }
        }
    }
    
    if png {
        let png_file = dot_file.with_extension("png");
        let output = Command::new("dot")
            .args(["-Tpng", "-o"])
            .arg(&png_file)
            .arg(dot_file)
            .output();
        
        match output {
            Ok(result) if result.status.success() => {
                println!("PNG file generated: {:?}", png_file);
            }
            Ok(result) => {
                eprintln!("Warning: Failed to generate PNG: {}", 
                    String::from_utf8_lossy(&result.stderr));
            }
            Err(e) => {
                eprintln!("Warning: graphviz not found, could not generate PNG: {}", e);
            }
        }
    }
    
    Ok(())
}