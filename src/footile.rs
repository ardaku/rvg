/*pub use footile::PathOp;

use pix::{rgb::Rgba8p, el::Pixel, Raster, ops::SrcOver};
use footile::{PathBuilder, Plotter};
use crate::rvg::*;

/// Render a graphic.
pub fn render_from_rvg(rvg: &[u8], raster: &mut Raster<Rgba8p>, x: u16, y: u16, graphic_width: u16) {
    let mut pathbuilder = PathBuilder::new().absolute();
    let offset_x = x as f32;
    let offset_y = y as f32;

    let rvg = Rvg::from_slice(rvg);
    let mut pts = vec![];

    println!("RENDING");

    for (byte_count, block) in rvg.iter().enumerate() {
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
                println!("Found Graphic!");
            
                let ar = u32::from_be_bytes(clone_into_array(&data[1..5]));
                let _bgc = u64::from_be_bytes(clone_into_array(&data[5..13])); // TODO
                let mut fill_color = Rgba8p::new(0, 0, 0, 0);
                let mut pen_color = Rgba8p::new(0, 0, 0, 0);
                let mut pen_width = 0;
                let width = raster.width();
                let height = raster.height();
                let mut p = Plotter::new(width, height);

                let width = graphic_width;
                let height = (graphic_width as f32 * (ar as f32 / 65536.0)) as u32;

                let mut i = 13;
                
                println!("Entering Loop!");
                
                loop {
                    if i >= data.len() { break };
                    let opcode = data[i];
                    i += 1;
                    println!("MACTING OPTPCODE {}", opcode);
                    match opcode {
                        0x10 => {
                            println!("MOVE");
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.move_to(x + offset_x, y + offset_y);
                            i += 2;
                        } // Move
                        0x11 => {
                            println!("LINE");
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.line_to(x + offset_x, y + offset_y);
                            i += 2;
                        } // Line
                        0x12 => {
                            println!("QAUD");
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
                            println!("QUB");
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
                            println!("ARC");
                            unimplemented!();
                        } // Arc
                        0x1F => {
                            println!("END");
                            let mut path_builder = PathBuilder::new().absolute();
                            std::mem::swap(&mut pathbuilder, &mut path_builder);
                            let path = path_builder.build();

                            println!(":PATH:");
                            for p in path.iter() {
                                use footile::PathOp::*;
                            
                                match p {
                                    Close() => { println!("CLOSE") },
                                    Move(x, y) => { println!("MOVE {} {}", x, y) },
                                    Line(x, y) => { println!("LINE {} {}", x, y) },
                                    Quad(x1, y1, x, y) => { println!("QUAD {} {} {} {}", x1, y1, x, y) },
                                    Cubic(x1, y1, x2, y2, x, y) => { println!("CUBIC {} {} {} {} {} {}", x1, y1, x2, y2, x, y) },
                                    PenWidth(w) => { println!("WIDTH {}", w) },
                                }
                            }

                            let alpha: u8 = fill_color.alpha().into();
                            if alpha != 0 { // Not transparent
                                raster.composite_matte((), p.fill(&path, footile::FillRule::NonZero), (), fill_color, SrcOver);
                                p.clear_matte();
                            }
                            let alpha: u8 = pen_color.alpha().into();
                            if alpha != 0 && pen_width != 0 { // Not transparent
                                raster.composite_matte((), p.stroke(&path), (), 
                                    pen_color, SrcOver
                                );
                                p.clear_matte();
                            }
                            pen_width = 0;
                        } // Close

                        // Solid
                        0x20 => {
                            println!("COLOR");
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            fill_color = Rgba8p::new(r, g, b, a);

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
                            println!("STROKE");
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            pen_color = Rgba8p::new(r, g, b, a);

                            i += 8;
                        } // Change stroke color
                        // Width
                        0x25 => {
                            println!("WDTH");
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

                        x => panic!("Parse error {:x} @{}!", x, byte_count)
                    }
                }
                println!("Done!");
                // Found Graphic
                return;
            }
            _ => panic!("Unsupported!"),
        }
    }
    panic!("There was no graphic in this file!");
}*/
