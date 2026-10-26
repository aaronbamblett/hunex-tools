# VCWT

Source spelling: `_VCWT()`.

## Operands

- [code] Operandless voice wait: dispatch T `0x770b0` calls `0x99c20`, which obtains voice-channel state through `0x98d40(-1)` and reads its first dword.

## Switch build

[code] Operandless voice wait: dispatch T `0x770b0` calls `0x99c20`, which obtains voice-channel state through `0x98d40(-1)` and reads its first dword. Zero continues at `0x76730`; nonzero branches to yield handling at `0x7c79c`. Thus its release condition is channel idle, not elapsed script time; it changes execution timing without starting a sound.

## Steam build

[code] Otherwise it submits native operation `0x19` through `0x140128390`, depending on that request, tracks the barrier, and attaches task vtable `0x140283d00` with filter `0/2/2`. Barrier executor S `0x1401243eb–0x1401244b7` waits for dependencies and polls actual playback through `0x14017cf30 → 0x14017ae50`, vtable `+0x40`. The generic task itself only checks request status; it becomes a playback barrier because this particular request polls playback.

## Differences

runtime cancellation and replacement behavior must be checked (S `0x1401243eb–0x1401244b7`). No finite-time escape is established here.

## Unverified

[inferred] One runtime check: compare the scr0003 two-cue sequence with Switch, including idle entry, muted/disabled voice, skip, consecutive barriers, and save/load. Hold the first cue about **2.1975 s**, the second about **4.0575 s**, before its separate 0.4-second scripted delay; verify idle returns promptly and no cue replays.

---

Engine facts here are lifted from the port's `docs/commands/VCWT.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
