use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;

const FORMAT_ANIM_ALBUM: [u8; 4] = [b'r', b'V', b'g', b'A'];

use std::convert::AsMut;

/// Helper function.
pub fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

///
#[repr(u8)]
pub enum BlockTypes {
    /// 2D points.
    Points2d = 0x00,
    /// 3D points.
    Points3d = 0x01,
    /// A graphic.
    Graphic = 0x02,
    /// A transition.
    Transition = 0x03,
    /// A bitmap.
    Bitmap = 0x04,
}

/// 
#[repr(u8)]
pub enum GraphicOps {
    // Fill (0x10-0x1F)
    Move = 0x10,
    Line = 0x11,
    Quad = 0x12,
    Cubic = 0x13,
    Arc = 0x14,
    Close = 0x1F,

    // Fill (0x20-0x2F)
    Solid = 0x20, // Fill with 1 color (before each vertex)
    Bitmap = 0x21, // Fill with bitmap - stretch (before Move)
    Tile = 0x22, // Fill tiled with pattern - Vector Graphics
    Pattern = 0x23, // Fill tiled with pattern - Bitmap
    Stroke = 0x24, // Change stroke color
    Width = 0x25, // Change stroke width
    Dashed = 0x26, // Change dash width (0=100% by default)

    // Join Style (0x30-0x3F)
    JoinMiter = 0x30, // `value` for amount
    JoinBevel = 0x31,
    JoinRound = 0x32,
}

/// An RVG Block.
pub struct Block(Vec<u8>);

impl Block {
    /// Create 2D point list Block.
    pub fn points2d(xy: &[(u16, u16)]) -> Self {
        let mut block = Block(vec![BlockTypes::Points2d as u8]);
        for (x,y) in xy.iter() {
            block.0.extend(x.to_be_bytes().iter());
            block.0.extend(y.to_be_bytes().iter());
        }
        block
    }

    /// Create 3D point list Block.
    pub fn points3d(xyz: &[(u16, u16, u16)]) -> Self {
        let mut block = Block(vec![BlockTypes::Points3d as u8]);
        for (x,y, z) in xyz.iter() {
            block.0.extend(x.to_be_bytes().iter());
            block.0.extend(y.to_be_bytes().iter());
            block.0.extend(z.to_be_bytes().iter());
        }
        block
    }

    /// Create a graphic Block.
    pub fn graphic(ar: u32, bgc: u64, ops: &[u8]) -> Self {
        let mut block = Block(vec![BlockTypes::Graphic as u8]);
        block.0.extend(ar.to_be_bytes().iter());
        block.0.extend(bgc.to_be_bytes().iter());
        block.0.extend(ops.iter());
        block
    }

    /// Read the block's bytes.
    pub fn read(&self) -> Option<(BlockTypes, &Vec<u8>)> {
        Some((match self.0[0] {
            0x00 => BlockTypes::Points2d,
            0x01 => BlockTypes::Points3d,
            0x02 => BlockTypes::Graphic,
            0x03 => return None,
            0x04 => return None,
            _ => return None,
        }, &self.0))
    }

/*    pub fn transition() -> Self {
        let mut block = Block(vec![BlockTypes::Transition as u8]);
        block
    }*/
}

pub struct BlockIter<'a> {
    rvg: &'a Rvg,
    cursor: usize,
}

impl<'a> Iterator for BlockIter<'a> {
    type Item = Block;

    fn next(&mut self) -> Option<Block> {
        let len = u32::from_be_bytes(clone_into_array(
            self.rvg.blocks.get(self.cursor..self.cursor + 4)?,
        )) as usize;
        self.cursor += 4;
        let ops = self.rvg.blocks.get(self.cursor..self.cursor + len)?.to_vec();
        self.cursor += len;

        Some(Block(ops))
    }
}

/// An RVG File.
pub struct Rvg {
    // Currently must be FORMAT_ANIM_ALBUM.
    format: [u8; 4],
    // Uncompressed Graphic Data
    blocks: Vec<u8>,
}

impl Default for Rvg {
    fn default() -> Self {
        Self::new()
    }
}

impl Rvg {
    /// Create a new empty graphic.
    pub fn new() -> Rvg {
        // File Footer (32 bits)
        let format = FORMAT_ANIM_ALBUM;
        let blocks = vec![];

        Rvg { format, blocks }
    }

    /// Add a block
    pub fn block(&mut self, block: Block) {
        self.blocks.extend((block.0.len() as u32).to_be_bytes().iter());
        self.blocks.extend(block.0);
    }

    /// Iterate over blocks
    pub fn iter(&self) -> BlockIter {
        BlockIter {
            rvg: self,
            cursor: 0,
        }
    }

    /// Create RVG file from file bytes.
    pub fn from_slice(buf: &[u8]) -> Rvg {
        let format = [
            buf[buf.len() - 4],
            buf[buf.len() - 3],
            buf[buf.len() - 2],
            buf[buf.len() - 1],
        ];
        let blocks = decompress_to_vec(&buf[..buf.len() - 4]).unwrap();
        Rvg { format, blocks }
    }

    /// Turn RVG graphic back into file bytes.
    pub fn into_vec(self) -> Vec<u8> {
        let mut data = compress_to_vec(self.blocks.as_slice(), 10);
        data.extend(self.format.iter());
        data
    }
}
