# hunex-lenzu

Decoder for the LenZu compressed stream used by HuneX engine archives, and a literal-only encoder. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

- `decompress(bytes)` decodes a stream and checks its header checksum.
- `compress_literal(bytes)` writes a stream that holds only literal runs. It is valid in the format but does not shrink data, and it is not yet confirmed to be accepted by the game.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
