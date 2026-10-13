# hunex-ccit

Reader and writer for the glyph tables (`.ccit`) of the HuneX engine. A decompressed table lists each glyph's character code and index, and optionally its position in a 1024 by 1024 atlas page. The crate parses a table exactly, writes it back byte for byte, looks up a glyph by character, and adds or replaces glyphs while keeping the codes sorted and the indices dense. It is `no_std`, has no dependencies, and builds for WebAssembly.

- `Table::parse` and `Table::to_bytes` read and write both record forms (with or without coordinates).
- `Table::find` and `Table::find_code` look up a glyph; `code_of` and `char_of` convert between characters and codes.
- `Table::insert` adds a glyph at the next index; `Table::replace` changes the coordinates of an existing glyph.
- `cell_of` gives the atlas page and cell origin of an index.

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
