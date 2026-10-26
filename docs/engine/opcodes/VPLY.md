# VPLY

Source spelling: `_VPLY(D10_00_00_0000_0,03fb5)`.

## Operands

- Shared VPLY/VPL2 entry T `0x773d0` reads the voice stem, derives speaker state via `0x2510`/`0x29fc0`, and derives the sound id from the name via `0x2570`.

- [code] Source operand 0 is the voice stem; operand 1's precise label/id role is not established by the shared name-derived stores.

- With a third operand, T `0x77498–0x774b8` reads field 2 and the subsequent path writes a control value through `[0x3519e8]` at `0x780b8`.

- Do not describe operand 1 as the value stored in those three voice-id globals.

- [code] Opcode `0x1a0`, handler S `0x1401086e0` (registration `0x1400e7909`), takes a filename string and control dword.

- It stages the first 0x78-byte pending record at `[[ctx+0x2508]+0x360]`, not the last record, and returns without yielding at `0x140108ade`.

- [code] R `recipes_data.rs:9000` emits pushes `(control=0, stem)` followed by `0x1a0` for two-field source; recipes at also carry a third control value for symbol-label forms.

- Operand 1 is omitted from this bytecode.

- [data] Mahoyo has **9,082** occurrences: `raw/1_0.chs`, file `0x1d68`, follows pushes of control 0 and voice stem `F70_1_0_0000_A` (dump convention in).

## Switch build

[code] Stages a voice for the next message; it does not itself start playback or wait for completion. Shared VPLY/VPL2 entry T `0x773d0` reads the voice stem, derives speaker state via `0x2510`/`0x29fc0`, and derives the sound id from the name via `0x2570`. Stores at `0x780f4–0x78120` update pending/current/replay ids `0x125a584`, `0x125a588`, `0x125a58c`; ordinary VPLY continues dispatch at `0x78124`. The message consumer T `0x6e804–0x6e87c`, `0x6ea50` starts the pending clip subject to playback/settings gates and clears pending state at `0x6e8a4`.

[code] Source operand 0 is the voice stem; operand 1's precise label/id role is not established by the shared name-derived stores. With a third operand, T `0x77498–0x774b8` reads field 2 and the subsequent path writes a control value through `[0x3519e8]` at `0x780b8`. Do not describe operand 1 as the value stored in those three voice-id globals.

## Steam build

[code] Opcode `0x1a0`, handler S `0x1401086e0` (registration `0x1400e7909`), takes a filename string and control dword. It stages the first 0x78-byte pending record at `[[ctx+0x2508]+0x360]`, not the last record, and returns without yielding at `0x140108ade`. Current/replay state is separate at state `+0x348`; merger S `0x14016bbde–0x14016bda3` consumes pending records. Native voice opening and configured gain run at `0x1401473e5–0x140147482`; submission uses channel 0 / `VA_Voice`.

[code] R `recipes_data.rs:9000` emits pushes `(control=0, stem)` followed by `0x1a0` for two-field source; recipes at also carry a third control value for symbol-label forms. Operand 1 is omitted from this bytecode. [data] Mahoyo has **9,082** occurrences: `raw/1_0.chs`, file `0x1d68`, follows pushes of control 0 and voice stem `F70_1_0_0000_A` (dump convention in).

## Differences

[code] VPLY alone supplies no playback barrier on either side (T `0x78124`; S `0x140108ade`). Substituting it for VPL2 delays voice until a message [data] One corpus call has a third value 0, in `dump/scr0212.txt:1`; the other 16,688 have two fields. [inferred] Losing that control or assuming filename remapping preserves character-volume identity can alter suppression/settings behavior (T `0x774ac`; S `0x14014743a`).

## Unverified

[inferred] One runtime check: play the cited cue with its speaker enabled, disabled, and with a changed speaker volume, recording voice onset against the next message and replaying it from backlog. Compare both engines; this settles deferred onset and remapped speaker/settings identity. Third-field suppression additionally needs the scr0212 cue; it is not proven by the ordinary two-field example (T `0x77498–0x774b8`; S `0x1401086e0`).

---

Engine facts here are lifted from the port's `docs/commands/VPLY.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
