use footile::{Rgba8, PixFmt};
use rvg::{Rvg, BlockTypes, clone_into_array};
use std::fs::{File};
use std::io::{Read};
use png::{self, HasParameters};

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
//    let pixels = vec![Rgba8::default(); 512 * 512];
    assert_eq!(args.len(), 2);
    let mut rvg = Vec::new();
    let mut f = File::open(&args[1]).unwrap();
    f.read_to_end(&mut rvg).unwrap();
    let (width, height, pixels) = rvg::graphic_from_rvg(rvg.as_slice());

    write_png(width, height, &pixels, &format!("{}.png", args[1])).unwrap();
}
