# hunex-hfa

Reader and writer for HFA script archives of the HuneX engine. An archive is a header, a table of named records and the entry data packed back to back. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

- `Archive::parse(bytes)` reads an archive and borrows the entry data.
- `Archive::write()` writes entries in order, with the layout the originals use.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
