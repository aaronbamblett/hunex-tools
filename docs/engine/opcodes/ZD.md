# ZD

Source spelling: `_ZD1b395($007927)`.

## Operands

- Its argument's decimal text ID is a different identity; suffix decoding reads the command name, not that token.

## Switch build

- [code] `ZD<five hex digits>(text-id)` participates in the same **three-digit hex group/two-digit hex sequence** bookkeeping as ZM, writing current/remembered suffix fields and consulting read/skip state (T `0x77c10–0x77cb0`, `0x77ccc–0x77cf0`). Its argument's decimal text ID is a different identity; suffix decoding reads the command name, not that token. [code] It **does not display dialogue or add a click wait**. After shared bookkeeping, T `0x7849c–0x784ac` sends Y to its label path and M to `0x7c380`; D instead continues at `0x76730`. No message routine or direct voice starter is called by that D-specific branch. Shared bookkeeping/cleanup is why this is not a completely effect-free dummy.

## Steam build

- [code] H canonicalizes ZM/ZZ/ZY only; ZD is not a compiler family or native message recipe (`src/source.rs:294–304`, `src/recipes_data.rs:10633–10722`). Renaming it to ZM would emit message opcode `0x140`, whose handler can install a yielding native task at S `0x14010af59–0x14010af6a`. [data] Mahoyo's 250 raw Steam dumps have message opcode `0x140` 23,139 times but no ZD-specific encoding; bytecode does not retain source family suffixes. Those message calls cannot be counted as ZD markers (`raw/1_0.chs:0x760` is an ordinary message).

## Differences

- [code] A rename would introduce extra dialogue/input waits absent on Switch. Dropping avoids that but loses current/history state written at T `0x77c98–0x77cb0`. [inferred] Equal-text neighboring ZD/ZM statements are not proof that ZD is a duplicate display; different suffixes and shared read/skip state can still matter (T `0x77c98`; `scr0119.txt:1`).

## Unverified

- [inferred] One paired trace at this marker should confirm no extra dialogue/click boundary, then inspect suffix/read-history state and replay the next genuine message under skip. This settles the remaining bookkeeping consequence (T `0x77c98`, `0x784a8`.

---

Engine facts here are lifted from the port's `docs/commands/ZD.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
