use bytemuck::{Pod, Zeroable};
use bytemuck_derive::{Pod, Zeroable};

use packed_struct_container::PackedStructContainer;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use sha2::{Digest, Sha256};



/// Example save data
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct SaveData {
    level: u32,
    score: u32,
}

/// Simple Merkle tree
#[derive(Debug, Clone)]
enum MerkleNode {
    Leaf(Vec<u8>),
    Internal(Vec<u8>, Box<MerkleNode>, Box<MerkleNode>),
}

impl MerkleNode {
    fn new_leaf(data: Vec<u8>) -> Self {
        MerkleNode::Leaf(data)
    }

    fn hash(&self) -> Vec<u8> {
        match self {
            MerkleNode::Leaf(data) => Sha256::digest(data).to_vec(),
            MerkleNode::Internal(_, left, right) => {
                let combined = [&left.hash()[..], &right.hash()[..]].concat();
                Sha256::digest(&combined).to_vec()
            }
        }
    }
}

/// Build a Merkle tree from a list of byte blocks
fn build_merkle_tree(blocks: &[Vec<u8>]) -> Option<MerkleNode> {
    if blocks.is_empty() {
        return None;
    }

    let mut nodes: Vec<MerkleNode> = blocks.iter().map(|b| MerkleNode::new_leaf(b.clone())).collect();

    while nodes.len() > 1 {
        let mut next_level = Vec::new();
        for i in (0..nodes.len()).step_by(2) {
            let left = nodes[i].clone();
            let right = if i + 1 < nodes.len() { nodes[i + 1].clone() } else { nodes[i].clone() };
            let combined_hash = [&left.hash()[..], &right.hash()[..]].concat();
            next_level.push(MerkleNode::Internal(combined_hash, Box::new(left), Box::new(right)));
        }
        nodes = next_level;
    }

    nodes.pop()
}

/// Save the game
fn save_game<P: AsRef<Path>>(path: P, container: &PackedStructContainer<SaveData>) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Prepare Merkle tree
    let byte_blocks: Vec<Vec<u8>> = container.iter()
        .map(|s| bytemuck::bytes_of(&s).to_vec())
        .collect();

    let root = build_merkle_tree(&byte_blocks).expect("Cannot save empty container");
    let root_hash = root.hash();

    // Write header: number of elements
    let len = container.len() as u32;
    file.write_all(&len.to_le_bytes())?;

    // Write Merkle root length + root hash
    let root_len = root_hash.len() as u32;
    file.write_all(&root_len.to_le_bytes())?;
    file.write_all(&root_hash)?;

    // Write container bytes
    for s in container.iter() {
        file.write_all(bytemuck::bytes_of(&s))?;
    }

    Ok(())
}

/// Load the game
fn load_game<P: AsRef<Path>>(path: P) -> std::io::Result<PackedStructContainer<SaveData>> {
    let mut file = File::open(path)?;

    // Read header
    let mut len_bytes = [0u8; 4];
    file.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;

    // Read Merkle root
    let mut root_len_bytes = [0u8; 4];
    file.read_exact(&mut root_len_bytes)?;
    let root_len = u32::from_le_bytes(root_len_bytes) as usize;
    let mut root_hash = vec![0u8; root_len];
    file.read_exact(&mut root_hash)?;

    // Read save data
    let mut buffer = vec![0u8; len * std::mem::size_of::<SaveData>()];
    file.read_exact(&mut buffer)?;

    // Convert bytes into SaveData slice
    let save_slice: &[SaveData] = bytemuck::cast_slice(&buffer);

    let container = PackedStructContainer::from_slice(save_slice);

    // Verify Merkle root
    let byte_blocks: Vec<Vec<u8>> = container.iter().map(|s| bytemuck::bytes_of(&s).to_vec()).collect();
    let root_computed = build_merkle_tree(&byte_blocks).expect("Empty container");
    if root_computed.hash() != root_hash {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Merkle root mismatch"));
    }

    Ok(container)
}

fn main() -> std::io::Result<()> {
    let data = [
        SaveData { level: 1, score: 100 },
        SaveData { level: 2, score: 200 },
    ];

    let container = PackedStructContainer::from_slice(&data);

    save_game("savefile.bin", &container)?;
    let loaded = load_game("savefile.bin")?;

    println!("Loaded {} entries:", loaded.len());
    for s in loaded.iter() {
        println!("{:?}", s);
    }

    Ok(())
}
