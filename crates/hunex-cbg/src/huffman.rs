//! The code tree of a stripe: built from the 256 byte weights, and read or written bit by bit.

use alloc::vec;
use alloc::vec::Vec;

use crate::bits::{BitReader, BitWriter};
use crate::Error;

#[derive(Clone, Copy)]
enum Node {
    Leaf(u8),
    Branch { zero: usize, one: usize },
}

pub(crate) struct Tree {
    nodes: Vec<Node>,
    root: usize,
}

/// Builds the tree. Leaves are the byte values; a leaf of weight 0 is never used. Repeatedly the
/// two unparented nodes of smallest weight are joined (ties go to the earlier node), and the
/// smaller becomes the 0-child. The build stops when the newest node's weight reaches the total.
pub(crate) fn build(weights: &[u64; 256]) -> Result<Tree, Error> {
    let mut nodes: Vec<Node> = (0..=255u8).map(Node::Leaf).collect();
    let mut weight: Vec<u64> = weights.to_vec();
    let mut total = 0u64;
    for &w in weights {
        total = total.checked_add(w).ok_or(Error::BadStream {
            what: "weight total",
        })?;
    }
    let mut open: Vec<usize> = (0..256).filter(|&i| weights[i] > 0).collect();
    if open.is_empty() {
        return Err(Error::BadStream { what: "no weights" });
    }
    while open.len() > 1 {
        open.sort_by_key(|&index| (weight[index], index));
        let zero = open.remove(0);
        let one = open.remove(0);
        let sum = weight[zero]
            .checked_add(weight[one])
            .ok_or(Error::BadStream { what: "weight sum" })?;
        nodes.push(Node::Branch { zero, one });
        weight.push(sum);
        let newest = nodes.len() - 1;
        open.push(newest);
        if sum >= total {
            break;
        }
    }
    let root = open[0];
    Ok(Tree { nodes, root })
}

impl Tree {
    /// Reads one symbol.
    pub(crate) fn read(&self, bits: &mut BitReader<'_>) -> Result<u8, Error> {
        let mut node = self.root;
        loop {
            match self.nodes[node] {
                Node::Leaf(symbol) => return Ok(symbol),
                Node::Branch { zero, one } => {
                    node = if bits.bit()? == 0 { zero } else { one };
                }
            }
        }
    }

    /// The code of every byte value: the path bits from the root, or `None` for an unused value.
    pub(crate) fn codes(&self) -> Vec<Option<Vec<u8>>> {
        let mut table = vec![None; 256];
        let mut stack = vec![(self.root, Vec::new())];
        while let Some((node, path)) = stack.pop() {
            match self.nodes[node] {
                Node::Leaf(symbol) => table[usize::from(symbol)] = Some(path),
                Node::Branch { zero, one } => {
                    let mut zero_path = path.clone();
                    zero_path.push(0);
                    let mut one_path = path;
                    one_path.push(1);
                    stack.push((zero, zero_path));
                    stack.push((one, one_path));
                }
            }
        }
        table
    }
}

/// Writes the code of each symbol, in order.
pub(crate) fn write_codes(
    writer: &mut BitWriter,
    codes: &[Option<Vec<u8>>],
    symbols: &[u8],
) -> Result<(), Error> {
    for &symbol in symbols {
        let code = codes[usize::from(symbol)]
            .as_ref()
            .ok_or(Error::BadStream {
                what: "unused symbol",
            })?;
        for &bit in code {
            writer.push(bit);
        }
    }
    Ok(())
}
