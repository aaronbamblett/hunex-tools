# SACL

Source spelling: `_SACL(29,1)`.

## Switch build

[code] T `0x7974c…0x797b4` reads slot (default 0) and a boolean (default false), setting bit `0x40` at saved state `+0x54+slot` for any nonzero value; `0x79bb4…0x79bc0` clears it for zero/absent. There is no explicit bounds check in this block. Draw T `0x86394…0x8642c` skips one displacement-addition block when bit 6 is set. It continues directly to dispatch, with no wait or audio. [inferred] This locks the sprite against that inherited/global displacement contribution; the producer's complete identity is not established.

## Steam build

[code] Opcode `0x25a`, S `0x140103a40`, accepts slots 0…63, normalizes operand 1 to boolean, stores slot `+0xac` and marks transform dirty, returning zero. Sync `0x14014a282…0x14014a2b4` calls `0x140078a10`, which stores the inverse at node `+0x1b8`. Position calculation `0x14007238c…0x140072442` conditionally adds a referenced object's displacement when this inverse flag is true. Recipe `_SACL:0` takes two fields.

[data] Mahoyo: 412 call(s) (`raw/1_0.chs`, assembly-dump line 506). These are static counts, not reachability.

## Differences

[code] Source stores a saved bit; Steam uses a dirty request plus inverse renderer flag, and accepts more slot ids (T `0x79790`; S `0x140103a6b`, `0x140078a10`). [data] All 450 source calls set operand 1 to 1 (`dump/scr*.txt:1`, 549-file survey). [inferred] The corpus does not exercise explicit unlock; effect layering and inherited displacement may differ despite matching boolean behavior.

## Unverified

[inferred] Which global/parent displacement is excluded, persistence, and reset defaults remain unverified. Put two identical sprites under the same global shock, apply SACL only to one as at scr0017 statement 1223, then clear/reset it and compare motion on both engines (T `0x86394`; S `0x14007238c`).
