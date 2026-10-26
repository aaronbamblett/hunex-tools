# SPRD

Source spelling: `_SPRD(31,img0082,0,0,2,2)`.

## Switch build

[code] Dispatch T `0x7750c…0x77578` reads image/name operand 1 through `0x56800`, parses integer operands 2…5 (defaults 0), and later slot operand 0 (default −1). It iterates the first/third endpoints and second/fourth endpoints with step `max(end-start,1)` (`0x79dd0…0x79f58`), builds image-piece names (`0x79f58…0x7a048`), resolves them through `0x56d40/0x33600` and allocates resource records in the 30-entry arrays starting `0x1355058`. Request `0xb5600` at `0x79f28` associates the selected piece with slot-derived saved values `+0x1404/+0x1454/+0x1458`. It continues to dispatch, without installing a script wait or requesting audio. [inferred] This prepares image pieces for later sprite drawing, rather than a timed screen transition.

## Steam build

[code] Opcode `0x220`, S `0x1400ff920`, pops `(slot,name,selector2,selector3,selector4,selector5)`, checks slot ≤63 and writes a per-slot preparation request: `+0x1c8` dirty, filename `+0x1d0`, selector fields `+0x208/+0x20c/+0x210/+0x214` and request flags `+0x218`. It inherits unspecified transform components from the slot (`0x1400ffa91…0x1400ffb33`) and returns zero (`0x1400ffc21`). Recipes `_SPRD:0` accept image or symbol names. Full piece-allocation consumer is unresolved.

[data] Mahoyo: 32 call(s) (`raw/1_2.chs`, assembly-dump line 27630). These are static counts, not reachability.

## Differences

[code] Switch uses shared 30-entry resource bookkeeping and can read saved slot fields; Steam queues the preparation on an individual slot (T `0x7a064…0x7a0d0`; S `0x1400ffb33…0x1400ffc09`). [inferred] Resource limits, generated filename variants and default state may differ. This handler itself does not wait; any later resource wait must be traced separately.

## Unverified

[inferred] Complete generated-piece naming, Steam preparation consumer and resource-readiness semantics remain unresolved. Replay scr0083 statements 115…169 with resource logging, showing each requested piece before SPRC; compare filenames, selected pieces and releases on both engines (T `0x79f58`; S `0x1400ff920`).
