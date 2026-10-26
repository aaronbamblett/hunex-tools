# DMMY

Source spelling: `_DMMY(F823,1)`.

## Operands

- [code] **Undispatched no-op**: first-letter D table entry at T `0x2355d6` leads to unknown-command path `0x76730`.

- The engine does not parse its operands, write the apparent flag destination, draw, play, or install a wait.

- [code] There is no DMMY recipe in H's command table (`src/recipes_data.rs:10724–11109`); no Steam instruction is needed to preserve the absent effect.

- The raw Steam dump has no DMMY-specific opcode to count; generic assignments must not be attributed to inert DMMY operands.

## Switch build

- [code] **Undispatched no-op**: first-letter D table entry at T `0x2355d6` leads to unknown-command path `0x76730`. The engine does not parse its operands, write the apparent flag destination, draw, play, or install a wait. It contributes to the generic interpreter statement budget (`0x76730–0x7673c`).

## Steam build

- [code] There is no DMMY recipe in H's command table (`src/recipes_data.rs:10724–11109`); no Steam instruction is needed to preserve the absent effect. Prior Mahoyo Switch dispatch also sends D to its miss path `0x57508`. This is Switch evidence, not a Steam handler address. [data] Mahoyo source inventory contains zero DMMY statements. The raw Steam dump has no DMMY-specific opcode to count; generic assignments must not be attributed to inert DMMY operands.

## Differences

- [code] DMMY operands can look like WKAD or WKST, but they are never executed by the source dispatcher (T `0x2355d6`, `0x76730`). Lowering those operands to actual flag writes would introduce new state changes. [inferred] Removing statements also removes their contribution to the 1,000-statement yield budget; no authored dependency on that budget is demonstrated (`0x76738`).

## Unverified

- [inferred] One Switch breakpoint trace across scr0082's dummy group should confirm all apparent destinations remain untouched and dispatch proceeds through `0x76730`. This checks the static no-op conclusion in actual execution.

---

Engine facts here are lifted from the port's `docs/commands/DMMY.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
