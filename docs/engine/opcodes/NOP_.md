# NOP_

Source spelling: `_NOP_()`.

## Operands

- [code] Operandless **interpreter yield**, not a no-op.

- It adds a scheduling boundary rather than a specified millisecond delay; exact elapsed time depends on its caller.

## Switch build

- [code] Operandless **interpreter yield**, not a no-op. The N letter case compares NOP_ at T `0x76728` and branches directly to the yield exit `0x7d00c`, returning current continuation x26. No flag, drawing or audio operation is executed in that handler (`0x76720–0x7672c`, `0x7d00c–0x7d030`). It adds a scheduling boundary rather than a specified millisecond delay; exact elapsed time depends on its caller.

## Steam build

- [code] H's command table has no NOP_ recipe (`src/recipes_data.rs:10724–11109`). No matching Steam opcode is identified; the source NOP_ cannot be treated as an existing VM return. In particular, RTM_ status 3 ends the VM and is not an equivalent yield (S `0x14007ddd0`). [data] Prior Mahoyo source inventory finds zero NOP_ statements. There is no source-specific NOP_ bytecode call to identify in the 250 raw CHS dumps.

## Differences

- [inferred] Pending image/effect updates can get less time before the next command, even though no fixed duration is encoded. Whether scr0202 relies on that boundary is unresolved (T `0x76728`; `scr0202.txt:1`, statement 1241). Replacing it with END_ or RTM_ would be a different control operation (S `0x14007ddd0`).

## Unverified

- [inferred] One paired frame/dispatch trace around scr0202 statement 1241 should measure when SPFC updates become visible relative to the next SEPL and show whether dropping the boundary matters (T `0x76728`. This timing check has not run.

---

Engine facts here are lifted from the port's `docs/commands/NOP_.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
