# IFP4

Source spelling: `_IFP4(,LV00200)`.

## Operands

- `empty,label`.

## Switch build

[code] `empty,label` enters T `0x7898c`, scans to the comma, and follows the shared label-copy/jump path `0x7aabc–0x7c310`.

- [code] For the corpus empty-first-field shape, there is no platform predicate before the jump. The handler directly skips to the label, changing the continuation pointer; no direct draw/audio or condition wait (`0x7898c–0x78ac0`, `0x7c304`).

## Steam build

- [code] H `_IFP4:0` uses platform value 0 and opcode `0x131`; H `compile::lower_special` compares that value to the opcode result and jumps on false. S `0x140109a30` pushes zero unconditionally. Thus this compiled form falls through. The instructions only manipulate the operand stack and IP, returning 0 without audiovisual work or a wait (S `0x140109a30–0x140109a4f`, `0x14007cb00`). [data] Mahoyo uses `0x131` 39 times across 250 raw files, including `raw/1_1.chs:0x401c`; the opcode serves both platform-source spellings, so these are not exclusive IFP4 counts.

## Differences

- [inferred] A later ZY branch may reconverge, but that does not undo commands dispatched in the intervening block (`scr0008.txt:1`, statements 521–525).

## Unverified

- [inferred] One paired scr0008 trace should record the next STCH after statement 521 in each engine and capture the resulting geometry. Visible geometry parity remains unverified (T `0x7898c`; S `0x140109a30`).
