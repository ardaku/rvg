# Header (64 bits)
## Format Tag
```rust
"RVG0" // 4 bytes
```
## Compression Algorithm
```rust
0u32 // Currently this is the only supported compression algorithm (DEFLATE).
```

# Compressed Data
## Header (128 bits)
```rust
width: u32,            //
height: u32,           //
background_color: u32, // sRGBA
type_size: u8,         // 1-128: number of bits per number (T)
unit_size: u8,         // 1-128: subpixel positioning bits
rvg_version: u16,      // must be zero
```

## Base Operations
```rust
0u32              // "Eof" End of File.
1u32 (id: u32)    // "Tag" ID Tag for next operation.
2u32              // "GroupOpen" Operations following will be grouped together.
3u32              // "GroupClose" Until this operation.
4u32 (color: u32) // "FillColor" Change fill color
5u32 (color: u32) // "StrokeColor" Change stroke color
6u32 (style: u32) // "StrokeJoin" Join style
7u32 (width: u32) // "StrokeWidth" Stroke width
8u32 ([T;16])     // "MatrixTransform3d"
9u32 ([T;9])      // "MatrixTransform2d"
10u32 ([T;3])     // "Move3d" (Start a path and) Move
11u32 ([T;2])     // "Move2d" (Start a path and) Move
12u32 ([T;3])     // "Line3d"
13u32 ([T;2])     // "Line2d"
14u32 ([T;3])     // "Quad3d"
15u32 ([T;2])     // "Quad2d"
16u32 ([T;3])     // "Cubic3d"
17u32 ([T;2])     // "Cubic2d"
18u32 ([u32;1])   // "BranchImage" [id]. On "BranchImage" rendering stops.
                  // Following branch images are alternate branches from the current frame image.
19u32 ([u32;2])   // "Animate" [anim: (style: u16, amount: i16), time_delay: (num: u16, den: u16)].
                  //             anim.style = 0: jump - no animation
                  //             anim.style = 1: linear (constant speed)
                  //             anim.style = 2: exponential (faster at beginning and end of animation)
                  //             anim.style = 3: fade
20u32 (...TODO)   // "QuaternionTransform3d"
21u32 (...TODO)   // "QuaternionTransform2d"
22u32             // "NonZero" fill rule - default fill rule
23u32             // "EvenOdd" fill rule
24u32 ([T;3])     // "Translate3d"
25u32 ([T;2])     // "Translate2d"
26u32 ([T;3])     // "TranslateFill3d" - Translate fill separate from stroke
27u32 ([T;2])     // "TranslateFill2d" - Translate fill separate from stroke
28u32 ([T;3])     // "TranslateStroke3d" - Translate stroke separate from fill
29u32 ([T;2])     // "TranslateStroke2d" - Translate stroke separate from fill
30u32 ([u32;2]..) // "Font" [font_id: u32, len: u32, data: &[u32]] - Load a font file and bind the font to an id.
31u32 ([u32;2]..) // "Text" [font_id: u32, len: u32, text: &str] - Draw text with font (use Translate2d/3d to position it).
32u32 ([u32;2]..) // "Image" [image_id: u32, len: u32, data: &[u32]] - Load an image file and bind the image to an id.
33u32 ([u32;3])   // "SetPattern" [image_id: u32, width: u32, height: u32] - Use image as pattern, scaling to width and height
34u32 ([u32;~9])  // "TextureRect2D" [image_id: u32, xywh: [T;4], tc_xywh: [T;4]] - Draw a 2D rectangle with texture.
35u32 ([u32;~11]) // "TextureRect3D" [image_id: u32, xyzwhd: [T;6], tc_xywh: [T;4]] - Draw a 3D rectangle with texture.
36u32 ([u32;~14]) // "TextureQuad2d" [image_id: u32, coords: [[xy: T; 2];4], tc_xywh: [T;4]] - Draw a 2D quad with texture.
37u32 ([u32;~18]) // "TextureQuad3d" [image_id: u32, coords: [[xy: T; 3];4], tc_xywh: [T;4]] - Draw a 3D quad with texture.
```

## SVG Extension
Currently no extensions.

## Inkscape Extension
Currently no extensions.
