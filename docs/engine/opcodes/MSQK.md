# MSQK

Source spelling: `_MSQK(500,5,1)`.

## Operands

- Operands are zero-based.

- Script operands are `(duration,xAmp,yAmp)`, defaults `(1000,5,5)`; duration outside unsigned 0…10000 resets to 1000.

- [code] `SHOK`, opcode `0x340`, S `0x1401054e0`, pops x, y, period, repetition/control and delay fields into global context `+0x70/+0xa8…+0xc8`, sets request/dirty flags and returns zero.

- [data] Mahoyo uses the matching opcode **87** times in the 250 supplied raw CHS files.

## Switch build

[code] Dispatch T `0x76edc` calls `0x3a3a0(duration,arg2,arg1)`. Script operands are `(duration,xAmp,yAmp)`, defaults `(1000,5,5)`; duration outside unsigned 0…10000 resets to 1000. Global state T `0x9e5288` holds duration, elapsed ms, scaled amplitudes and output. Update T `0x3db40…0x3dbbc` samples displacement until expiry and then zeros it; draw consumers include `0x3a9e4`, `0x3b530`, `0x3bf94`. This schedules a global shake without waiting or requesting audio.

## Steam build

[code] `SHOK`, opcode `0x340`, S `0x1401054e0`, pops x, y, period, repetition/control and delay fields into global context `+0x70/+0xa8…+0xc8`, sets request/dirty flags and returns zero. Recipe: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:5156`.

[data] Mahoyo uses the matching opcode **87** times in the 250 supplied raw CHS files. First occurrence: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_3.chs`, assembly-dump line 9661. All 250 dumps returned success; this is native usage, not proof of Tsukihime compatibility.

## Differences

[code] Source MSQK uses elapsed-ms random sampling; SHOK enters the separate global shock facility. Waveform, easing and coordinate scaling are not equivalent (T `0x3a3a0`, `0x3db40`; S `0x1401054e0`). Neither handler itself waits.

## Unverified

[inferred] Steam's final global-displacement application and exact duration of the corresponding SHOK request are unresolved. One runtime check: replay scr0496 statement 1363 on both engines and record global displacement and its expiry, including the immediately following voice (T `0x3db40`; S `0x1401054e0`).

---

Engine facts here are lifted from the port's `docs/commands/MSQK.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
