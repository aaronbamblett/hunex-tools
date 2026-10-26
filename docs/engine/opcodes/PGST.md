# PGST

Source spelling: `_PGST(-1)`.

## Operands

- [code] T `0x76f74–0x76f9c` evaluates operand 0 as a page identifier.

- In ordinary execution it stores a halfword at message-state `+0x8c4` through `0x78098–0x780a4` and continues.

- It pops the value, conditionally stores a full dword at state `+0x1188` when the state extent exceeds `0x462`, calls `0x1400e9510`, copies two global values to shared state `+0x320/+0x324`, and returns 0 (`0x140109983–0x1401099cf`).

- [data] Mahoyo raw corpus: 7,614 opcode calls; `raw/1_0.chs@0x94` pushes -1 using the negate operation before `0x12a`.

## Switch build

[code] T `0x76f74–0x76f9c` evaluates operand 0 as a page identifier. In ordinary execution it stores a halfword at message-state `+0x8c4` through `0x78098–0x780a4` and continues. In the alternate scan/search mode (`0x3519d8` points to `0x1354e50`) it compares the sign-extended 16-bit identifier with the target through `0x351a70`; a match clears that mode at `0x7c604–0x7c610`. No fade/audio operation or completion wait occurs in this path.

## Steam build

[code] R `recipes_data.rs:1790–1814` requires one integer and emits opcode `0x12a`. Registration S `0x1400e7683–0x1400e7692` selects handler `0x140109970`. It pops the value, conditionally stores a full dword at state `+0x1188` when the state extent exceeds `0x462`, calls `0x1400e9510`, copies two global values to shared state `+0x320/+0x324`, and returns 0 (`0x140109983–0x1401099cf`).

[inferred] This is page bookkeeping with additional Steam state synchronization; the complete meaning of the helper and copied globals is unresolved (S `0x1401099a5`).

[data] Mahoyo raw corpus: 7,614 opcode calls; `raw/1_0.chs@0x94` pushes -1 using the negate operation before `0x12a`.

## Differences

[code] Switch stores only 16 bits and sign-extends for page search, whereas Steam stores a dword (T `0x780a0`/`0x76f94`; S `0x14010999c`). [inferred] Large identifiers can diverge, and Steam's helper/global updates prevent classifying this as a harmless universal no-op.

## Unverified

[inferred] Search entry, read/backlog boundaries and save/load ownership remain unresolved. One runtime check: save on a page boundary, advance, reload and perform page search/backlog navigation while logging page state in both engines (T message `+0x8c4`; S state `+0x1188`, helper `0x1400e9510`).

---

Engine facts here are lifted from the port's `docs/commands/PGST.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
