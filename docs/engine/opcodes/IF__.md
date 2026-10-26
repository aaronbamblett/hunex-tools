# IF__

Source spelling: `_IF__(F026,ne,0,LV00000)`.

## Switch build

- [code] `left,comparison,right,label` is a jump on false for textual eq/ne/gt/ge/lt/le. Dispatch T `0x77ad4`, left resolution `0x77b4c–0x79bd8`, right resolution `0x7ba18–0x7ba70`, textual comparisons `0x7bf18–0x7bfd4`. True continues at `0x76730`; false parses operand 3 and resolves the label through `0x7bfe8–0x7c310`. Equality compares low 16 bits; ordered comparisons sign-extend 16 bits (`0x7bf44–0x7bfd0`). [code] F/G/underscore compatibility reads are recognized on this path; C is not handled as the relative destination bank here (T `0x77b58–0x77b80`, `0x7ba28–0x7ba70`). This reads flags and writes the script continuation pointer; it neither draws nor starts audio. A nonzero engine word through `[0x351a28]` suppresses the command (`0x77ad4–0x77ae0`). [code] The shared continuation path immediately resumes dispatch; the interpreter also has a 1,000-statement budget that yields at T `0x76738–0x7673c`. There is no condition-polling wait in IF__ itself. Numeric comparison selectors use a separate path (`0x7be20–0x7bec4`); textual semantics must not be extrapolated to it.

## Steam build

- [code] Compiler emits flag read `0xea`, right value, comparison `0x30..0x35`, target, then `0x19` with immediate 0 (H `compile::lower_special`; `recipes_data::COMMANDS`). Conditional jump S `0x14007cb00–0x14007cb5a` consumes target and predicate; immediate 0 selects predicate==0. It changes VM IP `+0x38` and returns 0 without a wait or direct audiovisual work. [code] Values are 32-bit and F/G/_D addressing is the Steam layout described in [WKST](WKST.md), with out-of-range reads returning zero (S `0x14007dc00–0x14007dcb3`). [data] Mahoyo has 150 flag reads and 1,071 conditional jumps in 250 raw scripts; these include entry dispatch and other predicates, not 1,071 recovered IF__ statements. `raw/1_0.chs:0x7c08–0x7c20` reads F024, compares to 1 and jumps on false before the return macro.

## Differences

- [code] Low-16-bit/signed-16-bit source comparisons and 32-bit Steam values can diverge after overflow or negative assignments (T `0x7bf44–0x7bfd0`; S `0x14007dcb0`). [inferred] Branch parity also depends on correctly restored flag ownership and special execution mode. No IF__ handler directly waits, but a wrong branch can enter a different later wait (`0x77ad4`; S `0x14007cb54`).

## Unverified

- [inferred] One paired trace of that entry condition with F026=0 and F026=1 should log the chosen continuation, then repeat a signed boundary value on an authored ordered comparison (T `0x7bf44`; S `0x14007cb00`). Native mode suppression and all boundary cases remain unexecuted.
