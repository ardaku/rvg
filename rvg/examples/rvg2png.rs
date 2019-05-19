use footile::{PathBuilder, Raster, Plotter, Rgba8, PixFmt};
use rvg::{Rvg, BlockTypes, clone_into_array, GraphicOps};
use std::fs::{File};
use std::io::{Read};
use png::{self, HasParameters};

fn png_from_rvg(rvg: Vec<u8>) -> (u32, u32, Vec<Rgba8>) {
    let mut pathbuilder = PathBuilder::new()
        .absolute();

    let rvg = Rvg::from_vec(rvg);
    let mut pts = vec![];

    for block in rvg.iter() {
        let (block_type, data) = block.read().unwrap();

        match block_type {
            BlockTypes::Points2d => for i in (1..data.len()).step_by(4) {
                let x = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                let y = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));

                pts.push((x, y, 0));
            }
            BlockTypes::Points3d => for i in (1..data.len()).step_by(6) {
                let x = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                let y = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                let z = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));

                pts.push((x, y, z));
            }
            BlockTypes::Graphic => {
                let ar = u32::from_be_bytes(clone_into_array(&data[1..5]));
                let _bgc = u64::from_be_bytes(clone_into_array(&data[5..13])); // TODO
                let mut fill_color = Rgba8::new(0, 0, 0, 0);
                let mut pen_color = Rgba8::new(0, 0, 0, 0);
                let mut pen_width = 0;
                let width = 512;
                let height = (512.0 * (ar as f32 / 65536.0)) as u32;
                let mut p = Plotter::new(width, height);
                let mut r = Raster::new(p.width(), p.height());

                let mut i = 9;
                loop {
                    match data[i] {
                        0x10 => {
                            print!("MOVE");
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.move_to(x, y);
                            println!("({},{})", x, y);
                            i += 2;
                        } // Move
                        0x11 => {
                            print!("LINE");
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let (x, y, _z) = pts[idx as usize];
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.line_to(x, y);
                            println!("({},{})", x, y);
                            i += 2;
                        } // Line
                        0x12 => {
                            print!("QUAD");
                            let idx = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let idy = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let (x1, y1, _z1) = pts[idx as usize];
                            let (x, y, _z) = pts[idy as usize];
                            let (x1, y1) = (x1 as f32 * width as f32, y1 as f32 * height as f32);
                            let (x, y) = (x as f32 * width as f32, y as f32 * height as f32);
                            pathbuilder = pathbuilder.quad_to(x1, y1, x, y);
                            println!("({},{},{},{})", x1, y1, x, y);
                            i += 4;
                        } // Quad
                        0x13 => {
                            print!("CUBIC");
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
                            println!("({},{},{},{})", x1, y1, x, y);
                            i += 6;
                        } // Cubic
                        0x14 => {
                            println!("ARC");
                            unimplemented!();
                        } // Arc
                        0x1F => {
                            println!("CLOSE");
                            let mut path_builder = PathBuilder::new();
                            std::mem::swap(&mut pathbuilder, &mut path_builder);
                            let path = path_builder.build();

                            if fill_color.alpha() != 0 { // Not transparent
                                r.over(
                                    p.fill(&path, footile::FillRule::NonZero),
                                    fill_color,
                                );
                            }
                            if pen_color.alpha() != 0 && pen_width != 0 { // Not transparent
                                r.over(
                                    p.stroke(&path),
                                    pen_color,
                                );
                            }
                        } // Close

                        // Solid
                        0x20 => {
                            println!("SOLID");
                            let r = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            let g = u16::from_be_bytes(clone_into_array(&data[i+2..i+4]));
                            let b = u16::from_be_bytes(clone_into_array(&data[i+4..i+6]));
                            let a = u16::from_be_bytes(clone_into_array(&data[i+6..i+8]));

                            // TODO: Full Rgba16 with pix.
                            let r = (r / 256).min(255) as u8;
                            let g = (g / 256).min(255) as u8;
                            let b = (b / 256).min(255) as u8;
                            let a = (a / 256).min(255) as u8;

                            fill_color = Rgba8::new(r, g, b, a);

                            println!("({},{},{},{})", r, b, b, a);
                            i += 8;
                        } // Fill with 1 color (before each vertex)
                        // Bitmap
                        0x21 => { println!("BITMAP"); } // Fill with bitmap - stretch (before Move)
                        // Tile
                        0x22 => { println!("TILE"); } // Fill tiled with pattern - Vector Graphics
                        // Pattern
                        0x23 => { println!("PATTERN"); } // Fill tiled with pattern - Bitmap
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

                            pen_color = Rgba8::new(r, g, b, a);

                            println!("({},{},{},{})", r, b, b, a);
                            i += 8;
                        } // Change stroke color
                        // Width
                        0x25 => {
                            println!("WIDTH");
                            let w = u16::from_be_bytes(clone_into_array(&data[i..i+2]));
                            pathbuilder = pathbuilder.pen_width(w as f32 * width as f32);
                            println!("({})", w);
                            i += 2;
                        } // Change stroke width
                        // Dashed
                        0x26 => { println!("DASHED"); } // Change dash width (0=100% by default)

                        // JoinMiter
                        0x30 => { println!("JOIN_MITER"); } // `value` for amount
                        // JoinBevel
                        0x31 => { println!("JOIN_BEVEL"); }
                        // JoinRound
                        0x32 => { println!("JOIN_ROUND"); }

                        _ => panic!("Parse error!")
                    }
                    i += 1;
                }
                return (width, height, r.as_slice().to_vec());
            }
            _ => panic!("Unsupported!"),
        }
    }
    panic!("There was no graphic in this file!");
}

pub fn write_png(width: u32, height: u32, pixels: &[Rgba8], filename: &str)
    -> std::io::Result<()>
{
    let fl = std::fs::File::create(filename)?;
    let ref mut bw = std::io::BufWriter::new(fl);
    let mut enc = png::Encoder::new(bw, width, height);
    enc.set(Rgba8::color_type()).set(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    let pix = Rgba8::as_u8_slice(pixels);
    writer.write_image_data(pix)?;
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pixels = vec![Rgba8::default(); 512 * 512];
    assert_eq!(args.len(), 2);
    let mut rvg = Vec::new();
    let mut f = File::open(&args[1]).unwrap();
    f.read_to_end(&mut rvg).unwrap();
    let (width, height, pixels) = png_from_rvg(rvg);

    write_png(width, height, &pixels, &format!("{}.png", args[1])).unwrap();
}
