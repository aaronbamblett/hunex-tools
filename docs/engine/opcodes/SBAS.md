# SBAS

Source spelling: `_SBAS(14,13)`.

## Operands

- Operands are zero-based.

- [code] Dispatch T `0x797b8…0x79820` parses `(child,parent)` with defaults child 0, parent −1 and stores a signed halfword in saved slot parent array `+0x93c + child*2` at `0x7981c`.

- Sprite creation copies/remaps links at `0x60c50`, `0x615a8…0x616c0`; draw composition reads parent transforms.

## Switch build

[code] Dispatch T `0x797b8…0x79820` parses `(child,parent)` with defaults child 0, parent −1 and stores a signed halfword in saved slot parent array `+0x93c + child*2` at `0x7981c`. This dispatch block has no explicit slot bounds check. Sprite creation copies/remaps links at `0x60c50`, `0x615a8…0x616c0`; draw composition reads parent transforms. It continues directly to `0x76730`: no yield, timer, wait or audio call. [inferred] The visible effect is that child placement/scale follows its parent; negative parent unbinds.

## Steam build

[code] Stock Steam has no matching native parent link or opcode handler. Source

[data] No SBAS opcode `0x25c` occurs in the 250 successfully dumped native Mahoyo CHS files

## Differences

[inferred] No standalone engine difference is recorded beyond the Switch and Steam behavior above.

## Unverified

[inferred] Runtime owner-guard behavior and saved parent restoration remain unresolved. One runtime check: play scr0515 page 10000's 23→24→25 chain while changing root zoom/rotation, save with children bound, restart and load, then unbind

---

Engine facts here are lifted from the port's `docs/commands/SBAS.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
