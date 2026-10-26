# STGS

Source spelling: `_STGS(0,0,0,0,0)`.

## Operands

- Operand 0 is slot 0–39, default 0.

- Operand 1 is initial opacity, absent = saved opacity at `+0x1e4 + slot*2`; operand 2 is target opacity, **absent = 0**.

- Operand 3 is duration, default 0, converted to `trunc(60*n/1000)` ticks (`0x636c8–0x636f0`).

- Operand 4 is interpolation selector, default 0; operand 5 is unread (`0x63708–0x63770`).

- [code] It saves target opacity as a halfword, divides initial/target by 255, and calls `0x33ad0` plus selector setter `0x338d0` (`0x63718–0x63748`).

- Missing/empty initial and target operands both compile as `0x80000000`, while duration/selector default 0.

## Switch build

[code] T dispatch `0x79400` → `0x63570`. Operand 0 is slot 0–39, default 0. Operand 1 is initial opacity, absent = saved opacity at `+0x1e4 + slot*2`; operand 2 is target opacity, **absent = 0**. Both are upper-clamped to 255, without a corresponding lower clamp in the inspected branches (`0x63624–0x63684`, `0x636b4`). Operand 3 is duration, default 0, converted to `trunc(60*n/1000)` ticks (`0x636c8–0x636f0`). Operand 4 is interpolation selector, default 0; operand 5 is unread (`0x63708–0x63770`).

[code] It saves target opacity as a halfword, divides initial/target by 255, and calls `0x33ad0` plus selector setter `0x338d0` (`0x63718–0x63748`). Live request has target alpha `+0x24`, delta `+0xbc`, timer `+0x8a/+0x8c`, selector `+0x88/+0x9a` (`0x33ad0–0x33b30`, `0x338d0`). It marks stage dirty, returns 0 and continues (`0x6374c–0x63770`, `0x79408`). It starts a fade; completion is a separate STCF mask-8 wait (T `0x33d70–0x33d8c`). No audio call is present in this path.

## Steam build

[code] Opcode `0x245` → S `0x140102520`, registration `0x1401067b9–0x1401067c7`; five-field recipe `recipes_data.rs:7791`. Missing/empty initial and target operands both compile as `0x80000000`, while duration/selector default 0. Handler scales alpha, resolves sentinels from pending/current state, sets request `+0x108`, initial/target `+0x110/+0x114`, duration `+0x118`, selector `+0x11c`; a nonpositive duration can also directly set current alpha (`0x1401025d1–0x140102699`). Return 0 (`0x1401026a5`). Sync applies opacity/mask state at `0x140149fa9–0x14014a1e3`; STCF supplies the barrier.

[data] Mahoyo: **6,835** occurrences; `1_0.chs@0x358` requests target 255 on slot 0 with retained initial alpha, then waits on mask 8 at `0x380`.

## Differences

[code] Missing target means zero on Switch but a retained pending/current value in the Steam compiler/handler; the upper clamp and storage widths also differ (T `0x636b4`, `0x63730`; S `0x140102639–0x140102686`). Source duration is quantized to ticks and short counters; Steam stores dword duration. [inferred] Explicit normal alpha endpoints may agree, but omitted targets, negative alpha, interrupted fades and timing/easing are not proven exact. A later wait can remain pending on an unrelated/stale command record ([STCF](STCF.md); S `0x14015d610`).

## Unverified

[inferred] Easing and actual fade/wait timing, including interrupted requests, remain unverified (T `0x33ad0`; S `0x140102520`, `0x14015d610`). **One runtime check:** begin a fade, interrupt it with an omitted-target STGS, then execute mask-8 STCF while recording alpha and completion in both engines.

---

Engine facts here are lifted from the port's `docs/commands/STGS.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
