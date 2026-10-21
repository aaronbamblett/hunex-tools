# STMA

Source spelling: `_STMA(3,1)`.

## Operands

- Operand 0 selects slot 0–39, default 0.

- Positive operand 1 selects an authored animation; nonpositive/absent id clears it.

- Operand 2 defaults to -1 and controls message/window suppression according to values 1/-1/2, engine mode and existing transition state (`0x96d64–0x96dec`).

- Operand 3 defaults to 0 and is passed to animation state setter `0x92e58` (`0x96e20–0x96e50`, `0x96eac–0x96eb0`).

- The id setter stores a halfword (`0x92e44`).

- Evaluator reads boundaries `0x976c0`, durations `0x97690`, frames `0x976f0`; timeline conversion is ×100/3 (`0x952a8–0x952c0`).

- Recipe `recipes_data.rs:7874` emits five values: slot, id, suppression policy, start/mode operand, and extra flags; its three-source-argument form supplies two zero tail values.

- [code] Policy 1, or conditional -1, writes native window/suppression request context `+0x2a0` (`0x140104cda–0x140104d52`).

## Switch build

[code] T dispatch `0x79844–0x79854` calls `0x96d30`. Operand 0 selects slot 0–39, default 0. Positive operand 1 selects an authored animation; nonpositive/absent id clears it. Operand 2 defaults to -1 and controls message/window suppression according to values 1/-1/2, engine mode and existing transition state (`0x96d64–0x96dec`). It is not an unconditional animation-completion wait. Operand 3 defaults to 0 and is passed to animation state setter `0x92e58` (`0x96e20–0x96e50`, `0x96eac–0x96eb0`). Its full mode semantics remain unverified.

[code] Positive ids are written to saved/live animation states at saved `+0xc34 + slot*12` and live `+0x14c + slot*0x24c`, with time reset through `0x92e50`; initial evaluation calls `0x951f0` (`0x96e78–0x96ef4`). The id setter stores a halfword (`0x92e44`). Evaluator reads boundaries `0x976c0`, durations `0x97690`, frames `0x976f0`; timeline conversion is ×100/3 (`0x952a8–0x952c0`). Handler returns 0 and dispatch continues (`0x96f74–0x96f8c`, `0x79854`). The effect is authored stage motion; waiting for its completion is separately handled by STCF bit `0x20` (T `0x33ce0–0x33d0c`). No sound start is established by this path.

## Steam build

[code] Actual compiler opcode is **`0x278` → S `0x140104c70`**, registered at `0x140106948–0x140106956`; animation startup is `0x140143bd0`. The similarly named older research about `0x140102d10` concerns opcode `0x252` (STBR), not this keyframe handler. Recipe `recipes_data.rs:7874` emits five values: slot, id, suppression policy, start/mode operand, and extra flags; its three-source-argument form supplies two zero tail values.

[code] Policy 1, or conditional -1, writes native window/suppression request context `+0x2a0` (`0x140104cda–0x140104d52`). Slot animation request is `+0x17c`, id `+0x184`, offset/mode `+0x188`, flags `+0x18c/+0x190`, pause/reset `+0x194..+0x198` (`0x140104e04–0x140104e3d`). The handler has zero-return paths (`0x14010508f`, `0x1401051f3`); this is an animation request, not an STCF task. [inferred] Exact effects of all suppression/flag combinations require the consumer, not the mere zero return.

[data] Mahoyo: **8,361** occurrences; `1_0.chs@0x1774` requests slot 3, animation 1, with three zero policy/tail values after sprite setup.

## Differences

A positive outlier 101960 occurs in scr0344. Source id storage is 16-bit. [inferred] Outliers, selector masks, loop/skip completion, pause and save/load need validation (T `0x92e44`, `0x951f0`; S `0x140143bd0`).

## Unverified

[inferred] Per-selector timing, operand 3/extra flags, outlier-id behavior and completion after pause/skip remain unresolved (T `0x96d30`, `0x951f0`; S `0x140104c70`). **One runtime check:** run the opening animation 344/Steam 8716 from a fresh script start, capture its start/end/timing, then pause/resume and issue STCF while tracing animation state and the pending record; compare with Switch.

---

Engine facts here are lifted from the port's `docs/commands/STMA.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
