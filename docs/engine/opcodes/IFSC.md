# IFSC

Source spelling: `_IFSC(,label)`.

## Operands

- [code] `empty,label` calls T `0x48b70`, which reads byte **`0xa1b208`**.

- Nonzero continues; zero copies operand 1 and resolves its label (dispatch `0x76ce8–0x76cf0`, scan `0x76cf4–0x76e20`, label jump `0x7c304`).

- [code] Recipe pushes 1, calls `0x130`, compares equal, and jumps on false (H `src/recipes_data.rs:905–926`; `src/compile.rs:967–980`).

- Handler S `0x140109a00` pushes engine state `[VM+0x2508]+0xc`, which native assignment updates from F024 (S `0x14007c015–0x14007c01f`).

## Switch build

- [code] `empty,label` calls T `0x48b70`, which reads byte **`0xa1b208`**. Nonzero continues; zero copies operand 1 and resolves its label (dispatch `0x76ce8–0x76cf0`, scan `0x76cf4–0x76e20`, label jump `0x7c304`). This is an engine-state predicate, not a timed wait; no direct drawing/audio occurs. The full lifecycle/meaning of that byte is not established by this getter (`0x48b70–0x48b78`).

## Steam build

- [code] Recipe pushes 1, calls `0x130`, compares equal, and jumps on false (H `src/recipes_data.rs:905–926`; `src/compile.rs:967–980`). Handler S `0x140109a00` pushes engine state `[VM+0x2508]+0xc`, which native assignment updates from F024 (S `0x14007c015–0x14007c01f`). The branch therefore jumps when that field is **not 1**. It has no task/wait, draw or audio effect (`0x140109a00–0x140109a28`). [data] Mahoyo uses `0x130` six times in 250 raw files, e.g. `raw/c_5.chs:0x404`, preceded by push 1.

## Differences

- [code] Switch tests a byte for zero/nonzero; Steam compares a dword with exactly 1. Even if both represent the same mode, values such as 2 need not take the same branch (`0x76cec`; H `src/compile.rs:975–978`). [inferred] Assuming this means a platform check would hide a state-mapping dependency. Branch errors can affect progression because the first occurrence precedes F053=1 (`scr0192.txt:1`, statements 10–11).

## Unverified

[inferred] No open engine check is recorded beyond the items above.

---

Engine facts here are lifted from the port's `docs/commands/IFSC.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
