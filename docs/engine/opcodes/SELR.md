# SELR

Source spelling: `_SELR(0,$042569)`.

## Switch build

[code] Registers a choice row: operand 0 is its authored result value, operand 1 its text reference/string, and operand 2 is an optional lock flag. The value store is T `0x796dc`; `0x796e0–0x796f8` sets the row's lock byte in `0x1357d5c` exactly when operand 2 equals 1. The count still increases at `0x796fc–0x79700`. Registration continues dispatch; selection/display belongs to SELD.

[code] Pointer selection rejects locked rows at T `0x817fc–0x81824`; initial focus and directional navigation consult the same lock array at `0x81954`, `0x81998`, `0x81a1c`. Locking preserves the row; it does not remove it. There is no demonstrated sound effect in this registration path.

## Steam build

[code] Opcode `0x160`, S `0x140049a40`, pops a value and obtains text through `0x14007ba70`, appends a 0x28-byte row record, and returns 0 at `0x140049bec`. It has no native lock operand and does not wait for selection. The compiler emits text then value (`_SELR:0`). Choice display/input is deferred to `0x161`.

[data] Mahoyo has 423 `0x160` instructions across 250 dumped CHS files; `raw/test_script.chs:0x2068` registers a scene-jump row after a string and value 0.

## Differences

[inferred] Appearance and input parity remain unverified.

[data] Surveyed authored values are 0..9; 6/824 registrations are locked, including two early calls in scr0070 (`dump/*.txt:1`). [inferred] The marker reserves the sign bit, so it is not a general mapping for arbitrary 32-bit authored values.

## Unverified

[inferred] Disabled styling, keyboard/mouse/controller behavior, all-locked behavior and save/load during choice presentation need runtime evidence; maximum row capacity and arbitrary value-range parity were not established. Enter scr0070's final choice with F057/G003 failing its unlock condition, attempt every input method on the locked second row, then repeat unlocked and verify original branch values (S `0x140155674`).
