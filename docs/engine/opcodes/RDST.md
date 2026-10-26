# RDST

Source spelling: `_RDST(n)`.

## Operands

- Operand 0 is an index; presence and unsigned range `0..575` are checked at `0x8fb0c–0x8fb24`.

## Switch build

[code] Dispatcher T `0x7795c–0x77974` calls `0x8faf0`. Missing, negative or larger values make no write. Valid input ORs `1 << (index & 15)` into the persistent halfword bank at `0x1252faa + 2*(0x6e + (index >> 4))` (`0x8fb28–0x8fb50`; base pointer `0x351550`).

[code] It returns zero and resumes ordinary dispatch, with no task or direct drawing/audio call (`0x8fb54–0x8fb60`, `0x77974`). Getter T `0x8fb70–0x8fba0` reads the same bank. This bank differs from RDED's completion bank at word `0x12c` and RTST's local session bank (`0x8f7b0`, `0x8fc30`).

[inferred] It records an entered/read marker rather than the validated completion operation performed by RDED; the full native consumer/lifetime meaning remains unresolved (T `0x8faf0`, `0x8fb70`, `0x8f910`).

## Steam build

[code] Compiler `_RDST(n)` pushes n and calls opcode `0x128` (`_RDST:0`). Handler S `0x140109910` consumes the index and, if the VM owner exists and index is below the container's bit count, sets a bit in the buffer pointed to by container+0x10, byte `index>>3`, bit `index&7` (`0x140109927–0x140109962`). Container is reached through VM+0x10 → owner+0xc8 → +0x28. It returns 0 immediately at `0x140109964`; there is no visual/audio or wait task here.

[data] Mahoyo uses `0x128` 205 times; `raw/1_0.chs:0x7bdc` sets index 0 after a timer sequence. These are native read-bit IDs, not demonstrated Tsukihime bank assignments (250 successful dumps; README).

## Differences

[inferred] Numerical index preservation does not establish identical state ownership: Switch's bank at persistent word `0x6e` and Steam's native bit container need a storage/consumer/lifecycle mapping. It does not restore the session and completion banks (T `0x8fb40`, `0x8f910`; S `0x14010992c`).

[code] Range handling differs: Switch hard-limits 575, whereas Steam uses the runtime container's bit count (`0x8fb20`; S `0x14010993f`). [data] Authored RDST indices span 1..562 (`dump/*.txt:1`). [inferred] Native skip/UI readers could interpret those IDs differently; exact behavior remains unverified rather than safely calling this pass-through exact (same addresses).

## Unverified

[inferred] Native Steam bit capacity, durable lifecycle and every reader of this bank remain unresolved. Enter scr0001 fresh, observe Switch's word `0x6e+(486>>4)` and Steam bit 486 before/after RDST, then save/reload/restart and compare skip/UI eligibility while keeping completion unset (T `0x8fb70`; S `0x140109910`).
