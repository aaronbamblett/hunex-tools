# STBR

Source spelling: `_STBR(4,23)`.

## Operands

- Operand 0 is slot 0–39, default 0.

- Operand 1 selects the slot's rendering/blend mode, default 0; low byte zero is stored as 255, otherwise its low byte is stored at saved stage `+0x7c + slot` (`0x621c8–0x62208`).

- If operand 2 is present and nonnegative, **it overrides operand 1** with the same zero-to-255 handling (`0x6220c–0x62244`).

- Operand 3 is not read (`0x62120–0x6225c`).

- It pops slot, mode and a **policy** value; accepts slots 0–63; converts modes through `0x1400a1830` (unsigned 0–25, otherwise zero) and stores result at `+0xcc`.

- [code] Compiler `recipes_data.rs:7021` accepts 1–4 source fields, defaults 0/0/0/-1 and emits fields in reverse order (`compile.rs:796`).

- The native handler consumes three; a fourth value can remain on the VM stack.

## Switch build

[code] T `0x7942c` → `0x62120`. Operand 0 is slot 0–39, default 0. Operand 1 selects the slot's rendering/blend mode, default 0; low byte zero is stored as 255, otherwise its low byte is stored at saved stage `+0x7c + slot` (`0x621c8–0x62208`). If operand 2 is present and nonnegative, **it overrides operand 1** with the same zero-to-255 handling (`0x6220c–0x62244`). Operand 3 is not read (`0x62120–0x6225c`). The handler prepares the stage snapshot and returns 0; dispatch continues at `0x76730` (`0x62164–0x62198`, `0x6224c`, `0x79434`).

[inferred] It changes sprite compositing on subsequent rendering, not keyframe motion; exact formulas for every selector remain untraced. The corresponding field is initialized by sprite creation at T `0x61bf4`, and Steam uses a rendering-node selector, not an animation-table index (S `0x14014a438–0x14014a464`). No sound or completion wait is established by this setter.

## Steam build

[code] Opcode `0x252` → S `0x140102d10` (registration `0x140106822–0x140106830`). It pops slot, mode and a **policy** value; accepts slots 0–63; converts modes through `0x1400a1830` (unsigned 0–25, otherwise zero) and stores result at `+0xcc`. Policy 1 sets `+0xd0=1`; policy -1 sets it only for mode 5; all other policies set zero (`0x140102d59–0x140102dde`). It returns 0 (`0x140102de5–0x140102df1`). Sync forwards `+0xcc/+0xd0` to node virtual methods at `0x14014a438–0x14014a4a0`.

[code] Compiler `recipes_data.rs:7021` accepts 1–4 source fields, defaults 0/0/0/-1 and emits fields in reverse order (`compile.rs:796`). The native handler consumes three; a fourth value can remain on the VM stack. [data] This also occurs in native Mahoyo: `1_0.chs@0x4a9c` follows four effective pushes, slot 22, mode 19, policy 0 and extra -1. Mahoyo total: **9,652** occurrences.

## Differences

[code] Operand 2 has different established roles: Switch replacement mode, Steam boolean-policy input (T `0x62214–0x62244`; S `0x140102dc7–0x140102dde`). Switch stores a byte with 0→255; Steam bounds the mode to 0–25 and maps it. [inferred] Explicit third operands and selectors outside Steam's enumeration require checking, even when compilation succeeds. Equality of corresponding blend formulas is not established by these setters.

## Unverified

[inferred] Full selector/formula correspondence, the intended policy flag and VM stack reset boundaries remain unverified (T `0x62208`; S `0x140102dc0`, `0x14014a464`). **One runtime check:** apply a three-operand STBR with different second/third values to an overlapping sprite and record both engines' effective blend mode and pixel output, also recording VM stack depth across the next statement.

---

Engine facts here are lifted from the port's `docs/commands/STBR.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
