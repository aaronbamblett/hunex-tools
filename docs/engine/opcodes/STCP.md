# STCP

Source spelling: `_STCP(1,1)`.

## Operands

- Operand 0 defaults to 0 and must be slot 0–39; operand 1 is priority/order, default 0.

- It prepares the stage snapshot if needed, writes the low 16 bits of priority to saved slot field `+0x234 + 2*slot`, and returns 0 (`0x62728–0x627f8`, especially `0x627b4–0x627e4`).

- [inferred] The write changes subsequent sprite draw ordering, without directly changing pixels or sound (same field is STCH operand 6 at T `0x61b5c`).

- Recipe requires two integers and pushes priority then slot (`recipes_data.rs:7701`).

- Handler accepts slots 0–63, writes priority as a **dword** at command slot `+0x9c`, and sets dirty/update flags (`0x140102ca7–0x140102d01`).

- [data] Mahoyo: **32,083** occurrences; `1_0.chs@0x158` sets slot 1 priority 1 before the first STCH at `0x21c`.

## Switch build

[code] T `0x79b88` → `0x62710`. Operand 0 defaults to 0 and must be slot 0–39; operand 1 is priority/order, default 0. It prepares the stage snapshot if needed, writes the low 16 bits of priority to saved slot field `+0x234 + 2*slot`, and returns 0 (`0x62728–0x627f8`, especially `0x627b4–0x627e4`). Dispatch resumes at `0x76730`; it does not wait. [inferred] The write changes subsequent sprite draw ordering, without directly changing pixels or sound (same field is STCH operand 6 at T `0x61b5c`).

## Steam build

[code] Opcode `0x251` → S `0x140102c80` (registration `0x14010680d–0x14010681b`). Recipe requires two integers and pushes priority then slot (`recipes_data.rs:7701`). Handler accepts slots 0–63, writes priority as a **dword** at command slot `+0x9c`, and sets dirty/update flags (`0x140102ca7–0x140102d01`). Return 0 (`0x140102d08`). Stage sync applies the priority through the node vtable and reorders at `0x14014a1ed–0x14014a22c`; there is no completion task or audio call.

[data] Mahoyo: **32,083** occurrences; `1_0.chs@0x158` sets slot 1 priority 1 before the first STCH at `0x21c`.

## Differences

[code] Source storage is 16-bit and destination storage 32-bit; source accepts 40 slots and destination 64 (T `0x627ac`, `0x627e4`; S `0x140102ca7`, `0x140102d01`). [inferred] Default insertion is exact, but out-of-range priority wrapping and ordering ties are not proven equivalent. It is a state setter, so STCP cannot substitute for waiting on movement or visibility (T `0x62710`; S `0x140102c80`).

## Unverified

[inferred] Tie ordering and values beyond signed/unsigned 16-bit limits remain unsettled (T `0x627e4`; S `0x14014a205`). **One runtime check:** overlap two slots, assign equal priorities and then boundary priorities, and compare ordering plus stored values on both engines.

---

Engine facts here are lifted from the port's `docs/commands/STCP.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
