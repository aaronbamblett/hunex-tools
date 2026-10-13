use alloc::vec;
use alloc::vec::Vec;

pub const GOB_BYTES: usize = 512;
pub const GOB_WIDTH: usize = 64;
pub const GOB_ROWS: usize = 8;
const MAX_BLOCK_HEIGHT: usize = 16;

/// The block height in GOBs for a texture with `rows` rows of units.
pub fn block_height(rows: usize) -> usize {
    let mut height = MAX_BLOCK_HEIGHT;
    while height > 1 && rows <= (height / 2) * GOB_ROWS {
        height /= 2;
    }
    height
}

/// Byte offset within a GOB of the byte at column `x` (0 to 63) and row `y` (0 to 7).
pub const fn gob_offset(x: usize, y: usize) -> usize {
    (x & 0xF)
        | ((y & 1) << 4)
        | (((x >> 4) & 1) << 5)
        | (((y >> 1) & 3) << 6)
        | (((x >> 5) & 1) << 8)
}

/// Reorders block-linear data into rows: `row_bytes` bytes per row of units, `rows` rows.
/// `cols` is the number of GOB columns and `block` the block height in GOBs.
pub fn deswizzle(
    source: &[u8],
    row_bytes: usize,
    rows: usize,
    cols: usize,
    block: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; row_bytes * rows];
    for row in 0..rows {
        let gob_row = row / GOB_ROWS;
        let in_gob_row = row % GOB_ROWS;
        let block_row = gob_row / block;
        let in_block = gob_row % block;
        for column in 0..row_bytes {
            let gob_column = column / GOB_WIDTH;
            let in_gob_column = column % GOB_WIDTH;
            let gob = (block_row * cols + gob_column) * block + in_block;
            let offset = gob * GOB_BYTES + gob_offset(in_gob_column, in_gob_row);
            out[row * row_bytes + column] = source.get(offset).copied().unwrap_or(0);
        }
    }
    out
}
