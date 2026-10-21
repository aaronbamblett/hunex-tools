# WTSV

Source spelling: `_WTSV()`.

## Operands

- No operands are evaluated, no save-completion predicate is polled, and no window/audio state or wait is installed by this command.

## Switch build

[code] T W-family comparisons `0x7805c–0x78078` do not recognize WTSV and continue at `0x76730`. No operands are evaluated, no save-completion predicate is polled, and no window/audio state or wait is installed by this command.

## Steam build

[code] There is no WTSV compiler recipe in R `recipes_data.rs`' command inventory and no matched opcode/handler. The comparison engine's no-op continuation is recorded at Mahoyo Switch `0x57508`. [inferred] That is not evidence of a Steam save-wait implementation.

[data] Raw CHS cannot provide an occurrence count for a source command that lowers to no matched opcode; zero named matches would not prove absence of save behavior elsewhere. The exact reference is the compiler command inventory (R `recipes_data.rs:10713–11090`).

## Differences

[inferred] Its spelling and placement after SELA must not be used to invent an autosave completion wait. The following explicit WTTM is a separate command with its own timing contract (T `0x7805c–0x78078`). Other save commands still need their own liveness analysis.

## Unverified

[inferred] One runtime check: trace scr0004 statement 57 across WTSV and the following WTTM, logging save activity and script cursor; WTSV should add no frame/task dependency on Switch (T `0x76730`). Any save wait observed elsewhere must be attributed to its actual command.

---

Engine facts here are lifted from the port's `docs/commands/WTSV.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
