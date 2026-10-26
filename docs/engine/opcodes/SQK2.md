# SQK2

Source spelling: `_SQK2(-1,1,4,0,600,0,0,0,0)`.

## Operands

- Operands are zero-based.

- [code] T `0x8e9b0` parses `(slot,xAmp,yAmp,period,lifetime,delay,fadeIn,fadeOut,unused)`, defaults `(0,15,15,0,-1,0,0,0,0)`, and installs category 1 through T `0x632f0`.

- Operand 8 is unread.

- [code] `STQK`, opcode `0x270`, handler S `0x140103bc0`, pops slot, x/y amplitudes, period, lifetime, delay and mode.

- It writes one slot-quake request at `+0x1a0…+0x1c0`; the callback S `0x1401455e0` consumes it.

- [data] Mahoyo uses the matching opcode **614** times in the 250 supplied raw CHS files.

## Switch build

[code] T `0x8e9b0` parses `(slot,xAmp,yAmp,period,lifetime,delay,fadeIn,fadeOut,unused)`, defaults `(0,15,15,0,-1,0,0,0,0)`, and installs category 1 through T `0x632f0`. Period is `max(2,trunc(3*p/50)+1)` frames, or one frame for negative p; lifetime and delay use `trunc(3*t/50)`. Operand 8 is unread. The setter copies parameters/timers to saved and live state and clears accumulated outputs. Consumer T `0x3e270`, random branches `0x3e56c/0x3e7d4`, produces sprite displacement with the fade controls. It starts asynchronously and returns to dispatch; no audio request.

## Steam build

[code] `STQK`, opcode `0x270`, handler S `0x140103bc0`, pops slot, x/y amplitudes, period, lifetime, delay and mode. It writes one slot-quake request at `+0x1a0…+0x1c0`; the callback S `0x1401455e0` consumes it. Input mode 0 selects random displacement (S `0x140145b16`); input mode 2 selects the sine branch (S `0x140145c5c`). The handler returns zero, without creating a wait. Recipe: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:8199`.

[data] Mahoyo uses the matching opcode **614** times in the 250 supplied raw CHS files. First occurrence: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_0.chs`, assembly-dump line 3762. All 250 dumps returned success; this is native usage, not proof of Tsukihime compatibility.

## Differences

[code] Steam has one quake state per slot; Switch has two banks, with categories 2 and 3 sharing bank 2 (T `0x235ed8`, setter `0x632f0`). Steam scaling, finite-duration decay, broadcast eligibility and source fade controls differ (T `0x3e270`; S `0x1401455e0…0x140145d97`). [inferred] A translated effect can replace or clear another effect that should continue. These start commands do not themselves wait; later completion waits need separate validation.

## Unverified

[inferred] Exact visible cadence, envelope and composition remain unmeasured. One runtime check: record `scr0001` statement 216 and a concurrent category-2 effect on both engines, tracking displacement each frame through start, fade and expiry (T `0x3e270`; S `0x1401455e0`).

---

Engine facts here are lifted from the port's `docs/commands/SQK2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
