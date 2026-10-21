# SELS

Source spelling: `_SELS(_D000,)`.

## Operands

- Operand 0 is the result destination; the surveyed second operand is empty.

- T `0x81150–0x81160` chooses system message `0x5a6 + F027`.

- [inferred] This is a system-modal decision; exact visual/audio transitions remain unverified (T `0x8fcc0`, `0x81150`).

- completion `0x140155ae8` writes authored values (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:3403–3505`).

- These demonstrate the replacement mechanism, not a dedicated hint-system command (README dump method).

## Switch build

[code] T `0x8fcc0` creates selection mode 2 without SELA's F1118 gate. Operand 0 is the result destination; the surveyed second operand is empty. T `0x81150–0x81160` chooses system message `0x5a6 + F027`. Confirmation stores No=0 or Yes=1 at `0x82eac–0x82ec0`, so the script waits for a player decision.

[inferred] This is a system-modal decision; exact visual/audio transitions remain unverified (T `0x8fcc0`, `0x81150`).

## Steam build

[code] No dedicated SELS recipe was identified in the compiler command table. completion `0x140155ae8` writes authored values (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:3403–3505`). This uses ordinary message/choice tasks rather than Switch mode 2.

[data] Mahoyo contains 423 SELR and 100 SELD opcodes in 250 dumped CHS files, including `mahoyo-steam/raw/test_script.chs:0x2068,0x2118`. These demonstrate the replacement mechanism, not a dedicated hint-system command (README dump method).

## Differences

Ordinary message processing can affect voice, window and click state differently from the Switch modal; that consequence needs observation rather than assuming parity (T `0x8fcc0`). F027 outside 1..25 has no demonstrated authored question (same source dispatcher).

## Unverified

[inferred] Modal timing, presentation, input behavior and save/load need runtime comparison. One check: run scr0529 for F027=1 and 25, take No then Yes, and compare question text, destination values, cleanup/return and extra message clicks with Switch (T `0x82eac`; S `0x140155ae8`).

---

Engine facts here are lifted from the port's `docs/commands/SELS.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
