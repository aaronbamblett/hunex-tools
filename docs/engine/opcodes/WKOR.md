# WKOR

Source spelling: `_WKOR(G022,2)`.

## Operands

- [code] `destination,value` performs a 16-bit OR, dispatch T `0x78514`, operand resolution `0x7a83c–0x7a8b8`, destination selection `0x7a8b8–0x7a910`, OR/store `0x7a914–0x7a918`.

- Reads the old word and operand; writes the low 16 bits; continues without waiting or a direct audiovisual effect (`0x7a91c`).

- [code] Native consumer T `0x2d150` reads persistent G through table `0x231858`; the corpus destinations occur there and gate an audio call at `0x2d24c`.

## Switch build

- [code] `destination,value` performs a 16-bit OR, dispatch T `0x78514`, operand resolution `0x7a83c–0x7a8b8`, destination selection `0x7a8b8–0x7a910`, OR/store `0x7a914–0x7a918`. F, G and underscore compatibility addresses are supported; this destination path has no C case. Reads the old word and operand; writes the low 16 bits; continues without waiting or a direct audiovisual effect (`0x7a91c`). [code] Native consumer T `0x2d150` reads persistent G through table `0x231858`; the corpus destinations occur there and gate an audio call at `0x2d24c`.

## Steam build

- [code] There is no single WKOR recipe. Existing VM OR `0x26` at S `0x14007cf90`, AND `0x25`, flag read `0xea` at `0x14007dc00`, and store `0xe9` at `0x14007bed0` implement the arithmetic. Expression support is H `src/recipes_data.rs:10286` and `src/compile.rs:420–454`; F/G words are 32-bit, so the mask matters. These instructions return immediately; they do not reproduce the native Switch consumer. [data] All 250 Mahoyo raw CHS dumps contain **zero `0x26` calls**; assignment/read primitives are present, e.g. `raw/1_0.chs:0xc4`. Absence of OR calls in this corpus does not mean the engine lacks the registered handler (S `0x14007cf90`).

## Differences

- [code] The lowering has no counterpart for native audio eligibility T `0x2d150`. G also shares Steam persistent variable storage rather than reproducing Tsukihime's complete system state (S `0x14007bf52`). [data] The source corpus contains **nineteen decimal 2 and two decimal 1 operands** (`phase4/scripts/scr*.txt:1`).

## Unverified

- [inferred] Save persistence and the final replay/audio eligibility consequence remain unverified. One paired G020 replay trace should apply both masks, repeat them, restart, and compare stored bits plus eligibility at T `0x2d150` against Steam; arithmetic alone cannot establish consumer parity.

---

Engine facts here are lifted from the port's `docs/commands/WKOR.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
