# IFNX

Source spelling: `_IFNX(,LV00300)`.

## Switch build

- [code] Undispatched no-op, despite its conditional-looking spelling. The I dispatch tree has no IFNX match; execution takes unknown-command T `0x76730`, increments the statement counter and continues. It does not read operands, resolve the label, change flag state, draw, play or install a wait (main first-letter table T `0x2355d0`). The generic 1,000-statement budget can still yield (`0x76738`).

## Steam build

- [code] The compiler has an empty-first-field/label recipe: push 1, call `0x131`, compare equal, jump on false (H `_IFNX:0`; `compile::lower_special`). S `0x140109a30` pushes zero, so this lowering would always jump. It returns immediately; no display/audio/wait occurs (S `0x140109a30–0x140109a4f`, conditional jump `0x14007cb00`). [data] Mahoyo's original 250 CHS files contain 39 calls to `0x131` (shared with IFP4), e.g. `raw/1_1.chs:0x401c`; bytecode alone does not recover the source spelling. Prior Switch inventory also finds IFNX undispatched in Mahoyo.

## Differences

- [code] Pass-through to the compiler is unsafe: it would jump, while Switch skips the statement (T `0x76730`; H `_IFNX:0`; S `0x140109a30`). Dropping also removes its contribution to the interpreter's statement budget (`0x76738`), so scheduler timing is not proven identical.

## Unverified

- [inferred] One Switch breakpoint trace at T `0x76730` with IFNX in the command buffer during scr0008 should confirm continuation through statement 525. Static evidence covers all identified dispatch passes; no game trace ran.
