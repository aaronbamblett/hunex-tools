# hunex-chs

Decoder and exact re-encoder for HuneX compiled script (`.chs`) files. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

- `Script::parse(bytes)` decodes a script into header facts, instructions and the string pool.
- `Script::encode()` rebuilds the bytes; for a parsed file the result is identical.
- `Script::lift()` and `Program::lower()` convert to and from a symbolic form with labels and strings.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX or TYPE-MOON. The crate does not include any game data.
