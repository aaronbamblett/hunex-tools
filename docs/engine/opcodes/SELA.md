# SELA

Source spelling: `_SELA(_D000,)`.

## Switch build

[code] The surveyed form supplies D000 and an empty operand, but mode 3 does not write D000. T `0x7ca88` calls the F1118-zero predicate `0x832c0`; only a zero flag opens `0x8fdf0`'s two-option save prompt. Initialization loads system message `0xd8` at `0x810cc–0x810f0`. T `0x8ff0c → 0x832b0(1)` sets F1118 before interaction finishes.

[code] Yes prepares Save at T `0x82058–0x8206c`, opens it through `0x820b0 → 0x51470`, and waits via `0x51fc0` at `0x820b8`. The interaction is a decision/save-screen barrier, not an autosave request. Specific sound behavior is unverified.

## Steam build

[code] There is no direct SELA recipe in the inspected compiler command table. The engine does have a callable Save entry S `0x1400fb8b0(0)`: it sets `0x140596fc4` and increments pause count at `0x1400fb921`; return codes are 0 new open, 2 pending, 1 blocked by another screen state. This is an engine function, not a proven script-opcode equivalent.

[data] Mahoyo's dumped scripts use the ordinary SELR/SELD facility (423/100 instructions; `raw/test_script.chs:0x2068,0x2118`). No dedicated SELA opcode was identified; native Save entry use is not evidence of a Mahoyo script save prompt (compiler table above).

## Differences

Without an equivalent save-prompt barrier, subsequent statements can run without waiting for a choice or Save closure. F1118 lifecycle also matters when entering through scene selection.

## Unverified

[inferred] Save closure/cancel timing and prompt presentation/audio need a paired runtime capture. Traverse scr0035 with F1118=0, choose Yes then cancel/close Save, and repeat with No and F1118=1; log when following WTTM/TOMP begins and confirm D000 stays unchanged (T `0x820b8`; S `0x1400fb8b0`).
