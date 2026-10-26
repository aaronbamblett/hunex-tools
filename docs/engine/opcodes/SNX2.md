# SNX2

Source spelling: `_SNX2(35,27)`.

## Operands

- Operand 0 selects a slot (default -1), operand 1 is a numeric resource parameter (default 0).

- Otherwise it snapshots scene state through `0x5ecd0`, calculates 1 + operand1/255, writes float slot `+0x248` using stride `0x24c` and mirrors it into persistent scene storage at `0x1256784 + slot*16 + 0x1454` (`0x4118–0x415c`).

- It accepts unsigned slot 0–63, locates a `0x220`-byte slot record, marks record `+0/+8` dirty and writes the second operand unchanged at record `+0x4c`, then returns 0 (`0x140102f87–0x140102fe7`).

## Switch build

[code] T dispatch `0x78f4c` calls `0x4090`. Operand 0 selects a slot (default -1), operand 1 is a numeric resource parameter (default 0). Negative slot skips the writes. Otherwise it snapshots scene state through `0x5ecd0`, calculates 1 + operand1/255, writes float slot `+0x248` using stride `0x24c` and mirrors it into persistent scene storage at `0x1256784 + slot*16 + 0x1454` (`0x4118–0x415c`). It returns zero and the dispatcher continues (`0x4164`, `0x78f50`).

[code] The float is consumed together with the SNX3 float during slot resource creation, T `0x3fd48–0x3fd70` → `0xb5600` → `0xa9650`. [inferred] This changes a visual/resource creation parameter, not a time delay; the exact parameter name and image transformation are unverified. No audio start or wait was established in this chain.

## Steam build

[code] R `recipes_data.rs:5255–5285` accepts two integers, reverses them for stack order, and emits opcode `0x255`; S registration `0x140106861–0x14010686f` binds handler `0x140102f60`. It accepts unsigned slot 0–63, locates a `0x220`-byte slot record, marks record `+0/+8` dirty and writes the second operand unchanged at record `+0x4c`, then returns 0 (`0x140102f87–0x140102fe7`). A slot within 0–63 but beyond the actual vector length leaves a null record pointer before the writes.

[code] Record fields beginning `+0x48` merge into the six-field resource key through `0x1400ff590`, called at `0x140148d8d`; key construction `0x14012b590` formats those six integers. [inferred] Equivalence to Switch's normalized float is unresolved; the raw store is not itself evidence of a scale mismatch.

[data] Mahoyo raw corpus: 1,833 calls; `raw/1_1.chs@0x2dd4` supplies slot 21 and parameter -35.

## Differences

[data] Source slots are 1–39 and parameter range -255–255 (`dump/scr*.txt:1`). [code] Switch has only a negative-slot check before indexed writes, whereas Steam rejects unsigned values above 63 and assumes accepted indices exist (T `0x4118`; S `0x140102f87–0x140102fd2`). [inferred] Resource-key conversion and Switch's 1+v/255 transform must be compared before asserting equal pixels; future out-of-range scripts can corrupt state or dereference null.

## Unverified

[inferred] One runtime check: compare the resource created around scr0001 statement 2259 with and without SNX2, logging the normalized Switch float and Steam resource key, and inspect the same slot's resulting image (T `0x4154`/`0x3fd70`; S `0x14012b590`). This settles the unresolved parameter meaning and parity.

---

Engine facts here are lifted from the port's `docs/commands/SNX2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
