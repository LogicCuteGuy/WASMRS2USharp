//! Performance reporting and visualization system

use crate::metrics::{PerformanceMetrics, VRChatMetrics, PerformanceGrade};
use crate::analysis::OptimizationRecommendation;
use crate::profiler::CompletedOperation;
use crate::multi_behavior_metrics::MultiBehaviorReport;
use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Performance reporter for generating various report formats
#[derive(Debug)]
pub struct PerformanceReporter {
    output_directory: String,
    pub report_templates: HashMap<ReportFormat, ReportTemplate>,
}

/// Available report formats
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Html,
    Markdown,
    Csv,
    Console,
}

/// Report template configuration
#[derive(Debug, Clone)]
pub struct ReportTemplate {
    pub format: ReportFormat,
    pub include_metrics: bool,
    pub include_recommendations: bool,
    pub include_profiling_data: bool,
    pub include_charts: bool,
    pub custom_sections: Vec<String>,
}

/// Complete performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub metadata: ReportMetadata,
    pub summary: ReportSummary,
    pub metrics: PerformanceMetrics,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub profiling_data: Vec<CompletedOperation>,
    pub vrchat_analysis: Option<VRChatMetrics>,
    pub trends: Option<PerformanceTrends>,
    pub multi_behavior_analysis: Option<MultiBehaviorReport>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub session_id: String,
    pub session_name: String,
    pub generated_at: DateTime<Utc>,
    pub generator_version: String,
    pub project_name: Option<String>,
    pub project_version: Option<String>,
}

/// High-level report summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub overall_grade: PerformanceGrade,
    pub overall_score: f64,
    pub compilation_time: f64, // seconds
    pub peak_memory_mb: f64,
    pub critical_issues: u32,
    pub high_priority_recommendations: u32,
    pub success_rate: f64,
    pub key_insights: Vec<String>,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub compilation_time_trend: TrendDirection,
    pub memory_usage_trend: TrendDirection,
    pub success_rate_trend: TrendDirection,
    pub historical_data: Vec<HistoricalDataPoint>,
}

/// Trend direction indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Unknown,
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    pub timestamp: DateTime<Utc>,
    pub compilation_time: f64,
    pub memory_usage: u64,
    pub success_rate: f64,
    pub overall_score: f64,
}

impl PerformanceReporter {
    /// Create a new performance reporter
    pub fn new() -> Self {
        let mut reporter = Self {
            output_directory: "performance_reports".to_string(),
            report_templates: HashMap::new(),
        };

        reporter.initialize_default_templates();
        reporter
    }

    /// Create reporter with custom output directory
    pub fn with_output_directory(directory: &str) -> Self {
        let mut reporter = Self::new();
        reporter.output_directory = directory.to_string();
        reporter
    }

    /// Generate a performance report for a session
    pub fn generate_session_report(
        &self,
        session_id: &str,
        metrics: &PerformanceMetrics,
        recommendations: &[OptimizationRecommendation],
    ) -> Result<()> {
        let report = self.create_performance_report(session_id, metrics, recommendations, &[])?;
        
        // Generate reports in all configured formats
        for (format, template) in &self.report_templates {
            self.generate_report_format(&report, format, template)?;
        }

        Ok(())
    }

    /// Generate a comprehensive report with profiling data
    pub fn generate_comprehensive_report(
        &self,
        session_id: &str,
        metrics: &PerformanceMetrics,
        recommendations: &[OptimizationRecommendation],
        profiling_data: &[CompletedOperation],
        vrchat_metrics: Option<&VRChatMetrics>,
    ) -> Result<PerformanceReport> {
        let mut report = self.create_performance_report(session_id, metrics, recommendations, profiling_data)?;
        report.vrchat_analysis = vrchat_metrics.cloned();
        
        // Generate all report formats
        for (format, template) in &self.report_templates {
            self.generate_report_format(&report, format, template)?;
        }

        Ok(report)
    }

    /// Generate report in specific format
    pub fn generate_report_format(
        &self,
        report: &PerformanceReport,
        format: &ReportFormat,
        template: &ReportTemplate,
    ) -> Result<String> {
        let content = match format {
            ReportFormat::Json => self.generate_json_report(report)?,
            ReportFormat::Html => self.generate_html_report(report, template)?,
            ReportFormat::Markdown => self.generate_markdown_report(report, template)?,
            ReportFormat::Csv => self.generate_csv_report(report)?,
            ReportFormat::Console => self.generate_console_report(report)?,
        };

        // Write to file
        let filename = self.get_report_filename(&report.metadata.session_id, format);
        let filepath = Path::new(&self.output_directory).join(&filename);
        
        // Ensure output directory exists
        if let Some(parent) = filepath.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&filepath, &content)?;
        println!("Generated {} report: {}", format_name(format), filepath.display());

        Ok(content)
    }

    /// Generate multi-behavior specific report
    pub fn generate_multi_behavior_report(
        &self,
        session_id: &str,
        multi_behavior_report: &MultiBehaviorReport,
    ) -> Result<()> {
        // Generate specialized multi-behavior report
        let report_content = self.generate_multi_behavior_markdown(multi_behavior_report)?;
        
        let filename = format!("multi_behavior_report_{}_{}.md", 
            session_id, 
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let filepath = std::path::Path::new(&self.output_directory).join(&filename);
        
        // Ensure output directory exists
        if let Some(parent) = filepath.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&filepath, &report_content)?;
        println!("Generated multi-behavior report: {}", filepath.display());

        // Also generate JSON version for programmatic access
        let json_content = serde_json::to_string_pretty(multi_behavior_report)?;
        let json_filename = format!("multi_behavior_report_{}_{}.json", 
            session_id, 
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        );
        let json_filepath = std::path::Path::new(&self.output_directory).join(&json_filename);
        std::fs::write(&json_filepath, &json_content)?;

        Ok(())
    }

    /// Generate multi-behavior markdown report
    fn generate_multi_behavior_markdown(&self, report: &MultiBehaviorReport) -> Result<String> {
        let mut md = String::new();
        
        md.push_str("# 🔄 Multi-Behavior Performance Report\n\n");
        md.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Executive Summary
        md.push_str("## 📊 Executive Summary\n\n");
        md.push_str(&format!("- **Total Behaviors:** {}\n", report.summary.total_behaviors));
        md.push_str(&format!("- **Shared Functions:** {}\n", report.summary.shared_functions));
        md.push_str(&format!("- **Code Sharing Efficiency:** {:.1}%\n", report.summary.sharing_efficiency * 100.0));
        md.push_str(&format!("- **Performance Score:** {:.1}/10\n", report.summary.performance_score * 10.0));
        md.push_str(&format!("- **Critical Issues:** {}\n", report.summary.critical_issues));
        md.push_str(&format!("- **Optimization Opportunities:** {}\n\n", report.summary.optimization_opportunities));

        // Performance Grade
        let grade = if report.summary.performance_score >= 0.9 {
            "🟢 Excellent"
        } else if report.summary.performance_score >= 0.7 {
            "🔵 Good"
        } else if report.summary.performance_score >= 0.5 {
            "🟡 Average"
        } else if report.summary.performance_score >= 0.3 {
            "🟠 Below Average"
        } else {
            "🔴 Poor"
        };
        
        md.push_str(&format!("**Overall Grade:** {}\n\n", grade));

        // Behavior Breakdown
        if !report.metrics.behavior_metrics.is_empty() {
            md.push_str("## 🎯 Behavior Analysis\n\n");
            md.push_str("| Behavior | Functions | Lines | Complexity | Dependencies | Calls In/Out |\n");
            md.push_str("|----------|-----------|-------|------------|--------------|---------------|\n");
            
            for (name, metrics) in &report.metrics.behavior_metrics {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.1}% | {} | {}/{} |\n",
                    name,
                    metrics.function_count,
                    metrics.generated_lines,
                    metrics.complexity_score * 100.0,
                    metrics.dependencies.len(),
                    metrics.incoming_calls,
                    metrics.outgoing_calls
                ));
            }
            md.push_str("\n");
        }

        // Code Sharing Analysis
        md.push_str("## 🔄 Code Sharing Analysis\n\n");
        md.push_str(&format!("- **Shared Functions:** {}\n", report.metrics.shared_functions_count));
        md.push_str(&format!("- **Sharing Efficiency:** {:.1}%\n", report.metrics.code_sharing_efficiency * 100.0));
        md.push_str(&format!("- **Inter-Behavior Calls:** {}\n", report.metrics.inter_behavior_calls));
        
        if report.metrics.optimization_metrics.memory_savings_bytes > 0 {
            md.push_str(&format!("- **Memory Savings:** {:.1}KB\n", 
                report.metrics.optimization_metrics.memory_savings_bytes as f64 / 1024.0));
        }
        md.push_str("\n");

        // Dependency Analysis
        md.push_str("## 🔗 Dependency Analysis\n\n");
        md.push_str(&format!("- **Dependency Edges:** {}\n", report.metrics.dependency_metrics.dependency_edges));
        md.push_str(&format!("- **Max Dependency Depth:** {}\n", report.metrics.dependency_metrics.max_dependency_depth));
        md.push_str(&format!("- **Average Depth:** {:.1}\n", report.metrics.dependency_metrics.avg_dependency_depth));
        md.push_str(&format!("- **Complexity Score:** {:.1}%\n", report.metrics.dependency_metrics.complexity_score * 100.0));
        
        if report.metrics.dependency_metrics.circular_dependencies > 0 {
            md.push_str(&format!("- **⚠️ Circular Dependencies:** {}\n", report.metrics.dependency_metrics.circular_dependencies));
        }
        md.push_str("\n");

        // Performance Issues
        if !report.metrics.performance_issues.is_empty() {
            md.push_str("## ⚠️ Performance Issues\n\n");
            for (i, issue) in report.metrics.performance_issues.iter().enumerate() {
                let severity_icon = match issue.severity {
                    crate::metrics::IssueSeverity::Critical => "🔴",
                    crate::metrics::IssueSeverity::High => "🟠",
                    crate::metrics::IssueSeverity::Medium => "🟡",
                    crate::metrics::IssueSeverity::Low => "🔵",
                    crate::metrics::IssueSeverity::Info => "ℹ️",
                };
                
                md.push_str(&format!("### {}. {} {:?}\n\n", i + 1, severity_icon, issue.severity));
                md.push_str(&format!("**Category:** {:?}\n\n", issue.category));
                md.push_str(&format!("{}\n\n", issue.description));
                
                if let Some(fix) = &issue.suggested_fix {
                    md.push_str(&format!("**Suggested Fix:** {}\n\n", fix));
                }
            }
        }

        // Optimization Recommendations
        if !report.recommendations.is_empty() {
            md.push_str("## 🚀 Optimization Recommendations\n\n");
            for (i, rec) in report.recommendations.iter().enumerate() {
                md.push_str(&format!("### {}. {:?} Optimization\n\n", i + 1, rec.optimization_type));
                md.push_str(&format!("**Estimated Improvement:** {:.1}%\n", rec.estimated_improvement));
                md.push_str(&format!("**Difficulty:** {:?}\n\n", rec.difficulty));
                md.push_str(&format!("{}\n\n", rec.description));
                
                if !rec.implementation_steps.is_empty() {
                    md.push_str("**Implementation Steps:**\n");
                    for (step_i, step) in rec.implementation_steps.iter().enumerate() {
                        md.push_str(&format!("{}. {}\n", step_i + 1, step));
                    }
                    md.push_str("\n");
                }
                
                if !rec.affected_behaviors.is_empty() {
                    md.push_str(&format!("**Affected Behaviors:** {}\n\n", rec.affected_behaviors.join(", ")));
                }
            }
        }

        // Performance Metrics
        md.push_str("## 📈 Detailed Metrics\n\n");
        md.push_str("### File Generation Performance\n\n");
        md.push_str(&format!("- **Behavior Generation:** {:.2}s\n", 
            report.metrics.file_generation_metrics.behavior_generation_time.as_secs_f64()));
        md.push_str(&format!("- **SharedRuntime Generation:** {:.2}s\n", 
            report.metrics.file_generation_metrics.shared_runtime_generation_time.as_secs_f64()));
        md.push_str(&format!("- **Prefab Generation:** {:.2}s\n", 
            report.metrics.file_generation_metrics.prefab_generation_time.as_secs_f64()));
        md.push_str(&format!("- **Total Files Generated:** {}\n", 
            report.metrics.file_generation_metrics.file_count));
        md.push_str(&format!("- **Total File Size:** {:.1}KB\n", 
            report.metrics.file_generation_metrics.total_file_size as f64 / 1024.0));
        md.push_str(&format!("- **Generation Efficiency:** {:.1} lines/s\n\n", 
            report.metrics.file_generation_metrics.generation_efficiency));

        // Recommendations Summary
        md.push_str("## 💡 Next Steps\n\n");
        if report.summary.critical_issues > 0 {
            md.push_str("1. **Address Critical Issues:** Fix circular dependencies and critical performance problems\n");
        }
        if report.summary.sharing_efficiency < 0.5 {
            md.push_str("2. **Improve Code Sharing:** Move common functionality to SharedRuntime\n");
        }
        if report.summary.optimization_opportunities > 0 {
            md.push_str("3. **Apply Optimizations:** Implement the recommended optimizations for better performance\n");
        }
        md.push_str("4. **Monitor Progress:** Re-run analysis after changes to track improvements\n\n");

        md.push_str("---\n");
        md.push_str("*Generated by UdonSharp Performance Analyzer*\n");

        Ok(md)
    }

    /// Print console summary
    pub fn print_console_summary(&self, report: &PerformanceReport) {
        println!("\n🚀 UdonSharp Performance Report");
        println!("═══════════════════════════════════════");
        println!("Session: {}", report.metadata.session_name);
        println!("Generated: {}", report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC"));
        println!();

        // Overall summary
        println!("📊 Overall Performance");
        println!("Grade: {} ({:.1}%)", grade_emoji(&report.summary.overall_grade), report.summary.overall_score * 100.0);
        println!("Compilation Time: {:.2}s", report.summary.compilation_time);
        println!("Peak Memory: {:.1}MB", report.summary.peak_memory_mb);
        println!("Success Rate: {:.1}%", report.summary.success_rate * 100.0);
        println!();

        // Key insights
        if !report.summary.key_insights.is_empty() {
            println!("💡 Key Insights");
            for insight in &report.summary.key_insights {
                println!("  • {}", insight);
            }
            println!();
        }

        // Top recommendations
        let top_recommendations: Vec<_> = report.recommendations.iter()
            .take(3)
            .collect();

        if !top_recommendations.is_empty() {
            println!("🎯 Top Recommendations");
            for (i, rec) in top_recommendations.iter().enumerate() {
                println!("  {}. {} ({}% improvement)", 
                    i + 1, 
                    rec.title, 
                    rec.estimated_improvement as u32
                );
            }
            println!();
        }

        // Issues summary
        if report.summary.critical_issues > 0 || report.summary.high_priority_recommendations > 0 {
            println!("⚠️  Issues Summary");
            if report.summary.critical_issues > 0 {
                println!("  Critical Issues: {}", report.summary.critical_issues);
            }
            if report.summary.high_priority_recommendations > 0 {
                println!("  High Priority Items: {}", report.summary.high_priority_recommendations);
            }
            println!();
        }

        println!("📁 Full report saved to: {}/", self.output_directory);
    }

    /// Create a performance report structure
    pub fn create_performance_report(
        &self,
        session_id: &str,
        metrics: &PerformanceMetrics,
        recommendations: &[OptimizationRecommendation],
        profiling_data: &[CompletedOperation],
    ) -> Result<PerformanceReport> {
        let metadata = ReportMetadata {
            session_id: session_id.to_string(),
            session_name: format!("Session {}", session_id),
            generated_at: Utc::now(),
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
            project_name: None,
            project_version: None,
        };

        let summary = self.create_report_summary(metrics, recommendations);
        
        Ok(PerformanceReport {
            metadata,
            summary,
            metrics: metrics.clone(),
            recommendations: recommendations.to_vec(),
            profiling_data: profiling_data.to_vec(),
            vrchat_analysis: None,
            trends: None,
            multi_behavior_analysis: None,
        })
    }

    /// Create report summary
    fn create_report_summary(
        &self,
        metrics: &PerformanceMetrics,
        recommendations: &[OptimizationRecommendation],
    ) -> ReportSummary {
        let overall_score = metrics.get_overall_score();
        let overall_grade = metrics.get_performance_grade();
        
        let critical_issues = recommendations.iter()
            .filter(|r| r.priority == crate::analysis::RecommendationPriority::Critical)
            .count() as u32;
            
        let high_priority_recommendations = recommendations.iter()
            .filter(|r| r.priority == crate::analysis::RecommendationPriority::High)
            .count() as u32;

        let success_rate = if metrics.step_count > 0 {
            1.0 - (metrics.failed_steps as f64 / metrics.step_count as f64)
        } else {
            1.0
        };

        let key_insights = self.generate_key_insights(metrics, recommendations);

        ReportSummary {
            overall_grade,
            overall_score,
            compilation_time: metrics.compilation.total_compilation_time.as_secs_f64(),
            peak_memory_mb: metrics.memory.peak_usage as f64 / 1_000_000.0,
            critical_issues,
            high_priority_recommendations,
            success_rate,
            key_insights,
        }
    }

    /// Generate key insights from metrics and recommendations
    fn generate_key_insights(
        &self,
        metrics: &PerformanceMetrics,
        recommendations: &[OptimizationRecommendation],
    ) -> Vec<String> {
        let mut insights = Vec::new();

        // Compilation performance insights
        if metrics.compilation.compilation_speed_loc_per_second > 500.0 {
            insights.push("Excellent compilation speed - well optimized build pipeline".to_string());
        } else if metrics.compilation.compilation_speed_loc_per_second < 100.0 {
            insights.push("Compilation speed could be improved - consider build optimizations".to_string());
        }

        // Memory insights
        let memory_efficiency = metrics.memory.get_efficiency_score();
        if memory_efficiency > 0.8 {
            insights.push("Good memory efficiency during compilation".to_string());
        } else if memory_efficiency < 0.5 {
            insights.push("Memory usage patterns suggest optimization opportunities".to_string());
        }

        // Recommendation insights
        let total_improvement = recommendations.iter()
            .map(|r| r.estimated_improvement)
            .sum::<f64>();
        
        if total_improvement > 50.0 {
            insights.push(format!("Significant optimization potential: up to {:.0}% improvement possible", total_improvement));
        }

        // Success rate insights
        if metrics.failed_steps == 0 {
            insights.push("Perfect compilation success rate - stable build process".to_string());
        } else if metrics.failed_steps > metrics.step_count / 4 {
            insights.push("High failure rate indicates build stability issues".to_string());
        }

        insights
    }

    /// Generate JSON report
    fn generate_json_report(&self, report: &PerformanceReport) -> Result<String> {
        serde_json::to_string_pretty(report)
            .map_err(|e| anyhow!("Failed to serialize JSON report: {}", e))
    }

    /// Generate HTML report
    fn generate_html_report(&self, report: &PerformanceReport, _template: &ReportTemplate) -> Result<String> {
        let mut html = String::new();
        
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>UdonSharp Performance Report</title>\n");
        html.push_str("<style>\n");
        html.push_str(include_str!("../templates/report.css"));
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");

        // Header
        html.push_str(&format!(
            "<h1>🚀 UdonSharp Performance Report</h1>\n<p>Session: {} | Generated: {}</p>\n",
            report.metadata.session_name,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Summary section
        html.push_str("<div class='summary'>\n");
        html.push_str(&format!(
            "<h2>📊 Summary</h2>\n<p>Grade: {} ({:.1}%)</p>\n<p>Compilation: {:.2}s | Memory: {:.1}MB | Success: {:.1}%</p>\n",
            grade_emoji(&report.summary.overall_grade),
            report.summary.overall_score * 100.0,
            report.summary.compilation_time,
            report.summary.peak_memory_mb,
            report.summary.success_rate * 100.0
        ));
        html.push_str("</div>\n");

        // Recommendations section
        if !report.recommendations.is_empty() {
            html.push_str("<div class='recommendations'>\n<h2>🎯 Recommendations</h2>\n<ul>\n");
            for rec in &report.recommendations {
                html.push_str(&format!(
                    "<li><strong>{}</strong> - {} ({:.0}% improvement)</li>\n",
                    rec.title, rec.description, rec.estimated_improvement
                ));
            }
            html.push_str("</ul>\n</div>\n");
        }

        html.push_str("</body>\n</html>");
        Ok(html)
    }

    /// Generate Markdown report
    fn generate_markdown_report(&self, report: &PerformanceReport, _template: &ReportTemplate) -> Result<String> {
        let mut md = String::new();
        
        md.push_str("# 🚀 UdonSharp Performance Report\n\n");
        md.push_str(&format!("**Session:** {}\n", report.metadata.session_name));
        md.push_str(&format!("**Generated:** {}\n\n", report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary
        md.push_str("## 📊 Performance Summary\n\n");
        md.push_str(&format!("- **Grade:** {} ({:.1}%)\n", grade_emoji(&report.summary.overall_grade), report.summary.overall_score * 100.0));
        md.push_str(&format!("- **Compilation Time:** {:.2}s\n", report.summary.compilation_time));
        md.push_str(&format!("- **Peak Memory:** {:.1}MB\n", report.summary.peak_memory_mb));
        md.push_str(&format!("- **Success Rate:** {:.1}%\n\n", report.summary.success_rate * 100.0));

        // Key insights
        if !report.summary.key_insights.is_empty() {
            md.push_str("## 💡 Key Insights\n\n");
            for insight in &report.summary.key_insights {
                md.push_str(&format!("- {}\n", insight));
            }
            md.push_str("\n");
        }

        // Recommendations
        if !report.recommendations.is_empty() {
            md.push_str("## 🎯 Optimization Recommendations\n\n");
            for (i, rec) in report.recommendations.iter().enumerate() {
                md.push_str(&format!("### {}. {}\n\n", i + 1, rec.title));
                md.push_str(&format!("**Priority:** {:?} | **Improvement:** {:.0}%\n\n", rec.priority, rec.estimated_improvement));
                md.push_str(&format!("{}\n\n", rec.description));
                
                if !rec.implementation_steps.is_empty() {
                    md.push_str("**Implementation Steps:**\n");
                    for step in &rec.implementation_steps {
                        md.push_str(&format!("1. {}\n", step));
                    }
                    md.push_str("\n");
                }
            }
        }

        // Detailed metrics
        md.push_str("## 📈 Detailed Metrics\n\n");
        md.push_str("### Compilation Metrics\n\n");
        md.push_str(&format!("- **Total Time:** {:.2}s\n", report.metrics.compilation.total_compilation_time.as_secs_f64()));
        md.push_str(&format!("- **Rust Compilation:** {:.2}s\n", report.metrics.compilation.rust_compilation_time.as_secs_f64()));
        md.push_str(&format!("- **WASM Generation:** {:.2}s\n", report.metrics.compilation.wasm_generation_time.as_secs_f64()));
        md.push_str(&format!("- **C# Generation:** {:.2}s\n", report.metrics.compilation.csharp_generation_time.as_secs_f64()));
        md.push_str(&format!("- **Lines of Code:** {}\n", report.metrics.compilation.rust_lines_of_code));
        md.push_str(&format!("- **Compilation Speed:** {:.1} LOC/s\n\n", report.metrics.compilation.compilation_speed_loc_per_second));

        md.push_str("### Memory Metrics\n\n");
        md.push_str(&format!("- **Peak Usage:** {:.1}MB\n", report.metrics.memory.peak_usage as f64 / 1_000_000.0));
        md.push_str(&format!("- **Average Usage:** {:.1}MB\n", report.metrics.memory.average_usage as f64 / 1_000_000.0));
        md.push_str(&format!("- **Memory Growth:** {:.1}MB\n", report.metrics.memory.memory_growth as f64 / 1_000_000.0));

        Ok(md)
    }

    /// Generate CSV report
    fn generate_csv_report(&self, report: &PerformanceReport) -> Result<String> {
        let mut csv = String::new();
        
        // Header
        csv.push_str("Metric,Value,Unit\n");
        
        // Add metrics
        csv.push_str(&format!("Overall Score,{:.3},percentage\n", report.summary.overall_score));
        csv.push_str(&format!("Compilation Time,{:.3},seconds\n", report.summary.compilation_time));
        csv.push_str(&format!("Peak Memory,{:.3},MB\n", report.summary.peak_memory_mb));
        csv.push_str(&format!("Success Rate,{:.3},percentage\n", report.summary.success_rate));
        csv.push_str(&format!("Critical Issues,{},count\n", report.summary.critical_issues));
        csv.push_str(&format!("High Priority Recommendations,{},count\n", report.summary.high_priority_recommendations));
        
        Ok(csv)
    }

    /// Generate console report
    fn generate_console_report(&self, report: &PerformanceReport) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("UdonSharp Performance Report\n");
        output.push_str("===========================\n\n");
        output.push_str(&format!("Session: {}\n", report.metadata.session_name));
        output.push_str(&format!("Generated: {}\n\n", report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        
        output.push_str("Performance Summary:\n");
        output.push_str(&format!("  Grade: {} ({:.1}%)\n", grade_emoji(&report.summary.overall_grade), report.summary.overall_score * 100.0));
        output.push_str(&format!("  Compilation: {:.2}s\n", report.summary.compilation_time));
        output.push_str(&format!("  Memory: {:.1}MB\n", report.summary.peak_memory_mb));
        output.push_str(&format!("  Success Rate: {:.1}%\n\n", report.summary.success_rate * 100.0));
        
        if !report.recommendations.is_empty() {
            output.push_str("Top Recommendations:\n");
            for (i, rec) in report.recommendations.iter().take(5).enumerate() {
                output.push_str(&format!("  {}. {} ({:.0}% improvement)\n", i + 1, rec.title, rec.estimated_improvement));
            }
        }
        
        Ok(output)
    }

    /// Initialize default report templates
    fn initialize_default_templates(&mut self) {
        self.report_templates.insert(ReportFormat::Json, ReportTemplate {
            format: ReportFormat::Json,
            include_metrics: true,
            include_recommendations: true,
            include_profiling_data: true,
            include_charts: false,
            custom_sections: Vec::new(),
        });

        self.report_templates.insert(ReportFormat::Html, ReportTemplate {
            format: ReportFormat::Html,
            include_metrics: true,
            include_recommendations: true,
            include_profiling_data: false,
            include_charts: true,
            custom_sections: Vec::new(),
        });

        self.report_templates.insert(ReportFormat::Markdown, ReportTemplate {
            format: ReportFormat::Markdown,
            include_metrics: true,
            include_recommendations: true,
            include_profiling_data: false,
            include_charts: false,
            custom_sections: Vec::new(),
        });

        self.report_templates.insert(ReportFormat::Console, ReportTemplate {
            format: ReportFormat::Console,
            include_metrics: true,
            include_recommendations: true,
            include_profiling_data: false,
            include_charts: false,
            custom_sections: Vec::new(),
        });
    }

    /// Get report filename for a format
    fn get_report_filename(&self, session_id: &str, format: &ReportFormat) -> String {
        let extension = match format {
            ReportFormat::Json => "json",
            ReportFormat::Html => "html",
            ReportFormat::Markdown => "md",
            ReportFormat::Csv => "csv",
            ReportFormat::Console => "txt",
        };
        
        format!("performance_report_{}_{}.{}", 
            session_id, 
            Utc::now().format("%Y%m%d_%H%M%S"), 
            extension
        )
    }
}

/// Get emoji for performance grade
fn grade_emoji(grade: &PerformanceGrade) -> &'static str {
    match grade {
        PerformanceGrade::A => "🟢 A",
        PerformanceGrade::B => "🔵 B", 
        PerformanceGrade::C => "🟡 C",
        PerformanceGrade::D => "🟠 D",
        PerformanceGrade::F => "🔴 F",
    }
}

/// Get format name for display
fn format_name(format: &ReportFormat) -> &'static str {
    match format {
        ReportFormat::Json => "JSON",
        ReportFormat::Html => "HTML",
        ReportFormat::Markdown => "Markdown",
        ReportFormat::Csv => "CSV",
        ReportFormat::Console => "Console",
    }
}

impl Default for PerformanceReporter {
    fn default() -> Self {
        Self::new()
    }
}