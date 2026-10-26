# CLO3

Source spelling: `_CLO3()`.

## Switch build

[code] Operand-free close request at T `0x7c394`: if open-word `0x9e5280` is zero, return immediately. Otherwise clear that word, set closing byte `0x9e5284`, and set message suppression `+0x89e=1`, then return the next statement at `0x7d00c`. It does not wait for closure. Alpha consumer `0x39d20–0x39d40` subtracts 16.666666/64 per tick from current alpha at `0x9e527c`, clamps to zero and clears the closing byte. The next message clears suppression at `0x6dc68`.

[inferred] Nominal full-alpha close time is 64 ms; a partially transparent window takes less time. Message rendering is suppressed until the next message (T `0x3ac9c–0x3acc4`).

## Steam build

[code] No CLO3 recipe exists. The Steam CLO4 handler writes close fields `+0x2cc..+0x2e0` and returns 0. Positive-duration requests start at full alpha if the open flag is set; already-closed requests still write fields (`0x14010b3da–0x14010b42f`).

[code] Close interpolation mode `0x10001` doubles elapsed/duration and clamps it (S `0x1400913f8`), so duration 128 requests nominal 64 ms.

[data] Mahoyo raw corpus has no CLO3 opcode; the carrier CLO4 opcode occurs 2,996 times, e.g. `raw/1_0.chs@0x320`.

## Differences

[code] Switch starts from current alpha and does nothing when closed; Steam may restart from full alpha and still writes close fields when closed (T `0x39d20`; S `0x14010b3e2–0x14010b3e9`). [inferred] A mid-fade close can brighten briefly. Suppression persistence through save/load is unmeasured. Substituting CLO2 would introduce a wait absent on Switch (S `0x14010b386`).

## Unverified

[inferred] Record scr0001 statement 544 at 30 fps, including repeated/mid-fade close and save/load before the next message; measure full-alpha-to-zero time and confirm glyphs stay hidden while subsequent script/audio continues (T `0x7c394`; S `0x14010b3b0`).
