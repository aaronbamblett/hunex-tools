# RTM_

Source spelling: `_RTM_()`.

## Operands

- [code] Operandless scene return at T `0x77d30`: pending byte through `[0x3519d8]` yields, and mode through `[0x351a28]` suppresses it (`0x77d38–0x77d4c`).

- No script operands or flags are popped.

## Switch build

- [code] Operandless scene return at T `0x77d30`: pending byte through `[0x3519d8]` yields, and mode through `[0x351a28]` suppresses it (`0x77d38–0x77d4c`). Normal path conditionally finalizes active state with `0x5db90`, clears its word, clears a local field, calls teardown `0x5eb20`, sets execution state **2** at `0x1354e8c`, and yields (`0x7c774–0x7c798`, `0x7c870–0x7c880`). [code] Teardown includes music handling and consumes the keep-music byte `0x1354e6d` at `0x5ebbc`, conditionally avoiding music stop `0x99e70`. Return is lifecycle control, not a timed wait; visible/audible effects come through cleanup.

## Steam build

- [code] Recipe emits `0xf4` (H `src/recipes_data.rs:1862–1883`), registered at S `0x14007a7e0` to `0x14007ddd0`. Handler returns scheduler status **3**. Dispatcher removes/destroys that VM (`0x14007e91a–0x14007e9c0`), destructor releases storage (`0x14007aab0`); outer scene teardown `0x1400e6b30` invokes reset `0x1400d78f0`. No script operands or flags are popped. [code] Outer reset preserves native music mode 1 but stops other modes with stop bits/duration 50; the handler alone does not stop music (S `0x1400d8bfc–0x1400d8ccd`). [data] Mahoyo uses `0xf4` 333 times in 250 raw files; `raw/1_0.chs:0x7d80` follows its scene transfer and precedes END_.

## Differences

- [code] Switch sets scene execution state and calls its teardown in the handler; Steam returns VM scheduler status and tears down through outer scene code (T `0x7c870`; S `0x14007ddd0`, `0x1400e6b30`). Ordinary music cleanup policy differs. [inferred] A scene can return correctly while losing music, control state or save metadata. Wait gating must be checked at the dispatcher/lifecycle boundary, not inferred from the operandless spelling (same addresses).

## Unverified

- [inferred] One paired ordinary return with active music and control state should compare cleanup, next scene, and a subsequent return after RTM2. The preserve case must consume its flag once; ordinary teardown must then resume (T `0x5eb20`; S `0x1400e6b30`).

---

Engine facts here are lifted from the port's `docs/commands/RTM_.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
