# RTMN

Source spelling: `_RTMN()`.

## Operands

- [code] Operandless keep-music return.

## Switch build

- [code] Operandless keep-music return. T `0x76fe4–0x76fec` stores a nonzero byte at **0x1354e6d**, then branches into RTM_'s pending/mode checks and return path (`0x77d30`, `0x7c774`, `0x7c870`). Teardown consumes the flag and skips music stop (`0x5ebbc`, `0x99e70`). It changes return/control state and cleanup policy; it does not start a track or wait for music completion.

## Steam build

- [code] There is no stock RTMN recipe/handler. The next ordinary teardown uses original music cleanup. No timer or playback-completion wait is introduced. [data] Mahoyo's source inventory has one RTMN. Native `0xf4` is used 333 times and does not itself mean keep music (`raw/1_0.chs:0x7d80`).

## Differences

- [code] Stock Steam teardown touches both music banks. The transient flag is not saved.

## Unverified

- [inferred] Run the documented inherited-music scr0002 return once, observing the pending→active→clear sequence, unchanged music records and continuous stream position, followed by an ordinary return. This is the one runtime check needed for continuity; it was not run here.

---

Engine facts here are lifted from the port's `docs/commands/RTMN.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
