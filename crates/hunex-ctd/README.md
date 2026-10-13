# hunex-ctd

Reader and writer for the script text (`.ctd`) files of the HuneX engine. A decompressed body is a list of CRLF-terminated UTF-8 lines; the line at index n is the text for script id `$` followed by six digits. The crate is `no_std`, depends only on hunex-lenzu, and builds for WebAssembly.

- `Text::from_bytes` reads a stored (LenZu) or decompressed body.
- `Text::write` and `Text::to_stored` write the body back, or as a LenZu stream.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
