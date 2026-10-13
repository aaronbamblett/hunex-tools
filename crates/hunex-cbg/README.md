# hunex-cbg

Reader and writer for the CompressedBG images (`.cbg`) of the HuneX engine. A file is split into horizontal stripes. Each stripe holds a spatial prediction of its rows, run-length coded and Huffman coded. The crate decodes a file to RGBA pixels, and encodes RGBA pixels back. The canonical encoder reproduces the 537 sample files byte for byte. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

- `decode` and `parse_header` read a file.
- `encode` writes the canonical form; `encode_simple` writes a larger form that decodes to the same pixels.
- Images above 16384 pixels a side, or above 64 Mpx in total, are rejected before anything is allocated.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
