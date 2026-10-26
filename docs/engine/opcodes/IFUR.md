# IFUR

Source spelling: `_IFUR(1,COMP_CHECK_OK,LV00000)`.

## Switch build

- [code] Dispatch T `0x78d58–0x78d70` parses operand 0 but calls completion helper `0x8fbb0(1)` unconditionally. Selector 1 requires completion bits 533..556 in persistent state: +0x29a bits 5..15 and +0x29c bits 0..12 (`0x8fbb8–0x8fbe0`). Zero return means complete and jumps using the first label operand; nonzero means incomplete and continues (`0x7ad84`, `0x7bff0`, `0x7c304`). The third supplied operand is not an automatically selected false branch on this path. [code] It reads completion bits produced by the RTST/RDED lifecycle, writes only the chosen script continuation, and returns to dispatch without a polling wait or direct audiovisual work (`0x78d58`, `0x8fbb0`).

## Steam build

- [code] No IFUR compiler recipe exists in H's command table (`recipes_data::COMMANDS`). Steam persistent-bit read S `0x14007db00` and conditional jump `0x14007cb00` are primitives, not a matching Tsukihime-completion handler. [data] Mahoyo's source-command inventory has zero IFUR uses. Its raw CHS has generic conditionals and bit machinery; there is no native IFUR opcode to count.

## Differences

- [code] Source progression state is missing even though Steam has persistent storage. Required current-index/session-bit validation and completion persistence are separate from simple VM flags (T `0x8f910`, `0x8fc30`). [inferred] Compiling this as always true/false would change completion gating; retaining it as unsupported exposes the gap (`scr0519.txt:1`, statement 5).

## Unverified

- [inferred] One paired completion-gate run should set exactly indices 533..556 complete, confirm COMP_CHECK_OK, then clear one required bit and confirm rejection; index 557 must not be required. Full completion/save lifecycle parity remains unresolved (T `0x8fbb0`).
