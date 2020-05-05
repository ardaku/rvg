# RVG Format Spec
RVG files are a binary SVG-like document, with 3D, animations, and albums.

## Example File Layout
After decompressing with zstandard, layout will look like this.  Floating point
numbers may only be NAN if they are closing a section of the file.

```
# RvgFile
FORMAT: u32                             # b"rVg\x00"
ATTRIBUTE_LIST: [Attribute]             # List of vertex attributes
VERTEX_LIST: [f32x(2+N)]                # 2D Points + Attributes (until NAN)
GROUP: [Group]                          # Groups until empty path
GRAPHICS: [Graphic]                     # Graphics
BITMAPS: [Bitmap]?                      # Optional Bitmaps (until EOF)

# Attribute
0u8: End                                # End attribute List
1u8: Z                                  # Z, 1 dimensions (until NAN)
2u8: UvTextureCoordinates               # UV Texcoords, 2 dim. (until NAN)
3u8: Rgb                                # RGB Gradient, 3 dim. (until NAN)
4u8: Rgba                               # RGBA Gradient, 4 dim. (until NAN)
5u8: Alpha                              # Alpha Gradient, 1 dim. (until NAN)
6u8: Normal2D                           # 2D Normal (until NAN)
7u8: Normal3D                           # 3D Normal (until NAN)
8u8: Normal4D                           # 4D Normal (until NAN)
9u8: StrokeWidth                        # Width of Stroke
16u8: UserDefined(Nu8)                  # Attr. N dimensions (until [NAN])

# Group (paths)
ATTRIBUTES: [u8]                        # List vertex attributes used until 0
PATH: [PathOp]                          # List PathOps

# PathOp
0u8: End                                # End list of PathOps
1u8: Close                              # Close PathOp
2u8: Move(u32, ...)                     # Move PathOp (Attribute Indices)
3u8: Line(u32, ...)                     # Line PathOp (Attribute Indices)
4u8: Quad(u32, u32, ...)                # Quad PathOp (Attribute Indices)
5u8: Cubic(u32, u32, u32, ...)          # Cubic PathOp (Attribute Indices)

# Graphic (Also "Model")
WIDTH: f32
HEIGHT: f32
GROUPS: [(u32, GroupProperties)]        # Group ID & Property list (MAX to end).
FRAMES: [Frame]

# GroupProperties
0u8: End
1u8: FillColorRgba(f32x4)
2u8: StrokeColorRgba(f32x4)
3u8: StrokeWidth(f32)
4u8: JoinStyle(u8)
5u8: FillRule(u8)
6u8: GlyphID(u32)
7u8: BitmapPattern(u32)
8u8: GroupPattern(u32)

# Frame
TRANSFORMS: [Transform]                 # One Transform For Each Group
DELAY: u16                              # Millis til next frame, 0 for nonlinear
ANIMATION: Animation                    # Animation Style

# Transform
OPS: [TransformOp]                      # List of Transform Operations

# TransformOp
0u8: End
1u8: Translate(x: f32, y: f32, z: f32)
2u8: Scale(x: f32, y: f32, z: f32)
3u8: Rotate(vx: f32, vy: f32, vz: f32, rot: f32)

# Animation
0u8: End
1u8: Jump - no animation
2u8: Linear (constant speed)
3u8: Faster at beginning and end of animation(amount_faster: f32)
4u8: Slower at beginning and end of animation(amount_faster: f32)
5u8: Fade
6u8: SrcOver each frame without clearing

# Bitmap
WIDTH: u16
HEIGHT: u16
SRGBA: [f32x4]
```
