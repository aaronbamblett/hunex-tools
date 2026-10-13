# hunex-image

Background images of the HuneX engine. An MZP image archive holds a header entry and one compressed tile per grid cell; the tiles abut at full size and are palette images. This crate assembles them into an RGBA canvas and splits an RGBA canvas back into tiles.

The encoder uses an exact palette when a tile has 256 colours or fewer. Otherwise it quantises the tile with a median cut, which is lossy. The crate is `no_std`, depends on hunex-mzp and hunex-mzx, and builds for WebAssembly.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
