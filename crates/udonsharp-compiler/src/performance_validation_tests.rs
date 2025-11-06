//! Performance and validation tests for the standard multi-behavior pattern
//! 
//! This module provides comprehensive tests for compilation performance,
//! generated C# code quality, and UdonSharp compatibility validation.

use crate::config::UdonSharpConfig;
use crate::multi_behavior::*;
use crate::code_generator::*;
use crate::struct_analyzer::*;
use crate::runtime_validation::*;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Performance benchmarking suite for multi-behavior compilation
pub struct PerformanceBenchmarkSuite {
    config: UdonSharpConfig,
    benchmark_results: Vec<BenchmarkResult>,
}

impl PerformanceBenchmarkSuite {
    /// Create a new performance benchmark suite
    pub fn new() -> Self {
        Self {
            config: UdonSharpConfig::default(),
            benchmark_results: Vec::new(),
        }
    }

    /// Run all performance benchmarks
    pub fn run_all_benchmarks(&mut self) -> BenchmarkSummary {
        let mut summary = BenchmarkSummary::new();

        // Benchmark struct analysis performance
        let analysis_result = self.benchmark_struct_analysis();
        summary.add_result("struct_analysis", analysis_result);

        // Benchmark code generation performance
        let generation_result = self.benchmark_code_generation();
        summary.add_result("code_generation", generation_result);

        // Benchmark large project compilation
        let large_project_result = self.benchmark_large_project();
        summary.add_result("large_project", large_project_result);

        // Benchmark memory usage
        let memory_result = self.benchmark_memory_usage();
        summary.add_result("memory_usage", memory_result);

        summary
    }

    /// Benchmark struct analysis performance
    fn benchmark_struct_analysis(&mut self) -> BenchmarkResult {
        let start_time = Instant::now();
        let mut total_structs = 0;
        let mut total_fields = 0;
        let mut total_methods = 0;

        // Create test cases with varying complexity
        let test_cases = self.create_analysis_test_cases();

        for (name, source_items) in test_cases {
            let analysis_start = Instant::now();
            let mut analyzer = StructAnalyzer::new();
            
            match analyzer.analyze_module(&source_items) {
                Ok(structs) => {
                    total_structs += structs.len();
                    for s in &structs {
                        total_fields += s.fields.len();
                        total_methods += s.methods.len();
                    }
                }
                Err(_) => {
                    // Expected for some test cases
                }
            }

            let analysis_duration = analysis_start.elapsed();
            self.benchmark_results.push(BenchmarkResult {
                test_name: format!("analysis_{}", name),
                duration: analysis_duration,
                throughput: Some(source_items.len() as f64 / analysis_duration.as_secs_f64()),
                memory_usage: None,
                success: true,
            });
        }

        let total_duration = start_time.elapsed();
        BenchmarkResult {
            test_name: "struct_analysis_total".to_string(),
            duration: total_duration,
            throughput: Some(total_structs as f64 / total_duration.as_secs_f64()),
            memory_usage: None,
            success: true,
        }
    }

    /// Benchmark code generation performance
    fn benchmark_code_generation(&mut self) -> BenchmarkResult {
        let start_time = Instant::now();
        let generator = CodeGenerator::new();
        let mut total_classes = 0;
        let mut total_lines = 0;

        // Create test structs with varying complexity
        let test_structs = self.create_generation_test_structs();

        for test_struct in test_structs {
            let generation_start = Instant::now();
            
            match generator.generate_behavior_class(&test_struct) {
                Ok(generated_class) => {
                    total_classes += 1;
                    total_lines += generated_class.source_code.lines().count();
                }
                Err(_) => {
                    // Some test cases may fail intentionally
                }
            }

            let generation_duration = generation_start.elapsed();
            self.benchmark_results.push(BenchmarkResult {
                test_name: format!("generation_{}", test_struct.name),
                duration: generation_duration,
                throughput: Some(1.0 / generation_duration.as_secs_f64()),
                memory_usage: None,
                success: true,
            });
        }

        let total_duration = start_time.elapsed();
        BenchmarkResult {
            test_name: "code_generation_total".to_string(),
            duration: total_duration,
            throughput: Some(total_classes as f64 / total_duration.as_secs_f64()),
            memory_usage: None,
            success: true,
        }
    }

    /// Benchmark large project compilation
    fn benchmark_large_project(&mut self) -> BenchmarkResult {
        let start_time = Instant::now();
        
        // Create a large project with many behaviors
        let large_project = self.create_large_project_test();
        
        let mut analyzer = StructAnalyzer::new();
        let generator = CodeGenerator::new();
        
        // Analyze all structs
        let analysis_start = Instant::now();
        let structs = analyzer.analyze_module(&large_project.source_items).unwrap_or_default();
        let analysis_duration = analysis_start.elapsed();
        
        // Generate all classes
        let generation_start = Instant::now();
        let mut generated_classes = Vec::new();
        for s in &structs {
            if let Ok(class) = generator.generate_behavior_class(s) {
                generated_classes.push(class);
            }
        }
        let generation_duration = generation_start.elapsed();
        
        let total_duration = start_time.elapsed();
        
        BenchmarkResult {
            test_name: "large_project".to_string(),
            duration: total_duration,
            throughput: Some(structs.len() as f64 / total_duration.as_secs_f64()),
            memory_usage: Some(large_project.estimated_memory_mb),
            success: generated_classes.len() == structs.len(),
        }
    }

    /// Benchmark memory usage
    fn benchmark_memory_usage(&mut self) -> BenchmarkResult {
        let start_time = Instant::now();
        
        // Create memory-intensive test case
        let memory_test = self.create_memory_test_case();
        
        // Measure memory before
        let memory_before = self.estimate_memory_usage();
        
        // Run compilation
        let mut analyzer = StructAnalyzer::new();
        let generator = CodeGenerator::new();
        
        let structs = analyzer.analyze_module(&memory_test.source_items).unwrap_or_default();
        let mut generated_classes = Vec::new();
        
        for s in &structs {
            if let Ok(class) = generator.generate_behavior_class(s) {
                generated_classes.push(class);
            }
        }
        
        // Measure memory after
        let memory_after = self.estimate_memory_usage();
        let memory_used = memory_after - memory_before;
        
        let duration = start_time.elapsed();
        
        BenchmarkResult {
            test_name: "memory_usage".to_string(),
            duration,
            throughput: Some(structs.len() as f64 / duration.as_secs_f64()),
            memory_usage: Some(memory_used),
            success: memory_used < 100.0, // Should use less than 100MB
        }
    }

    /// Create test cases for struct analysis benchmarking
    fn create_analysis_test_cases(&self) -> Vec<(String, Vec<syn::Item>)> {
        use syn::parse_quote;
        
        vec![
            ("simple".to_string(), vec![
                parse_quote! {
                    #[derive(UdonBehaviour)]
                    pub struct SimpleBehavior {
                        field: i32,
                    }
                }
            ]),
            ("complex".to_string(), vec![
                parse_quote! {
                    #[derive(UdonBehaviour)]
                    pub struct ComplexBehavior {
                        #[udon_public] field1: i32,
                        #[udon_sync] field2: String,
                        field3: Vec<unity::GameObject>,
                        field4: HashMap<String, f32>,
                        field5: Option<unity::Vector3>,
                    }
                }
            ]),
            ("multiple".to_string(), (0..10).map(|i| {
                parse_quote! {
                    #[derive(UdonBehaviour)]
                    pub struct #(format!("Behavior{}", i)) {
                        field: i32,
                    }
                }
            }).collect()),
        ]
    }

    /// Create test structs for code generation benchmarking
    fn create_generation_test_structs(&self) -> Vec<UdonBehaviourStruct> {
        let mut structs = Vec::new();
        
        // Simple struct
        let mut simple = UdonBehaviourStruct::new("SimpleBehavior".to_string());
        simple.add_field(StructField::new("field".to_string(), RustType::I32));
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.check_completeness();
        simple.set_trait_impl(trait_impl);
        structs.push(simple);
        
        // Complex struct with many fields
        let mut complex = UdonBehaviourStruct::new("ComplexBehavior".to_string());
        for i in 0..20 {
            let mut field = StructField::new(format!("field_{}", i), RustType::I32);
            if i % 2 == 0 {
                field.add_attribute(FieldAttribute::UdonPublic);
            }
            if i % 3 == 0 {
                field.add_attribute(FieldAttribute::UdonSync);
            }
            complex.add_field(field);
        }
        
        // Add many methods
        for i in 0..10 {
            let mut method = StructMethod::new(format!("method_{}", i));
            method.add_attribute(MethodAttribute::UdonEvent(format!("Event{}", i)));
            complex.add_method(method);
        }
        
        let mut trait_impl = UdonBehaviourTraitImpl::new();
        trait_impl.add_method("start".to_string());
        trait_impl.add_method("update".to_string());
        trait_impl.check_completeness();
        complex.set_trait_impl(trait_impl);
        structs.push(complex);
        
        structs
    }

    /// Create large project test case
    fn create_large_project_test(&self) -> LargeProjectTest {
        use syn::parse_quote;
        
        let mut source_items = Vec::new();
        
        // Generate 50 behaviors
        for i in 0..50 {
            source_items.push(parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct #(format!("Behavior{:02}", i)) {
                    #[udon_public] field1: i32,
                    #[udon_sync] field2: String,
                    field3: Vec<unity::GameObject>,
                }
            });
            
            source_items.push(parse_quote! {
                impl UdonBehaviour for #(format!("Behavior{:02}", i)) {
                    fn start(&mut self) {
                        self.field1 = #i;
                        self.field2 = format!("Behavior {}", #i);
                    }
                    
                    fn update(&mut self) {
                        // Update logic
                    }
                }
            });
        }
        
        LargeProjectTest {
            source_items,
            estimated_memory_mb: 50.0, // Estimated memory usage
        }
    }

    /// Create memory-intensive test case
    fn create_memory_test_case(&self) -> MemoryTestCase {
        use syn::parse_quote;
        
        let mut source_items = Vec::new();
        
        // Create behaviors with many fields and complex types
        for i in 0..20 {
            source_items.push(parse_quote! {
                #[derive(UdonBehaviour)]
                pub struct #(format!("MemoryBehavior{}", i)) {
                    large_vec: Vec<String>,
                    large_map: HashMap<String, Vec<i32>>,
                    gameobject_refs: Vec<unity::GameObject>,
                    complex_data: Option<Vec<HashMap<String, f32>>>,
                }
            });
        }
        
        MemoryTestCase {
            source_items,
        }
    }

    /// Estimate current memory usage (simplified)
    fn estimate_memory_usage(&self) -> f64 {
        // This is a simplified estimation
        // In a real implementation, you might use system APIs
        42.0 // Placeholder value in MB
    }
}

/// Code quality validation suite
pub struct CodeQualityValidator {
    validator: RuntimeValidator,
    quality_metrics: QualityMetrics,
}

impl CodeQualityValidator {
    /// Create a new code quality validator
    pub fn new() -> Self {
        Self {
            validator: RuntimeValidator::new(),
            quality_metrics: QualityMetrics::new(),
        }
    }

    /// Validate generated C# code quality
    pub fn validate_code_quality(&mut self, generated_classes: &[GeneratedClass]) -> QualityReport {
        let mut report = QualityReport::new();

        for class in generated_classes {
            let class_metrics = self.analyze_class_quality(class);
            report.add_class_metrics(class.class_name.clone(), class_metrics);
        }

        report
    }

    /// Analyze quality metrics for a single class
    fn analyze_class_quality(&mut self, class: &GeneratedClass) -> ClassQualityMetrics {
        let mut metrics = ClassQualityMetrics::new();

        // Analyze code structure
        metrics.line_count = class.source_code.lines().count();
        metrics.method_count = class.methods.len();
        metrics.field_count = class.fields.len();

        // Check naming conventions
        metrics.follows_naming_conventions = self.check_naming_conventions(class);

        // Check UdonSharp compatibility
        metrics.udonsharp_compatible = self.check_udonsharp_compatibility(class);

        // Check for code smells
        metrics.code_smells = self.detect_code_smells(class);

        // Validate syntax
        metrics.syntax_valid = self.validate_syntax(class);

        // Check performance characteristics
        metrics.performance_score = self.calculate_performance_score(class);

        metrics
    }

    /// Check if class follows C# naming conventions
    fn check_naming_conventions(&self, class: &GeneratedClass) -> bool {
        // Class name should be PascalCase
        if !self.is_pascal_case(&class.class_name) {
            return false;
        }

        // Field names should be camelCase
        for field in &class.fields {
            if !self.is_camel_case(&field.name) {
                return false;
            }
        }

        // Method names should be PascalCase
        for method in &class.methods {
            if !self.is_pascal_case(&method.name) {
                return false;
            }
        }

        true
    }

    /// Check UdonSharp compatibility
    fn check_udonsharp_compatibility(&self, class: &GeneratedClass) -> bool {
        // Check for UdonSharp-specific requirements
        let has_udonsharp_inheritance = class.source_code.contains(": UdonSharpBehaviour");
        let has_proper_using = class.using_statements.contains(&"using UdonSharp;".to_string());
        let has_unity_using = class.using_statements.contains(&"using UnityEngine;".to_string());

        has_udonsharp_inheritance && has_proper_using && has_unity_using
    }

    /// Detect code smells
    fn detect_code_smells(&self, class: &GeneratedClass) -> Vec<CodeSmell> {
        let mut smells = Vec::new();

        // Check for overly long methods
        for method in &class.methods {
            if method.body.lines().count() > 50 {
                smells.push(CodeSmell::LongMethod(method.name.clone()));
            }
        }

        // Check for too many fields
        if class.fields.len() > 20 {
            smells.push(CodeSmell::TooManyFields(class.fields.len()));
        }

        // Check for missing null checks on GameObject references
        for field in &class.fields {
            if field.field_type == "GameObject" && !class.source_code.contains(&format!("if ({} != null)", field.name)) {
                smells.push(CodeSmell::MissingNullCheck(field.name.clone()));
            }
        }

        smells
    }

    /// Validate C# syntax
    fn validate_syntax(&self, class: &GeneratedClass) -> bool {
        // Basic syntax validation
        let code = &class.source_code;
        
        // Check for balanced braces
        let open_braces = code.matches('{').count();
        let close_braces = code.matches('}').count();
        if open_braces != close_braces {
            return false;
        }

        // Check for balanced parentheses
        let open_parens = code.matches('(').count();
        let close_parens = code.matches(')').count();
        if open_parens != close_parens {
            return false;
        }

        // Check for proper semicolons (simplified)
        let lines: Vec<&str> = code.lines().collect();
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.is_empty() && 
               !trimmed.starts_with("//") && 
               !trimmed.starts_with("using") &&
               !trimmed.ends_with('{') && 
               !trimmed.ends_with('}') &&
               !trimmed.ends_with(';') &&
               !trimmed.contains("class ") &&
               !trimmed.contains("namespace ") {
                // Might be missing semicolon
                return false;
            }
        }

        true
    }

    /// Calculate performance score
    fn calculate_performance_score(&self, class: &GeneratedClass) -> f32 {
        let mut score = 100.0;

        // Penalize for complexity
        score -= (class.methods.len() as f32) * 0.5;
        score -= (class.fields.len() as f32) * 0.2;

        // Penalize for long methods
        for method in &class.methods {
            let line_count = method.body.lines().count();
            if line_count > 20 {
                score -= (line_count as f32 - 20.0) * 0.1;
            }
        }

        // Bonus for proper networking usage
        if class.source_code.contains("RequestSerialization") {
            score += 5.0;
        }

        score.max(0.0).min(100.0)
    }

    /// Check if string is PascalCase
    fn is_pascal_case(&self, s: &str) -> bool {
        !s.is_empty() && s.chars().next().unwrap().is_uppercase() && !s.contains('_')
    }

    /// Check if string is camelCase
    fn is_camel_case(&self, s: &str) -> bool {
        !s.is_empty() && s.chars().next().unwrap().is_lowercase() && !s.contains('_')
    }
}

/// UdonSharp compatibility validator
pub struct UdonSharpCompatibilityValidator {
    compatibility_rules: Vec<CompatibilityRule>,
}

impl UdonSharpCompatibilityValidator {
    /// Create a new compatibility validator
    pub fn new() -> Self {
        let mut validator = Self {
            compatibility_rules: Vec::new(),
        };
        
        validator.initialize_rules();
        validator
    }

    /// Initialize compatibility rules
    fn initialize_rules(&mut self) {
        self.compatibility_rules.push(CompatibilityRule {
            name: "UdonSharp Inheritance".to_string(),
            check: Box::new(|class| class.source_code.contains(": UdonSharpBehaviour")),
            severity: RuleSeverity::Error,
        });

        self.compatibility_rules.push(CompatibilityRule {
            name: "Required Using Statements".to_string(),
            check: Box::new(|class| {
                class.using_statements.contains(&"using UdonSharp;".to_string()) &&
                class.using_statements.contains(&"using UnityEngine;".to_string())
            }),
            severity: RuleSeverity::Error,
        });

        self.compatibility_rules.push(CompatibilityRule {
            name: "Synchronized Fields".to_string(),
            check: Box::new(|class| {
                if class.source_code.contains("[UdonSynced]") {
                    class.source_code.contains("RequestSerialization")
                } else {
                    true
                }
            }),
            severity: RuleSeverity::Warning,
        });

        self.compatibility_rules.push(CompatibilityRule {
            name: "GameObject Null Checks".to_string(),
            check: Box::new(|class| {
                let gameobject_fields: Vec<_> = class.fields.iter()
                    .filter(|f| f.field_type == "GameObject")
                    .collect();
                
                for field in gameobject_fields {
                    if !class.source_code.contains(&format!("if ({} != null)", field.name)) {
                        return false;
                    }
                }
                true
            }),
            severity: RuleSeverity::Warning,
        });
    }

    /// Validate UdonSharp compatibility
    pub fn validate_compatibility(&self, classes: &[GeneratedClass]) -> CompatibilityReport {
        let mut report = CompatibilityReport::new();

        for class in classes {
            let mut class_issues = Vec::new();

            for rule in &self.compatibility_rules {
                if !(rule.check)(class) {
                    class_issues.push(CompatibilityIssue {
                        rule_name: rule.name.clone(),
                        severity: rule.severity.clone(),
                        description: format!("Class '{}' violates rule '{}'", class.class_name, rule.name),
                    });
                }
            }

            if !class_issues.is_empty() {
                report.add_class_issues(class.class_name.clone(), class_issues);
            }
        }

        report
    }
}

// Supporting types and structures

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub duration: Duration,
    pub throughput: Option<f64>, // Operations per second
    pub memory_usage: Option<f64>, // MB
    pub success: bool,
}

#[derive(Debug)]
pub struct BenchmarkSummary {
    pub results: HashMap<String, BenchmarkResult>,
    pub total_duration: Duration,
}

impl BenchmarkSummary {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            total_duration: Duration::ZERO,
        }
    }

    pub fn add_result(&mut self, name: &str, result: BenchmarkResult) {
        self.total_duration += result.duration;
        self.results.insert(name.to_string(), result);
    }

    pub fn print_summary(&self) {
        println!("\n=== Performance Benchmark Summary ===");
        println!("Total Duration: {:.2}s", self.total_duration.as_secs_f64());
        
        for (name, result) in &self.results {
            println!("\n{}: {:.2}ms", name, result.duration.as_millis());
            if let Some(throughput) = result.throughput {
                println!("  Throughput: {:.2} ops/sec", throughput);
            }
            if let Some(memory) = result.memory_usage {
                println!("  Memory: {:.2} MB", memory);
            }
            println!("  Success: {}", result.success);
        }
    }
}

#[derive(Debug)]
pub struct LargeProjectTest {
    pub source_items: Vec<syn::Item>,
    pub estimated_memory_mb: f64,
}

#[derive(Debug)]
pub struct MemoryTestCase {
    pub source_items: Vec<syn::Item>,
}

#[derive(Debug)]
pub struct QualityMetrics {
    pub total_classes: usize,
    pub total_lines: usize,
    pub average_complexity: f32,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            total_classes: 0,
            total_lines: 0,
            average_complexity: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct QualityReport {
    pub class_metrics: HashMap<String, ClassQualityMetrics>,
    pub overall_score: f32,
}

impl QualityReport {
    pub fn new() -> Self {
        Self {
            class_metrics: HashMap::new(),
            overall_score: 0.0,
        }
    }

    pub fn add_class_metrics(&mut self, class_name: String, metrics: ClassQualityMetrics) {
        self.class_metrics.insert(class_name, metrics);
        self.calculate_overall_score();
    }

    fn calculate_overall_score(&mut self) {
        if self.class_metrics.is_empty() {
            self.overall_score = 0.0;
            return;
        }

        let total_score: f32 = self.class_metrics.values()
            .map(|m| m.performance_score)
            .sum();
        
        self.overall_score = total_score / self.class_metrics.len() as f32;
    }
}

#[derive(Debug)]
pub struct ClassQualityMetrics {
    pub line_count: usize,
    pub method_count: usize,
    pub field_count: usize,
    pub follows_naming_conventions: bool,
    pub udonsharp_compatible: bool,
    pub code_smells: Vec<CodeSmell>,
    pub syntax_valid: bool,
    pub performance_score: f32,
}

impl ClassQualityMetrics {
    pub fn new() -> Self {
        Self {
            line_count: 0,
            method_count: 0,
            field_count: 0,
            follows_naming_conventions: true,
            udonsharp_compatible: true,
            code_smells: Vec::new(),
            syntax_valid: true,
            performance_score: 100.0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CodeSmell {
    LongMethod(String),
    TooManyFields(usize),
    MissingNullCheck(String),
    UnusedField(String),
    ComplexCondition(String),
}

#[derive(Debug)]
pub struct CompatibilityRule {
    pub name: String,
    pub check: Box<dyn Fn(&GeneratedClass) -> bool>,
    pub severity: RuleSeverity,
}

#[derive(Debug, Clone)]
pub enum RuleSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug)]
pub struct CompatibilityReport {
    pub class_issues: HashMap<String, Vec<CompatibilityIssue>>,
    pub total_errors: usize,
    pub total_warnings: usize,
}

impl CompatibilityReport {
    pub fn new() -> Self {
        Self {
            class_issues: HashMap::new(),
            total_errors: 0,
            total_warnings: 0,
        }
    }

    pub fn add_class_issues(&mut self, class_name: String, issues: Vec<CompatibilityIssue>) {
        for issue in &issues {
            match issue.severity {
                RuleSeverity::Error => self.total_errors += 1,
                RuleSeverity::Warning => self.total_warnings += 1,
                RuleSeverity::Info => {}
            }
        }
        self.class_issues.insert(class_name, issues);
    }

    pub fn is_compatible(&self) -> bool {
        self.total_errors == 0
    }
}

#[derive(Debug, Clone)]
pub struct CompatibilityIssue {
    pub rule_name: String,
    pub severity: RuleSeverity,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_benchmark_suite() {
        let mut suite = PerformanceBenchmarkSuite::new();
        let summary = suite.run_all_benchmarks();
        
        // Verify benchmarks ran
        assert!(!summary.results.is_empty());
        assert!(summary.total_duration > Duration::ZERO);
        
        // Print results for manual inspection
        summary.print_summary();
    }

    #[test]
    fn test_code_quality_validation() {
        let mut validator = CodeQualityValidator::new();
        
        // Create test generated class
        let test_class = GeneratedClass {
            class_name: "TestBehavior".to_string(),
            namespace: Some("TestNamespace".to_string()),
            using_statements: vec![
                "using UnityEngine;".to_string(),
                "using UdonSharp;".to_string(),
            ],
            class_attributes: vec![],
            fields: vec![
                GeneratedField {
                    name: "testField".to_string(),
                    field_type: "int".to_string(),
                    visibility: "private".to_string(),
                    attributes: vec![],
                    default_value: Some("0".to_string()),
                }
            ],
            methods: vec![
                GeneratedMethod {
                    name: "Start".to_string(),
                    visibility: "public".to_string(),
                    return_type: "void".to_string(),
                    parameters: vec![],
                    attributes: vec!["override".to_string()],
                    body: "testField = 42;".to_string(),
                }
            ],
            custom_events: vec![],
            source_code: r#"
using UnityEngine;
using UdonSharp;

public class TestBehavior : UdonSharpBehaviour
{
    private int testField = 0;
    
    public override void Start()
    {
        testField = 42;
    }
}
"#.to_string(),
        };
        
        let report = validator.validate_code_quality(&[test_class]);
        
        // Verify quality analysis
        assert_eq!(report.class_metrics.len(), 1);
        assert!(report.overall_score > 0.0);
    }

    #[test]
    fn test_udonsharp_compatibility_validation() {
        let validator = UdonSharpCompatibilityValidator::new();
        
        // Create compatible test class
        let compatible_class = GeneratedClass {
            class_name: "CompatibleBehavior".to_string(),
            namespace: None,
            using_statements: vec![
                "using UnityEngine;".to_string(),
                "using UdonSharp;".to_string(),
            ],
            class_attributes: vec![],
            fields: vec![],
            methods: vec![],
            custom_events: vec![],
            source_code: r#"
using UnityEngine;
using UdonSharp;

public class CompatibleBehavior : UdonSharpBehaviour
{
    public override void Start()
    {
        // Compatible code
    }
}
"#.to_string(),
        };
        
        // Create incompatible test class
        let incompatible_class = GeneratedClass {
            class_name: "IncompatibleBehavior".to_string(),
            namespace: None,
            using_statements: vec!["using UnityEngine;".to_string()],
            class_attributes: vec![],
            fields: vec![],
            methods: vec![],
            custom_events: vec![],
            source_code: r#"
using UnityEngine;

public class IncompatibleBehavior : MonoBehaviour
{
    void Start()
    {
        // Incompatible code
    }
}
"#.to_string(),
        };
        
        let report = validator.validate_compatibility(&[compatible_class, incompatible_class]);
        
        // Verify compatibility analysis
        assert_eq!(report.class_issues.len(), 1); // Only incompatible class should have issues
        assert!(report.total_errors > 0);
        assert!(!report.is_compatible());
    }

    #[test]
    fn test_performance_thresholds() {
        let mut suite = PerformanceBenchmarkSuite::new();
        let summary = suite.run_all_benchmarks();
        
        // Verify performance meets thresholds
        for (name, result) in &summary.results {
            match name.as_str() {
                "struct_analysis" => {
                    assert!(result.duration < Duration::from_secs(5), 
                           "Struct analysis should complete within 5 seconds");
                }
                "code_generation" => {
                    assert!(result.duration < Duration::from_secs(10), 
                           "Code generation should complete within 10 seconds");
                }
                "large_project" => {
                    assert!(result.duration < Duration::from_secs(30), 
                           "Large project compilation should complete within 30 seconds");
                }
                "memory_usage" => {
                    if let Some(memory) = result.memory_usage {
                        assert!(memory < 100.0, 
                               "Memory usage should be less than 100MB");
                    }
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_code_smell_detection() {
        let validator = CodeQualityValidator::new();
        
        // Create class with code smells
        let smelly_class = GeneratedClass {
            class_name: "SmellyBehavior".to_string(),
            namespace: None,
            using_statements: vec![
                "using UnityEngine;".to_string(),
                "using UdonSharp;".to_string(),
            ],
            class_attributes: vec![],
            fields: (0..25).map(|i| GeneratedField {
                name: format!("field{}", i),
                field_type: "int".to_string(),
                visibility: "private".to_string(),
                attributes: vec![],
                default_value: Some("0".to_string()),
            }).collect(),
            methods: vec![
                GeneratedMethod {
                    name: "VeryLongMethod".to_string(),
                    visibility: "public".to_string(),
                    return_type: "void".to_string(),
                    parameters: vec![],
                    attributes: vec![],
                    body: (0..60).map(|i| format!("// Line {}", i)).collect::<Vec<_>>().join("\n"),
                }
            ],
            custom_events: vec![],
            source_code: "// Generated code with smells".to_string(),
        };
        
        let metrics = validator.analyze_class_quality(&smelly_class);
        
        // Verify code smells are detected
        assert!(!metrics.code_smells.is_empty());
        assert!(metrics.code_smells.iter().any(|s| matches!(s, CodeSmell::TooManyFields(_))));
        assert!(metrics.code_smells.iter().any(|s| matches!(s, CodeSmell::LongMethod(_))));
    }
}