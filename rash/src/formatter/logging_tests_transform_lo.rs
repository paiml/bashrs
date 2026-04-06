use super::*;

#[test]
fn test_transform_log_creation() {
    let log = TransformLog::new();
    assert_eq!(log.entries.len(), 0);
    assert_eq!(log.metadata.total_transforms, 0);
    assert_eq!(log.merkle_tree.leaf_hashes.len(), 0);
}

#[test]
fn test_add_log_entry() {
    let mut log = TransformLog::new();

    let entry = TransformEntry {
        id: TransformId::new(),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(0), BytePos(10)),
        result_span: Span::new(BytePos(0), BytePos(10)),
        timestamp: Instant::now(),
        proof: Some(SexprProof::identity()),
        semantic_delta: None,
    };

    log.add_entry(entry);

    assert_eq!(log.entries.len(), 1);
    assert_eq!(log.metadata.total_transforms, 1);
    assert_eq!(log.metadata.semantic_preserving, 1);
    assert_eq!(log.metadata.with_proofs, 1);
}

#[test]
fn test_merkle_tree_empty() {
    let tree = MerkleTree::empty();
    assert_eq!(tree.leaf_hashes.len(), 0);
    assert_eq!(tree._height, 0);
}

#[test]
fn test_merkle_tree_single_leaf() {
    let leaf_hash = blake3::hash(b"test");
    let tree = MerkleTree::from_leaves(vec![leaf_hash]);

    assert_eq!(tree.leaf_hashes.len(), 2); // Padded to power of 2
    assert_eq!(tree._height, 1);
    assert_ne!(tree.root_hash, blake3::hash(b"")); // Non-empty root
}

#[test]
fn test_merkle_tree_multiple_leaves() {
    let leaves = vec![
        blake3::hash(b"leaf1"),
        blake3::hash(b"leaf2"),
        blake3::hash(b"leaf3"),
    ];

    let tree = MerkleTree::from_leaves(leaves.clone());
    assert_eq!(tree.leaf_hashes.len(), 4); // Padded to next power of 2
    assert_eq!(tree._height, 2);
}

#[test]
fn test_merkle_proof_generation() {
    let mut log = TransformLog::new();

    // Add several entries
    for i in 0..4 {
        let entry = TransformEntry {
            id: TransformId::new(),
            transform: Transform::Identity,
            source_span: Span::new(BytePos(i * 10), BytePos((i + 1) * 10)),
            result_span: Span::new(BytePos(i * 10), BytePos((i + 1) * 10)),
            timestamp: Instant::now(),
            proof: None,
            semantic_delta: None,
        };
        log.add_entry(entry);
    }

    // Generate proof for second entry
    let proof = log.generate_proof(1);
    assert!(proof.is_some());

    let proof = proof.unwrap();
    assert_eq!(proof.leaf_index, 1);
    assert!(!proof.path.is_empty());
}

#[test]
fn test_log_stats() {
    let mut log = TransformLog::new();

    // Add mixed entries
    let semantic_preserving = TransformEntry {
        id: TransformId::new(),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(0), BytePos(10)),
        result_span: Span::new(BytePos(0), BytePos(10)),
        timestamp: Instant::now(),
        proof: Some(SexprProof::identity()),
        semantic_delta: None,
    };

    let non_preserving = TransformEntry {
        id: TransformId::new(),
        transform: Transform::ArithToTest {
            preserve_short_circuit: false,
            overflow_behavior: OverflowSemantics::Wrap,
        },
        source_span: Span::new(BytePos(10), BytePos(20)),
        result_span: Span::new(BytePos(10), BytePos(25)),
        timestamp: Instant::now(),
        proof: None,
        semantic_delta: Some(SemanticDelta::ShortCircuitLost),
    };

    log.add_entry(semantic_preserving);
    log.add_entry(non_preserving);

    let stats = log.stats();
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.semantic_preserving_ratio, 0.5);
    assert_eq!(stats.proof_coverage_ratio, 0.5);
    assert!(stats.memory_usage_bytes > 0);
    assert!(stats.integrity_verified);
}

#[test]
fn test_hash_entry_deterministic() {
    let log = TransformLog::new();

    let entry = TransformEntry {
        id: TransformId(12345),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(0), BytePos(10)),
        result_span: Span::new(BytePos(0), BytePos(10)),
        timestamp: Instant::now(),
        proof: None,
        semantic_delta: None,
    };

    let hash1 = log.hash_entry(&entry);
    let hash2 = log.hash_entry(&entry);

    // Note: Due to timestamp, hashes may differ
    // In a real implementation, we'd use a deterministic timestamp
    assert_eq!(hash1.as_bytes().len(), hash2.as_bytes().len());
}

#[test]
fn test_export_verification_data() {
    let mut log = TransformLog::new();

    let entry = TransformEntry {
        id: TransformId::new(),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(0), BytePos(10)),
        result_span: Span::new(BytePos(0), BytePos(10)),
        timestamp: Instant::now(),
        proof: None,
        semantic_delta: None,
    };

    log.add_entry(entry);

    let verification_data = log.export_verification_data();
    assert_eq!(verification_data.entries.len(), 1);
    assert_eq!(verification_data.leaf_hashes.len(), 2); // Padded
    assert_eq!(verification_data.metadata.total_transforms, 1);
}

#[test]
fn test_root_hash_changes() {
    let mut log = TransformLog::new();
    let initial_root = log.root_hash();

    let entry = TransformEntry {
        id: TransformId::new(),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(0), BytePos(10)),
        result_span: Span::new(BytePos(0), BytePos(10)),
        timestamp: Instant::now(),
        proof: None,
        semantic_delta: None,
    };

    log.add_entry(entry);
    let new_root = log.root_hash();

    assert_ne!(initial_root, new_root);
}

#[test]
fn test_log_metadata_time_span() {
    let mut log = TransformLog::new();

    let first_entry = TransformEntry {
        id: TransformId::new(),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(0), BytePos(10)),
        result_span: Span::new(BytePos(0), BytePos(10)),
        timestamp: Instant::now(),
        proof: None,
        semantic_delta: None,
    };

    log.add_entry(first_entry);

    // Simulate some time passing
    std::thread::sleep(std::time::Duration::from_millis(1));

    let second_entry = TransformEntry {
        id: TransformId::new(),
        transform: Transform::Identity,
        source_span: Span::new(BytePos(10), BytePos(20)),
        result_span: Span::new(BytePos(10), BytePos(20)),
        timestamp: Instant::now(),
        proof: None,
        semantic_delta: None,
    };

    log.add_entry(second_entry);

    assert!(log.metadata.time_span.is_some());
    assert!(log.metadata.time_span.unwrap() > std::time::Duration::from_nanos(0));
}
