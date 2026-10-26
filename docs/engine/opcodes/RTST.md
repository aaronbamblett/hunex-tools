# RTST

Source spelling: `_RTST(486)`.

## Operands

- [code] T `0x77bb0 → 0x8fc30` takes an index `0..575`; missing/invalid input does nothing.

- [code] T `0x8fc94–0x8fcac` also writes current index at local +0xdc if the index is 533..557 or F024 (local +0x30) is nonzero.

- [data] Mahoyo has 205 `0x128` calls, e.g. `mahoyo-steam/raw/1_0.chs:0x7bdc`, operand 0.

## Switch build

[code] T `0x77bb0 → 0x8fc30` takes an index `0..575`; missing/invalid input does nothing. It sets a **local session** bit at `0x1251b5a + 2*(0xc9 + (index>>3))`, mask `1<<(index&7)`. This is distinct from RDST's persistent bank and RDED's completion bank.

[code] T `0x8fc94–0x8fcac` also writes current index at local +0xdc if the index is 533..557 or F024 (local +0x30) is nonzero. RDED uses current index/session state at `0x8f958–0x8f978`; other readers include `0x8f830`, `0x13498`, `0x14d44`, `0x80170`. It resumes dispatch at `0x77bb4`, with no direct drawing, audio or wait task.

## Steam build

[code] No direct session/current-index handler is identified. Closest traced primitives are VM variable access S `0x14007bed0`, `0x14007dc00` and persistent bit read/write `0x14007db00`, `0x14007e5e0`. Native read-bit opcode `0x128` / `0x140109910` sets a bit immediately without these current-index rules (compiler RDST recipe `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:1815–1841`).

[data] Mahoyo has 205 `0x128` calls, e.g. `mahoyo-steam/raw/1_0.chs:0x7bdc`, operand 0. This demonstrates its native marking primitive, not Tsukihime's RTST session bank; no direct RTST recipe was identified (250 successful dumps).

## Differences

[inferred] Without RTST's session and current-index writes, faithful RDED validation is lost even if completion writes are restored in isolation. A plain persistent bit also loses session lifetime and the conditional current-index update (T `0x8fc30`, `0x8fc94`, `0x8f910`).

[data] Authored indices span 1..557 (`phase4/scripts/*.txt:1`). [inferred] Steam storage must have explicit ownership/reset rules; native F/G variables and native bit IDs are not automatically free Tsukihime storage (S `0x14007dc00`).

## Unverified

[inferred] Full reset/load/session lifetime and all UI readers remain unresolved. One check: enter index 533 (scr0495/scr0531), inspect local +0x216 bit 5 and current index 533, confirm persistent completion +0x29a bit 5 remains clear, then reload/restart and compare lifetimes (T `0x8fc30`, `0x8f910`).

---

Engine facts here are lifted from the port's `docs/commands/RTST.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
