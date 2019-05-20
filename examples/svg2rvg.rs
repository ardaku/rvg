//! Convert an SVG into an RVG.

use rvg;
use usvg;
use usvg::svgdom::WriteBuffer;
use usvg::svgdom::{
    AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathSegment, LengthUnit, Transform
};
use std::io::Write;
use rvg::{Block, GraphicOps};

pub fn search_add(pts: &mut Vec<(u16,u16)>, pt: (u16,u16)) -> u16 {
    for i in 0..pts.len() {
        if pts[i] == pt {
            return i as u16;
        }
    }
    pts.push(pt);
    return pts.len() as u16 - 1;
}

/// Convert an SVG string into RVG byte data.
fn rvg_from_svg(svg: &str) -> Vec<u8> {
    // Build a new RVG.
    let mut pts = vec![];
    let mut ops = vec![];

    // Simplify SVG with usvg.
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).unwrap();
    let svg = tree
        .to_svgdom()
        .with_write_opt(&usvg::svgdom::WriteOptions::default())
        .to_string();
//    println!("SVG: {}", svg);

    // Render
    let doc = Document::from_str(&svg).unwrap();
    let mut iter = doc.root().descendants().svg();

    let (width, height) = if let Some((id, node)) = iter.next() {
        if id == ElementId::Svg {
            let attrs = node.attributes();
            let width;
            let height;

//            println!("{:?}", attrs);

            if let Some(&AttributeValue::ViewBox(ref v)) =
                attrs.get_value(AttributeId::ViewBox)
            {
                width = v.w as f32;
                height = v.h as f32;
            } else if let Some(&AttributeValue::Length(ref v)) =
                attrs.get_value(AttributeId::Width)
            {
                width = v.num as f32;
                if let Some(&AttributeValue::Length(ref v)) =
                    attrs.get_value(AttributeId::Height)
                {
                    height = v.num as f32;
                } else {
                    panic!("Height unspecified!");
                }
            } else {
                panic!("Width unspecified!");
            }

            (width, height)
        } else {
            panic!("Not an SVG!");
        }
    } else {
        panic!("SVG is an empty file!");
    };

    println!("WH: ({} {})", width, height);

    let ar = (65536.0 * height / width) as u32;
    let bgc = 0u64; // TODO

    for (id, node) in iter {
        match id {
            ElementId::Path => {
                let mut old_x = 0.0f32;
                let mut old_y = 0.0f32;

                let attrs = node.attributes();

                let (red, green, blue) = if let Some(&AttributeValue::Color(ref c)) =
                    attrs.get_value(AttributeId::Fill)
                {
                    ((c.red as u16) * 256,
                    (c.green as u16) * 256,
                    (c.blue as u16) * 256)
                } else {
                    (0, 0, 0)
                };
                let alpha = if attrs.get_value(AttributeId::Fill) != Some(&AttributeValue::None) { 65535u16 } else { 0u16 };
                ops.push(GraphicOps::Solid as u8);
                ops.extend(red.to_be_bytes().iter());
                ops.extend(green.to_be_bytes().iter());
                ops.extend(blue.to_be_bytes().iter());
                ops.extend(alpha.to_be_bytes().iter());

                if let Some(&AttributeValue::Color(ref c)) =
                    attrs.get_value(AttributeId::Stroke)
                {
                    let red = (c.red as u16) * 256;
                    let green = (c.green as u16) * 256;
                    let blue = (c.blue as u16) * 256;
                    let alpha = 65535u16;

                    ops.push(GraphicOps::Stroke as u8);
                    ops.extend(red.to_be_bytes().iter());
                    ops.extend(green.to_be_bytes().iter());
                    ops.extend(blue.to_be_bytes().iter());
                    ops.extend(alpha.to_be_bytes().iter());
                }

                if let Some(&AttributeValue::Length(ref w)) =
                    attrs.get_value(AttributeId::StrokeWidth)
                {
                    assert!(w.unit == LengthUnit::Px || w.unit == LengthUnit::None);
                    println!("WIDTH");
                    let width = (w.num as f32) / width;
                    let width = (width * 65535.0) as u16;

                    ops.push(GraphicOps::Width as u8);
                    ops.extend(width.to_be_bytes().iter());
                }

                let transform = if let Some(&AttributeValue::Transform(ref t)) =
                    attrs.get_value(AttributeId::Transform)
                {
                    *t
                } else {
                    Transform::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
                };

                if let Some(&AttributeValue::Path(ref path)) =
                    attrs.get_value(AttributeId::D)
                {
                    for seg in path.iter() {
                        match seg {
                            PathSegment::MoveTo {abs, mut x, mut y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                transform.apply_to(&mut x, &mut y);
                                old_x = x as f32;
                                old_y = y as f32;

                                // Generate MoveTo command.
                                let x = old_x / width;
                                let y = old_y / height;
                                println!("MOVE {} {}", x, y);
                                let x = (x * 16384 as f32 + 24576.0) as u16;
                                let y = (y * 16384 as f32 + 24576.0) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Move as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::LineTo {abs, mut x, mut y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                transform.apply_to(&mut x, &mut y);
                                old_x = x as f32;
                                old_y = y as f32;

                                // Generate LineTo command.
                                let x = old_x / width;
                                let y = old_y / height;
                                println!("LINE {} {}", x, y);
                                let x = (x * 16384 as f32 + 24576.0) as u16;
                                let y = (y * 16384 as f32 + 24576.0) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::HorizontalLineTo {abs, x} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = transform.apply(*x, 0.0).0 as f32;

                                // Generate LineTo command.
                                let x = ((old_x / width) * 16384 as f32 + 24576.0) as u16;
                                let y = ((old_y / height) * 16384 as f32 + 24576.0) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::VerticalLineTo {abs, y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_y = transform.apply(0.0, *y).1 as f32;

                                // Generate LineTo command.
                                let x = ((old_x / width) * 16384 as f32 + 24576.0) as u16;
                                let y = ((old_y / height) * 16384 as f32 + 24576.0) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::Quadratic {abs, mut x1, mut y1, mut x, mut y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                transform.apply_to(&mut x1, &mut y1);
                                transform.apply_to(&mut x, &mut y);

                                old_x = x as f32;
                                old_y = y as f32;
                                let old_x1 = x1 as f32;
                                let old_y1 = y1 as f32;

                                let x = old_x / width;
                                let y = old_y / height;
                                let x1 = old_x1 / width;
                                let y1 = old_y1 / height;
                                println!("QUAD {} {}", x, y);
                                let x = (x * 16384 as f32 + 24576.0) as u16;
                                let y = (y * 16384 as f32 + 24576.0) as u16;
                                let x1 = (x1 * 16384 as f32 + 24576.0) as u16;
                                let y1 = (y1 * 16384 as f32 + 24576.0) as u16;

                                let i = search_add(&mut pts, (x1,y1));
                                let j = search_add(&mut pts, (x,y));

                                ops.push(GraphicOps::Quad as u8);
                                ops.extend(i.to_be_bytes().iter());
                                ops.extend(j.to_be_bytes().iter());
                            }
                            PathSegment::CurveTo {
                                abs,
                                mut x1,
                                mut y1,
                                mut x2,
                                mut y2,
                                mut x,
                                mut y,
                            } => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                transform.apply_to(&mut x1, &mut y1);
                                transform.apply_to(&mut x2, &mut y2);
                                transform.apply_to(&mut x, &mut y);

                                old_x = x as f32;
                                old_y = y as f32;
                                let old_x1 = x1 as f32;
                                let old_y1 = y1 as f32;
                                let old_x2 = x2 as f32;
                                let old_y2 = y2 as f32;

                                // Generate LineTo command.
                                let x = old_x / width;
                                let y = old_y / height;
                                let x1 = old_x1 / width;
                                let y1 = old_y1 / height;
                                let x2 = old_x2 / width;
                                let y2 = old_y2 / height;
                                println!("CUBE {} {}", x, y);
                                let x = (x * 16384 as f32 + 24576.0) as u16;
                                let y = (y * 16384 as f32 + 24576.0) as u16;
                                let x1 = (x1 * 16384 as f32 + 24576.0) as u16;
                                let y1 = (y1 * 16384 as f32 + 24576.0) as u16;
                                let x2 = (x2 * 16384 as f32 + 24576.0) as u16;
                                let y2 = (y2 * 16384 as f32 + 24576.0) as u16;

                                let i = search_add(&mut pts, (x1,y1));
                                let j = search_add(&mut pts, (x2,y2));
                                let k = search_add(&mut pts, (x,y));

                                ops.push(GraphicOps::Cubic as u8);
                                ops.extend(i.to_be_bytes().iter());
                                ops.extend(j.to_be_bytes().iter());
                                ops.extend(k.to_be_bytes().iter());
                            }
                            PathSegment::ClosePath {abs} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = 0.0;
                                old_y = 0.0;
                                // Shall be implemented as just a line back to last move.
                                // unimplemented!();
                            }
                            a => {
                                println!("WARNING: Path Unknown {:?}", a);
                            }
                        }
                    }
//                    if ops[ops.len() - 1] != GraphicOps::Close as u8 {
                        ops.push(GraphicOps::Close as u8);
//                    }
                }

                // END PATH
            }
            ElementId::Defs => { /* IGNORE */ }
            a => {
                println!("WARNING: Element Unknown \"{}\"", a);
            }
        }
    }

    // Do the encoding.
    let mut rvg = rvg::Rvg::new();

    rvg.block(Block::points2d(pts.as_slice()));
    rvg.block(Block::graphic(ar, bgc, ops.as_slice()));

    rvg.into_vec()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let svg = std::fs::read_to_string(&args[1]).unwrap();

    let data = rvg_from_svg(&svg);

    let fl = std::fs::File::create(format!("{}.rvg", args[1])).unwrap();
    let ref mut bw = std::io::BufWriter::new(fl);
    bw.write_all(data.as_slice()).unwrap();
}
