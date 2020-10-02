use crate::Graphic;
use footile::{Path2D, Plotter};
use pix::{
    chan::Ch8, el::Pixel, matte::Matte8, ops::SrcOver, rgb::SRgba8, Raster,
    Region,
};

pub struct ScaledRegion {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ScaledRegion {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        ScaledRegion {
            x,
            y,
            width,
            height,
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
            (
                1.0,
                1.0,
                (r.x as i32, r.y as i32, i32::MAX as u32, i32::MAX as u32)
                    .into(),
            )
        } else {
            (
                r.width / raster.width() as f32,
                r.height / raster.height() as f32,
                (r.x as i32, r.y as i32, r.width as u32, r.height as u32)
                    .into(),
            )
        }
    };

    // We can't render these types of RVGs with footile yet.
    assert!(graphic.attributes.is_empty());
    assert!(graphic.bitmaps.is_empty());
    assert!(graphic.models.len() == 1);

    let model = &graphic.models[0];
    let mut p = Plotter::new(Raster::with_clear(
        model.width as u32,
        model.height as u32,
    ));

    for (group_id, group_props) in &model.groups {
        let mut path = Path2D::default().absolute();

        println!("Building Path….");

        let mut fill_color = SRgba8::new(0, 0, 0, 0);
        let mut stroke_color = SRgba8::new(0, 0, 0, 0);
        for prop in group_props {
            use crate::GroupProperty::*;
            match *prop {
                FillColorRgba([red, green, blue, alpha]) => {
                    fill_color = SRgba8::new(red, green, blue, alpha)
                }
                StrokeColorRgba([red, green, blue, alpha]) => {
                    stroke_color = SRgba8::new(red, green, blue, alpha)
                }
                StrokeWidth(w) => path = path.pen_width(w),
                JoinStyle(_) => unimplemented!(),
                FillRule(_) => unimplemented!(),
                GlyphID(_) => unimplemented!(),
                BitmapPattern(_) => unimplemented!(),
                GroupPattern(_) => unimplemented!(),
            }
        }

        for pathop in &graphic.group[*group_id as usize] {
            match *pathop {
                crate::PathOp::Close() => path = path.close(),
                crate::PathOp::Move(a) => {
                    let (x, y) = (
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    path = path.move_to(x, y);
                }
                crate::PathOp::Line(a) => {
                    let (x, y) = (
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    path = path.line_to(x, y);
                }
                crate::PathOp::Quad(a, b) => {
                    let (bx, by) = (
                        graphic.vertex_list[a as usize * 2] * xs,
                        graphic.vertex_list[a as usize * 2 + 1] * ys,
                    );
                    let (cx, cy) = (
                        graphic.vertex_list[b as usize * 2] * xs,
                        graphic.vertex_list[b as usize * 2 + 1] * ys,
                    );
                    path = path.quad_to(bx, by, cx, cy);
                }
                crate::PathOp::Cubic(a, b, c) => {
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
                    path = path.cubic_to(bx, by, cx, cy, dx, dy);
                }
            }
        }

        let path = path.finish();

        if fill_color.alpha() != Ch8::new(0u8) {
            let fill =
                p.fill(footile::FillRule::NonZero, &path, Matte8::new(255));

            let temp_raster: Raster<
                pix::el::Pix1<
                    P::Chan,
                    pix::matte::Matte,
                    pix::chan::Premultiplied,
                    pix::chan::Linear,
                >,
            > = Raster::with_raster(fill);

            raster.composite_matte(
                dst_region,
                &temp_raster,
                (),
                fill_color.convert(),
                SrcOver,
            );
            let mut pr = p.raster();
            pr.clear();
            p = Plotter::new(pr);
        }
        if stroke_color.alpha() != Ch8::new(0u8) {
            let stroke = p.stroke(&path, Matte8::new(255));

            let temp_raster: Raster<
                pix::el::Pix1<
                    P::Chan,
                    pix::matte::Matte,
                    pix::chan::Premultiplied,
                    pix::chan::Linear,
                >,
            > = Raster::with_raster(stroke);

            raster.composite_matte(
                dst_region,
                &temp_raster,
                (),
                stroke_color.convert(),
                SrcOver,
            );
            let mut pr = p.raster();
            pr.clear();
            p = Plotter::new(pr);
        }
    }
}
