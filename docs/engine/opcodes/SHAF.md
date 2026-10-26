# SHAF

Source spelling: `_SHAF()`.

## Operands

- Operands are zero-based.

- With no operand it invokes `0x2f760(0,0,0,0)`, clearing global shock active/counter/output state at `0x9e52a8` and saved halfword `+0x19ae` (`0x6010c…0x6012c`).

- An optional operand instead calls `0x2fbf0` (`0x60138…0x60154`); its semantics are outside the corpus form established here.

- [data] Native Mahoyo has **193** matching opcode occurrences across 250 supplied CHS files, all dumped successfully.

## Switch build

[code] T `0x79740` calls `0x600d0`. With no operand it invokes `0x2f760(0,0,0,0)`, clearing global shock active/counter/output state at `0x9e52a8` and saved halfword `+0x19ae` (`0x6010c…0x6012c`). An optional operand instead calls `0x2fbf0` (`0x60138…0x60154`); its semantics are outside the corpus form established here. It returns zero and has no audio request or completion wait.

## Steam build

[code] Opcode `0x341`, S `0x1401055a0`, consumes one stack field without using its value, sets global dirty/stop flags at context `+0x70/+0x74/+0xa8/+0xac` and returns zero. The source-form compiler recipe supplies a dummy 0 to this handler: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:5125`.

[data] Native Mahoyo has **193** matching opcode occurrences across 250 supplied CHS files, all dumped successfully. First: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_0.chs`, assembly-dump line 30. These are static counts, not reachability.

## Differences

[data] All 397 corpus calls are operand-free (`/var/tmp/tsukihime-port/phase4/scripts/scr*.txt:1`, 549-file survey). [code] Switch directly clears shock state while Steam queues stop flags (T `0x2f760`; S `0x1401055a0`). [inferred] The same update may retain one engine's previous displacement longer. This does not clear the two per-slot quake banks; source scene starts issue SQS2/SHS2/SSS2 separately (`scr0001.txt:1`, statements 12…15).

## Unverified

[inferred] One runtime check: issue SHAF midway through a SHOK with a slot quake also active, record the first zero-global-displacement frame and continued slot motion in both engines (T `0x2f760`; S `0x1401055a0`). Optional-operand semantics and native reset/save timing remain unresolved.

---

Engine facts here are lifted from the port's `docs/commands/SHAF.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
