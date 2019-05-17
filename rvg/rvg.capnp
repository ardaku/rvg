# RVG IDL file

@0x952387ffc904d293;

# A high precision color (RGBA 16-16-16-16).
struct Color {
    r @0 :UInt16;
    g @1 :UInt16;
    b @2 :UInt16;
    a @3 :UInt16;
}

struct Bitmap {
    pitch @0 :UInt16;
    pixels @1 :List(Color);
}

# At the root of the file, an RVG is a Graphic.
struct Graphic {
    # By default, main axis is X, this will change it to Y if true.
    mainAxis @0 :Bool = false;
    # By default RVGs are 2D.
    threeDimensions @1 :Bool = false;
    # 2nd axis in relation to main.  Range: 0...65,535 -> (0 = equal length, 1-65535 = less length)
    aspectRatio @2 :UInt16 = 0;
    # Now we just list the operations.
    operations @3 :List(Operation);
    # This is either 2-tuples (2D) or 3-tuples (3D) depending on `three_dimensions` flag.
    positions @4 :List(UInt16);
    # Colors
    colors @5 :List(Color);
    # Bitmaps
    bitmaps @6 :List(Bitmap);
    #
    glyphs @7 :List(Graphic);
}

enum PathOp {
    # Uses `value` for position index
    move @0;
    line @1;
    quad @2;
    cubic @3;
    arc @4;

    # Fill (0x10-0x1F)
    solid @5; # Fill with 1 color (before each vertex)
    bitmap @6; # Fill with bitmap - stretch (before Move)
    tile @7; # Fill tiled with pattern - Vector Graphics
    pattern @8; # Fill tiled with pattern - Bitmap
    stroke @9; # Change stroke color
    width @10; # Change stroke width
    dashed @11; # Change dash width (0=100% by default)

    # Join Style (0x20-0x2F)
    joinMiter @12; # `value` for amount
    joinBevel @13;
    joinRound @14;
}

struct Operation {
    pathOp @0: PathOp;
    value @1: UInt16;
}


