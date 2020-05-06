//! Convert an SVG into an RVG.

use std::io::Write;
use rvg::{GroupProperty, Graphic, PathOp, Model};
use usvg::{NodeKind, Paint, PathSegment};

pub fn search_add(pts: &mut Vec<f32>, pt: &[f64]) -> u32 {
    let stride = pt.len();
    let pt = (pt[0] as f32, pt[1] as f32);

    for i in (0..pts.len()).step_by(stride) {
        if pt.0 == pts[i] && pt.1 == pts[i + 1] {
            return i as u32;
        }
    }
    pts.push(pt.0);
    pts.push(pt.1);
    return pts.len() as u32 - 1;
}

/// Convert an SVG string into RVG byte data.
fn rvg_from_svg<W: Write>(svg: &str, w: W) {
    let mut group = Vec::new();
    let mut groups = Vec::new();

    // Build a new RVG.
    let mut pts = vec![];

    // Simplify SVG with usvg.
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).unwrap();

    // Render
    let mut iter = tree.root().descendants();

    let (width, height): (f32, f32) = if let Some(node) = iter.next() {
        match *node.borrow() {
            NodeKind::Svg(svg) => (svg.size.width() as f32, svg.size.height() as f32),
            _ => panic!("Not an SVG!"),
        }
    } else {
        panic!("SVG is an empty file!");
    };

    println!("WH: ({} {})", width, height);

    for node in iter {
        match &*node.borrow() {
            NodeKind::Path(path) => {
                let mut properties = Vec::new();
            
                // Fill Color if it exists.
                if let Some(fill) = &path.fill {
                    if let Paint::Color(c) = fill.paint {
                        let alpha = (fill.opacity.value() * 255.0) as u8;
                        properties.push(GroupProperty::FillColorRgba([c.red, c.green, c.blue, alpha]));
                    } else {
                        panic!("Linked paint server not supported!");
                    };
                }

                // Stroke Width & Color
                if let Some(stroke) = &path.stroke {
                    // Color
                    if let Paint::Color(c) = stroke.paint {
                        properties.push(GroupProperty::StrokeColorRgba([c.red, c.green, c.blue, (stroke.opacity.value() * 255.0) as u8]));
                    } else {
                        panic!("Linked paint server not supported!");
                    };
                    
                    properties.push(GroupProperty::StrokeWidth(stroke.width.value() as f32));
                }

                let mut pathops = vec![];

                for subpath in path.data.subpaths() {
                    for segment in subpath.0 {
                        match *segment {
                            PathSegment::MoveTo {x, y} => {
                                let i = search_add(&mut pts, &[x, y]);
                                pathops.push(PathOp::Move(i));
                            }
                            PathSegment::LineTo {x, y} => {
                                let i = search_add(&mut pts, &[x, y]);
                                pathops.push(PathOp::Line(i));
                            }
                            PathSegment::CurveTo {
                                x1,
                                y1,
                                x2,
                                y2,
                                x,
                                y,
                            } => {
                                let i = search_add(&mut pts, &[x1, y1]);
                                let j = search_add(&mut pts, &[x2,y2]);
                                let k = search_add(&mut pts, &[x,y]);
                                pathops.push(PathOp::Cubic(i, j, k));
                            }
                            PathSegment::ClosePath {} => {
                                pathops.push(PathOp::Close());
                            }
                        }
                    }
                }

                groups.push((group.len() as u32, properties));
                group.push(pathops);

                // END PATH
            }
            a => {
                println!("WARNING: Element Unknown \"{:?}\"", a);
            }
        }
    }

    // Do the encoding.
    let graphic = Graphic {
        attributes: Vec::new(), // Don't use any attributes
        vertex_list: Vec::new(),
        group,
        models: vec![Model {
            width, height, groups, frames: vec![rvg::Frame {
                transforms: Vec::new(),
                delay: 0,
                animation: rvg::Animation::Done,
            }],
        }],
        bitmaps: Vec::new(),
    };
    graphic.save(w).unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let svg = std::fs::read_to_string(&args[1]).unwrap();

    let fl = std::fs::File::create(format!("{}.rvg", args[1])).unwrap();
    let ref mut bw = std::io::BufWriter::new(fl);
    rvg_from_svg(&svg, bw);
}
