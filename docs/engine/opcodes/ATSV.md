# ATSV

Source spelling: `_ATSV()`.

## Operands

- Operand 0 is an optional mode: the absent form selects 2; present mode uses its low two bits at `0x76888–0x76894`, and bit 3 returns to ordinary dispatch at `0x76868`.

- [code] It reads/clears the halfword at `0xe05e58` via `0x3515d8`, conditionally calls `0x5db90`, calls `0x40140(0)`, and dispatches mode-dependent save work through `0x5a9c0` at `0x79024–0x79038` or `0x7ae4c–0x7ae5c`.

- The common tail can continue or yield according to the saved halfword at `0x7b094–0x7b0c0`.

- [code] Opcode `0x10e`, S `0x140108f70`, snapshots engine state, pops a mode at `0x140109320–0x140109338`, installs a task at VM+0x24e8 (`0x140109430–0x140109440`) and returns 1.

- Compiler recipes pass an integer unchanged, or push mode 2 when absent (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:10–64`).

## Switch build

[code] T `0x76834–0x768e8` dispatches autosave. Operand 0 is an optional mode: the absent form selects 2; present mode uses its low two bits at `0x76888–0x76894`, and bit 3 returns to ordinary dispatch at `0x76868`. Thus the authored mode 8 form bypasses this path. Execution byte `0x1354e50` can instead send entry to yield (`0x76844–0x76850`; pointer at `0x3519d8`).

[code] It reads/clears the halfword at `0xe05e58` via `0x3515d8`, conditionally calls `0x5db90`, calls `0x40140(0)`, and dispatches mode-dependent save work through `0x5a9c0` at `0x79024–0x79038` or `0x7ae4c–0x7ae5c`. The common tail can continue or yield according to the saved halfword at `0x7b094–0x7b0c0`. The complete asynchronous save lifecycle is not established here.

[inferred] Its purpose is a save checkpoint, with no demonstrated dialogue/audio effect in the dispatcher. Calling it a fixed timer or an unconditional immediate save would overstate the static evidence (T `0x76834`, `0x79024`).

## Steam build

[code] Opcode `0x10e`, S `0x140108f70`, snapshots engine state, pops a mode at `0x140109320–0x140109338`, installs a task at VM+0x24e8 (`0x140109430–0x140109440`) and returns 1. Compiler recipes pass an integer unchanged, or push mode 2 when absent (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:10–64`). Mode 8 still performs handler work before the task bypasses saving.

[code] Task update S `0x14014f300` gates on engine-state +8/+0xc; mode 2 tests `0x140598044`, other ordinary modes test `0x140598040`, and mode 8 finishes without saving (`0x14014f318–0x14014f34c`). It starts job `0x1400ddc40`, polls at `0x14014f39a–0x14014f3c4`, then may wait for globals `0x140598a40` and `0x140594c3c` to clear (`0x14014f428–0x14014f445`). Those globals have additional service/counter dependencies.

[data] Mahoyo has 182 `0x10e` calls; `mahoyo-steam/raw/1_0.chs:0x100` follows a push of 2. This establishes native usage, not safety under Tsukihime's initialization sequence (250 successful dumps; README).

## Differences

[inferred] **Pass-through is not proven exact and can stall indefinitely.** The trace proves which opcode remains pending, not which task predicate/job/global is responsible. A circular dependency is not established (S `0x14014f300`).

[data] Working scr0001 also calls ATSV but sets G010=1 beforehand; that is a source difference, not a proven fix. [code] Switch masks ordinary modes to two bits and bypasses bit 3; Steam receives the original integer and has native mode-specific checks, so even identical spelling does not establish equal semantics (T `0x76868–0x76894`; S `0x140109338`, `0x14014f32f`).

## Unverified

[inferred] The actual pending task state and correct initialization dependency remain unknown. One check: cold-start scr0088 with a debugger logging task +0xc0, mode +0x38, engine +8/+0xc, job status and both final globals at each `0x14014f300` update; compare the same checkpoint reached naturally from scr0087. That distinguishes precondition, job and final-global stalls without removing autosave.

---

Engine facts here are lifted from the port's `docs/commands/ATSV.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
