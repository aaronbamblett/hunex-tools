# SPFC

Source spelling: `_SPFC(29,aki_a_30_01_n01,,,,,12,)`.

## Operands

- Operand 0's full role and the names of the two adjustment controls remain unresolved.

## Switch build

[code] Dispatch T `0x79308…0x79368` reads name operand 1 via `0x56800`, then integer fields 4/5 used to construct a variant name (`0x7a1f4…0x7a264`). Registry miss returns without changing state (`0x7a268`). Field 6 defaults 0; fields 2/3 default 0 (`0x7a274…0x7a28c`, `0x7a428…0x7a47c`); field 7 is a string, default empty, parsed for tint (`0x7a488…0x7a4b0`, default `0xffffff` at `0x7ac6c`). It reuses/allocates a record in the 30-entry arrays at `0x1355058`, suppressing duplicate requests with equal id/fields (`0x7acc8…0x7acfc`). Resource setup at `0x7b3b0…0x7b404` passes field 6, `1+arg2/255`, `1+arg3*0.01` and tint to `0xb5600`, then stores them. It continues dispatch without waiting or audio.

## Steam build

[code] Opcode `0x222`, S `0x140100230`, pops slot, name and six integer fields, builds a preparation record and appends it to global context `+0x270` (dirty at `+0x268`, insert call `0x1400ecd50` at `0x1401004d1`), returning zero. This is a global request list, not the SPRD per-slot block. Compiler recipes encode image/symbol, adjustment, variant, field-6 and optional color forms: `_SPFC:0`, generic forms. Full target resource consumer is unresolved.

[data] Mahoyo: 84 call(s) (`raw/1_2.chs`, assembly-dump line 12847). These are static counts, not reachability.

## Differences

[data] Source arities are 482 eight-field, four nine-field and one seven-field statements (`dump/scr*.txt:1`, 549-file survey). [code] The source shared 30-entry pool and target queued record list differ (T `0x7ac84…0x7acfc`; S `0x1401004ab…0x1401004d1`). [inferred] Variant names, tint packing, duplicate suppression, resource limits, the ninth tail and capture timing need reconciliation; successful compilation does not establish those semantics.

## Unverified

[inferred] Operand 0, full adjustment meanings, ninth-field behavior and Steam resource consumer remain unresolved. Replay scr0202 statement 1240 and its immediately following SPFC, logging resolved variant names, tint/adjustment values and resource handles through the next SPRC; compare pixels and duplicate-resource behavior with Switch (T `0x7b3b0`; S `0x140100230`).
