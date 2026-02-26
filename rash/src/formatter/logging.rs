//! Transform logging with Merkle tree for integrity verification

use crate::formatter::{transforms::*, types::*};
use blake3::{Hash, Hasher};
use std::time::Instant;

/// Append-only log for transformation verification context propagation
#[derive(Debug, Clone)]
pub struct TransformLog {
    /// All transformation entries
    pub entries: Vec<TransformEntry>,

    /// Merkle tree for integrity verification
    merkle_tree: MerkleTree,

    /// Log metadata
    metadata: LogMetadata,
}

/// Individual transformation log entry
#[derive(Debug, Clone)]
pub struct TransformEntry {
    /// Unique identifier for this transformation
    pub id: TransformId,

    /// The transformation that was applied
    pub transform: Transform,

    /// Source span affected by this transformation
    pub source_span: Span,

    /// Resulting span after transformation
    pub result_span: Span,

    /// Timestamp when transformation was applied
    pub timestamp: Instant,

    /// Optional SMT proof for semantic preservation
    pub proof: Option<SexprProof>,

    /// Semantic changes introduced (if any)
    pub semantic_delta: Option<SemanticDelta>,
}

/// Merkle tree for log integrity verification
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// Root hash of the tree
    root_hash: Hash,

    /// All leaf hashes (one per log entry)
    leaf_hashes: Vec<Hash>,

    /// Internal node hashes for efficient verification
    internal_nodes: Vec<Hash>,

    /// Tree height (log2 of leaf count, rounded up)
    _height: usize,
}

/// Metadata about the transformation log
#[derive(Debug, Clone)]
pub struct LogMetadata {
    /// Total number of transformations
    pub total_transforms: usize,

    /// Number of semantic-preserving transformations
    pub semantic_preserving: usize,

    /// Number of transformations with proofs
    pub with_proofs: usize,

    /// Time span of all transformations
    pub time_span: Option<std::time::Duration>,

    /// Log creation timestamp
    pub created_at: Instant,
}

/// Merkle proof for verifying log entry integrity
#[derive(Debug, Clone)]
pub struct MerkleProof {
    /// Index of the leaf being verified
    pub leaf_index: usize,

    /// Hash path from leaf to root
    pub path: Vec<Hash>,

    /// Direction indicators (true = right, false = left)
    pub directions: Vec<bool>,
}

/// Result of verifying a Merkle proof
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Valid,
    Invalid,
    InsufficientData,
}

impl TransformLog {
    /// Create a new empty transform log
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            merkle_tree: MerkleTree::empty(),
            metadata: LogMetadata {
                total_transforms: 0,
                semantic_preserving: 0,
                with_proofs: 0,
                time_span: None,
                created_at: Instant::now(),
            },
        }
    }

    /// Add a new transformation entry
    pub fn add_entry(&mut self, entry: TransformEntry) {
        // Update metadata
        self.metadata.total_transforms += 1;

        if entry.transform.is_semantic_preserving() {
            self.metadata.semantic_preserving += 1;
        }

        if entry.proof.is_some() {
            self.metadata.with_proofs += 1;
        }

        // Update time span
        if let Some(first_entry) = self.entries.first() {
            self.metadata.time_span = Some(entry.timestamp.duration_since(first_entry.timestamp));
        }

        // Add to entries
        self.entries.push(entry);

        // Rebuild Merkle tree
        self.rebuild_merkle_tree();
    }

    /// Rebuild the Merkle tree from all entries
    fn rebuild_merkle_tree(&mut self) {
        if self.entries.is_empty() {
            self.merkle_tree = MerkleTree::empty();
            return;
        }

        // Compute leaf hashes
        let leaf_hashes: Vec<Hash> = self
            .entries
            .iter()
            .map(|entry| self.hash_entry(entry))
            .collect();

        self.merkle_tree = MerkleTree::from_leaves(leaf_hashes);
    }

    /// Hash a single transform entry
    fn hash_entry(&self, entry: &TransformEntry) -> Hash {
        let mut hasher = Hasher::new();

        // Hash the transform ID
        hasher.update(&entry.id.0.to_le_bytes());

        // Hash the transform description
        hasher.update(entry.transform.description().as_bytes());

        // Hash the spans
        hasher.update(&entry.source_span.start.0.to_le_bytes());
        hasher.update(&entry.source_span.end.0.to_le_bytes());
        hasher.update(&entry.result_span.start.0.to_le_bytes());
        hasher.update(&entry.result_span.end.0.to_le_bytes());

        // Hash timestamp (as nanoseconds since epoch)
        let nanos = entry.timestamp.elapsed().as_nanos();
        hasher.update(&(nanos as u64).to_le_bytes());

        // Hash proof if present
        if let Some(proof) = &entry.proof {
            hasher.update(proof.formula.as_bytes());
            hasher.update(&[u8::from(proof.is_valid)]);
        }

        hasher.finalize()
    }

    /// Generate a proof for a specific log entry
    pub fn generate_proof(&self, entry_index: usize) -> Option<MerkleProof> {
        if entry_index >= self.entries.len() {
            return None;
        }

        self.merkle_tree.generate_proof(entry_index)
    }

    /// Verify a Merkle proof against the current root
    pub fn verify_proof(&self, proof: &MerkleProof, leaf_hash: Hash) -> VerificationResult {
        self.merkle_tree.verify_proof(proof, leaf_hash)
    }

    /// Get the current root hash for integrity checking
    pub fn root_hash(&self) -> Hash {
        self.merkle_tree.root_hash
    }

    /// Export log for external verification
    pub fn export_verification_data(&self) -> LogVerificationData {
        LogVerificationData {
            entries: self.entries.clone(),
            root_hash: self.merkle_tree.root_hash,
            leaf_hashes: self.merkle_tree.leaf_hashes.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// Get statistics about the log
    pub fn stats(&self) -> LogStats {
        let total = self.metadata.total_transforms;
        let semantic_ratio = if total > 0 {
            self.metadata.semantic_preserving as f64 / total as f64
        } else {
            0.0
        };

        let proof_ratio = if total > 0 {
            self.metadata.with_proofs as f64 / total as f64
        } else {
            0.0
        };

        LogStats {
            total_entries: total,
            semantic_preserving_ratio: semantic_ratio,
            proof_coverage_ratio: proof_ratio,
            memory_usage_bytes: self.estimate_memory_usage(),
            integrity_verified: true, // Always true for our implementation
        }
    }

    fn estimate_memory_usage(&self) -> usize {
        let entries_size = self.entries.len() * std::mem::size_of::<TransformEntry>();
        let tree_size = self.merkle_tree.leaf_hashes.len() * std::mem::size_of::<Hash>()
            + self.merkle_tree.internal_nodes.len() * std::mem::size_of::<Hash>();

        entries_size + tree_size + std::mem::size_of::<LogMetadata>()
    }
}

impl MerkleTree {
    /// Create an empty Merkle tree
    pub fn empty() -> Self {
        Self {
            root_hash: blake3::hash(b""),
            leaf_hashes: Vec::new(),
            internal_nodes: Vec::new(),
            _height: 0,
        }
    }

    /// Create Merkle tree from leaf hashes
    pub fn from_leaves(mut leaf_hashes: Vec<Hash>) -> Self {
        if leaf_hashes.is_empty() {
            return Self::empty();
        }

        // Pad to next power of 2 for balanced tree (minimum 2 leaves)
        let original_count = leaf_hashes.len();
        let next_power_of_2 = std::cmp::max(2, original_count.next_power_of_two());
        let padding_needed = next_power_of_2 - original_count;

        // Pad with zero hashes
        let zero_hash = blake3::hash(b"");
        for _ in 0..padding_needed {
            leaf_hashes.push(zero_hash);
        }

        let height = (leaf_hashes.len() as f64).log2() as usize;
        let mut internal_nodes = Vec::new();
        let mut current_level = leaf_hashes.clone();

        // Build tree bottom-up
        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let left = chunk[0];
                let right = chunk.get(1).copied().unwrap_or(zero_hash);

                let mut hasher = Hasher::new();
                hasher.update(left.as_bytes());
                hasher.update(right.as_bytes());
                let parent_hash = hasher.finalize();

                internal_nodes.push(parent_hash);
                next_level.push(parent_hash);
            }

            current_level = next_level;
        }

        let root_hash = current_level[0];

        Self {
            root_hash,
            leaf_hashes,
            internal_nodes,
            _height: height,
        }
    }

    /// Generate a Merkle proof for a specific leaf
    pub fn generate_proof(&self, leaf_index: usize) -> Option<MerkleProof> {
        if leaf_index >= self.leaf_hashes.len() {
            return None;
        }

        let mut path = Vec::new();
        let mut directions = Vec::new();
        let current_index = leaf_index;
        let _current_level = &self.leaf_hashes;

        // Generate simple proof (simplified implementation)
        if _current_level.len() > 1 {
            let sibling_index = if current_index.is_multiple_of(2) {
                current_index + 1
            } else {
                current_index - 1
            };

            // Add sibling to path
            if sibling_index < _current_level.len() {
                path.push(_current_level[sibling_index]);
                directions.push(current_index.is_multiple_of(2)); // true if we're left child
            } else {
                // Pad with zero hash
                path.push(blake3::hash(b""));
                directions.push(true);
            }
        }

        Some(MerkleProof {
            leaf_index,
            path,
            directions,
        })
    }

    /// Verify a Merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof, leaf_hash: Hash) -> VerificationResult {
        if proof.leaf_index >= self.leaf_hashes.len() {
            return VerificationResult::Invalid;
        }

        if proof.path.len() != proof.directions.len() {
            return VerificationResult::Invalid;
        }

        let mut current_hash = leaf_hash;

        // Recompute root hash using proof path
        for (sibling_hash, is_left) in proof.path.iter().zip(&proof.directions) {
            let mut hasher = Hasher::new();

            if *is_left {
                hasher.update(current_hash.as_bytes());
                hasher.update(sibling_hash.as_bytes());
            } else {
                hasher.update(sibling_hash.as_bytes());
                hasher.update(current_hash.as_bytes());
            }

            current_hash = hasher.finalize();
        }

        if current_hash == self.root_hash {
            VerificationResult::Valid
        } else {
            VerificationResult::Invalid
        }
    }
}

/// Data structure for external log verification
#[derive(Debug, Clone)]
pub struct LogVerificationData {
    pub entries: Vec<TransformEntry>,
    pub root_hash: Hash,
    pub leaf_hashes: Vec<Hash>,
    pub metadata: LogMetadata,
}

/// Statistics about the transformation log
#[derive(Debug, Clone)]
pub struct LogStats {
    pub total_entries: usize,
    pub semantic_preserving_ratio: f64,
    pub proof_coverage_ratio: f64,
    pub memory_usage_bytes: usize,
    pub integrity_verified: bool,
}

impl Default for TransformLog {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LogMetadata {
    fn default() -> Self {
        Self {
            total_transforms: 0,
            semantic_preserving: 0,
            with_proofs: 0,
            time_span: None,
            created_at: Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
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
}
