# MVOL

Source spelling: `_MVOL(,,`001:5000,,`017:80)`.

## Operands

- [code] Changes current BGM volume over tag 001 duration to tag 017 target.

- Handler T `0x651a0` searches the tags at `0x65444–0x654d8`, narrowing both to signed 16 bits.

- It reads remembered volume `0x1354e70`, computes a rate proportional to absolute volume delta / max(duration,1), calls `0x9a220`, then stores the new target at `0x65554`.

## Switch build

[code] Changes current BGM volume over tag 001 duration to tag 017 target. Handler T `0x651a0` searches the tags at `0x65444–0x654d8`, narrowing both to signed 16 bits. It reads remembered volume `0x1354e70`, computes a rate proportional to absolute volume delta / max(duration,1), calls `0x9a220`, then stores the new target at `0x65554`. The sound helper updates target/rate only when the current music channel state is active (`0x9a220–0x9a258`). Returns 0 at `0x65560`: asynchronous gain change, no voice or music-completion wait.

## Steam build

[code] Opcode `0x186`, handler S `0x1401074e0`; R `recipes_data.rs:1681` emits `(target, duration, unused=0)` pushes. The handler discards the top field, clamps duration <=0 to 1, selects current BGM channel through `0x1405aaf28`, and marks its pending record: active `+0`, gain flag `+0x80`, sentinel `+0x84=-1`, target `+0x88`, duration `+0x8c`. It returns 0 without constructing a task (`0x1401074f4–0x1401075a5`). Native state merging copies the target into current/active music state (`0x14016b7ae–0x14016b7df`).

[data] Mahoyo has **213** calls; `raw/1_0.chs` file `0x5270` uses target 0 / duration 100 / unused 0. It follows an SE-start opcode, demonstrating independent pending audio controls.

## Differences

[code] Switch uses signed-16-bit operands and a rate based on remembered target delta; Steam stores target/duration as dwords in a pending command and clamps duration to at least 1 (T `0x65490`, `0x654cc`, `0x65514`; S `0x14010751e–0x14010759e`). [inferred] Interrupted ramps, negative/out-of-range targets, and MVOL without active music may differ. Neither path promises that later script execution waits until the requested volume is reached (T `0x65560`; S `0x1401075a5`).

## Unverified

[inferred] One runtime check: measure the cited five-second BGM gain change, interrupt it with another MVOL midway, and repeat with no active BGM before starting a track. Compare envelope, remembered target, and continuation timing on both engines; this resolves the remembered-volume versus pending-state uncertainty (T `0x65514–0x65554`; S `0x140107576–0x14010759e`).

---

Engine facts here are lifted from the port's `docs/commands/MVOL.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
