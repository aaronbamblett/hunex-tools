# CLO4

Source spelling: `_CLO4(1000)`.

## Operands

- [code] T `0x7c3c8–0x7c44c` parses optional duration (default 0).

- Duration is interpreted as milliseconds by the 60 Hz slope; complete mid-fade behavior still needs measurement.

- [code] R `recipes_data.rs:182–233` accepts no operand (emits zero) or one integer; opcode `0x152` is registered to S `0x14010b3b0`.

- [inferred] Thus reaching zero takes about half the supplied duration on that path.

## Switch build

[code] T `0x7c3c8–0x7c44c` parses optional duration (default 0). Alternate execution/skip flags can force an immediate request. Shared paths `0x7c728` and `0x7c884–0x7c8ac` write target alpha 0 at `0x125a510` and a negative slope derived from -16.666666/duration at `0x125a514`. It returns the next statement through `0x7d00c`; closure is asynchronous.

[inferred] This is a timed close, distinct from CLO3's fixed 64-ms close and suppression flag (T `0x7c394`, `0x7c890`). Duration is interpreted as milliseconds by the 60 Hz slope; complete mid-fade behavior still needs measurement.

## Steam build

[code] R `recipes_data.rs:182–233` accepts no operand (emits zero) or one integer; opcode `0x152` is registered to S `0x14010b3b0`. Positive values write close request `+0x2cc`, start `+0x2d4`, target zero `+0x2d8`, duration `+0x2dc`, easing `0x10001` at `+0x2e0`, then return 0. Nonpositive values take immediate-close path `0x14010b43b`. Open windows explicitly start at `0x10000`; closed windows use start -1/duration zero (`0x14010b3da–0x14010b42f`).

[code] Consumer `0x14014ab9e` forwards the fields; interpolation `0x1400913f8–0x140091410` clamps twice elapsed/duration. [inferred] Thus reaching zero takes about half the supplied duration on that path.

[data] Mahoyo raw corpus: 2,996 opcode calls; `raw/1_0.chs@0x320` is a zero-duration call.

## Differences

[inferred] The static easing comparison predicts authored closes can finish twice as fast on Steam (T `0x7c890`; S `0x1400913f8`). Steam's full-alpha restart can also differ during a partial fade (S `0x14010b3e2`). [data] Source numeric durations span 0–8000, and none is 128 (`dump/scr*.txt:1`).

## Unverified

[inferred] One runtime check: compare scr0001 statement 2760 on both engines while recording alpha and next-statement time, starting from full and partial opacity; verify whether duration 1000 reaches zero at 1000 or 500 ms (T `0x125a510/+4`; S `0x14014ab9e`).

---

Engine facts here are lifted from the port's `docs/commands/CLO4.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
