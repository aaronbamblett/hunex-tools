# GOTO

Source spelling: `_GOTO(LBL_018)`.

## Switch build

- [code] `label` is an unconditional local branch. T `0x779a8` skips it when the word through `[0x351a28]` is nonzero; otherwise it copies the argument and calls label resolver `0x592c0` at `0x7c304`. The returned pointer becomes x26 and execution immediately resumes (`0x7c30c–0x7c310`). It changes no flags and has no direct wait, drawing or audio effect. [code] The general interpreter budget still yields after 1,000 statements (T `0x76738–0x7673c`); this is distinct from a GOTO wait.

## Steam build

- [code] Compiler resolves a label to a bytecode offset, pushes it and emits `0x18` (H `compile::lower_special`, recipe `_GOTO:label`). Handler S `0x14007cad0` pops the target and assigns VM IP +0x38, returning 0 (`0x14007cae2–0x14007caff`). It neither changes flag banks nor performs a task, sound or draw. [data] Mahoyo's 250 raw files contain 198 `0x18` calls, including macro/label lowering; these are not 198 recovered GOTO source statements. `raw/1_0.chs:0x7c40` follows the return macro's target push.

## Differences

- [code] Source label lookup at runtime becomes resolved bytecode offsets; Steam does not implement the Switch guard in `0x18` (same addresses; H `compile::lower_special`). [inferred] A malformed target or a branch cycle can stall progression even though GOTO itself does not wait. Scheduler fairness of long source loops versus VM loops is unverified (T `0x76738`; S `0x14007cad0`).

## Unverified

- [inferred] One paired trace through scr0021's two branch outcomes should verify the target statement and state carried across the branch, including any special source mode in use (T `0x7c304`; S `0x14007cad0`).
