use footile::{PathBuilder, Plotter};
use pix::chan::Ch8;
use pix::el::Pixel;
use pix::ops::SrcOver;
use pix::rgb::{Rgba8p, SRgba8};
use pix::{Raster, Region};
use std::fs::File;
use std::io::Read;
use rvg::Graphic;

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

pub struct ScaledRegion {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ScaledRegion {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        ScaledRegion {
            x, y, width, height,
        }
    }
}

impl From<()> for ScaledRegion {
    fn from(_rhs: ()) -> Self {
        ScaledRegion::new(0.0, 0.0, f32::INFINITY, f32::INFINITY)
    }
}

impl From<(f32, f32, f32, f32)> for ScaledRegion {
    fn from(rhs: (f32, f32, f32, f32)) -> Self {
        ScaledRegion::new(rhs.0, rhs.1, rhs.2, rhs.3)
    }
}

pub fn render<P, R>(raster: &mut Raster<P>, graphic: &Graphic, region: R)
    where
        R: Into<ScaledRegion>,
        P: Pixel<Alpha = pix::chan::Premultiplied, Gamma = pix::chan::Linear>,
        P::Chan: From<Ch8>,
{
    let (xs, ys, dst_region): (_, _, Region) = {
        let r: ScaledRegion = region.into();
        if r.width.is_infinite() || r.height.is_infinite() {
            (1.0, 1.0, (r.x as i32, r.y as i32, i32::MAX as u32, i32::MAX as u32).into())
        } else {
            (r.width / raster.width() as f32, r.height / raster.height() as f32, (r.x as i32, r.y as i32, r.width as u32, r.height as u32).into())
        }
    };

    // We can't render these types of RVGs with footile yet.
    assert!(graphic.attributes.is_empty());
    assert!(graphic.bitmaps.is_empty());
    assert!(graphic.models.len() == 1);

    let model = &graphic.models[0];
    let mut p = Plotter::new(model.width as u32, model.height as u32);

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
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    pathbuilder = pathbuilder.move_to(x, y);
                }
                rvg::PathOp::Line(a) => {
                    let (x, y) = (
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    pathbuilder = pathbuilder.line_to(x, y);
                }
                rvg::PathOp::Quad(a, b) => {
                    let (bx, by) = (
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    let (cx, cy) = (
                        graphic.vertex_list[b as usize * 2] * xs,
                        graphic.vertex_list[b as usize * 2 + 1] * ys,
                    );
                    pathbuilder = pathbuilder.quad_to(bx, by, cx, cy);
                }
                rvg::PathOp::Cubic(a, b, c) => {
                    let (bx, by) = (
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    let (cx, cy) = (
                        graphic.vertex_list[b as usize * 2] * xs,
                        graphic.vertex_list[b as usize * 2 + 1] * ys,
                    );
                    let (dx, dy) = (
                        graphic.vertex_list[c as usize * 2] * xs,
                        graphic.vertex_list[c as usize * 2 + 1] * ys,
                    );
                    pathbuilder = pathbuilder.cubic_to(bx, by, cx, cy, dx, dy);
                }
            }
        }

        let path = pathbuilder.build();

        if fill_color.alpha() != Ch8::new(0u8) {
            let fill = p.fill(&path, footile::FillRule::NonZero);
        
            let temp_raster: Raster<pix::el::Pix1<P::Chan, pix::matte::Matte, pix::chan::Premultiplied, pix::chan::Linear>> = Raster::with_raster(fill);

            raster.composite_matte(
                dst_region,
                &temp_raster,
                (),
                fill_color.convert(),
                SrcOver,
            );
        }
        if stroke_color.alpha() != Ch8::new(0u8) {
            let stroke = p.stroke(&path);
        
            let temp_raster: Raster<pix::el::Pix1<P::Chan, pix::matte::Matte, pix::chan::Premultiplied, pix::chan::Linear>> = Raster::with_raster(stroke);
        
            raster.composite_matte(
                dst_region,
                &temp_raster,
                (),
                stroke_color.convert(),
                SrcOver,
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let mut rvg = Vec::new();
    let mut f = File::open(&args[1]).unwrap();
    f.read_to_end(&mut rvg).unwrap();

    let graphic = Graphic::load(std::io::Cursor::new(&rvg)).unwrap();

    let model = &graphic.models[0];
    let mut raster = Raster::<Rgba8p>::with_clear(model.width as u32, model.height as u32);

    /*rvg::*/render(&mut raster, &graphic, ());

    write_png(Raster::with_raster(&raster), &format!("{}.png", args[1]))
        .unwrap();
}
