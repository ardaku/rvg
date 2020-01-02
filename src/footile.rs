use pix::{Rgba8, Raster, RasterBuilder, Alpha};
use footile::{PathBuilder, Plotter};
use crate::rvg::*;

/// Render a graphic.
pub fn graphic_from_rvg(rvg: &[u8]) -> (u32, u32, Vec<Rgba8>) {
    let mut pathbuilder = PathBuilder::new().absolute();

    let rvg = Rvg::from_slice(rvg);
    let mut pts = vec![];

    for block in rvg.iter() {
        let (block_type, data) = block.read().unwrap();

        match block_type {
            BlockTypes::Points2d => for i in (1..data.len()).step_by(4) {
                let mut x = u16::from_be_bytes(clone_into_array(&data[i..i+2])) as f32;
                let mut y = u16::from_be_bytes(clone_into_array(&data[i+2..i+4])) as f32;
                x -= 24576.0;
                y -= 24576.0;
                x /= 16384.0;
                y /= 16384.0;
                pts.push((x, y, 0.0));
            }
            BlockTypes::Points3d => for i in (1..data.len()).step_by(6) {
                let mut x = u16::from_be_bytes(clone_into_array(&data[i..i+2])) as f32;
                let mut y = u16::from_be_bytes(clone_into_array(&data[i+2..i+4])) as f32;
                let mut z = u16::from_be_bytes(clone_into_array(&data[i+4..i+6])) as f32;
                x -= 24576.0;
                y -= 24576.0;
                z -= 24576.0;
                x /= 16384.0;
                y /= 16384.0;
                z /= 16384.0;
                pts.push((x, y, z));
            }
            BlockTypes::Graphic => {
                let ar = u32::from_be_bytes(clone_into_array(&data[1..5]));
                let _bgc = u64::from_be_bytes(clone_into_array(&data[5..13])); // TODO
                let mut fill_color = Rgba8::with_alpha(0, 0, 0, 0);
                let mut pen_color = Rgba8::with_alpha(0, 0, 0, 0);
                let mut pen_width = 0;
                let width = 512;
                let height = (512.0 * (ar as f32 / 65536.0)) as u32;
                let mut p = Plotter::new(width, height);
                let mut r = RasterBuilder::new().with_clear(p.width(), p.height());

                let mut i = 13;
                loop {
                    if i >= data.len() { break };
                    let opcode = data[i];
                    i += 1;
                    match opcode {
                        0x10 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.move_to(x, y);
                            i += 2;
                        } // Move
                        0x11 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.line_to(x, y);
                            i += 2;
                        } // Line
                        0x12 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let idy = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let (x1, y1, _z1) = pts[idx as usize];
                            let (x, y, _z) = pts[idy as usize];
                            let (x1, y1) = (x1 as f32 * width as f32, y1 as f32 * height as f32);
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.quad_to(x1, y1, x, y);
                            i += 4;
                        } // Quad
                        0x13 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let idy = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let idz = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let (x1, y1, _z1) = pts[idx as usize];
                            let (x2, y2, _z2) = pts[idy as usize];
                            let (x, y, _z) = pts[idz as usize];
                            let (x1, y1) = (x1 as f32 * width as f32, y1 as f32 * height as f32);
                            let (x2, y2) = (x2 as f32 * width as f32, y2 as f32 * height as f32);
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.cubic_to(x1, y1, x2, y2, x, y);
                            i += 6;
                        } // Cubic
                        0x14 => {
                            unimplemented!();
                        } // Arc
                        0x1F => {
                            let mut path_builder = PathBuilder::new().absolute();
                            std::mem::swap(&mut pathbuilder, &mut path_builder);
                            let path = path_builder.build();
    
                            let alpha: u8 = fill_color.alpha().value().into(); 
                            if alpha != 0 { // Not transparent
                                pixops::raster_over(
                                    &mut r,
                                    p.fill(&path, footile::FillRule::NonZero),
                                    fill_color,
                                    0,
                                    0,
                                );
                                p.clear_mask();
                            }
                            let alpha: u8 = pen_color.alpha().value().into();
                            if alpha != 0 && pen_width != 0 { // Not transparent
                                pixops::raster_over(
                                    &mut r,
                                    p.stroke(&path),
                                    pen_color,
                                    0,
                                    0,
                                );
                                p.clear_mask();
                            }
                            pen_width = 0;
                        } // Close

                        // Solid
                        0x20 => {
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            fill_color = Rgba8::with_alpha(r, g, b, a);

                            i += 8;
                        } // Fill with 1 color (before each vertex)
                        // Bitmap
                        0x21 => { } // Fill with bitmap - stretch (before Move)
                        // Tile
                        0x22 => { } // Fill tiled with pattern - Vector Graphics
                        // Pattern
                        0x23 => { } // Fill tiled with pattern - Bitmap
                        // Stroke
                        0x24 => {
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            pen_color = Rgba8::with_alpha(r, g, b, a);

                            i += 8;
                        } // Change stroke color
                        // Width
                        0x25 => {
                            let w = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            pen_width = w;
                            let w = w as f32 / std::u16::MAX as f32;
                            let w = w * width as f32;
                            pathbuilder = pathbuilder.pen_width(w);
                            i += 2;
                        } // Change stroke width
                        // Dashed
                        0x26 => { } // Change dash width (0=100% by default)

                        // JoinMiter
                        0x30 => { } // `value` for amount
                        // JoinBevel
                        0x31 => { }
                        // JoinRound
                        0x32 => { }

                        x => panic!("Parse error {:x}!", x)
                    }
                }
                return (width, height, r.as_slice().to_vec());
            }
            _ => panic!("Unsupported!"),
        }
    }
    panic!("There was no graphic in this file!");
}

/// Render a graphic.
pub fn render_from_rvg(rvg: &[u8], raster: &mut Raster<Rgba8>, x: u16, y: u16, graphic_width: u16) {
    let mut pathbuilder = PathBuilder::new().absolute();
    let offset_x = x as f32;
    let offset_y = y as f32;

    let rvg = Rvg::from_slice(rvg);
    let mut pts = vec![];

    for block in rvg.iter() {
        let (block_type, data) = block.read().unwrap();

        match block_type {
            BlockTypes::Points2d => for i in (1..data.len()).step_by(4) {
                let mut x = u16::from_be_bytes(clone_into_array(&data[i..i+2])) as f32;
                let mut y = u16::from_be_bytes(clone_into_array(&data[i+2..i+4])) as f32;
                x -= 24576.0;
                y -= 24576.0;
                x /= 16384.0;
                y /= 16384.0;
                pts.push((x, y, 0.0));
            }
            BlockTypes::Points3d => for i in (1..data.len()).step_by(6) {
                let mut x = u16::from_be_bytes(clone_into_array(&data[i..i+2])) as f32;
                let mut y = u16::from_be_bytes(clone_into_array(&data[i+2..i+4])) as f32;
                let mut z = u16::from_be_bytes(clone_into_array(&data[i+4..i+6])) as f32;
                x -= 24576.0;
                y -= 24576.0;
                z -= 24576.0;
                x /= 16384.0;
                y /= 16384.0;
                z /= 16384.0;
                pts.push((x, y, z));
            }
            BlockTypes::Graphic => {
                let ar = u32::from_be_bytes(clone_into_array(&data[1..5]));
                let _bgc = u64::from_be_bytes(clone_into_array(&data[5..13])); // TODO
                let mut fill_color = Rgba8::with_alpha(0, 0, 0, 0);
                let mut pen_color = Rgba8::with_alpha(0, 0, 0, 0);
                let mut pen_width = 0;
                let width = raster.width();
                let height = raster.height();
                let mut p = Plotter::new(width, height);

                let width = graphic_width;
                let height = (graphic_width as f32 * (ar as f32 / 65536.0)) as u32;

                let mut i = 13;
                loop {
                    if i >= data.len() { break };
                    let opcode = data[i];
                    i += 1;
                    match opcode {
                        0x10 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.move_to(x + offset_x, y + offset_y);
                            i += 2;
                        } // Move
                        0x11 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.line_to(x + offset_x, y + offset_y);
                            i += 2;
                        } // Line
                        0x12 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let idy = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let (x1, y1, _z1) = pts[idx as usize];
                            let (x, y, _z) = pts[idy as usize];
                            let (x1, y1) = (x1 as f32 * width as f32, y1 as f32 * height as f32);
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.quad_to(x1 + offset_x, y1 + offset_y, x + offset_x, y + offset_y);
                            i += 4;
                        } // Quad
                        0x13 => {
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let idy = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let idz = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let (x1, y1, _z1) = pts[idx as usize];
                            let (x2, y2, _z2) = pts[idy as usize];
                            let (x, y, _z) = pts[idz as usize];
                            let (x1, y1) = (x1 as f32 * width as f32, y1 as f32 * height as f32);
                            let (x2, y2) = (x2 as f32 * width as f32, y2 as f32 * height as f32);
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.cubic_to(x1 + offset_x, y1 + offset_y, x2 + offset_x, y2 + offset_y, x + offset_x, y + offset_y);
                            i += 6;
                        } // Cubic
                        0x14 => {
                            unimplemented!();
                        } // Arc
                        0x1F => {
                            let mut path_builder = PathBuilder::new().absolute();
                            std::mem::swap(&mut pathbuilder, &mut path_builder);
                            let path = path_builder.build();

                            let alpha: u8 = fill_color.alpha().value().into();
                            if alpha != 0 { // Not transparent
                                pixops::raster_over(
                                    raster,
                                    p.fill(&path, footile::FillRule::NonZero),
                                    fill_color,
                                    0,
                                    0,
                                );
                                p.clear_mask();
                            }
                            let alpha: u8 = pen_color.alpha().value().into();
                            if alpha != 0 && pen_width != 0 { // Not transparent
                                pixops::raster_over(
                                    raster,
                                    p.stroke(&path),
                                    pen_color,
                                    0,
                                    0,
                                );
                                p.clear_mask();
                            }
                            pen_width = 0;
                        } // Close

                        // Solid
                        0x20 => {
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            fill_color = Rgba8::with_alpha(r, g, b, a);

                            i += 8;
                        } // Fill with 1 color (before each vertex)
                        // Bitmap
                        0x21 => { } // Fill with bitmap - stretch (before Move)
                        // Tile
                        0x22 => { } // Fill tiled with pattern - Vector Graphics
                        // Pattern
                        0x23 => { } // Fill tiled with pattern - Bitmap
                        // Stroke
                        0x24 => {
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            pen_color = Rgba8::with_alpha(r, g, b, a);

                            i += 8;
                        } // Change stroke color
                        // Width
                        0x25 => {
                            let w = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            pen_width = w;
                            let w = w as f32 / std::u16::MAX as f32;
                            let w = w * width as f32;
                            pathbuilder = pathbuilder.pen_width(w);
                            i += 2;
                        } // Change stroke width
                        // Dashed
                        0x26 => { } // Change dash width (0=100% by default)

                        // JoinMiter
                        0x30 => { } // `value` for amount
                        // JoinBevel
                        0x31 => { }
                        // JoinRound
                        0x32 => { }

                        x => panic!("Parse error {:x}!", x)
                    }
                }
                return;
            }
            _ => panic!("Unsupported!"),
        }
    }
    panic!("There was no graphic in this file!");
}
