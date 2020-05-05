//! Convert an SVG into an RVG.

use std::io::Write;
use rvg::{Block, GraphicOps};
use usvg::{NodeKind, Paint, PathSegment};

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

    // Render
    let mut iter = tree.root().descendants();

    let (width, height): (f64, f64) = if let Some(node) = iter.next() {
        match *node.borrow() {
            NodeKind::Svg(svg) => {
                (svg.size.width(), svg.size.height())
            }
            _ => {
                panic!("Not an SVG!");
            }
        }
    } else {
        panic!("SVG is an empty file!");
    };

    println!("WH: ({} {})", width, height);

    let ar = (65536.0 * height / width) as u32;
    let bgc = 0u64; // TODO

    for node in iter {
        match &*node.borrow() {
            NodeKind::Path(path) => {
                // Fill Color
                let (red, green, blue, alpha) = if let Some(fill) = &path.fill {
                    let (red, green, blue) = if let Paint::Color(c) = fill.paint {
                        ((c.red as f64) / 255.0,
                        (c.green as f64) / 255.0,
                        (c.blue as f64) / 255.0)
                    } else {
                        (0.0, 0.0, 0.0)
                    };
                    let alpha = fill.opacity.value();
                    ops.push(GraphicOps::Solid as u8);
                    (red, green, blue, alpha)
                } else {
                    (0.0, 0.0, 0.0, 0.0)
                };
                ops.extend(((red * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                ops.extend(((green * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                ops.extend(((blue * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                ops.extend(((alpha * (u16::MAX as f64)) as u16).to_be_bytes().iter());

                // Stroke Width & Color
                if let Some(stroke) = &path.stroke {
                    // Color
                    let (red, green, blue) = if let Paint::Color(c) = stroke.paint {
                        ((c.red as f64) / 255.0,
                        (c.green as f64) / 255.0,
                        (c.blue as f64) / 255.0)
                    } else {
                        (0.0, 0.0, 0.0)
                    };
                    let alpha = stroke.opacity.value();
                    ops.push(GraphicOps::Stroke as u8);
                    ops.extend(((red * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                    ops.extend(((green * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                    ops.extend(((blue * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                    ops.extend(((alpha * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                    
                    // Width
                    let stroke_width = stroke.width.value() / width;
                    ops.push(GraphicOps::Width as u8);
                    ops.extend(((stroke_width * (u16::MAX as f64)) as u16).to_be_bytes().iter());
                }

                for subpath in path.data.subpaths() {
                    for segment in subpath.0 {
                        match segment {
                            PathSegment::MoveTo {x, y} => {
                                // Generate MoveTo command.
                                println!("MOVE {} {}", x, y);
                                let x = (x * 16384 as f64 + 24576.0) as u16;
                                let y = (y * 16384 as f64 + 24576.0) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Move as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::LineTo {x, y} => {
                                println!("LINE {} {}", x, y);
                                let x = (x * 16384 as f64 + 24576.0) as u16;
                                let y = (y * 16384 as f64 + 24576.0) as u16;
                                let i = search_add(&mut pts, (x,y));
                                ops.push(GraphicOps::Line as u8);
                                ops.extend(i.to_be_bytes().iter());
                            }
                            PathSegment::CurveTo {
                                x1,
                                y1,
                                x2,
                                y2,
                                x,
                                y,
                            } => {
                                let x = (x * 16384 as f64 + 24576.0) as u16;
                                let y = (y * 16384 as f64 + 24576.0) as u16;
                                let x1 = (x1 * 16384 as f64 + 24576.0) as u16;
                                let y1 = (y1 * 16384 as f64 + 24576.0) as u16;
                                let x2 = (x2 * 16384 as f64 + 24576.0) as u16;
                                let y2 = (y2 * 16384 as f64 + 24576.0) as u16;

                                let i = search_add(&mut pts, (x1,y1));
                                let j = search_add(&mut pts, (x2,y2));
                                let k = search_add(&mut pts, (x,y));

                                ops.push(GraphicOps::Cubic as u8);
                                ops.extend(i.to_be_bytes().iter());
                                ops.extend(j.to_be_bytes().iter());
                                ops.extend(k.to_be_bytes().iter());
                            }
                            PathSegment::ClosePath {} => { }
                        }
                    }
                    ops.push(GraphicOps::Close as u8);
                }

                // END PATH
            }
            a => {
                println!("WARNING: Element Unknown \"{:?}\"", a);
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
