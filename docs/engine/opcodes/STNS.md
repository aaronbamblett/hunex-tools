# STNS

Source spelling: `_STNS(17,11)`.

## Operands

- Operand 0 selects slot 0–39, default 0; operand 1 is a noise/effect mode, default 0 (`0x64708–0x64744`).

## Switch build

[code] T dispatch `0x79858–0x79860` calls `0x64670`. It validates the low-halfword unsigned value against 192, replacing larger values with zero, then stores its low byte into saved `+0x13b4 + slot` and live `+0x1b0`. It also stores a derived companion byte: mode <=2 gives 3, mode >2 gives 1 (`0x64748–0x64798`). Saved companion is `+0x13dc + slot`, live companion `+0x1b1`. Stage snapshot/dirty state is maintained; handler returns 0 and dispatch continues (`0x646b8–0x646ec`, `0x64768`, `0x6479c–0x647ac`, `0x79860`).

[inferred] “Noise/effect mode” identifies the sprite rendering control, not a proven named noise algorithm. Exact pixels cannot be inferred from mode 11 alone; no sound or completion wait is established by the handler (T `0x64670–0x647ac`).

## Steam build

[code] Opcode `0x257` → S `0x1401030a0`, registration `0x14010688b–0x140106899`; two-integer recipe `_STNS:0`. Accepts slots 0–63, splits operand 1 by signed division by 10, stores quotient at command slot `+0x38` and remainder at `+0x34`, then sets dirty/update and returns 0 (`0x1401030c7–0x140103145`). Sync uses quotient through converter `0x1400a1a20`, remainder through resource-name lookup `0x1400a12b0`, and installs a texture/parameter using `0x140075460` (`0x14014a4a6–0x14014a5d2`). This is a graphics request, not a wait.

[data] Mahoyo: 90 occurrences; `1_0.chs@0x60ec` supplies slot 26 and mode 1 after STBR.

## Differences

[code] Source mode 11 stores byte 11 plus companion 1; Steam mode 11 produces remainder 1 and quotient 1. Source restricts a halfword to 0–192; Steam uses signed dword division with no matching mode bound in this handler (T `0x64758–0x64798`; S `0x140103112–0x14010313f`). [inferred] These are different representations; their final visual equivalence is not established by the arithmetic alone. Resource lookup failure can leave the previous texture/effect rather than install the intended one (S `0x14014a502–0x14014a5d7`).

## Unverified

[inferred] Source mode/companion consumers, actual noise textures and intensity correspondence remain unresolved (T `+0x1b0/+0x1b1`; S `0x1400a12b0`, `0x1400a1a20`). Apply mode 11 to the scr0054 sprite and trace the final texture/parameters while capturing a short frame sequence in both engines.
