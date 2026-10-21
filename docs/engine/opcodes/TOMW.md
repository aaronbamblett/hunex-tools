# TOMW

Source spelling: `_TOMW()`.

## Operands

- [code] Operandless movie barrier: dispatch T `0x7706c–0x7707c → 0x7c358`.

- It first checks execution byte `0x1354e50`, yielding if nonzero; otherwise it reads the halfword at local-state `0x1251b5a+0x48` via pointer `0x351300`.

- It initiates no new movie or direct audio effect itself; the complete meaning/lifetime of that local halfword remains unverified (T `0x7c368`).

- [code] Opcode `0x1c1`, S `0x140106cb0`, pops no operands.

- Compiler operandless recipe calls 449 (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:8778–8800`).

- [code] Vtable update S `0x140156250` reads movie handle `0x1405aaf10`, transfers native event bits into `0x1405aaf20` (`0x1401562a1–0x14015630c`), then runs a multi-state player/control sequence.

- It reads player `0x140596f78`, +0x7c, +0xb0, time/error state and event latch at `0x140156327–0x1401563ce`; later stages inspect engine audio vectors, issue a stop request with duration 0xbb8 and poll `0x140160410` (`0x1401563e0–0x1401566a7`).

- This is more than a single read of the Switch local halfword; exact player-facing meaning of each state is unresolved.

## Switch build

[code] Operandless movie barrier: dispatch T `0x7706c–0x7707c → 0x7c358`. It first checks execution byte `0x1354e50`, yielding if nonzero; otherwise it reads the halfword at local-state `0x1251b5a+0x48` via pointer `0x351300`. A nonzero value retains the statement position for another pass; zero permits advancement. Both cases yield through `0x7d00c` (`0x7c358–0x7c37c`).

[inferred] This is a completion/control barrier for the asynchronous movie path, with at least a dispatch yield even when clear. It initiates no new movie or direct audio effect itself; the complete meaning/lifetime of that local halfword remains unverified (T `0x7c368`).

## Steam build

[code] Opcode `0x1c1`, S `0x140106cb0`, pops no operands. It creates a 0x60-byte task, vtable `0x140284000`, captures engine state and pointers to `0x14059c1b0`/`0x1405a08f0`, attaches it to VM+0x24e8 and returns 1 (`0x140106cd1–0x140106d71`). Compiler operandless recipe calls 449 (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:8778–8800`).

[code] Vtable update S `0x140156250` reads movie handle `0x1405aaf10`, transfers native event bits into `0x1405aaf20` (`0x1401562a1–0x14015630c`), then runs a multi-state player/control sequence. It reads player `0x140596f78`, +0x7c, +0xb0, time/error state and event latch at `0x140156327–0x1401563ce`; later stages inspect engine audio vectors, issue a stop request with duration 0xbb8 and poll `0x140160410` (`0x1401563e0–0x1401566a7`). This is more than a single read of the Switch local halfword; exact player-facing meaning of each state is unresolved.

[data] Mahoyo has one TOMW, `mahoyo-steam/raw/opening.chs:0x3b4`, after its single TOMP at `0x2b8` with intervening operations including opcode `0x201`. Counts come from 250 successful dumps.

## Differences

[inferred] **Same spelling does not establish equal waits.** TOMW can run after Steam TOMP has already blocked through playback, and its native task has event/player/audio stages absent from Switch's short barrier. A pending state can delay later dialogue; exact deadlock conditions have not been demonstrated (T `0x7c358`; S `0x140156250`).

Missing or skipped movies, abort input, and BGM can therefore produce behavior that cannot be inferred from the source form alone (S `0x140156327`, `0x140156673`).

## Unverified

[inferred] The Switch halfword's full lifecycle and all Steam task stages, including already-ended/aborted/missing-movie behavior, need runtime evidence. One check: instrument scr0002's TOMP→TOMW sequence for natural completion and abort, logging T local +0x48 and S task +0x58/player +0x7c, event latch and audio requests until the next script command executes (T `0x7c358`; S `0x140156250`).

---

Engine facts here are lifted from the port's `docs/commands/TOMW.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
