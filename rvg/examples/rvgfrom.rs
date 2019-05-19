//! Convert an SVG into an RVG.

use rvg;
use usvg;
use usvg::svgdom::WriteBuffer;
use usvg::svgdom::{
    AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathSegment,
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
    println!("SVG: {}", svg);

    // Render
    let doc = Document::from_str(&svg).unwrap();
    let mut iter = doc.root().descendants().svg();

    let (width, height) = if let Some((id, node)) = iter.next() {
        if id == ElementId::Svg {
            let attrs = node.attributes();
            let width;
            let height;

            println!("{:?}", attrs);

            if let Some(&AttributeValue::Length(ref v)) =
                attrs.get_value(AttributeId::Width)
            {
                width = v.num as u32;
            } else {
                panic!("Width unspecified!");
            }
            if let Some(&AttributeValue::Length(ref v)) =
                attrs.get_value(AttributeId::Height)
            {
                height = v.num as u32;
            } else {
                panic!("Height unspecified!");
            }

            (width, height)
        } else {
            panic!("Not an SVG!");
        }
    } else {
        panic!("SVG is an empty file!");
    };

    let ar = (65536.0 * height as f64 / width as f64) as u32;
    let bgc = 0u64; // TODO

    for (id, node) in iter {
        match id {
            ElementId::Path => {
                let mut old_x = 0.0f32;
                let mut old_y = 0.0f32;

                let attrs = node.attributes();

                if let Some(&AttributeValue::Color(ref c)) =
                    attrs.get_value(AttributeId::Fill)
                {
                    let red = (c.red as u16) * 256;
                    let green = (c.green as u16) * 256;
                    let blue = (c.blue as u16) * 256;
                    let alpha = 65535u16;

                    ops.push(GraphicOps::Solid as u8);
                    ops.extend(red.to_be_bytes().iter());
                    ops.extend(green.to_be_bytes().iter());
                    ops.extend(blue.to_be_bytes().iter());
                    ops.extend(alpha.to_be_bytes().iter());
                }

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

                // TODO: Stroke Width

                if let Some(&AttributeValue::Path(ref path)) =
                    attrs.get_value(AttributeId::D)
                {
                    for seg in path.iter() {
                        println!("{:?}", seg);
                        match seg {
                            PathSegment::MoveTo {abs, x, y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = *x as f32;
                                old_y = *y as f32;

                                // Generate MoveTo command.
                                let x = ((old_x / width as f32) * std::u16::MAX as f32) as u16;
                                let y = ((old_y / height as f32) * std::u16::MAX as f32) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Move as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::LineTo {abs, x, y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = *x as f32;
                                old_y = *y as f32;

                                // Generate LineTo command.
                                let x = ((old_x / width as f32) * std::u16::MAX as f32) as u16;
                                let y = ((old_y / height as f32) * std::u16::MAX as f32) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::HorizontalLineTo {abs, x} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = *x as f32;

                                // Generate LineTo command.
                                let x = ((old_x / width as f32) * std::u16::MAX as f32) as u16;
                                let y = ((old_y / height as f32) * std::u16::MAX as f32) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::VerticalLineTo {abs, y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_y = *y as f32;

                                // Generate LineTo command.
                                let x = ((old_x / width as f32) * std::u16::MAX as f32) as u16;
                                let y = ((old_y / height as f32) * std::u16::MAX as f32) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::Quadratic {abs, x1, y1, x, y} => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = *x as f32;
                                old_y = *y as f32;
                                let x = ((old_x / width as f32) * std::u16::MAX as f32) as u16;
                                let y = ((old_y / height as f32) * std::u16::MAX as f32) as u16;
                                let x1 = ((*x1 as f32 / width as f32) * std::u16::MAX as f32) as u16;
                                let y1 = ((*y1 as f32 / height as f32) * std::u16::MAX as f32) as u16;
                                let i = search_add(&mut pts, (x1,y1));
                                let j = search_add(&mut pts, (x,y));

                                ops.push(GraphicOps::Quad as u8);
                                ops.extend(i.to_be_bytes().iter());
                                ops.extend(j.to_be_bytes().iter());
                            }
                            PathSegment::CurveTo {
                                abs,
                                x1,
                                y1,
                                x2,
                                y2,
                                x,
                                y,
                            } => {
                                if *abs == false {
                                    panic!("Relative not support.");
                                }
                                old_x = *x as f32;
                                old_y = *y as f32;
                                let x = ((old_x / width as f32) * std::u16::MAX as f32) as u16;
                                let y = ((old_y / height as f32) * std::u16::MAX as f32) as u16;
                                let x1 = ((*x1 as f32 / width as f32) * std::u16::MAX as f32) as u16;
                                let y1 = ((*y1 as f32 / height as f32) * std::u16::MAX as f32) as u16;
                                let x2 = ((*x2 as f32 / width as f32) * std::u16::MAX as f32) as u16;
                                let y2 = ((*y2 as f32 / height as f32) * std::u16::MAX as f32) as u16;
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
                                // Shall be implemented as just a line back to last move.
                                unimplemented!();
                            }
                            a => {
                                println!("WARNING: Path Unknown {:?}", a);
                            }
                        }
                    }
                }

                // END PATH
            }
            a => {
                println!("WARNING: Element Unknown {}", a);
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
