use zstd::stream::Encoder;
use ruzstd::streaming_decoder::StreamingDecoder;
use std::io::prelude::*;

const FORMAT_HEADER: [u8; 4] = [b'r', b'V', b'g', b'\x00'];

/// Pixel data
pub struct Bitmap {
    width: u16,
    height: u16,
    srgba: Vec<u8>,
}

/// 
#[derive(PartialEq)]
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
    pub fn load<R: Read>(mut reader: R) -> Option<Graphic> {
        let mut reader = StreamingDecoder::new(&mut reader).unwrap();
        let mut buf = vec![];
        let len = reader.read_to_end(&mut buf).unwrap();
        dbg!(len);
        let mut buf = buf.iter().cloned();

        // FORMAT
        let header = [buf.next()?, buf.next()?, buf.next()?, buf.next()?];
        if header != FORMAT_HEADER {
            eprintln!("Headers do not match: {:?} ≠ {:?}", header, FORMAT_HEADER);
            return None;
        }
        
        // ATTRIBUTE_LIST
        let mut attributes = Vec::new();
        loop {
            attributes.push(match buf.next()? {
                0 => break,
                1 => Attribute::Z,
                2 => Attribute::UvTextureCoordinates,
                3 => Attribute::Rgb,
                4 => Attribute::Rbga,
                5 => Attribute::Alpha,
                6 => Attribute::Normal2D,
                7 => Attribute::Normal3D,
                8 => Attribute::Normal4D,
                9 => Attribute::StrokeWidth,
                10 => Attribute::UserDefined(buf.next()?),
                u => panic!("Unknown attribute {}", u),
            });
        }
        
        // VERTEX_LIST
        let mut vertex_list = Vec::new();
        loop {
            let value: f32 = f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]);
            match value {
                x if x.is_nan() => break,
                x => vertex_list.push(x),
            }
        }

        // GROUP
        let mut group = Vec::new();
        'g: loop {
            let mut path = Vec::new();
            'p: loop {
                path.push(match buf.next()? {
                    0 => break 'p,
                    1 => PathOp::Close(),
                    2 => PathOp::Move({
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }),
                    3 => PathOp::Line({
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }),
                    4 => PathOp::Quad({
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }, {
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }),
                    5 => PathOp::Cubic({
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }, {
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }, {
                        u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])
                    }),
                    u => panic!("Unknown path {}", u),
                });
            }
            if path.is_empty() {
                break 'g;
            }
            group.push(path);
        }
        
        // MODELS
        let mut models = Vec::new();
        'm: loop {
            let width = f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]);
            if width.is_nan() {
                break 'm;
            }
            let height = f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]);
            
            let mut groups = Vec::new();
            'g2: loop {
                let group_id = u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]);
                if group_id == u32::MAX {
                    break 'g2;
                }
                
                let mut group_props = Vec::new();
                'p2: loop {
                    group_props.push(match buf.next()? {
                        0 => break 'p2,
                        1 => GroupProperty::FillColorRgba([buf.next()?, buf.next()?, buf.next()?, buf.next()?]),
                        2 => GroupProperty::StrokeColorRgba([buf.next()?, buf.next()?, buf.next()?, buf.next()?]),
                        3 => GroupProperty::StrokeWidth(f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        4 => GroupProperty::JoinStyle(buf.next()?),
                        5 => GroupProperty::FillRule(buf.next()?),
                        6 => GroupProperty::GlyphID(u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        7 => GroupProperty::BitmapPattern(u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        8 => GroupProperty::GroupPattern(u32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        u => panic!("Unknown group property: {}", u),
                    });
                }
                
                groups.push((group_id, group_props));
            }
            
            let mut frames = Vec::new();
            'f: loop {
                let mut transforms = Vec::new();
                't: loop {
                    transforms.push(match buf.next()? {
                        0 => break 't,
                        1 => Transform::Translate(f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        2 => Transform::Scale(f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        3 => Transform::Rotate(f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?]), f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                        u => panic!("Unknown transform: {}", u),
                    });
                }
                let delay = u16::from_le_bytes([buf.next()?, buf.next()?]);
                let animation = match buf.next()? {
                    0 => Animation::Done,
                    1 => Animation::Jump,
                    2 => Animation::Linear,
                    3 => Animation::ExpA(f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                    4 => Animation::ExpB(f32::from_le_bytes([buf.next()?, buf.next()?, buf.next()?, buf.next()?])),
                    5 => Animation::Fade,
                    6 => Animation::Layer,
                    u => panic!("Unknown animation: {}", u),
                };
                
                let done = animation == Animation::Done;
                
                frames.push(Frame { transforms, delay, animation });
                
                if done {
                    break 'f;
                }
            }
            models.push(Model {frames, groups, width, height});
        }
            
        // BITMAPS
        let mut bitmaps = Vec::new();
        while let Some(a) = buf.next() {
            let width = u16::from_le_bytes([a, buf.next()?]);
            let height = u16::from_le_bytes([buf.next()?, buf.next()?]);
            let mut srgba = Vec::new();
            for _ in 0..(width*height*4) {
                srgba.push(buf.next()?);
            }
            bitmaps.push(Bitmap {
                width, height, srgba
            });
        }
        
        println!("Load Success!!");

        Some(Graphic {
            attributes, bitmaps, group, models, vertex_list,
        })
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
            encoder.write(&[0]).ok()?;
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
