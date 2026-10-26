# VPL2

Source spelling: `_VPL2(B90_00_01_0000_0,03d00)`.

## Operands

- [code] Immediate variant of VPLY with source `(voice stem, label/id)`; both share T `0x773d0` and name-derived pending/replay stores `0x780f4–0x78120`.

## Switch build

[code] Immediate variant of VPLY with source `(voice stem, label/id)`; both share T `0x773d0` and name-derived pending/replay stores `0x780f4–0x78120`. The VPL2 branch copies speaker state `0x125a590 → 0x1256682`, clears pending speaker, checks playback permissions and character configuration, then starts voice at `0x781e4 → 0xa9580 → 0x99b20`. It clears pending voice at `0x781f4` and continues dispatch. It does not wait for the clip to finish; VCWT is a separate barrier.

## Steam build

[code] No distinct stock VPL2 handler is used. It returns 0, with no completion wait.

## Differences

[inferred] A following message replaying the same cue, a muted cue releasing VCWT too early, or a restore retaining an obsolete request would break Switch timing.

## Unverified

[inferred] One runtime check: run the scr0003 cue with output-device mute but engine voice enabled, then repeat with character disabled, skip, and save/load around the barrier. The action after VCWT should follow approximately **2.1975 s** of voice/animation, and the next message must not replay it. no runtime result is claimed here.

---

Engine facts here are lifted from the port's `docs/commands/VPL2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
