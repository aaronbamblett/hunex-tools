# SNX4

Source spelling: `_SNX4(4,11,11)`.

## Operands

- [code] The supplied Switch dispatch has no SNX4 handler: first-letter table T `0x2355d0` and S-family comparisons fall through to `0x76730`; the inspected live entries for SNX2/SNX3 instead reach `0x78f4c`/`0x799c8`.

- Operands are not parsed; there are no command state writes, audio/visual requests, or waits.

- R `recipes_data.rs:5317–5423` supplies three stack values, treating the surveyed empty component as zero.

- Handler accepts unsigned slot 0–63, marks record `+0/+8` dirty, stores operands 1/2 at `+0xa0/+0xa4`, and returns 0 (`0x14010302b–0x140103098`).

## Switch build

[code] The supplied Switch dispatch has no SNX4 handler: first-letter table T `0x2355d0` and S-family comparisons fall through to `0x76730`; the inspected live entries for SNX2/SNX3 instead reach `0x78f4c`/`0x799c8`. Operands are not parsed; there are no command state writes, audio/visual requests, or waits. The explicit no-handler finding.

## Steam build

[code] Same-named Mahoyo command has opcode `0x256`, handler S `0x140102ff0` (registration `0x140106876–0x140106884`). R `recipes_data.rs:5317–5423` supplies three stack values, treating the surveyed empty component as zero. Handler accepts unsigned slot 0–63, marks record `+0/+8` dirty, stores operands 1/2 at `+0xa0/+0xa4`, and returns 0 (`0x14010302b–0x140103098`). Those fields drive image-variant selection: `0x140148d43–0x140148da2` chooses explicit or inherited values and passes them to `0x1400a1500`.

[data] Mahoyo raw corpus: 9,470 calls; `raw/1_0.chs@0x7458` supplies slot 33, component 0, component 20. [inferred] This is real image-variant state in Steam, not a compatible Switch no-op.

## Differences

[inferred] Re-enabling a compiler-supported command solely because it compiles would change image-variant selection (S `0x140103085`, `0x140148d43`). It does not introduce a wait itself, but variant/resource use could change subsequent behavior. Exact variant geometry/appearance is unresolved (S `0x1400a1500`).

## Unverified

[inferred] One runtime check: trace scr0001 statement 66 on Switch and compare the port's next STCH with an isolated Steam SNX4-enabled variant, logging variant indices and loaded image names (T `0x76730`; S `0x140148da2`).

---

Engine facts here are lifted from the port's `docs/commands/SNX4.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
