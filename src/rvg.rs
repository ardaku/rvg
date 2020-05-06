use zstd::stream::Encoder;
use ruzstd::frame_decoder::FrameDecoder;
use std::io::prelude::*;

const FORMAT_HEADER: [u8; 4] = [b'r', b'V', b'g', b'\x00'];

/// Pixel data
pub struct Bitmap {
    width: u16,
    height: u16,
    srgba: Vec<u8>,
}

/// 
pub enum Animation {
    /// Must be the last value.
    Done,
    /// No animation
    Jump,
    /// Constant speed
    Linear,
    /// Faster at beginning and end of animation(amount_faster: f32)
    ExpA(f32),
    /// Slower at beginning and end of animation
    ExpB(f32),
    /// Fade
    Fade,
    /// SrcOver each frame without clearing
    Layer,
}

/// 
pub enum Transform {
    Translate(f32, f32, f32),
    Scale(f32, f32, f32),
    Rotate(f32, f32, f32, f32),
}

/// 
pub struct Frame {
    pub transforms: Vec<Transform>,
    pub delay: u16,
    pub animation: Animation,
}

/// 
pub enum GroupProperty {
    FillColorRgba([u8; 4]),
    StrokeColorRgba([u8; 4]),
    StrokeWidth(f32),
    JoinStyle(u8),
    FillRule(u8),
    GlyphID(u32),
    BitmapPattern(u32),
    GroupPattern(u32),
}

/// 
pub struct Model {
    pub width: f32,
    pub height: f32,
    pub groups: Vec<(u32, Vec<GroupProperty>)>,
    pub frames: Vec<Frame>,
}

/// An RVG PathOp
pub enum PathOp {
    /// Close Path
    Close(),
    /// Move To
    Move(u32),
    /// Line To
    Line(u32),
    /// Quadratic Curve To
    Quad(u32, u32),
    /// Cubic Curve To
    Cubic(u32, u32, u32),
}

/// A vertex attribute.
pub enum Attribute {
    /// Z dimension (depth)
    Z,
    /// U,V Texture Coordinates
    UvTextureCoordinates,
    /// Vertex Gradient
    Rgb,
    /// Vertex Gradient
    Rbga,
    /// Vertex Gradient
    Alpha,
    /// Normal metadata
    Normal2D,
    /// Normal metadata
    Normal3D,
    /// Normal metadata
    Normal4D,
    /// Stroke Width
    StrokeWidth,
    /// User Defined metadata
    UserDefined(u8),
}

/// An RVG graphic that has been parsed, or will be parsed.
pub struct Graphic {
    pub attributes: Vec<Attribute>,
    pub vertex_list: Vec<f32>,
    pub group: Vec<Vec<PathOp>>,
    pub models: Vec<Model>,
    pub bitmaps: Vec<Bitmap>,
}

impl Graphic {
    pub fn load<R: Read>(reader: R) -> Option<Graphic> {
        None
    }

    #[cfg(feature = "zstd")]
    pub fn save<W: Write>(&self, writer: W) -> Option<()> {
        let mut encoder = Encoder::new(writer, 21).ok()?.auto_finish();

        // FORMAT
        encoder.write(&FORMAT_HEADER).ok()?;
        
        // ATTRIBUTE_LIST
        for attribute in &self.attributes {
            match attribute {
                Attribute::Z => encoder.write(&[1]).ok()?,
                Attribute::UvTextureCoordinates => encoder.write(&[2]).ok()?,
                Attribute::Rgb => encoder.write(&[3]).ok()?,
                Attribute::Rbga => encoder.write(&[4]).ok()?,
                Attribute::Alpha => encoder.write(&[5]).ok()?,
                Attribute::Normal2D => encoder.write(&[6]).ok()?,
                Attribute::Normal3D => encoder.write(&[7]).ok()?,
                Attribute::Normal4D => encoder.write(&[8]).ok()?,
                Attribute::StrokeWidth => encoder.write(&[9]).ok()?,
                Attribute::UserDefined(n) => encoder.write(&[16, *n]).ok()?,
            };
        }
        encoder.write(&[0]).ok()?;
        
        // VERTEX_LIST
        for vertex in &self.vertex_list {
            encoder.write(&vertex.to_le_bytes()).ok()?;
        }
        encoder.write(&f32::NAN.to_le_bytes()).ok()?;
        
        // GROUP
        for group in &self.group {
            for op in group {
                match op {
                    PathOp::Close() => { encoder.write(&[1]).ok()?; },
                    PathOp::Move(index) => {
                        let a = index.to_le_bytes();
                        encoder.write(&[2, a[0], a[1], a[2], a[3]]).ok()?;
                    }
                    PathOp::Line(index) => {
                        let a = index.to_le_bytes();
                        encoder.write(&[3, a[0], a[1], a[2], a[3]]).ok()?;
                    }
                    PathOp::Quad(one, two) => {
                        let a = one.to_le_bytes();
                        let b = two.to_le_bytes();
                        encoder.write(&[4, a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3]]).ok()?;
                    }
                    PathOp::Cubic(one, two, three) => {
                        let a = one.to_le_bytes();
                        let b = two.to_le_bytes();
                        let c = three.to_le_bytes();
                        encoder.write(&[5, a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], c[0], c[1], c[2], c[3]]).ok()?;
                    }
                }
            }
        }
        encoder.write(&[0]).ok()?;
        
        // MODELS
        for model in &self.models {
            encoder.write(&model.width.to_le_bytes()).ok()?;
            encoder.write(&model.height.to_le_bytes()).ok()?;
            
            // GROUPS
            for (group_id, group_props) in &model.groups {
                encoder.write(&group_id.to_le_bytes()).ok()?;
                for prop in group_props {
                    use GroupProperty::*;
                    match *prop {
                        FillColorRgba([r, g, b, a]) => {
                            encoder.write(&[1, r, g, b, a]).ok()?;
                        }
                        StrokeColorRgba([r, g, b, a]) => {
                            encoder.write(&[2, r, g, b, a]).ok()?;
                        }
                        StrokeWidth(width) => {
                            encoder.write(&[3]).ok()?;
                            encoder.write(&width.to_le_bytes()).ok()?;
                        }
                        JoinStyle(style) => {
                            encoder.write(&[4, style]).ok()?;
                        }
                        FillRule(rule) => {
                            encoder.write(&[5, rule]).ok()?;
                        }
                        GlyphID(id) => {
                            let a = id.to_le_bytes();
                            encoder.write(&[6, a[0], a[1], a[2], a[3]]).ok()?;
                        }
                        BitmapPattern(id) => {
                            let a = id.to_le_bytes();
                            encoder.write(&[7, a[0], a[1], a[2], a[3]]).ok()?;
                        }
                        GroupPattern(id) => {
                            let a = id.to_le_bytes();
                            encoder.write(&[8, a[0], a[1], a[2], a[3]]).ok()?;
                        }
                    }
                }
                encoder.write(&[0]).ok()?;
            }
            encoder.write(&u32::MAX.to_le_bytes()).ok()?;
            
            // FRAMES
            for frame in &model.frames {
                for transform in &frame.transforms {
                    use Transform::*;
                    match transform {
                        Translate(x, y, z) => {
                            encoder.write(&[1]).ok()?;
                            encoder.write(&x.to_le_bytes()).ok()?;
                            encoder.write(&y.to_le_bytes()).ok()?;
                            encoder.write(&z.to_le_bytes()).ok()?;
                        }
                        Scale(x, y, z) => {
                            encoder.write(&[2]).ok()?;
                            encoder.write(&x.to_le_bytes()).ok()?;
                            encoder.write(&y.to_le_bytes()).ok()?;
                            encoder.write(&z.to_le_bytes()).ok()?;
                        }
                        Rotate(x, y, z, w) => {
                            encoder.write(&[3]).ok()?;
                            encoder.write(&x.to_le_bytes()).ok()?;
                            encoder.write(&y.to_le_bytes()).ok()?;
                            encoder.write(&z.to_le_bytes()).ok()?;
                            encoder.write(&w.to_le_bytes()).ok()?;
                        }
                    }
                }
                encoder.write(&[0]).ok()?;
                encoder.write(&frame.delay.to_le_bytes()).ok()?;
                match frame.animation {
                    Animation::Done => encoder.write(&[0]).ok()?,
                    Animation::Jump => encoder.write(&[1]).ok()?,
                    Animation::Linear => encoder.write(&[2]).ok()?,
                    Animation::ExpA(amt_faster) => {
                        let a = amt_faster.to_le_bytes();
                        encoder.write(&[3, a[0], a[1], a[2], a[3]]).ok()?
                    },
                    Animation::ExpB(amt_faster) => {
                        let a = amt_faster.to_le_bytes();
                        encoder.write(&[4, a[0], a[1], a[2], a[3]]).ok()?
                    },
                    Animation::Fade => encoder.write(&[5]).ok()?,
                    Animation::Layer => encoder.write(&[6]).ok()?,
                };
            }
        }
        encoder.write(&f32::NAN.to_le_bytes()).ok()?;

        // BITMAPS
        for bitmap in &self.bitmaps {
            encoder.write(&bitmap.width.to_le_bytes()).ok()?;
            encoder.write(&bitmap.height.to_le_bytes()).ok()?;
            encoder.write(&bitmap.srgba).ok()?;
        }

        Some(())
    }
}



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

/*

use std::convert::AsMut;

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
        dbg!(&blocks);
        Rvg { format, blocks }
    }

    /// Turn RVG graphic back into file bytes.
    pub fn into_vec(self) -> Vec<u8> {
        dbg!(&self.blocks);
        let mut data = compress_to_vec(self.blocks.as_slice(), 10);
        data.extend(self.format.iter());
        data
    }
}*/
