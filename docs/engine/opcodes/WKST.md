# WKST

Source spelling: `_WKST(F823,eq,0)`.

## Switch build

- [code] Assign `destination,operator,value`; dispatch T `0x77104`, operator parse `0x7a52c`, value parse `0x7b270`, destination/store `0x7b2fc–0x7b388`. The surveyed `eq` form reads operand 2, resolves a literal or F/G/compatibility variable, then stores its low 16 bits. Non-`e` operator behavior is outside the established semantics (`0x7a540–0x7a544`). [code] `F<n>` is a local 16-bit word at `0x1251b5a+2*n`; `G<n>` is persistent at `0x1252faa+2*n`. `C<n>` is F[n+signed(F823)], not F[n] (`0x7b324–0x7b36c`). Underscore compatibility destinations, including `_D000`, use G[n−600] for n=600..700 inclusive, otherwise F[n] (`0x7b34c–0x7b388`). D is not a separately allocated bank. Index arithmetic sign-extends the parsed low 16 bits; these stores do not establish a safe arbitrary index range. [code] Assignment returns to ordinary dispatch at T `0x76730`; it has no direct drawing/audio call or wait (`0x7b36c–0x7b370`). State consumers can change later presentation.

## Steam build

- [code] Generic assignment is opcode `0xe9`, S `0x14007bed0`: stack inputs are value, operator string, index, bank. The operator string is popped but the handler stores the value without comparing it (`0x14007bf0a–0x14007bf27`). It returns 0 immediately.

[code] F=bank 0: 1,280 32-bit VM variables, `[VM+0x50]`, count `[VM+0x4c]` (S `0x14007a921–0x14007a941`). G=bank 1: 256 32-bit system variables, via `[VM+0x10]+0xc8`, array +0x18/count +0x10 (`0x1400cea03–0x1400cea23`, `0x14007bf52–0x14007bf71`). `_D`=bank 2 aliases indices 600..700 to G[0..100], all others to F (`0x14007bf2a–0x14007bf45`; H `src/source.rs:98`). There is no C bank in the source parser (`src/source.rs:123`). Invalid ordinary indices skip the store; assignment still checks special F indices afterward (`0x14007bfa7–0x14007bfb5`). [code] F024/F025/F026 update native control fields; F1104..F1109 update message fields (`0x14007bfb5–0x14007c0b6`). Message recipes omit many zero/63 assignments, retain F823 as compiler state, or emit message task `0x141` on its reset (H `_WKST:0`; S `0x14010b030`). [data] Mahoyo has 3,057 calls to `0xe9` and 7,409 to `0x141` in 250 raw CHS files; e.g. `raw/1_0.chs:0xc4` assigns F023=1 and calls `0x141`. Counts refer to bytecode calls, not recovered source statements.

## Differences

- [code] Switch stores truncate to 16 bits; generic Steam stores preserve 32 bits. Steam also attaches native meaning to indices shared with source state (T `0x7b36c`; S `0x14007bfb5`). [inferred] Relative message/read/skip state needs behavioral validation; successfully compiling renamed operands cannot prove equivalence. The specialized Steam tasks are not an implementation of the source C array (T `0x7b324`; S `0x14010b030`; H `_WKST:3`). Persistent G ownership/save lifecycle also needs an audit before claiming state parity.

## Unverified

- [inferred] Safe Switch bank bounds and complete native consumers/save lifecycle are not established by these stores. One paired runtime trace should exercise a multi-message page, C847 increments and 63, voice/auto/skip and save/load, watching F823 plus the actual C-derived addresses against Steam's message tasks (T `0x7b324`; S `0x14010b030/0x14010b120`). No such trace ran.
