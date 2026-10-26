# STZ4

Source spelling: `_STZ4(4,984,984,984,984,0,0,0)`.

## Operands

- Operand 0 is slot 0–39, default 0.
- Operands 1/2 are initial x/y zoom and 3/4 target x/y zoom, in per-mille units.
- Operand 5 is duration, default 0, converted to `trunc(60*n/1000)` ticks (`0x62b44–0x62b6c`).
- Operand 6 is an interpolation selector, default 0; operand 7 is a scale-mode byte retained from `+0x874` when absent (`0x62c6c–0x62ca4`).

## Switch build

[code] T dispatch `0x793d4–0x793dc` calls `0x62a30`. Omitted initial values come from saved `+0x7d4/+0x824`; provided target values replace those saved halfwords, while omitted targets retain them (`0x62ad0–0x62c58`).

[code] Setter T `0x33210` stores target scales, mode, counters and deltas into live slot state (`+0x9c/+0x9e/+0xa2`, progress at `+0x90`, `+0xe4..+0xec`); selector setter `0x33300` writes `+0xa0/+0x8e`. Handler marks stage dirty and returns 0 (`0x62ca8–0x62cf8`); the dispatcher continues. It starts/scales a sprite over later updates and does not itself wait or play sound. [inferred] Visual scaling follows from the stored scales and interpolation deltas, T `0x33210–0x332fc`; STCF mask bit 4 checks `+0x84`, whereas this setter writes `+0x90`; its predicate does not directly inspect the latter (`0x33cb0–0x33da0`). A blanket “STCF waits for every kind of zoom” claim is unsupported.

## Steam build

[code] Opcode `0x246` → S `0x1401026c0` (registration `0x1401067ce–0x1401067dc`), eight-field recipe `_STZ4:0`. Defaults for fields 1–4 and 7 are `0x80000000` (retain-state sentinel), duration/selector default 0. It converts scales by /1000, resolves missing values from the pending request or current scale, and writes request `+0x124`, scales `+0x12c..+0x138`, mode `+0x13c`, duration `+0x140`, selector `+0x144` (`0x1401027c1–0x1401028f3`). Return 0 (`0x1401028ff`), not a barrier. Sync consumes completed/immediate requests at `0x140149dd3–0x140149f11`; mode maps to a boolean by equality to 1.

[data] Mahoyo: 20,257 occurrences; `1_0.chs@0x598` supplies slot 1, four scale values 1000 and three zero tail fields, following sprite setup.

## Differences

[code] Switch stores zoom as halfwords and uses tick/short-counter interpolation; Steam retains float scale and a dword duration/selector request (T `0x62bf8`, `0x62c2c`, `0x33210`; S `0x1401028af–0x1401028f3`). Switch absent mode retains a byte, whereas Steam resolves its sentinel against pending/current mode. [inferred] Explicit common values may agree, but large/negative scales, sub-tick durations, mode flags and interrupted requests are not proven exact. Neither immediate handler supplies a completion guarantee.

## Unverified

[inferred] Easing selectors, scale-mode interpretation and timing equivalence remain unverified (T `0x33300`; S `0x1401028e5–0x1401028f3`). Record a nonuniform scale tween with omitted endpoints and a sub-frame duration, interrupt it midway, and compare per-frame scales and STCF completion on both engines.
