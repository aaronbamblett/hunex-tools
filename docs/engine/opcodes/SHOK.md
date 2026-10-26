# SHOK

Source spelling: `_SHOK(1,3,100,18)`.

## Operands

- Operands are `(xAmp,yAmp,period,repeat,delay)`: amplitudes default 0, period defaults 300 ms (`0x5f548`), repeat defaults 3 and delay defaults 0; operand 4 uses signed-16 conversion (`0x5fab8`).

## Switch build

[code] Dispatch T `0x7927c…0x79294` invokes `0x5f260` unless the global suppression flag is set. T `0x30020(1,x,y,period,repeat,delay)` stores a pending delayed request at `0x9e5368/0x9e536c`, or immediately initializes global shock state `0x9e52a8`: active flag, counters, period `3*p/50` frames (zero becomes 1), repeat at least 1, amplitudes and initial phase. Update T `0x300e0` consumes it. T `0x5fb04` returns zero: no script wait or audio request. The precise waveform needs a complete consumer trace.

## Steam build

[code] Opcode `0x340`, S `0x1401054e0`, pops five integers in script order into global context: x → `+0xb0`, y → `+0xb4`, period → `+0xbc`, repeat → `+0xc0`, delay → `+0xc4`; it resets elapsed/request fields and sets `+0x70/+0x74/+0xa8/+0xc8`. It returns zero. The global sync calls effect startup S `0x140144be0` at `0x14014a8ba`. Recipes: `_SHOK:source`; the four-field recipe supplies delay 0.

[data] Mahoyo: 87 call(s) (`raw/1_3.chs`, assembly-dump line 9661). These are static counts, not reachability.

## Differences

[code] Switch stores signed-16 operands and converts period to frames; Steam receives dwords and uses its own global effect task (T `0x5f260`, `0x30020`; S `0x1401054e0`, `0x140144be0`). [inferred] Equal operands do not prove equal amplitude scaling, phase, repeat endpoints or delay. SHAF can interrupt this state; neither command waits for completion.

## Unverified

[inferred] Exact Steam waveform, coordinate scaling and repeat/expiry boundary remain unverified. Capture scr0036 statement 1637 through its full repeat count on both engines, measuring displacement each update and then issuing SHAF to compare clearing (T `0x300e0`; S `0x140144be0`).
