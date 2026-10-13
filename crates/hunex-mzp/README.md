# hunex-mzp

Reader and writer for the MZP containers of the HuneX engine: a six-byte magic, an entry count, a table of 8-byte records giving each entry's sector and span, and the entries, each padded with 0xFF to an 8-byte boundary. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
