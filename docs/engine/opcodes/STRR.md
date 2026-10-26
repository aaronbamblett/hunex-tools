# STRR

Source spelling: `_STRR(16,-1386)`.

## Operands

- Operands are zero-based.

- It writes a signed centidegree angle at saved state `+0xbe4 + 2*slot` (store `0x632cc`), wraps angles to −17999…18000 and maps −99999 to −32768, then snapshots state and marks it dirty.

## Switch build

[code] T `0x63190` accepts `(slot,clipAngle)` for slots 0…39. It writes a signed centidegree angle at saved state `+0xbe4 + 2*slot` (store `0x632cc`), wraps angles to −17999…18000 and maps −99999 to −32768, then snapshots state and marks it dirty. Draw-list T `0x861c4…0x861ec` takes the slot or parent angle. Renderer T `0x885c8…0x88840` uses rotated clip quads; zero uses axis-aligned scissor T `0xb1650`. It changes the clip window, independently of sprite rotation, and does not wait or request audio.

## Steam build

[code] No matching native clip-angle opcode was established. Region setter S `0x1400762f0` takes no angle; image-rotation STRT S `0x140102920` is a different operation. Region-composite path S `0x14008aebb…0x14008c1ad` is the candidate render facility; equivalence is not established.

[data] No native counterpart occurrence was identified in the 250 successfully dumped Mahoyo CHS files

## Differences

[code] Rotated windows become axis-aligned. STCH operand 19 writes the same field (T `0x61880`); a following STRR does not recover it. [inferred] Rotating the sprite instead would also rotate its image and would not restore the intended clipping.

## Unverified

[inferred] Child/composite render hooks and exact inherited clip geometry remain unresolved. One runtime check: compare scr0038's 90-degree window with Switch while holding the image rotation fixed; the boundary must rotate independently and clip the child correctly (T `0x885c8`; S `0x14008aebb`).

---

Engine facts here are lifted from the port's `docs/commands/STRR.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
