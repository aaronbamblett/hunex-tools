# END_

Source spelling: `_END_()`.

## Switch build

[code] Operandless end-of-script control, dispatch T `0x76954` to `0x7c59c`.

- [code] If the pending byte through `[0x3519d8]` is set, it returns a null continuation and yields (`0x7c5a0–0x7c5b0`). Otherwise it conditionally finalizes active state, calls teardown `0x5eb20`, restores the saved continuation from `0x1354e98`, sets execution state 1, and yields (`0x7c828–0x7c854`, shared state store `0x7d008`). [code] This differs from RTM_, which sets state 2 after teardown rather than restoring that saved pointer (`0x7c870–0x7c880`). Cleanup can affect audiovisual state; END_ has no independent duration or playback-completion predicate.

## Steam build

- [code] Recipe emits 0x12c (H `_END_:0`), registered at S `0x1400e76a1` to 0x14007ddd0, the same status-3 return handler as RTM_'s `0xf4`. It pops no operands and causes VM removal/destruction through the scheduler (`0x14007e91a–0x14007e9c0`). The matching engine's outer scene cleanup supplies later audiovisual effects. [data] Mahoyo has 249 `0x12c` calls in 250 raw files, e.g. `raw/1_0.chs:0x7d90`, following its `0xf4` return.

## Differences

- [code] Switch END_ preserves/restores a continuation and sets state 1; RTM_ sets state 2. Native Steam opcodes `0x12c` and `0xf4` return the same status 3 (T `0x7c828–0x7c880`; S `0x1400e76a1`, `0x14007a7e0`). [inferred] The example END_ follows a return and may be unreachable in normal execution, which can hide a lifecycle mismatch if END_ is reached directly (`scr0001.txt:1`, statements 8434–8435).

## Unverified

- [inferred] One paired trace that actually reaches an authored END_ should compare the saved continuation, subsequent caller behavior and cleanup; testing only RTM_ does not establish END_ parity (T `0x7c84c`; S `0x14007ddd0`). No runtime execution was performed.
