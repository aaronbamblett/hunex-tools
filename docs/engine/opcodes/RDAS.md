# RDAS

Source spelling: `_RDAS(486)`.

## Operands

- [code] Operand 0 is a scene/read index.

## Switch build

[code] T `0x78cd4–0x78d34` tests execution/skip state and completion via `0x8f7b0`; it can yield at `0x7cd78`. The completion getter accepts `0..575`, returning 0 outside that range, and reads persistent bank word `0x12c + (index>>4)`, bit `index&15` (`0x8f7b0–0x8f7e0`). Calls `0x3c5f0`/`0x3c6f0` also inspect transition state.

[code] Helper T `0x8fa50` requires guards including `0x5b060 == false`, word `0xe05e58 == 0`, `0x43690 == true`, and an already-completed index, then calls `0x436d0(1)` to write control word `0x9e63c8`. [inferred] It changes skip/control timing rather than writing a read bit; exact meanings of all guards and resulting visible/audio effects remain unresolved (`0x8fa60–0x8fad4`).

## Steam build

[code] The nearest traced skip-target facility is opcode `0x122`, S `0x140109890`, writing engine control fields +0x388..+0x390; it does not query Tsukihime's completion bank. It pops two dwords, writes +0x388=1, clears +0x38c/+0x390 for a negative top argument or enables +0x38c and stores the other argument at +0x390, and returns 0 without a wait task (`0x140109895–0x14010990a`). Consequently its control fields do not establish RDAS equivalence.

[data] Mahoyo has four `0x122` calls: `raw/d_8.chs:0x23314` follows target offset 0x23360 and numeric operand 0x7f. It also has 205 native read marks and 182 autosaves (`1_0.chs:0x7bdc,0x100`; 250 successful dumps). These establish native skip/checkpoint primitives, not use of a dedicated RDAS; no matching RDAS recipe was identified (`recipes_data.rs`, command table beginning at line 10727).

## Differences

[inferred] First-visit/replay skip behavior and entry-transition timing can differ even when dialogue displays correctly. The `0x122` facility does not supply completion state or all predicates, and neither does it replace the RDED completion writer (T `0x78cd4`, `0x8fa50`; S `0x140109890`).

[data] Authored indices span 1..557; some later scripts repeat the entry operations (`dump/*.txt:1`). [inferred] Waiting behavior must be traced separately from the numerical index range; the same argument does not imply the same completion source (T `0x8f7b0`).

## Unverified

[inferred] Every guard's player-facing meaning and a faithful Steam control mapping remain unresolved. Compare a fresh visit and completed-scene replay of scr0001 with identical skip input, logging T `0x78cd4`/`0x8fa50` guards, yield duration and the matching Steam transition/unread-stop state (S `0x140109890`).
