//! Convert an SVG into an RVG.

use rvg;
use usvg;
use usvg::svgdom::WriteBuffer;
use usvg::svgdom::{
    AttributeId, AttributeValue, Document, ElementId, FilterSvg, PathSegment,
};

fn rvg_from_svg() -> Vec<u8> {
}

fn main() {
    // Build a new RVG.
    let mut rvg = rvg::Rvg::new();
    let mut pts = vec![];

    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let svg = std::fs::read_to_string(&args[1]).unwrap();

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

    let (rvg_axis, rvg_ratio) = if width != height {
        (0, false)
    } else if width > height {
        ((65536.0 * height as f64 / width as f64) as u16, false)
    } else {
        ((65536.0 * width as f64 / height as f64) as u16, false)
    };

    for (id, node) in iter {
        match id {
            ElementId::Path => {
                let mut pathbuilder = PathBuilder::new();
                let mut old_x = 0.0f32;
                let mut old_y = 0.0f32;

                let attrs = node.attributes();
                if let Some(&AttributeValue::Path(ref path)) =
                    attrs.get_value(AttributeId::D)
                {
                    for seg in path.iter() {
                        println!("{:?}", seg);
                        match seg {
                            PathSegment::MoveTo { abs, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.move_to(*x as f32, *y as f32);
                                old_x = *x as f32;
                                old_y = *y as f32;
                            }
                            PathSegment::LineTo { abs, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.line_to(*x as f32, *y as f32);
                                old_x = *x as f32;
                                old_y = *y as f32;
                            }
                            PathSegment::HorizontalLineTo { abs, x } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.line_to(*x as f32, old_y);
                                old_x = *x as f32;
                            }
                            PathSegment::VerticalLineTo { abs, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.line_to(old_x, *y as f32);
                                old_y = *y as f32;
                            }
                            PathSegment::Quadratic { abs, x1, y1, x, y } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.quad_to(
                                    *x1 as f32, *y1 as f32, *x as f32, *y as f32,
                                );
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
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.cubic_to(
                                    *x1 as f32, *y1 as f32, *x2 as f32, *y2 as f32,
                                    *x as f32, *y as f32,
                                ); // TODO: verify order.
                            }
                            PathSegment::ClosePath { abs } => {
                                if *abs {
                                    pathbuilder = pathbuilder.absolute();
                                } else {
                                    pathbuilder = pathbuilder.relative();
                                }
                                pathbuilder = pathbuilder.close();
                            }
                            a => {
                                println!("WARNING: Path Unknown {:?}", a);
                            }
                        }
                    }
                }

                let path = pathbuilder.build();

                if let Some(&AttributeValue::Color(ref c)) =
                    attrs.get_value(AttributeId::Fill)
                {
                    r.over(
                        p.fill(&path, footile::FillRule::NonZero),
                        Rgba8::rgb(c.red, c.green, c.blue),
                    );
                }

                if let Some(&AttributeValue::Color(ref c)) =
                    attrs.get_value(AttributeId::Stroke)
                {
                    r.over(
                        p.fill(&path, footile::FillRule::NonZero),
                        Rgba8::rgb(c.red, c.green, c.blue),
                    );
                }
                // END PATH
            }
            a => {
                println!("WARNING: Element Unknown {}", a);
            }
        }
    }

    // Return pixels
    (width, height, r.as_slice().to_vec())
}
