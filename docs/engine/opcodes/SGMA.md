# SGMA

Source spelling: `_SGMA(2,22551)`.

## Operands

- Operands are zero-based.

- [code] T `0x97960` reads `(slot,animationId,frameOffset)`; missing/empty offset is 0.

- [data] Native Mahoyo has **19** matching opcode occurrences across 250 supplied CHS files, all dumped successfully.

## Switch build

[code] T `0x97960` reads `(slot,animationId,frameOffset)`; missing/empty offset is 0. It captures slot state into mutable animation frame `start(id)+offset` via boundary lookup `0x976c0`, with the write at `0x979e0…0x97a20`. This changes a later animation's frame data rather than starting or waiting for that animation. No audio request. The captured-field set and invalid-index behavior are not fully enumerated here.

## Steam build

[code] Opcode `0x279`, S `0x140105200`, stores the capture id in slot `+0x19c` and returns zero. Sync consumer `0x14014956a/0x140149578` calls frame writer `0x1400c1380`, which normally uses boundary(id) only (`0x1400c1395`). Recipe `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:5094` accepts two fields.

[data] Native Mahoyo has **19** matching opcode occurrences across 250 supplied CHS files, all dumped successfully. First: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_5.chs`, assembly-dump line 1977. These are static counts, not reachability.

## Differences

[code] Stock Steam captures only the first frame (S `0x1400c1395`). Capture is deferred through a slot request on Steam, whereas the Switch handler writes the frame directly (T `0x979e0`; S `0x140105200`, `0x140149578`). [inferred] Multiple captures before a sync, save/load between request and consumption, and out-of-range frame offsets may differ; the packing range does not establish frame bounds.

## Unverified

[inferred] Capture timing, complete copied fields and persistence remain unresolved. One runtime check: replay scr0365's captures of slot 10 into frames 0 and 1 of source animation 10505, then play it on slot 7; compare both captured frames and a save/load between capture and playback.

---

Engine facts here are lifted from the port's `docs/commands/SGMA.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
