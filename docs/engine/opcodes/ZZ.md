# ZZ

Source spelling: `_ZZ00b01(START)`.

## Operands

- The label argument is independent of the hex suffix; an empty label still carries suffix bookkeeping.

## Switch build

- [code] `ZZ<five hex digits>(optional-label)` is a label/history marker, not a dialogue statement. Shared Z handling decodes a three-hex-digit group and two-hex-digit sequence, stores current and remembered group/sequence state and checks per-group history/skip data (T `0x77c10–0x77cb0`, `0x77ccc–0x77cf0`). [code] At T `0x7849c–0x784ac`, only M reaches display and Y takes its special label path; Z continues at `0x76730`. Thus normal passage through ZZ adds no dialogue/click wait or direct audio. Local branches resolve named targets through `0x592c0` (`0x7c304–0x7c310`).

## Steam build

- [code] H canonicalizes all-hex ZZ suffixes (`src/source.rs:294–304`). Its label recipes accept empty form or a symbol and emit no command body (`_ZZ:0, _ZZ:1`). A named label defines a compiled offset; the prologue also emits name comparisons via `0xf7` and conditional jumps for scene entry (H `compile::lower_special`). There is no one-to-one runtime ZZ opcode. [code] S `0x14007cad0`/`0x14007cb00` perform offset branches; these manipulate IP immediately and do not reproduce suffix history state. Empty labels produce no history operation (H `_ZZ:0`). [data] Mahoyo's 250 raw CHS files contain 1,071 conditional jumps and 198 unconditional jumps, including compiler-generated labels/macros. Its `raw/1_0.chs:0x4c–0x60` compares START through entry dispatch; these counts cannot recover the number of source ZZ markers.

## Differences

- [code] A named label and suffix history identity are independent. Keeping label reachability does not keep the shared Z state writes (T `0x77c98–0x77cb0`; H `src/source.rs:294`). [inferred] Save/read/skip behavior at empty or branch-entry ZZ markers may diverge without any missing visible text. Message tokens such as `$NNNNNN` belong to ZM/ZD arguments, not ZZ's label namespace (`scr0000.txt:1`, statements 0–2; H `_ZZ:1`).

## Unverified

- [inferred] One paired non-default scene-entry run targeting an authored ZZ should compare the first executed statement and subsequent read/skip/save behavior, watching T's suffix fields. A successful offset branch alone cannot settle history parity (T `0x77c98`, `0x592c0`; H `compile::compile`).
