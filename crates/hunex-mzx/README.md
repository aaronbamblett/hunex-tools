# hunex-mzx

Decoder and encoder for the MZX0 compressed streams that hold the image tiles of the HuneX engine. The encoder is greedy and uses runs, back-references and literals. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
