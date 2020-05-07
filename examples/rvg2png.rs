use footile::{PathBuilder, Plotter};
use pix::chan::Ch8;
use pix::el::Pixel;
use pix::ops::SrcOver;
use pix::rgb::{Rgba8p, SRgba8};
use pix::Raster;
use std::fs::File;
use std::io::Read;

pub fn write_png(
    raster: Raster<SRgba8>,
    filename: &str,
) -> std::io::Result<()> {
    let fl = std::fs::File::create(filename)?;
    let ref mut bw = std::io::BufWriter::new(fl);
    let mut enc = png_pong::FrameEncoder::new(bw);
    enc.still(&raster).unwrap();
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let mut rvg = Vec::new();
    let mut f = File::open(&args[1]).unwrap();
    f.read_to_end(&mut rvg).unwrap();

    let graphic = rvg::Graphic::load(std::io::Cursor::new(&rvg)).unwrap();

    // We can't render these types of RVGs with footile yet.
    assert!(graphic.attributes.is_empty());
    assert!(graphic.bitmaps.is_empty());
    assert!(graphic.models.len() == 1);

    let model = &graphic.models[0];
    let mut p = Plotter::new(model.width as u32, model.height as u32);
    let mut raster = Raster::<Rgba8p>::with_clear(p.width(), p.height());

    for (group_id, group_props) in &model.groups {
        let mut pathbuilder = PathBuilder::new().absolute();

        println!("Building Path….");

        let mut fill_color = SRgba8::new(0, 0, 0, 0);
        let mut stroke_color = SRgba8::new(0, 0, 0, 0);
        for prop in group_props {
            use rvg::GroupProperty::*;
            match *prop {
                FillColorRgba([r, g, b, a]) => {
                    fill_color = SRgba8::new(r, g, b, a)
                }
                StrokeColorRgba([r, g, b, a]) => {
                    stroke_color = SRgba8::new(r, g, b, a)
                }
                StrokeWidth(w) => pathbuilder = pathbuilder.pen_width(w),
                JoinStyle(_) => unimplemented!(),
                FillRule(_) => unimplemented!(),
                GlyphID(_) => unimplemented!(),
                BitmapPattern(_) => unimplemented!(),
                GroupPattern(_) => unimplemented!(),
            }
        }

        for pathop in &graphic.group[*group_id as usize] {
            match *pathop {
                rvg::PathOp::Close() => pathbuilder = pathbuilder.close(),
                rvg::PathOp::Move(a) => {
                    let (x, y) = (
                        graphic.vertex_list[a as usize * 2],
                        graphic.vertex_list[a as usize * 2 + 1],
                    );
                    pathbuilder = pathbuilder.move_to(x, y);
                }
                rvg::PathOp::Line(a) => {
                    let (x, y) = (
                        graphic.vertex_list[a as usize * 2],
                        graphic.vertex_list[a as usize * 2 + 1],
                    );
                    pathbuilder = pathbuilder.line_to(x, y);
                }
                rvg::PathOp::Quad(a, b) => {
                    let (bx, by) = (
                        graphic.vertex_list[a as usize * 2],
                        graphic.vertex_list[a as usize * 2 + 1],
                    );
                    let (cx, cy) = (
                        graphic.vertex_list[b as usize * 2],
                        graphic.vertex_list[b as usize * 2 + 1],
                    );
                    pathbuilder = pathbuilder.quad_to(bx, by, cx, cy);
                }
                rvg::PathOp::Cubic(a, b, c) => {
                    let (bx, by) = (
                        graphic.vertex_list[a as usize * 2],
                        graphic.vertex_list[a as usize * 2 + 1],
                    );
                    let (cx, cy) = (
                        graphic.vertex_list[b as usize * 2],
                        graphic.vertex_list[b as usize * 2 + 1],
                    );
                    let (dx, dy) = (
                        graphic.vertex_list[c as usize * 2],
                        graphic.vertex_list[c as usize * 2 + 1],
                    );
                    pathbuilder = pathbuilder.cubic_to(bx, by, cx, cy, dx, dy);
                }
            }
        }

        let path = pathbuilder.build();

        if fill_color.alpha() != Ch8::new(0u8) {
            raster.composite_matte(
                (),
                p.fill(&path, footile::FillRule::NonZero),
                (),
                fill_color.convert(),
                SrcOver,
            );
        }
        if stroke_color.alpha() != Ch8::new(0u8) {
            raster.composite_matte(
                (),
                p.stroke(&path),
                (),
                stroke_color.convert(),
                SrcOver,
            );
        }
    }

    //     rvg::render_from_rvg(rvg.as_slice(), &mut raster, 0, 0, 512);

    write_png(Raster::with_raster(&raster), &format!("{}.png", args[1]))
        .unwrap();
}
