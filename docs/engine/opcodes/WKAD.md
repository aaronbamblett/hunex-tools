# WKAD

Source spelling: `_WKAD(F823,1)`.

## Switch build

- [code] Add `destination,value`; dispatcher T `0x786e4`, relative C address `0x7aa14–0x7aa28`, read/add/store `0x7aa6c–0x7aa7c`. Addition wraps modulo 65,536 and continues immediately; it does not directly draw, play or wait. [code] F and G are local/persistent 16-bit words; C<n> adds signed F823 to n before addressing F; underscore compatibility indices 600..700 address G[n−600], otherwise F[n] (T `0x7aa14–0x7aa78`). `_D000` aliases F000 rather than a separate D allocation.

## Steam build

- [code] Generic addition is opcode `0xeb`, S `0x14007dce0`: pops bank, index, increment, resolves bank 2's 600..700 alias, then adds to a 32-bit F/G word. Bank 0 uses the 1,280-word VM allocation; bank 1 uses the 256-word system allocation. Out-of-range indices do nothing; return is 0, with no direct wait/audio/draw (`0x14007dd26–0x14007dda7`; S `0x14007a921`, `0x1400cea03`). [code] H `_WKAD:0` treats F823+=1 as compile-state F823=1 with no emitted arithmetic. F847..F853+=1 emit `0x142`, which updates native message state rather than these VM variables (H; S `0x14010b120–0x14010b1bf`). A generic flag/integer recipe now exists (H). The older compiler-gap list is stale for that checkout. [data] Mahoyo uses `0xeb` 10 times and message opcode `0x142` 3,285 times. Example: `raw/wik_select_a.chs:0x360` adds 1 to F020; `raw/1_0.chs:0x6f8` calls `0x142` (250 raw CHS dumps).

## Differences

- [code] Both the destination and width can change: Switch C847 with F823=1 updates F848; renamed F847+=1 emits a native message operation. Generic Steam addition does not truncate to 16 bits (T `0x7aa1c–0x7aa78`; S `0x14007dd9d`; H `_WKAD:2`). [inferred] Arithmetic route counters and message counters need separate validation.

## Unverified

- [inferred] Whether the specialized `0x142` task reproduces every source page/counter pattern is unresolved. One paired trace of a page with both F823+=1 and C847+=1, followed by auto/skip and a save/load, should compare the relative words and native message progress (T `0x7aa14`; S `0x14010b120`).
