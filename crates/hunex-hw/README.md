# hunex-hw

Reader and writer for the `.hw` audio files of the HuneX engine. A `.hw` file is a 64-byte header followed by an unmodified Ogg Vorbis stream. The crate is `no_std`, depends only on hunex-ogg, and builds for WebAssembly.

- `Hw::parse` checks the header against the Ogg data and returns the payload.
- `wrap` writes the header for an Ogg Vorbis file.
- `vorbis_info` reads the channel count, sample rate and length of an Ogg Vorbis file.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
