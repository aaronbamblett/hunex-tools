# CLO5

Source spelling: `_CLO5(2000)`.

## Switch build

[code] T `0x7c450–0x7c4e0` parses an optional duration, forces zero in alternate mode, then selects an opening target/slope using current transition state. Shared path `0x7c73c–0x7c770` writes target 1 at `0x125a510`, slope at `0x125a514`, clears message suppression `+0x89e`, sets window alpha/open state through `0x2f350`/`0x2f340`, and returns the next statement. Positive duration uses 16.666666/duration; there is no explicit completion wait.

## Steam build

[code] The matching semantic command is CLO6, opcode `0x153`, handler S `0x14010b480` (registration `0x1400e77a1–0x1400e77b0`). It pops duration, writes open request flags `+0x2a0/+0x2a4/+0x2a8/+0x2cc`, target `0x10000`, duration `+0x2dc`, easing `0x10000`, and returns 0. An already-open flag collapses the positive-duration request to duration zero; nonpositive duration is immediate (`0x14010b49c–0x14010b54e`).

[code] Same-spelled Steam CLO5 instead compiles to `0x154` with remapped modes (H `_CLO5:0, _CLO5:1, _CLO5:2, _CLO5:3`); it is not this timed opening operation.

[data] Mahoyo raw corpus has 12 CLO6 opcode calls; `raw/1_5.chs@0x25c2c` pushes 300.

## Differences

Passing a Switch timed-open form to the same-named Steam command would not preserve the timed-open meaning. [data] Surveyed durations are nonnegative, 0–3000 (`dump/scr*.txt:1`). [inferred] Reopening partway through a close may differ because Steam collapses requests based on its open flag (S `0x14010b4aa`).

## Unverified

[inferred] Replay scr0003 statement 164 from closed and partially closing states; compare the alpha curve, message suppression release and continued execution during the 2000-ms opening (T `0x7c750`; S `0x14010b480`).
