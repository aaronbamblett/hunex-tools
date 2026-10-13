# hunex-switch-text

Script text of the HuneX Switch RomFS. `script_text.mrg` holds five language pairs; each pair is a table of big-endian offsets and the text it indexes. A `$` reference is a zero-based line index. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
