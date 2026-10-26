# STCF

Source spelling: `_STCF(0,8)`.

## Operands

- Operand 0 selects slot 0–39 (default 0); any value outside that unsigned range selects all 40, not an invalid-slot no-op (`0x62d3c–0x62d4c`).
- Operand 1 is a component mask, default `0x3f`; it is stored as a byte per selected slot (`0x62d80`, `0x62db4`).
- Operand 2 is not read by this handler (`0x62d00–0x62df4`).

## Switch build

[code] Dispatch T `0x7cce8` calls `0x62d00`.

[code] The handler clears/rebuilds the selected-slot bitset and sets interpreter pending bit `0x20` (`0x62d68–0x62df4`). Pending processing T `0x7d398–0x7d3e0` polls `0x33cb0`. Mask bits 1/2/4/8 test positive movement/uniform-zoom/rotation/opacity counters at live slot `+0x78/+0x7e/+0x84/+0x8a`; bit `0x10` tests region progress `+0x13c`; bit `0x20` tests animation active and unpaused (`0x33cc0–0x33d8c`, `0x92e98`). It yields until every selected component is idle. It changes no image or sound itself; its visible consequence is delaying later statements. [inferred] Effect follows from the dispatcher and pending poll, rather than a rendering call (`0x7ccf0`, `0x33cb0`).

## Steam build

[code] Recipe opcode `0x27f` → S `0x140105360` (registration `0x140106987–0x140106995`). Compiler defaults are slot 0, mask 63, operand 2 = 1 (`_STCF:0`). The handler pops all three. It allocates a task, stores slot/mask at task `+0x58/+0x5c`, sets task `+0x60` to `0x12201` when operand 2 is zero and zero otherwise, attaches it to VM `+0x24e8`, and normally returns 1 (`0x1401053b9–0x14010545c`). With global `0x1405aaee0` set, it takes an immediate completion/cleanup path and returns 0 (`0x140105423–0x14010544a`).

[code] Update S `0x14015d500` polls `0x14015d610`: match command records by slot/class mask and query type 1 via `0x14008ce40`, type 2 via `0x14013ff90`, type 3 via `0x14015a7b0`. A busy record keeps the task pending (`0x14015d83d`). Type 2 already has status-flag exceptions (`0x14015d739–0x14015d784`); “Steam never recognizes pause” is not established. No drawing/audio call is introduced by the wait itself (`0x140105360–0x140105479`).

[data] Mahoyo: 4,261 opcode occurrences in the 250 raw CHS files; `1_0.chs@0x380` pushes operand 2 = 1, mask 8, slot 0 after an opacity change at `0x358`.

## Differences

[code] Switch truncates the mask to eight bits and ignores operand 2; Steam retains the dword mask and uses operand 2 for task flags. Switch selects 40 slots; Steam's record system uses 64-slot state (S `0x1401052d3–0x1401052df`). Those differences matter for nonstandard operands as well as default waits.

The suspected first wait has not been confirmed by a live IP trace. Paused animation has not been established as the cause; the old-slot busy record remains unidentified. A wait can remain pending even after the visible sprite appears settled. [inferred] This follows from the record scan, not a demonstrated identity of the stalled record (S `0x14015d610`).

## Unverified

Trace the pending Steam record and compare it with Switch's selected counters and pause state. This distinguishes stale/paused animation from another pending component without removing waits.
