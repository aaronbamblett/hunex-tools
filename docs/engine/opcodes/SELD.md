# SELD

Source spelling: `_SELD(_D000,1)`.

## Operands

- Operand 0 is the result destination; operand 1 is an optional choice ID.

## Switch build

[code] Starts choice mode 1 (T `0x7caa4`) using rows registered by SELR. T `0x7ce5c–0x7ce6c` stores that ID at `0x1357da0`; `0x8ff30` reads three integers per ID from `0x23df18` and sets history bits in `0x1252faa`. ID 1 maps to 605/606/607.

[code] Entry reads seen byte `0x125654e[id]` at T `0x7ce8c–0x7ce98`, passes its nonzero status to `0x3c440(1,seen)`, and the speed selector `0x3c460` consumes it. Choice processing waits for confirmation; `0x81ab0–0x81ad0` stores the selected authored value, not row position, into the result destination. Confirmation ORs `1 << option_value` into the seen byte at `0x81ae4–0x81b10`. History/seen storage is copied at `0x4fcc0–0x4fcc4`, `0x4fcfc–0x4fd08`.

[inferred] Visible effect is an interactive choice and timing depends on confirmation/seen-choice controls; no specific audio behavior is established by this trace (T `0x81ab0`, `0x3c440`).

## Steam build

[code] Opcode `0x161`, S `0x140049c20`, pops four stack values and constructs a choice task with chosen index -1 at `0x140049dc3`. The source recipe accepts only destination D000 and an empty second operand, pushes 0,2,0,2 and calls 353 (`_SELD:0`). Completion S `0x140155ae8–0x140155b5a` resolves the row to its authored value and writes the destination. The popped values are stored at task +0x70/+0x74/+0x78/+0x7c; completion interprets +0x70/+0x74 as destination bank/index (bank 2 is the 600..700 compatibility alias). It attaches the task to VM+0x24e8 and returns 1, so execution waits on the choice task (S `0x140049e0f–0x140049e35`, `0x140155b04–0x140155b5a`). It has no Tsukihime choice-ID field.

[data] Mahoyo has 100 `0x161` calls; `raw/test_script.chs:0x2118` follows row registration and pushes the same four constants. Its choice row count is 423 (250 successful dumps; README).

## Differences

[inferred] This is approximate even when the branch result is right: first-visit/replay timing and history consumers differ. Durable storage alone would not restore entry marking, confirmation marking and skip behavior together (T `0x8ff30`, `0x81ae4`, `0x3c440`).

[data] There are 83 numeric-ID calls and 87 empty-ID calls; all 22 early calls carry IDs.

## Unverified

[inferred] The full history-consumer set, durable reset/load rules, seen styling, the second native bank/index pair's full role, and timing remain unresolved. Enter scr0004 fresh, select each branch across save/reload/restart, and compare result value, history bits and seen-choice skip timing on Switch and Steam (T `0x125654e`, `0x1252faa`; S `0x140155ae8`).
