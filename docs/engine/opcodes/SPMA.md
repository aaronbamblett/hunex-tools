# SPMA

Source spelling: `_SPMA(1)`.

## Operands

- Operands are zero-based.

- Operand 0 pauses when equal to 1, otherwise resumes (`0x96fd4…0x96fe4`).

- [code] Opcode `0x27a`, S `0x140105290`, pops a dword and visits global command records selecting slot <64 and class exactly `0x20`.

- It marks the slot dirty (`+0x17c`) and stores the operand at `+0x198` (`0x140105322…0x140105334`), returning zero.

- Update `0x14016b19a…0x14016b1ee` propagates pause into animation state +0x1c when no new animation id is supplied.

- [data] Native Mahoyo has **3** matching opcode occurrences across 250 supplied CHS files, all dumped successfully.

## Switch build

[code] Dispatch T `0x794ec` calls `0x96f90`. Operand 0 pauses when equal to 1, otherwise resumes (`0x96fd4…0x96fe4`). It iterates all 40 slots and changes two per-slot animation states through `0x92e6c`, which stores pause at state +8. Time update `0x92e74…0x92e94` does not advance paused states. It does not itself wait or request audio. Switch wait query `0x33cf4…0x33d0c` considers animation busy only when active and unpaused; other transform components can still be busy.

## Steam build

[code] Opcode `0x27a`, S `0x140105290`, pops a dword and visits global command records selecting slot <64 and class exactly `0x20`. It marks the slot dirty (`+0x17c`) and stores the operand at `+0x198` (`0x140105322…0x140105334`), returning zero. Update `0x14016b19a…0x14016b1ee` propagates pause into animation state +0x1c when no new animation id is supplied. Recipe `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:6836` includes the general integer control recipe.

[data] Native Mahoyo has **3** matching opcode occurrences across 250 supplied CHS files, all dumped successfully. First: `/var/tmp/tsukihime-port/mahoyo-steam/raw/wik_z_3.chs`, assembly-dump line 523. These are static counts, not reachability.

## Differences

[code] Switch visits both animation states in every slot; Steam visits existing class-0x20 records and stores the raw dword. Source values are only 0/1: 189 pauses and 10 resumes (`/var/tmp/tsukihime-port/phase4/scripts/scr*.txt:1`, 549-file survey). [inferred] Exact treatment of newly created animations after pausing is not proved. A later STCF can still wait on another component; Steam's type-2 wait branch already has state-flag exceptions, so “Steam never recognizes paused animations” is unsupported.

## Unverified

[inferred] One runtime check: at scr0088 statements 1527…1537 inspect live VM IP, wait slot/mask and matching records at S `0x14015d610`, compare active/paused components with Switch T `0x33cb0`, and resume to confirm the specific completion condition. Static screenshots do not identify one exact STCF.

---

Engine facts here are lifted from the port's `docs/commands/SPMA.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
