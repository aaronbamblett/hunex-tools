# SNX3

Source spelling: `_SNX3(21,32)`.

## Operands

- Operand 0 selects a slot (default -1), operand 1 defaults to zero.

## Switch build

[code] T dispatch `0x799c8` calls `0x4180`. For nonnegative slots it snapshots scene state, computes 1 + operand1*0.01, writes float slot `+0x24c` using stride `0x24c`, and mirrors it at `0x1256784 + slot*16 + 0x1458` (`0x4208–0x424c`). It returns zero; dispatch continues at `0x799cc`.

[code] T `0x3fd5c–0x3fd70` loads both SNX2/SNX3 floats into resource creation `0xb5600`, then `0xa9650`. [inferred] It changes a slot visual/resource parameter; its exact transformation and any interaction with SNX2 remain unresolved. It installs no wait and no audio start was established.

## Steam build

[code] H `_SNX3:0` accepts two integers and emits opcode `0x254`; S registration `0x14010684c–0x14010685a` binds `0x140102ed0`. The handler pops slot then value, accepts unsigned slot 0–63, marks record `+0/+8` dirty and writes raw value at `+0x50` (`0x140102ef7–0x140102f57`). As with SNX2, a slot beyond actual vector length leaves a null pointer before stores.

[code] This field participates in the six-integer resource key copied by `0x1400ff590` and formatted by `0x14012b590` on the render resource path (`0x140148d8d–0x140148ddc`). [inferred] The relationship of that key to Switch's normalized float is not established.

[data] Mahoyo raw corpus: 1,830 calls; `raw/1_0.chs@0x4e68` pushes 62 then slot 25.

## Differences

[data] Surveyed source slots are 1–39 and values -78–127 (`dump/scr*.txt:1`). [code] Switch skips negative slots but shows no upper bound before the writes; Steam uses unsigned 0–63 and an assumed vector entry (T `0x4208`; S `0x140102ef7–0x140102f42`). [inferred] Do not label this a text-window command or a proven percentage scale solely from its name/formula; exact effect remains unresolved.

## Unverified

[inferred] Compare the slot around scr0001 statement 4764 with SNX3 enabled/disabled, logging Switch's float and Steam's key plus the resulting image, while holding SNX2 fixed (T `0x4244`; S `0x14012b590`).
