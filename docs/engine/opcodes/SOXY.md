# SOXY

Source spelling: `_SOXY(14,12,12,12,12)`.

## Operands

- Operands are zero-based.

- [code] Dispatch T `0x799d0` reads slot (operand 0, default 0), then only operands 3 and 4 (default 0), and stores signed halfwords at saved state `+0x98c+slot*2` and `+0x9dc+slot*2` (`0x79a04…0x79a20`, `0x79d48…0x79d84`).

- Operands 1/2/5/6 are not read on this dispatch path.

- Restore copies these fields into live slot state (`0x7e3b8…0x7e3cc`); draw consumes the second through integer division by 3 and a position addition (`0x86430…0x8649c`).

- [code] Opcode `0x259`, S `0x140103960`, consumes seven fields but stores only slot operand 0 plus operands 1,2,5: slot `+0x70=arg1`, `+0x74=arg2`, `+0x78=arg5`; it marks slot and transform dirty and returns zero (`0x1401039a5…0x140103a3c`).

- [data] Native Mahoyo has **182** matching opcode occurrences across 250 supplied CHS files, all dumped successfully.

## Switch build

[code] Dispatch T `0x799d0` reads slot (operand 0, default 0), then only operands 3 and 4 (default 0), and stores signed halfwords at saved state `+0x98c+slot*2` and `+0x9dc+slot*2` (`0x79a04…0x79a20`, `0x79d48…0x79d84`). Operands 1/2/5/6 are not read on this dispatch path. Restore copies these fields into live slot state (`0x7e3b8…0x7e3cc`); draw consumes the second through integer division by 3 and a position addition (`0x86430…0x8649c`). It continues dispatch without waiting or audio. [inferred] These are draw-offset controls; the full axis/coordinate interpretation of both fields is unresolved.

## Steam build

[code] Opcode `0x259`, S `0x140103960`, consumes seven fields but stores only slot operand 0 plus operands 1,2,5: slot `+0x70=arg1`, `+0x74=arg2`, `+0x78=arg5`; it marks slot and transform dirty and returns zero (`0x1401039a5…0x140103a3c`). The 1…7-field recipe pads 1…4 with zero and 5/6 with one: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:5428`. No completion task is created.

[data] Native Mahoyo has **182** matching opcode occurrences across 250 supplied CHS files, all dumped successfully. First: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_1.chs`, assembly-dump line 2120. These are static counts, not reachability.

## Differences

[code] Operand selection differs: Switch uses fields 3/4, Steam 1/2/5 (T `0x79d7c/0x79d80`; S `0x140103a24…0x140103a2c`). [data] 71 of 208 source calls have textually unequal 1/2 versus 3/4 pairs; the other 137 being textually equal does not prove coordinate equivalence. First unequal call: scr0049 statement 583 (`/var/tmp/tsukihime-port/phase4/scripts/scr0049.txt:1`). [inferred] The operand mismatch can apply a different offset even with valid syntax; scaling/axis interpretation still needs tracing.

## Unverified

[inferred] Exact Steam node consumer, full source first-field interpretation, and whether the offset is relative to pivot or image origin remain unresolved. One runtime check: replay scr0049 statement 583 on both engines with a visible marked pivot, recording node position and sprite edges immediately before/after; distinguish operand-pair choice from scaling (T `0x79d48`; S `0x140103960`).

---

Engine facts here are lifted from the port's `docs/commands/SOXY.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
