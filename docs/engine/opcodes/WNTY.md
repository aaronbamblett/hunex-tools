# WNTY

Source spelling: `_WNTY(1)`.

## Operands

- [code] T `0x78084–0x78094` evaluates operand 0 through `0x55520`, tests its low 16 bits, normalizes it to a boolean, and calls `0x39dd0`.

- That setter writes window/message mode at `0x9e52a0` and message-state halfword `+0x63e`.

## Switch build

[code] T `0x78084–0x78094` evaluates operand 0 through `0x55520`, tests its low 16 bits, normalizes it to a boolean, and calls `0x39dd0`. That setter writes window/message mode at `0x9e52a0` and message-state halfword `+0x63e`. The dispatcher resumes at `0x76730`; this command itself does not wait, display a message, or start audio.

[inferred] This is message/window presentation state, not a timer. The exact visible distinction between modes 0 and 1 is not established by the setter alone (T `0x39dd0–0x39dec`).

## Steam build

[code] There is no one-to-one WNTY opcode in these recipes. R `recipes_data.rs:10329` records compiler state `Wnty`; ordinary 0/1 forms emit no instruction and carry `zero_unproven: true`. When compiler state F822 is 1, it emits mode 1/2 to opcode `0x158`, then clears its compiler F822 state. S registration `0x1400e77cd–0x1400e77dc` binds `0x158` to the CLO2 handler `0x14010b280`, which installs a task and normally yields.

[code] R `compile.rs:860–910` consumes Wnty during message lowering: message mode is Wnty+1, overridden to 0 by a following flag assignment of 63 before the next WNTY/message, or to 1 by text containing `@n`; the emitted message call is `0x140`. This is compile-time propagation, not a runtime mode setter.

[data] Across 250 Mahoyo raw CHS files, opcode `0x158` occurs 16 times; example `raw/archive.chs@0x118` follows a push of 1. Message presentation also uses `0x140`; the 16 calls are not a count of authored WNTY statements.

## Differences

[code] Switch normalizes low-16-bit nonzero values; the surveyed compiler recipes accept only literal 0/1 (T `0x78088`; R `recipes_data.rs:10329–10430`). [inferred] Compile-time state can differ from runtime state across jumps. The F822 path introduces a yielding CLO2 task, whereas Switch WNTY itself continues immediately (S `0x14010b386`; T `0x78094`).

## Unverified

[inferred] Visible mode meaning, branch-entry parity, and F822 behavior remain unmeasured. One runtime check: trace a two-message page with a branch into its second message, in both WNTY modes and with F822 set, logging mode and whether execution pauses at WNTY (T `0x39dd0`; S `0x14010b280`/`0x14010a170`).

---

Engine facts here are lifted from the port's `docs/commands/WNTY.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
