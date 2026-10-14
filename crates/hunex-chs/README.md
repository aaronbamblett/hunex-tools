# hunex-chs

Decoder and exact re-encoder for HuneX compiled script (`.chs`) files. The crate is `no_std`, has no dependencies, and builds for WebAssembly.

- `Script::parse(bytes)` decodes a script into header facts, instructions and the string pool.
- `Script::encode()` rebuilds the bytes; for a parsed file the result is identical.
- `Script::lift()` and `Program::lower()` convert to and from a symbolic form with labels and strings.
- `source::parse(text)` reads Switch script text (`_CMD(args);`) into statements with typed arguments.
- `normalize::normalize(&program)` drops Debug instructions and resolves string and jump targets, for comparing compiled output against real files.
- `compile::compile(&statements, &scene_names)` lowers statements to a `Program` with the engine recipe table (92 commands, 204 recipes); the scene number map is caller input. It fails with the statement offset on anything unproven.
- `compare::compare(&input)` classifies a compiled scene against Steam as exact, shape (source-numeric drift only) or authored (reviewed manifest edits).

Part of the [hunex-tools](https://github.com/aaronbamblett/hunex-tools) project. Unofficial; not affiliated with HuneX. The crate does not include any game data.
