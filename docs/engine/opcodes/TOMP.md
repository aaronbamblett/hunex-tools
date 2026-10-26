# TOMP

Source spelling: `_TOMP(4)`.

## Switch build

[code] Dispatch T `0x7705c–0x77068 → 0x77784` checks execution byte `0x1354e50`, then calls `0x6ea60` and yields when that helper returns nonzero. The helper parses operand 0 (default 0), resets state `0xe05e58`, marks local +0x89e and submits `0x40550(0,0,n...)` (`0x6ea60–0x6eb38`). The special parsed value 7 has an additional gate at `0x6eaa0–0x6eab8`.

[code] Movie selection uses row n in table T `0x357868`, 0x30-byte records containing an index and filename pointer at +8. The table has 134 rows and a -1 sentinel.

[inferred] TOMP submits movie state and yields for setup, while authored script execution can continue before movie completion; TOMW is the explicit completion barrier. A setup yield must not be confused with waiting for the entire movie (T `0x77798`, `0x7c358`). Visible/audible output is the selected movie; exact BGM interaction remains unverified.

## Steam build

[code] Native opcode `0x1c0`, S `0x140106ab0`, consumes no row operand. It clears previous movie handle state at `0x140106ae4–0x140106b10`, selects one of four language-specific opening filenames at `0x140106b6a–0x140106b9a`, creates task vtable `0x140284088`, attaches it to VM+0x24e8 and returns 1 (`0x140106c6e–0x140106c7d`). Compiler source form calls 448 (`_TOMP:source`).

[code] Task update S `0x140156970` waits for native activity/counter gates (`0x1401569a7–0x1401569e0`), opens via `0x1400ef960`, and remains pending while the player at `0x140596f78` has state +0x7c=2 (`0x140156a8f–0x140156aaa`). Abort cleanup calls `0x1400efc70` and clears the handle/control globals (`0x140156aac–0x140156adf`). Thus the native start task also blocks through playback.

[data] Mahoyo has one `0x1c0`, `raw/opening.chs:0x2b8`, after a timer, and one later TOMW at `0x3b4`; it has zero `0x1c2` calls (250 successful dumps; README).

## Differences

[inferred] Timing is not exact: Steam blocks inside TOMP where Switch can execute following fades/timers/sprite operations before TOMW. Preserving both source commands therefore does not preserve overlap (S `0x140156a8f`).

[data] Surveyed script operands are 4..45, so those authored row numbers fit; this does not prove each build supplies every name (`dump/*.txt:1`). [inferred] Resource lookup, abort, BGM and missing-file handling need runtime evidence (S `0x140156a02`).

## Unverified

[inferred] Resource lookup, natural/abort completion, overlap and BGM parity remain unverified. Record scr0002 from TOMP through TOMW on both engines, logging command timestamps/player state and audio, once naturally and once with abort input (T `0x77784`, `0x7c358`; S `0x140156970`, `0x140156250`).
