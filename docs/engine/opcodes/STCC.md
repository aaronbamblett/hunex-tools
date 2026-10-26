# STCC

Source spelling: `_STCC(-1)`.

## Operands

- Operand 0 defaults to slot 0; slots 0–39 clear individually, any negative value clears all 40, and values above 39 return without clearing (`0x61e18–0x61f70`).

## Switch build

[code] T dispatch `0x78980` calls `0x61de0`. No other operand is read. The handler prepares the saved stage snapshot when necessary (`0x61f14–0x61f48`), then resets image id to -1, position/pivot/order to zero, scale to 1000, opacity to 255, region/clip angle, mask, effect and animation state (`0x61fb0–0x620dc`, animation resets `0x62038–0x62054`).

[code] It sets the stage dirty flag and returns 0; dispatcher resumes at T `0x76730` (`0x61e40`, `0x620f0–0x62110`, `0x78988`). It requests removal/reset of the sprite on the next stage update, without waiting for animation completion or emitting sound. [inferred] Visible removal follows from image id -1 and dirty state, not a synchronous screen presentation (`0x61fb0`, `0x61e40`).

## Steam build

[code] Opcode `0x241` → S `0x140101830`, registered at `0x140106765–0x140106773`; one integer is emitted by `_STCC:0`. A negative operand loops over 64 slots (`0x140101891–0x1401018ae`); a nonnegative operand must be below 64 (`0x140101c1f`). It replaces slot command state with defaults, marks dirty/create/update flags and, where appropriate, requests animation-resource cleanup (`0x1401019f9–0x140101be7`, individual path `0x140101c77–0x140101cf7`). Return 0; no completion task (`0x140101bed`).

[data] Mahoyo: 36,437 occurrences; `1_0.chs@0x6a8` clears slot 0 after a fade opcode at `0x690`, before the next debug/source statement (raw dump).

## Differences

[code] Broadcast differs: Switch clears 40 slots, Steam 64 (T `0x61f0c`; S `0x1401018ae`). A negative clear can affect Steam-only slots.

## Unverified

[inferred] Resource cleanup completion and parent/child placement after a clear are not established by the immediate reset (T `0x62038`; S `0x140101ce3`). Clear a moving parent with a visible child while recording slot state and the next rendered frame in both engines, including a broadcast clear with a Steam-only slot populated.
