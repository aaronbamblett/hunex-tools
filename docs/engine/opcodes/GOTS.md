# GOTS

Source spelling: `_GOTS(_D000,SEL0_0,SEL0_1,SEL0_2)`.

## Operands

- [code] `selector,label0,label1...` is a zero-based computed local branch.

## Switch build

- [code] T `0x769f0` yields while the pending byte through `[0x3519d8]` is nonzero, and skips in the mode through `[0x351a28]` (`0x769f8–0x76a0c`). This path accepts an underscore compatibility selector; it resolves local/persistent aliasing, reads its 16-bit word, then scans to the selected label (`0x76a68–0x76ae0`, `0x79ba8–0x7c310`). The ordinary corpus selector is `_D000` (local F000). [code] Selection values 0..32766 take the label-counting path; larger raw values take the alternate scan at `0x79490` (`0x76acc–0x76adc`). Behavior for invalid/out-of-list selectors must not be inferred from valid choices. The branch changes the continuation pointer and has no direct draw/audio call or choice UI; SELD supplies the result separately (`0x7c304`).

## Steam build

- [code] Compiler reverses supplied labels, pads missing targets with sentinel `0xffffffff`, pushes selector index/bank, and emits `0x162` (H `compile::lower_special`, `_GOTS:0`; sentinel constant `compile::SKIP_SENTINEL`). Handler S `0x140049e60` pops bank, index and 19 target offsets, resolves F/G/compatibility banks, and changes IP when the signed selector is 0..18; values outside that range fall through (`0x14004a043–0x14004a0c9`). It returns 0 without a task or audiovisual action. [code] Compiler arity is 2..19 total operands, so it accepts at most 18 authored labels, despite the VM's 19 target slots (H `_GOTS:0`). An in-range but unprovided target selects the padded sentinel; the handler does not check for it before storing IP (S `0x14004a0bf–0x14004a0c6`). [data] Mahoyo has 78 `0x162` calls in 250 raw files; `raw/test_script.chs:0x21d0` uses bank 2/index 0 after target offsets.

## Differences

- [code] Switch selector decoding is narrower than Steam's generic bank support; Steam's explicit range is 19 slots. Invalid values and missing targets have different paths (T `0x76a6c`, `0x76ad4`; S `0x14004a0a5`). [inferred] Padding is not safe fall-through. A bad selector may put Steam IP at the sentinel (H `compile::lower_special`; S `0x14004a0c6`).

## Unverified

- [inferred] One paired run selecting every option at that three-choice site should log D000 and the first target statement. Out-of-list behavior and the exact source fallback scan remain unresolved (T `0x79490`; S `0x14004a0a5`).
