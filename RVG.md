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

## Operations
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
18u32 ([u32;3])   // "NextImage" [id, anim: (style: u16, amount: i16), time_delay: (num: u16, den: u16)]
                  //             anim.style = 0: linear
                  //             anim.style = 1: exponential (faster at beginning and end of animation)
                  //
                  // On "NextImage" rendering stops.
19u32             // "BranchImage"
```
