# RVG
Resizable Vector Graphics is a binary encoding for vector graphics.  It is designed to be created from an SVG with no data loss.  It uses Flatbuffers and is compressed with DEFLATE (plus it doesn't use XML), and thus is quite a bit smaller than an SVG.

## rvgshow
This program will display an RVG file.

## rvgfrom
This program will create an RVG from an SVG.

## rvgedit
This program lets you edit RVG files the same way inkscape lets you edit SVG files.

# Developing
We use Cap'n Proto.
```
cargo install capnpc
```
