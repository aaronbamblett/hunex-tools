# FANX

Source spelling: `_FANX(1,0,0)`.

## Operands

- No operands are parsed, no command state is written, no wait is installed and no visible/audible effect is requested by this spelling in the supplied executable.

- That handler advances the operand stack cursor by six cells, does not read their values, returns 0 and creates no fade task (`0x1400ff8e0–0x1400ff911`).

- [data] Mahoyo raw corpus: 154 opcode calls; `raw/1dot5_3.chs@0x3cf98` pushes 0,256,0,0,0,1 before the call.

## Switch build

[code] The F dispatch compares only ADE and ADS at T `0x76978–0x76994`; FANX follows the non-match branch to `0x76730`. No operands are parsed, no command state is written, no wait is installed and no visible/audible effect is requested by this spelling in the supplied executable.

## Steam build

[code] R `recipes_data.rs:522–636` accepts only literal shapes (1,0,0) and (1,0,0,0), supplies six stack values, and emits opcode `0x201`. Registration S `0x1401066d2–0x1401066e0` binds `0x1400ff8e0`. That handler advances the operand stack cursor by six cells, does not read their values, returns 0 and creates no fade task (`0x1400ff8e0–0x1400ff911`).

[data] Mahoyo raw corpus: 154 opcode calls; `raw/1dot5_3.chs@0x3cf98` pushes 0,256,0,0,0,1 before the call. The opcode's proximity/name does not make it a nonblocking FADS implementation.

## Differences

[inferred] Treating FANX as an asynchronous transition would be unsupported: both inspected paths do no transition work (T `0x76994`; S `0x1400ff8e0`). [code] Compiler support is restricted to two literal shapes even though the Steam handler ignores values (R `recipes_data.rs:524`).

## Unverified

[inferred] Scheduler-boundary effects of many no-op commands have not been measured. One runtime check: step scr0001 statement 212 and confirm no fade task/window change in either engine before the following STMA (T `0x76730`; S `0x1400ff8e0`).

---

Engine facts here are lifted from the port's `docs/commands/FANX.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
