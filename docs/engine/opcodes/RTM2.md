# RTM2

Source spelling: `_RTM2()`.

## Switch build

[code] Operandless arm keep-music without returning.

- [code] T `0x78ac4–0x78acc` writes nonzero byte `0x1354e6d` and continues at ordinary dispatch `0x76730`. It does not execute RTM_'s return checks, draw/play or yield itself. The next teardown consumes the flag at `0x5ebbc`, bypassing music stop `0x99e70`.

## Steam build

- [code] There is no stock RTM2 opcode or handler; ordinary returns use `0xf4` (`raw/1_0.chs:0x7d80`). [data] Stock Mahoyo's 250 raw CHS files have zero `0x261` calls.

## Differences

- [code] Switch RTM2 arms cleanup policy rather than returning immediately; Steam has no matching keep-music return (T `0x78ac4`; S `0x14007ddd0`). Actual save/load mid-flag behavior is unverified. [inferred] Treating RTM2 as a return would skip the following completion mark in scr0003; treating it as a no-op would lose music continuity (`scr0003.txt:1`, statements 2126–2128).

## Unverified

- [inferred] One scr0003 ending trace should confirm RDED executes after RTM2, then observe one flag consumption and continuous authored track 2 across the following return; an ordinary next return must clean music normally.
