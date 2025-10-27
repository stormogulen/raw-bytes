// save/src/merkle.rs
use sha2::{Sha256, Digest};
use std::cmp::max;

/// A Merkle tree node: either raw data (`Leaf`) or combined hash (`Internal`)
#[derive(Debug, Clone)]
pub enum MerkleNode {
    Leaf(Vec<u8>),
    Internal(Vec<u8>, Box<MerkleNode>, Box<MerkleNode>),
}

impl MerkleNode {
    /// Create a leaf node from raw data
    pub fn from_data(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        MerkleNode::Leaf(hash.to_vec())
    }

    /// Return the hash of this node
    pub fn hash(&self) -> Vec<u8> {
        match self {
            MerkleNode::Leaf(h) => h.clone(),
            MerkleNode::Internal(h, _, _) => h.clone(),
        }
    }

    /// Depth of the tree
    pub fn depth(&self) -> u32 {
        match self {
            MerkleNode::Leaf(_) => 0,
            MerkleNode::Internal(_, left, right) => 1 + max(left.depth(), right.depth()),
        }
    }
}

/// Build a Merkle tree from a list of byte chunks
pub fn build_merkle_tree(chunks: &[Vec<u8>]) -> MerkleNode {
    let mut nodes: Vec<MerkleNode> = chunks.iter().map(|d| MerkleNode::from_data(d)).collect();

    while nodes.len() > 1 {
        let mut next = Vec::new();
        for pair in nodes.chunks(2) {
            let left = pair[0].clone();
            let right = pair.get(1).cloned().unwrap_or_else(|| left.clone());

            let mut hasher = Sha256::new();
            hasher.update(left.hash());
            hasher.update(right.hash());
            let combined_hash = hasher.finalize().to_vec();

            next.push(MerkleNode::Internal(combined_hash, Box::new(left), Box::new(right)));
        }
        nodes = next;
    }

    nodes.pop().expect("no nodes built")
}

/// Verify that the provided chunks produce the expected root hash
pub fn verify_merkle_tree(chunks: &[Vec<u8>], expected_root: &[u8]) -> bool {
    let root = build_merkle_tree(chunks);
    root.hash() == expected_root
}
