# hunex-nopus

Reader for the Nintendo OPUS audio files of the Switch release, and a remuxer to Ogg Opus (RFC 7845). The packets are copied byte for byte. The crate is `no_std`, depends only on hunex-ogg, and builds for WebAssembly.

The output is an intermediate for re-encoding: the game's engine plays Ogg Vorbis, not Opus. See the main README for the path to `.hw`.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
