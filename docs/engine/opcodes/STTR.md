# STTR

Source spelling: `_STTR(3,26)`.

## Operands

- Operand 0 selects slot 0–39, default 0; operand 1 is a mask/transition resource number, default 0; operand 2 is a mode packed in the upper four bits of the saved halfword.
- Operand 3 is not read in this inline path.

## Switch build

[code] T dispatch `0x79a50–0x79ac0`, with continuation `0x7a29c–0x7a32c`, implements STTR inline. It skips the operation while either of two mode/status halfwords is set (`0x79a50–0x79a6c`). Positive resource n produces `(n-1) | (mode<<12)`; zero/negative resource stores -1 (`0x7a2c4–0x7a2f0`).

[code] It writes saved mask field `+0x8ec + 2*slot`, updates live `+0x118` for an active slot, and requests resource n-1 through T `0x4c400` (`0x7a2f4–0x7a328`). Resource helper caches the index and opens/clears the mask carrier `0x1c0c` (`0x4c408–0x4c494`). Dispatch continues at `0x76730` without a completion wait (`0x7a32c`). [inferred] Its visual effect is a mask used by subsequent opacity/transition rendering, not an immediate screen fade or audio action. The mode nibble's complete interpretation remains untraced.

## Steam build

[code] Opcode `0x25b` → S `0x140103af0`, registration `0x1401068df–0x1401068ed`; four-field recipe defaults 0/0/0/256 (`_STTR:fields`). Handler accepts slots 0–63, stores raw resource/mode/parameter at command slot `+0x3c/+0x40/+0x44`, sets dirty/update and returns 0 (`0x140103b3f–0x140103bb1`).

[code] Sync resolves resource through S `0x1400a11b0`: only indices 0–14 are accepted, indexing pointer table `0x1405805e0`; negative or >=15 returns null (`0x1400a11b0–0x1400a11ca`, caller `0x14014a069–0x14014a0d7`). A null/missing resource takes the no-installed-mask branch (`0x14014a153–0x14014a1d2`). Opacity rendering consumes the mask and parameter through `0x140091450`/`0x140075280`. No STTR wait task is created.

[data] Mahoyo: 26 occurrences; `5b_14.chs@0x4fc7c` supplies slot 23, resource -1, mode 0, parameter 256 after STRT.

## Differences

[code] Switch resource n is resolved as n-1 and mode is packed into a halfword; Steam stores raw n and independently looks it up in a 15-entry table (T `0x7a2e0–0x7a328`; S `0x140103b99`, `0x1400a11b0`). [data] Among the 110 regularly spelled corpus calls, 64 have positive decimal resource values and 53 have values above 14, up to 47 (`dump/scr*.txt`). [inferred] Those 53 cannot resolve their intended resource through this unpatched lookup; even smaller numbers need resource-identity verification. Operand 3 adds another Steam-only parameter. Compilation success does not restore the source masks.

## Unverified

[inferred] Correct source-mask assets, packed mode semantics, missing-resource fallback appearance and parameter-256 equivalence remain unresolved (T `0x7a2e8`, `0x4c400`; S `0x14014a069–0x14014a1d2`). Replay the scr0049 resource-26 transition while recording both resolved mask resources and opacity progression; capture the mask edge/output in both engines.
