# STRA

Source spelling: `_STRA(12,1)`.

## Operands

- Operand 0 defaults to 0; -1 broadcasts to all 40 slots, 0–39 selects one, other values exit (`0x6442c–0x64444`, `0x6447c–0x64490`).
- Operand 1 is a per-slot auxiliary animation/effect selector, default 0, saved as a halfword at `+0x1364 + slot*2`.
- Operand 2 is duration, default 0; it is sign-extended as a halfword before conversion to `trunc(60*n/1000)` ticks (`0x64448–0x644f4`).

## Switch build

[code] T dispatch `0x78030–0x78038` calls `0x643a0`.

[code] The setter writes live fields only when slot `+0x14` is positive: nonzero selector with zero duration stores selector at `+0x238`, clears `+0x23a` and writes the counter at `+0x23c`; zero selector uses `+0x23e/+0x240`. Timed branches use the corresponding counters without the same immediate selector store (`0x644f8–0x64650`). It returns 0 and dispatch continues (`0x64654–0x64664`, `0x78038`). [inferred] This controls a separate sprite effect, not STMA's keyframe id: neither the handler nor its dispatch calls the STMA setter `0x92e44`. The exact visible effect and selector table remain unverified; no wait or audio start appears in this setter.

## Steam build

[code] Opcode `0x258` → S `0x140103150`, registration `0x1401068a0–0x1401068ae`. Recipes `_STRA:0` accept two or three integers and default duration to 0. Negative slot broadcasts over 64 slots; a single slot below 64 writes request `+0x168`, disable flag `+0x16c=(selector<=0)`, selector `+0x170`, duration `+0x174` (`0x140103150–0x140103959`, single-slot path `0x14010390c–0x140103947`). Returns 0 (`0x140103952`); it does not create a completion barrier.

[data] Mahoyo: 54 occurrences; `1dot5_3.chs@0x217c` supplies slot -1, selector 1 and duration 0 after STCP.

## Differences

[code] Source limits broadcast to -1 and 40 slots, narrows selector/duration and gates live writes on slot state; Steam accepts any negative slot for a 64-slot broadcast and retains dword selector/duration requests (T `0x64438–0x6454c`; S `0x140103150–0x140103959`). [inferred] A Tsukihime selector is not demonstrated to name the same effect in Steam.

## Unverified

[inferred] The source/Steam selector consumers and precise visible effect, duration envelope and save/restore semantics remain unverified (T live `+0x238..+0x240`; S `+0x170/+0x174`). Capture the actual scr0001 selector-1 sequence and a timed stop while logging these fields and their consumers in both engines; compare effect output and expiry without assuming the selector tables correspond.
