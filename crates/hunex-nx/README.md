# hunex-nx

Switch textures of the HuneX engine. An NXX entry is a 16-byte header and a zlib or gzip body, holding a BNTX texture container. This crate unwraps the body, reads the textures, reverses the block-linear layout, and decodes BC4, BC7 and RGBA8 to RGBA.

BC3 textures are reported as unsupported: the channel reading is not settled. The crate is `no_std` and builds for WebAssembly. It depends on `bcdec_rs` (MIT) for block decoding and on `miniz_oxide` (MIT, Zlib or Apache-2.0) for inflate.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
