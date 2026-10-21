# FADS

Source spelling: `_FADS(1,0,0)`.

## Operands

- It reads skip/control state through `0x3515d8`, scene state through `0x351620`, and phase bytes `0x1354e58/+4/+8`.

- [code] Core `0x602c0–0x604ec` parses operand 0 (transition direction/mode, default 0), operand 1 (timing value, default 0), operand 2 (effect selector, default 0), and operand 3 (message-window policy, default -1), then commits scene state through `0x5d610`.

- Operand 3 equal to 1 explicitly suppresses the message window at `0x60410`.

- [inferred] Operand 1 is the authored transition duration in milliseconds; the full effect-selector table and Switch operands 4/5 are not established here.

- [code] R `recipes_data.rs:407–521` accepts 1–6 operands and emits `0x200`, filling defaults 0,0,0,-1,256,0 in source order.

- That handler pops six values; operand 2 indexes a 15-entry transition resource table when in 0–14 (`0x1400ff67b–0x1400ff698`), operand 3 controls closing the message window (`0x1400ff69f–0x1400ff71e`), operand 4 goes to task `+0x80`, and operand 5 selects input mask `0x11` versus `0x32` (`0x1400ff844–0x1400ff862`).

- `raw/1_0.chs@0x2fc` has reverse-order pushes 0,256,0,0,0,1, corresponding to mode 1 with zero duration/effect and explicit window policy 0.

## Switch build

[code] T dispatch `0x76978–0x769cc` recognizes FADS and enters a multi-update transition state machine. It reads skip/control state through `0x3515d8`, scene state through `0x351620`, and phase bytes `0x1354e58/+4/+8`. It checks pending slot resources using `0x2f2d0` and readiness through `0xb54f0`, then calls `0x602c0` (T `0x7c9c0–0x7ca74`). Retry paths restore the current statement with `x26=x23` at `0x7cd78`; this is a real yielding barrier.

[code] Core `0x602c0–0x604ec` parses operand 0 (transition direction/mode, default 0), operand 1 (timing value, default 0), operand 2 (effect selector, default 0), and operand 3 (message-window policy, default -1), then commits scene state through `0x5d610`. Operand 3 equal to 1 explicitly suppresses the message window at `0x60410`. [inferred] Operand 1 is the authored transition duration in milliseconds; the full effect-selector table and Switch operands 4/5 are not established here. The immediate effect is a scene transition, with possible window closure; no audio start is present in the inspected core.

## Steam build

[code] R `recipes_data.rs:407–521` accepts 1–6 operands and emits `0x200`, filling defaults 0,0,0,-1,256,0 in source order. S registration `0x1401066bd–0x1401066cb` binds it to `0x1400ff5f0`. That handler pops six values; operand 2 indexes a 15-entry transition resource table when in 0–14 (`0x1400ff67b–0x1400ff698`), operand 3 controls closing the message window (`0x1400ff69f–0x1400ff71e`), operand 4 goes to task `+0x80`, and operand 5 selects input mask `0x11` versus `0x32` (`0x1400ff844–0x1400ff862`).

[code] It creates a fade task, stores mode/timing at task `+0x58/+0x84`, installs it at interpreter `+0x24e8`, and returns 1 (`0x1400ff728–0x1400ff8b1`). Update `0x14015da40` polls fade handles at task `+0x98/+0xa0` through `0x14013ff90`; it is not the STCF command-record scan.

[data] Mahoyo raw corpus: 24,435 opcode `0x200` calls. `raw/1_0.chs@0x2fc` has reverse-order pushes 0,256,0,0,0,1, corresponding to mode 1 with zero duration/effect and explicit window policy 0.

## Differences

[data] Source numeric effect operands span 0–49 (`dump/scr*.txt:1`); Steam's resource-name lookup only accepts 0–14 (S `0x1400ff686`). [inferred] This is an operand compatibility risk, not proof that every selector above 14 hangs. A pending fade/resource handle can keep the task alive; unlike WTTM this is not solely a finite timer (S `0x14015da7b–0x14015db3f`).

## Unverified

[inferred] Effect appearance, selector fallback, skip behavior and handle completion remain unverified. One runtime check: replay a corpus FADS with selector above 14 while logging both fade handles to completion and comparing the corresponding Switch transition (T `0x602c0`; S `0x14015da40`).

---

Engine facts here are lifted from the port's `docs/commands/FADS.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
