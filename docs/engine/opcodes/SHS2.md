# SHS2

Source spelling: `_SHS2(-1)`.

## Operands

- Operands are zero-based.

- [code] Handler T `0x8f120` parses slot (default 0) and calls `0x63450(slot,2)`.

- Categories 2 and 3 alias the second bank through T `0x235ed8`.

- [data] Mahoyo uses the matching opcode **295** times in the 250 supplied raw CHS files.

## Switch build

[code] Handler T `0x8f120` parses slot (default 0) and calls `0x63450(slot,2)`. Slot >39 is ignored; any negative slot clears all 40 slots. It immediately clears bank 2, including a category-3 sway and its saved/live parameters, timers and displacement outputs, then continues dispatch. Categories 2 and 3 alias the second bank through T `0x235ed8`. This is a stop, not a wait for natural completion; no audio call.

## Steam build

[code] Matching rewritten opcode `STQS` is `0x272`, S `0x140103e60`. It sets the stop request on Steam's single slot-quake state and returns zero. Negative slot broadcasts across 64 slots, rather than Switch's 40. Registration: S `0x140106909…0x140106917`.

[data] Mahoyo uses the matching opcode **295** times in the 250 supplied raw CHS files. First occurrence: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_0.chs`, assembly-dump line 3933. All 250 dumps returned success; this is native usage, not proof of Tsukihime compatibility.

## Differences

[code] Steam has one quake state per slot; Switch has two banks, with categories 2 and 3 sharing bank 2 (T `0x235ed8`, setter `0x632f0`). Steam scaling, finite-duration decay, broadcast eligibility and source fade controls differ (T `0x3e270`; S `0x1401455e0…0x140145d97`). [inferred] A translated effect can replace or clear another effect that should continue. These clear commands do not themselves wait; later completion waits need separate validation.

## Unverified

[inferred] One runtime check: start category 1 and category 2/3 together on a slot, issue SHS2, and observe that only bank 2, including a category-3 sway stops on Switch; compare the translated scene in Steam (T `0x63450`; S `0x140103e60`). Native update latency after a stop remains unmeasured.

---

Engine facts here are lifted from the port's `docs/commands/SHS2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
