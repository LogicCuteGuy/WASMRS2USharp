//! Tests for WASM analysis and behavior identification

use crate::analyzer::OopBehaviorAnalyzer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_behavior_analyzer_creation() {
        let analyzer = OopBehaviorAnalyzer::new();
        
        // Test that analyzer is created with empty state
        assert!(analyzer.get_exported_functions().is_empty());
        assert!(analyzer.get_udon_behaviour_functions().is_empty());
    }

    #[test]
    fn test_call_graph_creation() {
        let analyzer = OopBehaviorAnalyzer::new();
        
        // Test that call graph can be created (even if empty)
        let call_graph = analyzer.build_call_graph();
        assert!(call_graph.is_ok());
        
        let graph = call_graph.unwrap();
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
        assert!(graph.reverse_edges.is_empty());
    }

    #[test]
    fn test_behavior_unit_identification() {
        let analyzer = OopBehaviorAnalyzer::new();
        
        // Test that behavior units can be identified (even if empty)
        let behavior_units = analyzer.identify_behavior_units();
        assert!(behavior_units.is_ok());
        
        let units = behavior_units.unwrap();
        assert!(units.is_empty()); // No UdonBehaviour functions in empty analyzer
    }

    #[test]
    fn test_cross_behavior_dependency_analysis() {
        let analyzer = OopBehaviorAnalyzer::new();
        let call_graph = analyzer.build_call_graph().unwrap();
        
        // Test that cross-behavior dependencies can be analyzed
        let dependencies = analyzer.analyze_cross_behavior_dependencies(&call_graph);
        assert!(dependencies.is_ok());
        
        let deps = dependencies.unwrap();
        assert!(deps.is_empty()); // No dependencies in empty graph
    }

    #[test]
    fn test_circular_dependency_detection() {
        let analyzer = OopBehaviorAnalyzer::new();
        let call_graph = analyzer.build_call_graph().unwrap();
        
        // Test that circular dependencies can be detected
        let circular_deps = analyzer.detect_circular_dependencies(&call_graph);
        assert!(circular_deps.is_empty()); // No circular dependencies in empty graph
    }

    #[test]
    fn test_analysis_result_building() {
        let mut analyzer = OopBehaviorAnalyzer::new();
        
        // Test that analysis can be performed on empty WASM
        let empty_wasm = vec![
            0x00, 0x61, 0x73, 0x6d, // WASM magic number
            0x01, 0x00, 0x00, 0x00, // WASM version
        ];
        
        let result = analyzer.analyze(&empty_wasm);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.behavior_units.is_empty());
        assert!(analysis.udon_behaviour_functions.is_empty());
        assert!(analysis.exported_functions.is_empty());
        assert!(analysis.cross_behavior_dependencies.is_empty());
        assert!(analysis.circular_dependencies.is_empty());
    }

    #[test]
    fn test_udon_behaviour_attribute_checking() {
        let analyzer = OopBehaviorAnalyzer::new();
        
        // Test checking for non-existent attributes
        assert!(!analyzer.has_udon_behaviour_attribute("non_existent_function"));
        assert!(analyzer.get_udon_behaviour_attribute("non_existent_function").is_none());
    }
}