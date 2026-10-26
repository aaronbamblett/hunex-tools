# SESP

Source spelling: `_SESP(-1,)`.

## Switch build

[code] Stops SE immediately. Dispatch T `0x7921c–0x79228` selects inline handler `0x79874`: read operand 0, narrow to signed 16 bits, reduce modulo 20. A negative channel expands to all channels 0..19 (`0x79880–0x798c8`). Calls stop `0x9a7a0` per selected channel unless the engine gate suppresses it, clears ids through `[0x351a10]` and remembered/queued SE records at base `0x1354e58`, then continues dispatch (`0x79910–0x79970`). It does not poll natural sound completion or wait for a loop to end. The corpus's optional second field is empty (`dump/scr0140.txt:1`, `scr0439.txt:1`).

## Steam build

[code] Opcode `0x194`, S `0x140107ab0`; H `_SESP:0` emits `(duration=0, delay=0, channel)` in push order. Handler reads channel and two control values; negative channel explicitly iterates 20 SE records (`0x140107af5–0x140107f17`). Single-channel path bounds the vector at state `+0x330/+0x338`. Each selected record gets active `+0`, stop flags `+0x70/+0x74`, duration `+0x78`, delay `+0x7c`; it returns 0 with no task (`0x140107b75–0x140107b90`, `0x140107f36–0x140107fa3`). Actual stopping is applied through native pending-state processing, rather than a blocking script call.

[data] Mahoyo has 140 calls; `raw/1dot5_1.chs` file `0xc7b0` follows a music-stop opcode and uses channel 0 with both controls 0.

## Differences

[code] Switch explicitly clears queued state in addition to stopping (T `0x79928–0x79944`); Steam records pending stops, with negative-channel handling separate from SEFD's single-channel handler (S `0x140107ab0`, `0x140107fb0`). Switch modulo wrapping differs from Steam bounds checks for out-of-range positive channels. [inferred] A delayed start could race a pending stop differently; do not replace SESP with an SE completion wait, especially for loops (T `0x79950–0x79970`; S `0x140107f9f`).

## Unverified

[inferred] Schedule delayed looping SE on two channels, issue the all-channel stop, then wait past the scheduled onset. Compare silence, cancellation and next-statement timing on Switch and Steam; this resolves deferred-state clearing and pending-stop latency (T `0x79910–0x79970`; S `0x140107b30–0x140107f17`).
