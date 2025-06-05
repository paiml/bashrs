use super::*;

#[test]
fn test_playground_system_creation() {
    let result = PlaygroundSystem::new(false);
    assert!(result.is_ok(), "Should be able to create playground system");
}

#[test]
fn test_document_store_operations() {
    let mut store = DocumentStore::new().expect("Should create document store");

    store
        .load_content("fn main() { println!(\"Hello\"); }")
        .expect("Should load content");

    let content = store.get_content();
    assert!(content.contains("Hello"), "Content should be loaded");

    assert_eq!(store.get_version(), 1, "Version should increment");
}

#[test]
fn test_computation_graph() {
    let graph = ComputationGraph::new().expect("Should create computation graph");

    // Test basic operations without playground feature
    graph.mark_full_rebuild();
    let result = graph.process_pending();
    assert!(result.is_ok(), "Should process pending computations");
}

#[test]
fn test_render_pipeline() {
    let _pipeline = RenderPipeline::new().expect("Should create render pipeline");
    // Basic test - detailed testing would require playground feature
    // Render pipeline created successfully
}

#[test]
fn test_keymap_engine() {
    let _vi_engine = KeymapEngine::new(true).expect("Should create VI keymap engine");
    let _emacs_engine = KeymapEngine::new(false).expect("Should create Emacs keymap engine");

    // Basic creation test
    // Keymap engines created successfully
}

#[cfg(test)]
#[path = "property_tests.rs"]
mod property_tests;

#[cfg(feature = "playground")]
mod integration_tests {
    use super::*;

    #[test]
    fn test_playground_integration() {
        let mut playground =
            PlaygroundSystem::new(true).expect("Should create playground with VI mode");

        playground
            .load_content("fn main() { println!(\"Test\"); }")
            .expect("Should load test content");

        // Test that document is loaded
        let content = playground.get_document_content();
        assert!(content.contains("Test"), "Content should be loaded");
    }

    #[test]
    fn test_adaptive_debouncer() {
        let mut debouncer = crate::playground::transpiler::AdaptiveDebouncer::new();

        let delay1 = debouncer.calculate_delay();
        let delay2 = debouncer.calculate_delay();

        // Second delay should be longer due to burst detection
        assert!(delay2 >= delay1, "Burst should increase delay");
    }
}
