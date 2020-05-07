use pix::rgb::{Rgba8p, SRgba8};
use pix::{Raster};
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 2);
    let mut rvg = Vec::new();
    let mut f = File::open(&args[1]).unwrap();
    f.read_to_end(&mut rvg).unwrap();

    let graphic = Graphic::load(std::io::Cursor::new(&rvg)).unwrap();

    let model = &graphic.models[0];
    let mut raster = Raster::<Rgba8p>::with_clear(model.width as u32, model.height as u32);

    rvg::render(&mut raster, &graphic, ());

    write_png(Raster::with_raster(&raster), &format!("{}.png", args[1]))
        .unwrap();
}
