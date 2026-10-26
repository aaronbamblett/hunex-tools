# SPRC

Source spelling: `_SPRC()`.

## Switch build

[code] Operand-free dispatch T `0x79500` calls `0x57210`. It walks 30 resource records starting `0x1355058`; for active positive ids it releases the associated resource with `0xb5730` and clears id/parameter arrays at +0, +0x78, +0xf0, +0x168, restoring +0x1e0 to `0xffffff` (`0x5723c…0x57264`). It returns to `0x76730`; no script wait or audio call. [inferred] It clears the shared piece/resource preparation used by SPRD and SPFC.

## Steam build

[code] Opcode `0x221`, S `0x1400ffc40`, takes no operands. It walks Steam slot records and marks preparation state dirty at `+0x1c8` and clear-request flag `+0x21c`, then returns zero (`0x1400ffd17…0x140100225`). The companion SPRD writes the same preparation block (S `0x1400ffb33…0x1400ffc09`). Recipe `_SPRC:0`.

[data] Mahoyo: 38 call(s) (`raw/1_2.chs`, assembly-dump line 13339). These are static counts, not reachability.

## Differences

[code] Switch directly releases a shared 30-entry pool; Steam marks per-slot requests for later consumption (T `0x57210`; S `0x1400ffc40`). [inferred] Clear timing and handling of still-visible prepared resources may differ.

## Unverified

[inferred] Exact Steam consumer/release path and clear timing remain unresolved. At scr0083 statement 169, observe requested pieces, live sprite ownership and resource handles immediately before/after SPRC on both engines; confirm visible sprites survive or disappear in the same way (T `0x57210`; S `0x1400ffc40`).
