# SHK2

Source spelling: `_SHK2(17,0,3,300,-1,0)`.

## Operands

- Operands are zero-based.

- [code] T `0x8eda0` parses `(slot,xAmp,yAmp,period,lifetime,delay,fadeIn,fadeOut,modulation)`, defaults `(0,0,15,300,-1,0,0,0,1)`.

- Default period is 18 frames.

- [code] `STQK`, opcode `0x270`, handler S `0x140103bc0`, pops slot, x/y amplitudes, period, lifetime, delay and mode.

- It writes one slot-quake request at `+0x1a0…+0x1c0`; the callback S `0x1401455e0` consumes it.

- [data] Mahoyo uses the matching opcode **614** times in the 250 supplied raw CHS files.

## Switch build

[code] T `0x8eda0` parses `(slot,xAmp,yAmp,period,lifetime,delay,fadeIn,fadeOut,modulation)`, defaults `(0,0,15,300,-1,0,0,0,1)`. It installs category 2 in bank 2 through T `0x632f0`. Supplied period becomes `trunc(3*p/50)` frames; consumer T `0x3e3e8` substitutes 60 for zero. Default period is 18 frames. T `0x3e3d8…0x3e538` applies sine displacement, fades and modulation. The saved/live parameters change immediately; updates animate the output without a script wait or audio call.

## Steam build

[code] `STQK`, opcode `0x270`, handler S `0x140103bc0`, pops slot, x/y amplitudes, period, lifetime, delay and mode. It writes one slot-quake request at `+0x1a0…+0x1c0`; the callback S `0x1401455e0` consumes it. Input mode 0 selects random displacement (S `0x140145b16`); input mode 2 selects the sine branch (S `0x140145c5c`). The handler returns zero, without creating a wait. Recipe: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:8199`.

[data] Mahoyo uses the matching opcode **614** times in the 250 supplied raw CHS files. First occurrence: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_0.chs`, assembly-dump line 3762. All 250 dumps returned success; this is native usage, not proof of Tsukihime compatibility.

## Differences

[code] Steam has one quake state per slot; Switch has two banks, with categories 2 and 3 sharing bank 2 (T `0x235ed8`, setter `0x632f0`). Steam scaling, finite-duration decay, broadcast eligibility and source fade controls differ (T `0x3e270`; S `0x1401455e0…0x140145d97`). [inferred] A translated effect can replace or clear another effect that should continue. These start commands do not themselves wait; later completion waits need separate validation.

## Unverified

[inferred] Source oscillator phase/modulation and finite-life envelope have not been measured against Steam. One runtime check: replay `scr0140` statement 3542 with category 1 also active and compare displacement and expiry frame by frame, then replace bank 2 with sway (T `0x3e3d8`; S `0x140145c5c`).

---

Engine facts here are lifted from the port's `docs/commands/SHK2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
