# SEPL

Source spelling: `_SEPL(0,1,MOON_SE03001,,`001:4000,,`017:50)`.

## Operands

- [code] Starts/replaces a sound effect in source `(channel, loop/control, sound stem, tagged suffix)`.

- Channel is signed-16-bit, reduced modulo 20 at `0x6a418–0x6a440`; operand 1 is tested for loop/reuse at `0x6a710–0x6a7c0`.

- [code] Tag 001 is fade duration, 017 volume clamped to 0..100 (absent sentinel -1 retained separately), 018 pan (64-centered, mapped by `(value-64)*1000/64`), 068 deferred execution, 070 sound/voice category (`0x6a1d4–0x6a230`, `0x6a460–0x6a658`, `0x6a7e0–0x6a898`).

- Tag 068 routes to `0x65fe0` with action 1; this schedules per-channel state rather than waiting.

- [code] Tag 070 controls per-character configuration: category is stored at sound-channel `+0x34` by `0x9a780`; mask/master/per-category gains can mute/scale it instead of ordinary SE gain (T `0x6a834–0x6a898`, `0x9a2c0`/`0x9a410`/`0x9a560`).

- R `recipes_data.rs:3537` emits reverse-order pushes, defaulting delay/fade to 0, pan to 64, volume to 100.

- Handler bounds the channel against state `+0x330/+0x338`, selects a 0xa8-byte record, writes filename and flags, loop `+0x3c`, volume `+0x40`, pan `+0x44`, fade `+0x54`, delay `+0x58`; returns without a wait (`0x140107678–0x1401076c4`, `0x1401079d6–0x140107a72`).

## Switch build

[code] Starts/replaces a sound effect in source `(channel, loop/control, sound stem, tagged suffix)`. Handler T `0x69f10` scans tags rather than fixed suffix positions. Channel is signed-16-bit, reduced modulo 20 at `0x6a418–0x6a440`; operand 1 is tested for loop/reuse at `0x6a710–0x6a7c0`. Sound lookup `0x900` supplies a registry id at `0x6a72c`. Empty name follows a stop path at `0x6a664–0x6a6b4`.

[code] Tag 001 is fade duration, 017 volume clamped to 0..100 (absent sentinel -1 retained separately), 018 pan (64-centered, mapped by `(value-64)*1000/64`), 068 deferred execution, 070 sound/voice category (`0x6a1d4–0x6a230`, `0x6a460–0x6a658`, `0x6a7e0–0x6a898`). Tag 068 routes to `0x65fe0` with action 1; this schedules per-channel state rather than waiting. Immediate audio uses `0x9a2c0` or fade-play `0x9a410`, then pan `0x9a670`; remembered volume/pan/id/loop/category are retained around state base `0x1354e58` (`0x6a724–0x6a984`). Handler returns 0 (`0x6a98c`), so neither a looping clip nor delayed start blocks script execution.

[code] Tag 070 controls per-character configuration: category is stored at sound-channel `+0x34` by `0x9a780`; mask/master/per-category gains can mute/scale it instead of ordinary SE gain (T `0x6a834–0x6a898`, `0x9a2c0`/`0x9a410`/`0x9a560`).

## Steam build

[code] Opcode `0x190`, S `0x1401075b0`, seven stack fields: channel, loop, name, volume, pan, fade, delay (popped in that order). R `recipes_data.rs:3537` emits reverse-order pushes, defaulting delay/fade to 0, pan to 64, volume to 100. Handler bounds the channel against state `+0x330/+0x338`, selects a 0xa8-byte record, writes filename and flags, loop `+0x3c`, volume `+0x40`, pan `+0x44`, fade `+0x54`, delay `+0x58`; returns without a wait (`0x140107678–0x1401076c4`, `0x1401079d6–0x140107a72`). No category field is popped.

[data] Mahoyo has **4,687** calls; `raw/1_0.chs` file `0x4fc0` starts SEETC02 on channel 0, loop 0, volume 100, pan 64, fade/delay 0.

## Differences

[code] Switch wraps channel numbers modulo 20; Steam bounds against its vector. Switch volume/durations are narrowed and pan transformed; Steam stores dwords (T `0x6a418–0x6a648`; S `0x1401076b1`, `0x1401079d6–0x140107a0a`). Explicit zero fade and absent volume have distinct Switch branches. [inferred] Category settings, same-loop reuse, deferred scheduling, and explicit/default operand distinctions must not be inferred from audible success at defaults. SEPL itself is not a natural-completion barrier (T `0x6a98c`; S `0x140107a72`).

## Unverified

[inferred] One runtime comparison: use a category-070 SEV cue from scr0166, with character volume enabled/disabled and unequal to SE volume, plus an authored delayed/faded cue. Record onset, loop reuse, gain and pan against Switch. This distinguishes missing category semantics from delayed/mixer differences; static reading does not establish either audible match (T `0x65fe0`; S `0x1401079d6–0x140107a0a`).

---

Engine facts here are lifted from the port's `docs/commands/SEPL.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
