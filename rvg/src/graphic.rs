use miniz_oxide::inflate::decompress_to_vec;
use miniz_oxide::deflate::compress_to_vec;

/// An RVG Graphic.
pub struct Graphic {
    buf: Vec<u8>,
}

impl Graphic {
    /// Create a new empty graphic.
    pub fn new() -> Graphic {
        // File Footer (32 bits)
        let version = 1u16; // VERSION 1
        let compression = 0u16; // DEFLATE

        // Graphic Header (64 bits)
        let details = 0u16; // main_axis, three_dimensions
        let aspect_ratio = 0u16;
        let background_rgba = 0x_00_00_00_00_u32;
        //
        let mut file = Vec::new();
        file.extend(version.to_be_bytes().iter());
        file.extend(compression.to_be_bytes().iter());
        let mut graphic = Vec::new();
        graphic.extend(details.to_be_bytes().iter());
        graphic.extend(aspect_ratio.to_be_bytes().iter());
        graphic.extend(background_rgba.to_be_bytes().iter());
        file.extend();
        Self::from_vec(file)
    }

    /// Create RVG graphic from file bytes.
    pub fn from_vec(buf: Vec<u8>) -> Graphic {
        let 
        Graphic { buf }
    }

    /// Turn RVG graphic back into file bytes.
    pub fn into_vec(self) -> Vec<u8> {
        let mut file = compress_to_vec(graphic.as_slice(), 10);

        self.buf
    }
}
