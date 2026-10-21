# MSAD

Source spelling: `_MSAD($000120)`.

## Operands

- The source operand is message text/reference.

- R `recipes_data.rs:1613` selects `Special::Message`; R `compile.rs:861–915` emits message metadata `0x7b`, then pushes `(mode,1,0,0,text)` and calls `0x140`.

- The handler reads text/auxiliary string and numeric fields (`0x14010a264–0x14010a2b9`), constructs native message state, and can attach its task at `[ctx+0x24e8]` (`0x14010af59–0x14010af6a`).

## Switch build

[code] A message continuation/display command, not a music-start or audio-delay command. Dispatch T `0x79008–0x79018` selects `0x7c7b0`, which conditionally copies saved message state and clears its `+0x7be` flag before calling `0x6e430`. That routine appends source text to the existing message buffer, removes a terminal `@k` when present, limits the buffer to 0x300 bytes, and processes inline controls (`0x6e4e4–0x6e5e8`). The source operand is message text/reference. [data] Empty content and control-only content occur in the corpus (`dump/scr0001.txt:1`, `dump/scr*.txt:1`).

[code] It updates message rendering/state through `0xaa0d0` and can consume VPLY's pending voice through `0x6e804–0x6e87c`, start it at `0x6ea50`, then clear pending at `0x6e8a4`. Dispatch exits through `0x7d00c` after setting up the message (`0x7c80c–0x7c814`). [inferred] Message progression depends on text/input/control state, not simply the clip's end; VCWT is the explicit voice barrier, and MSAD must not be treated as one (T `0x770b0`, `0x6e430`).

## Steam build

[code] Compiler mapping is message opcode `0x140`, handler S `0x14010a170` (registration `0x1400e7725`), shared with ordinary messages. R `recipes_data.rs:1613` selects `Special::Message`; R `compile.rs:861–915` emits message metadata `0x7b`, then pushes `(mode,1,0,0,text)` and calls `0x140`. Mode is 0 when later WKST eq 63 is seen before the next message/window boundary, 1 for text containing `@n`, otherwise WNTY+1. The handler reads text/auxiliary string and numeric fields (`0x14010a264–0x14010a2b9`), constructs native message state, and can attach its task at `[ctx+0x24e8]` (`0x14010af59–0x14010af6a`). Voice-state merging is S `0x14016bbde–0x14016bda3`.

[data] Mahoyo has **23,139 message-opcode 0x140 calls**, including `raw/1_0.chs` file `0x760` (mode 1, numeric fields 1/0/0, text reference $000000). CHS bytecode alone cannot split that count into original MSAD versus ZM source names, since this compiler mapping shares the opcode (R `recipes_data.rs:1613`).

## Differences

[code] Switch's continuation buffer is bounded to 0x300 bytes and strips a trailing `@k`; the compiler's message lowering replaces `^` with a newline and has no MSAD-specific append branch (T `0x6e4e4–0x6e5b4`; R `compile.rs:861–915`). [inferred] Prior text retention, control-only messages, input/auto-mode progression and voice replay can differ. Waiting forever for text-state flags is a potential integration failure if the shared message mode does not produce the state expected by following script; it is not demonstrated by static reading (R `compile.rs:870–890`; T `0x6e8a4–0x6e8b4`).

## Unverified

[inferred] One runtime check: compare the cited ZM→MSAD pair on both engines, then a control-only/empty MSAD with pending VPLY, using manual and auto advance. Observe retained text, line placement, input release and whether voice starts once; this settles whether shared mode lowering preserves continuation and progression (T `0x6e430`; S `0x14010a170`; R `compile.rs:861–915`).

---

Engine facts here are lifted from the port's `docs/commands/MSAD.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
