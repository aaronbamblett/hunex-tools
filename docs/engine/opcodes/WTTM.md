# WTTM

Source spelling: `_WTTM(1,1)`.

## Operands

- [code] T `0x7c614–0x7c69c` evaluates operand 0 as milliseconds and converts it to a 60 Hz count with signed truncation of 3*t/50.

- T `0x7cef0–0x7cf04` enforces a minimum count of 1 and stores a halfword at `0x125a4fc`.

- Presence of a numeric third field enables subtraction of the saved signed counter through `0x351ab0` (`0x7ceac–0x7ceec`); its full authored meaning is unresolved.

- [code] Operand 1 is an interruptibility boolean, defaulting to zero when absent in the inspected scanner; it is stored as a dword at `0x125a508` (`0x7cf08–0x7d008`).

- Updates at `0x7f074–0x7f108` decrement/clamp the signed 16-bit counter and, when the boolean is zero, allow an input path to clear it.

- [code] R `recipes_data.rs:10462–10523` accepts 1–3 operands, defaults them to 0,1,0, and emits opcode `0x110`.

- S `0x1401094c0` pops duration, interrupt policy and timing mode; clamps negative duration to zero; stores duration at task `+0x60`, third operand at `+0x64`, and sets input mask `0x12201` only when operand 1 is zero.

- [data] Mahoyo raw corpus: 4,536 opcode calls; `raw/1_0.chs@0xa1c` pushes 0,1,1000 in stack order, requesting the common 1000-ms, policy-1 form.

## Switch build

[code] T `0x7c614–0x7c69c` evaluates operand 0 as milliseconds and converts it to a 60 Hz count with signed truncation of 3*t/50. T `0x7cef0–0x7cf04` enforces a minimum count of 1 and stores a halfword at `0x125a4fc`. Presence of a numeric third field enables subtraction of the saved signed counter through `0x351ab0` (`0x7ceac–0x7ceec`); its full authored meaning is unresolved.

[code] Operand 1 is an interruptibility boolean, defaulting to zero when absent in the inspected scanner; it is stored as a dword at `0x125a508` (`0x7cf08–0x7d008`). The handler returns the next statement at `0x7d00c`, but runtime dispatch is gated by counters (`0x7daf0–0x7db30`). Updates at `0x7f074–0x7f108` decrement/clamp the signed 16-bit counter and, when the boolean is zero, allow an input path to clear it. This command delays following script execution; it does not itself start audio or a fade.

## Steam build

[code] R `recipes_data.rs:10462–10523` accepts 1–3 operands, defaults them to 0,1,0, and emits opcode `0x110`. S `0x1401094c0` pops duration, interrupt policy and timing mode; clamps negative duration to zero; stores duration at task `+0x60`, third operand at `+0x64`, and sets input mask `0x12201` only when operand 1 is zero. It normally installs the task at interpreter `+0x24e8` and returns 1 (`0x14010951a–0x1401095d1`).

[code] Update `0x140084280–0x140084574` handles timing mode 1 specially, accumulates clock deltas into `+0x78`, and completes when that reaches duration; mask/input and engine pause state affect progress (`0x14008432b`, `0x140084370`, `0x14008448d–0x1400844e4`).

[data] Mahoyo raw corpus: 4,536 opcode calls; `raw/1_0.chs@0xa1c` pushes 0,1,1000 in stack order, requesting the common 1000-ms, policy-1 form.

## Differences

[code] Switch quantizes to 60 Hz and stores signed 16-bit ticks; Steam retains a nonnegative dword duration. Switch zero-duration requests still store at least one tick (T `0x7cf00`); Steam stores zero (S `0x140109523`). Omitted policy differs: Switch's scanner initializes it to zero, compiler defaults it to 1 (T `0x7cf0c`; R `recipes_data.rs:10490`). [inferred] Long durations can overflow Switch's signed tick count; third-operand presence semantics also need comparison. Steam pause/input state can prolong a timer, but this is not an animation-record completion wait (S `0x140084370–0x1400844e4`).

## Unverified

[inferred] One runtime check: instrument a timer sequence using 0, 1, 400 and 300000 ms, with omitted/0/1 policy, input held/released and a pause/resume; compare time to the next statement and counter values on both engines (T `0x125a4fc`; S task `+0x60/+0x78`). Third-operand compensation remains outside the authored corpus survey.

---

Engine facts here are lifted from the port's `docs/commands/WTTM.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
