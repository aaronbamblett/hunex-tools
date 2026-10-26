# CLO2

Source spelling: `_CLO2()`.

## Operands

- [code] Operand-free T dispatch `0x7c32c–0x7c354` calls reset helper `0x54c70(1)`, clears open-word through `0x2f340(0)`, writes alpha zero through `0x2f350(0)`, clears a global halfword through `0x3517b8`, and sets suppression `+0x89e=1` through `0x7c3b8`.

- [code] R `recipes_data.rs:160–181` emits operand-free opcode `0x151`, registered at S `0x1400e7775–0x1400e7784` to `0x14010b280`.

- [code] The task is constructed by `0x14015b390` and uses updater `0x14015b680`, which reads elapsed time, engine/window state, and resource state; its exact full completion contract remains unresolved.

## Switch build

[code] Operand-free T dispatch `0x7c32c–0x7c354` calls reset helper `0x54c70(1)`, clears open-word through `0x2f340(0)`, writes alpha zero through `0x2f350(0)`, clears a global halfword through `0x3517b8`, and sets suppression `+0x89e=1` through `0x7c3b8`. It immediately returns the next statement at `0x7d00c`; it does not install a fade-completion wait. The visible implication is immediate window/text closure, without an audio start in this path.

## Steam build

[code] R `recipes_data.rs:160–181` emits operand-free opcode `0x151`, registered at S `0x1400e7775–0x1400e7784` to `0x14010b280`. It sets window request fields `+0x2a0/+0x2a4/+0x2cc..+0x2e0`, constructs a task at `0x14010b2da`, supplies task mode/indices 8,4,3,6,5 (`0x14010b328–0x14010b344`), stores the task at interpreter `+0x24e8`, and normally returns 1 (`0x14010b386–0x14010b397`). The special fast path at `0x14010b351` clears state and returns 0.

[code] The task is constructed by `0x14015b390` and uses updater `0x14015b680`, which reads elapsed time, engine/window state, and resource state; its exact full completion contract remains unresolved. This is more than an immediate state setter.

[data] Mahoyo raw corpus: 157 opcode calls, e.g. startup macro `raw/1_0.chs@0x110` following debug source `Include/define_macro.h`, line 21.

## Differences

[inferred] Full visible reset parity remains unverified: Steam processes close-request fields through its native consumer, while Switch resets its own window immediately. CLO2 remains distinct from CLO3.

## Unverified

[inferred] Visible reset parity remains unresolved. Compare window reset and next-statement timing with Switch (T `0x7c334`; S `0x14010b280`).

---

Engine facts here are lifted from the port's `docs/commands/CLO2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
