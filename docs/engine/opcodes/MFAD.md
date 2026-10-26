# MFAD

Source spelling: `_MFAD(,,`001:4000,,`011:0)`.

## Switch build

[code] Fades/stops current music. Handler T `0x64da0` reads tag 001 as signed-16-bit duration and tag 011 as wait control (`0x6503c–0x650cc`, `0x65134–0x65148`). Positive duration computes a fade rate from remembered BGM volume and calls `0x9a1d0`; nonpositive duration stops through `0x99e30`. Remembered track becomes -1 at `0x65130`/`0x65194`. Audio state at `0x98cf0(-1)` is changed to fade/stop mode by `0x9a1d0`.

[code] Explicit tag 011:0 returns 0 without yielding (`0x65138–0x65184`). Missing/nonzero wait control writes an approximately duration/16 countdown through `[0x351508]` and returns 1 (`0x6514c–0x65174`); it is a timed script yield, not a clip-end predicate. The dispatcher branches on that return at `0x78404`.

## Steam build

[code] Opcode `0x185`, S `0x1401072e0`; H `_MFAD:0` emits pushes `(wait, duration, unused=0)`. The handler discards the top field, reads duration and wait, and selects the current BGM channel from global `0x1405aaf28`. It marks pending music record `+0/+0x70/+0x74`, stores duration at `+0x78`, and returns 0 for wait=0 (`0x140107308–0x14010739a`, `0x1401074a4`). Nonzero wait constructs task vtable `0x140283d00`, channel=current / kind=1 / mask=2, and yields via `[ctx+0x24e8]` (`0x14010740a–0x1401074c7`).

[code] That task polls tracked request status through S `0x140123b10`, not playback directly; skip/direct-mode and cancellation paths can release it (`0x14013e330–0x14013e4c1`). [data] Mahoyo has 328 calls; `raw/1_0.chs` file `0x66d0` uses wait 0 / duration 10000 / unused 0.

## Differences

[code] Switch's wait-enabled branch counts down script time, whereas Steam's task checks audio requests (T `0x6514c–0x65174`; S `0x14013e3cf`). Switch duration is narrowed to signed 16 bits at `0x650b8`; Steam stores a dword at `0x140107394`. [inferred] Out-of-range durations can diverge. Request completion could fail to release if its underlying request remains pending; no deadline is established by this task. These wait risks are outside the observed 011:0 corpus shape.

## Unverified

[inferred] Record the cited four-second fade with following SE/voice, comparing the gain envelope and next-statement timing against Switch; include a wait-enabled variant in the same comparison to expose countdown versus request-completion release. Engine audio must remain active even if the output device is muted (T `0x65124`, S `0x140107398`).
