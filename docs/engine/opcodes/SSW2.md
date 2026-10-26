# SSW2

Source spelling: `_SSW2(11,7,4,1000,2000,1,-1,0)`.

## Switch build

[code] T `0x8f1b0` reads `(slot,ampMin,ampMax,periodMin,periodMax,multiplier,lifetime,delay)`, defaults `(0,7,15,1000,2000,1,-1,0)`. Periods/lifetime/delay convert to 60 Hz frames. Category 3 occupies bank 2, shared with SHK2, through T `0x235ed8` and `0x632f0`. T `0x3e318…0x3e790` independently resamples amplitude and period for both axes and produces sine sway. Installation is nonblocking; draw/update consumes the stored output; no audio call.

## Steam build

[code] The current counterpart is `STSW`, opcode `0x274`, handler S `0x140104520`, which selects internal mode 5 without waiting. Generator S `0x140145700…0x140145ae4` uses fixed amplitude 5…10 and periods 60…120 frames, with an extra y factor 0.3. Registration S `0x14010691e…0x14010692c`.

[data] Mahoyo: 1 call(s) (`raw/nz3.chs`, assembly-dump line 14325).

## Differences

[code] Steam has one quake state per slot; Switch has two banks, with categories 2 and 3 sharing bank 2 (T `0x235ed8`, setter `0x632f0`). Steam scaling, finite-duration decay, broadcast eligibility and source fade controls differ (T `0x3e270`; S `0x1401455e0…0x140145d97`). [inferred] A translated effect can replace or clear another effect that should continue. These start commands do not themselves wait; later completion waits need separate validation.

[code] Steam fixed ranges and vertical scaling cannot preserve the authored ranges. Fractional multiplier 0.5 is outside `effect_args`' integer contract.

## Unverified

[inferred] Fractional multiplier semantics and actual sampled trajectories remain unresolved. Compare scr0079's varied sway with a finite 800-ms scr0424 sway, recording both axes and stopping bank 2 while bank 1 continues (T `0x3e318`; S `0x140145700`).
