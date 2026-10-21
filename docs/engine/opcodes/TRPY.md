# TRPY

Source spelling: `_TRPY(28)`.

## Operands

- [code] TRPY has no Switch handler: first-letter dispatch table T `0x2355d0` and T-family comparisons lead to `0x76730`.

- Operand 0 is not evaluated, and the command has no state writes, visible/audible effect, or completion wait in the supplied executable.

- It pops a signed index, handles indices 1–26 through a jump table at `0x140109c54`, checks persistent bitsets and invokes `0x14004c0b0`/`0x14004b850`/`0x14004ba00` on selected paths, then returns 0 (`0x140109a72–0x140109c51`).

- [data] Mahoyo raw corpus: 35 calls; `raw/1_5.chs@0x2d01c` pushes index 1.

## Switch build

[code] TRPY has no Switch handler: first-letter dispatch table T `0x2355d0` and T-family comparisons lead to `0x76730`. Operand 0 is not evaluated, and the command has no state writes, visible/audible effect, or completion wait in the supplied executable.

## Steam build

[code] R `recipes_data.rs:8800–8824` requires one integer and emits opcode `0x138`. Registration S `0x1400e76f1–0x1400e7700` selects `0x140109a50`. It pops a signed index, handles indices 1–26 through a jump table at `0x140109c54`, checks persistent bitsets and invokes `0x14004c0b0`/`0x14004b850`/`0x14004ba00` on selected paths, then returns 0 (`0x140109a72–0x140109c51`). Values outside 1–26 skip the indexed branch but still run the common configuration scan at `0x140109be3`.

[inferred] This appears to update achievement/progression-related state, not text timing; the exact names and external effects of the service calls remain unverified (S `0x140109ad0–0x140109b83`).

[data] Mahoyo raw corpus: 35 calls; `raw/1_5.chs@0x2d01c` pushes index 1.

## Differences

[data] The only source use has operand 28 (`dump/scr0529.txt:1`, statement 211). [code] That is outside Steam's indexed range, but Steam still performs its common configuration scan (S `0x140109a7d`, `0x140109be3`). [inferred] Even this out-of-range value must not be treated as proving Steam has a complete no-op.

## Unverified

[inferred] One runtime check: trace scr0529 statement 211 on Switch and compare a standalone Steam opcode with operand 28, observing configuration/service calls and following-statement execution (T `0x76730`; S `0x140109a50`). This would settle externally observable side effects of the shared scan.

---

Engine facts here are lifted from the port's `docs/commands/TRPY.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
