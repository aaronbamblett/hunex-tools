# JUMP

Source spelling: `_JUMP(240,START)`.

## Switch build

- [code] `scene_number[,entry_label]` transfers scenes. First-letter J dispatch goes directly to T `0x76e24`; operand 0 is matched as an unsigned 16-bit ID against the 549-entry scene-number table starting `0x235744` (`0x76e3c–0x76e80`). Pending byte through `[0x3519d8]` yields before transfer; mode through `[0x351a28]` suppresses it (`0x76e24–0x76e30`, `0x77bbc–0x77bcc`). [code] The following scan counts/parses the optional entry operand (`0x77dcc–0x77e84`); this is a scene transfer rather than an in-script label branch. It reads scene mapping and engine execution state and changes scene/entry state. No standalone sound/image operation is identified at dispatch; scene lifecycle determines subsequent effects (T `0x76e24`, `0x77dcc`).

## Steam build

- [code] H `compile::lower_special` requires a caller-supplied scene-name mapping, emits entry value (default 0), setup `0xf6`, scene-name string, and transfer `0xf3`. Unknown numeric scene IDs are compile errors (`compile::scene_name`). `0xf6` is S `0x14007dde0`; `0xf3` reaches `0x14007c440` through `0x14007ddc0` (registration `0x14007a7d2`). The transfer core loads scene data and returns VM scheduler status; its failure paths report errors (`0x14007c6eb–0x14007c7a1`). [code] These operations consume entry/name rather than a Switch numeric registry ID. Transfer is lifecycle work, not a timer or predicate wait; no direct equivalence of every Switch mode guard follows from name mapping (same handlers). [data] Mahoyo's 250 raw files contain 576 calls each to `0xf6` and `0xf3`. Example: `raw/1_0.chs:0x7d5c–0x7d70` sets entry 0 and transfers to 1_1.

## Differences

- [code] Switch uses a 16-bit numeric scene lookup; Steam opens named CHS scenes and resolves the entry through generated dispatch. Named ZZ labels are registered in the compiler prologue (T `0x76e50`; H `compile::compile`). [inferred] Missing entries, special source execution modes and teardown/audio lifecycle need validation; a resolved filename does not prove that loading the target reproduces all Switch state (`0x77bbc`; S `0x14007c440`).

## Unverified

- [inferred] One paired transfer trace should take an authored non-default entry, confirm the first target statement, and compare inherited flags/music plus an ordinary return. This settles actual entry/lifecycle parity beyond static scene-name resolution (T `0x76e24`; S `0x14007c440`).
