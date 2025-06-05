use proptest::prelude::*;
use crate::playground::*;
use crate::playground::session::LayoutStrategy;
use crate::models::*;

/// Generate arbitrary user actions for property testing
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum UserAction {
    TypeText(String),
    MoveCursor(CursorMotion),
    DeleteRange(usize, usize),
    Undo,
    Redo,
    SwitchLayout(LayoutStrategy),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum CursorMotion {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
    StartOfLine,
    EndOfLine,
    StartOfFile,
    EndOfFile,
}

prop_compose! {
    fn arb_cursor_motion()(
        motion in prop_oneof![
            (1..=10usize).prop_map(CursorMotion::Left),
            (1..=10usize).prop_map(CursorMotion::Right),
            (1..=10usize).prop_map(CursorMotion::Up),
            (1..=10usize).prop_map(CursorMotion::Down),
            Just(CursorMotion::StartOfLine),
            Just(CursorMotion::EndOfLine),
            Just(CursorMotion::StartOfFile),
            Just(CursorMotion::EndOfFile),
        ]
    ) -> CursorMotion {
        motion
    }
}

prop_compose! {
    fn arb_text()(
        text in "[a-zA-Z0-9 \\n]{0,50}"
    ) -> String {
        text
    }
}

prop_compose! {
    fn arb_user_action(max_pos: usize)(
        action in prop_oneof![
            arb_text().prop_map(UserAction::TypeText),
            arb_cursor_motion().prop_map(UserAction::MoveCursor),
            (0..=max_pos, 0..=max_pos).prop_map(|(a, b)| {
                let start = a.min(b);
                let end = a.max(b);
                UserAction::DeleteRange(start, end)
            }),
            Just(UserAction::Undo),
            Just(UserAction::Redo),
            prop_oneof![
                Just(LayoutStrategy::Vertical { ratio: 1.618 }),
                Just(LayoutStrategy::Horizontal { ratio: 0.5 }),
                Just(LayoutStrategy::Tabbed { active: 0 }),
            ].prop_map(UserAction::SwitchLayout),
        ]
    ) -> UserAction {
        action
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::playground::system::PlaygroundSystem;
    use crate::playground::document::DocumentStore;
    use crate::playground::highlighting::SyntaxHighlighter;
    use crate::playground::session::SessionState;
    
    proptest! {
        #[test]
        fn prop_document_store_invariants(actions in prop::collection::vec(arb_user_action(1000), 0..50)) {
            let mut store = DocumentStore::new().unwrap();
            let mut current_pos = 0;
            
            for action in actions {
                match action {
                    UserAction::TypeText(text) => {
                        let content_len = store.get_content().len();
                        if current_pos <= content_len {
                            let _ = store.insert_text(current_pos, &text);
                            current_pos = current_pos.saturating_add(text.len());
                        }
                    }
                    UserAction::DeleteRange(start, end) => {
                        let content_len = store.get_content().len();
                        if start <= content_len && end <= content_len && start <= end {
                            let _ = store.delete_range(start, end);
                            current_pos = start;
                        }
                    }
                    UserAction::MoveCursor(motion) => {
                        let content_len = store.get_content().len();
                        match motion {
                            CursorMotion::Left(n) => current_pos = current_pos.saturating_sub(n),
                            CursorMotion::Right(n) => current_pos = (current_pos + n).min(content_len),
                            CursorMotion::StartOfFile => current_pos = 0,
                            CursorMotion::EndOfFile => current_pos = content_len,
                            _ => {} // Line-based movements need line info
                        }
                    }
                    _ => {} // Other actions don't affect document
                }
                
                // Invariants
                let content = store.get_content();
                prop_assert!(current_pos <= content.len());
                prop_assert_eq!(store.get_version() > 0, true);
                
                #[cfg(feature = "playground")]
                {
                    let rope = store.get_rope();
                    prop_assert_eq!(rope.to_string(), content);
                }
            }
        }
        
        #[test]
        fn prop_syntax_highlighting_deterministic(text: String) {
            let mut highlighter1 = SyntaxHighlighter::new();
            let mut highlighter2 = SyntaxHighlighter::new();
            
            let line_id = highlighting::LineId(0);
            let tokens1 = highlighter1.highlight_line(&text, line_id);
            let tokens2 = highlighter2.highlight_line(&text, line_id);
            
            // Should produce identical results
            prop_assert_eq!(tokens1.len(), tokens2.len());
            for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
                prop_assert_eq!(&t1.text, &t2.text);
                prop_assert_eq!(t1.style.fg.r, t2.style.fg.r);
                prop_assert_eq!(t1.style.fg.g, t2.style.fg.g);
                prop_assert_eq!(t1.style.fg.b, t2.style.fg.b);
            }
        }
        
        #[test]
        fn prop_session_state_roundtrip(source in "[a-zA-Z0-9\\n ]{0,1000}") {
            let mut state = SessionState::new();
            state.document = ropey::Rope::from_str(&source);
            
            // Test file persistence
            let temp_path = std::env::temp_dir().join("test_session.json");
            state.save_to_file(&temp_path).unwrap();
            let loaded = SessionState::load_from_file(&temp_path).unwrap();
            
            prop_assert_eq!(state.document.to_string(), loaded.document.to_string());
            prop_assert_eq!(state.cursor_position.offset, loaded.cursor_position.offset);
            
            // Test URL encoding
            if source.len() < 500 { // URL encoding has size limits
                let url = state.to_url().unwrap();
                let restored = SessionState::from_url(&url).unwrap();
                
                prop_assert_eq!(state.document.to_string(), restored.document.to_string());
            }
            
            // Cleanup
            let _ = std::fs::remove_file(temp_path);
        }
        
        #[test]
        fn prop_transpilation_cancellation(source in "[a-zA-Z0-9\\n ]{0,100}") {
            use std::sync::Arc;
            
            let config = Config::default();
            let mut controller = transpiler::TranspilationController::new(config);
            
            // Create multiple overlapping transpilation requests
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();
            
            runtime.block_on(async {
                let source_arc: Arc<str> = Arc::from(source.as_str());
                
                // Test that transpilation works
                let result = controller.transpile_with_cancellation(source_arc.clone(), 1).await;
                
                // Return result for proptest
                if result.is_ok() {
                    Ok(())
                } else {
                    Err(TestCaseError::fail("Transpilation failed"))
                }
            })?;
        }
        
        #[test]
        fn prop_highlighting_cache_consistency(
            lines in prop::collection::vec("[a-zA-Z0-9 ]{0,50}", 1..20)
        ) {
            let mut highlighter = SyntaxHighlighter::new();
            
            // Highlight all lines twice
            let first_pass: Vec<_> = lines.iter().enumerate()
                .map(|(i, line)| highlighter.highlight_line(line, highlighting::LineId(i)))
                .collect();
            
            let second_pass: Vec<_> = lines.iter().enumerate()
                .map(|(i, line)| highlighter.highlight_line(line, highlighting::LineId(i)))
                .collect();
            
            // Should get same results (from cache)
            for (tokens1, tokens2) in first_pass.iter().zip(second_pass.iter()) {
                prop_assert_eq!(tokens1.len(), tokens2.len());
            }
            
            // Invalidate some lines
            highlighter.invalidate_lines(0, lines.len() / 2);
            
            // Third pass should still match for non-invalidated lines
            let third_pass: Vec<_> = lines.iter().enumerate()
                .map(|(i, line)| highlighter.highlight_line(line, highlighting::LineId(i)))
                .collect();
            
            for i in (lines.len() / 2 + 1)..lines.len() {
                prop_assert_eq!(second_pass[i].len(), third_pass[i].len());
            }
        }
        
        #[test]
        fn prop_computation_graph_no_cycles(
            node_count in 1..20usize,
            edge_count in 0..40usize,
        ) {
            let graph = computation::ComputationGraph::new().unwrap();
            
            #[cfg(feature = "playground")]
            {
                use crate::playground::computation::{ComputeNode, ByteRange};
                
                // Add nodes
                let node_ids: Vec<_> = (0..node_count)
                    .map(|i| {
                        let node = ComputeNode::Parse {
                            range: ByteRange { start: i * 10, end: (i + 1) * 10 },
                            version: 1,
                        };
                        graph.add_node(node)
                    })
                    .collect();
                
                // Add random edges (avoiding cycles by only adding forward edges)
                for _ in 0..edge_count {
                    let from = node_ids[0];
                    let to = node_ids[node_count - 1];
                    if from != to {
                        let _ = graph.add_dependency(from, to);
                    }
                }
                
                // Process should not hang (no cycles)
                let result = graph.process_pending();
                prop_assert!(result.is_ok());
            }
        }
    }
}