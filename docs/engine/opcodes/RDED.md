# RDED

Source spelling: `_RDED(486)`.

## Operands

- [code] Dispatcher T `0x78d38 → 0x8f910` validates operand 0 as index `0..575`, requires it to equal current index at local `0x1251b5a+0xdc`, and requires its RTST session bit.

- [code] No direct validated-completion equivalent is identified.

- Native opcode `0x128` / S `0x140109910` immediately sets a native read bit, while general persistent-bit writing `0x14007e5e0` serializes through system writer `0x140036b30`, `0x140036da1–0x140036df3`.

## Switch build

[code] Dispatcher T `0x78d38 → 0x8f910` validates operand 0 as index `0..575`, requires it to equal current index at local `0x1251b5a+0xdc`, and requires its RTST session bit. Success ORs `1 << (index & 15)` into persistent `0x1252faa + 2*(0x12c + (index>>4))` and returns 0; rejection returns 1 (`0x8f930–0x8f9b8`).

[code] On successful marking with F024 nonzero, dispatch conditionally calls `0x5db90`, calls cleanup `0x5eb20`, sets execution state 2 and yields through `0x7d00c` (`0x7c858`). The marking helper itself draws/plays nothing; cleanup's full visible/audible consequences are not settled. Readers include RDAS, IFUR and native UI eligibility (`0x78d18`, `0x8fab0`, `0x8fbb0`, `0x14ca4–0x14d48`).

## Steam build

[code] No direct validated-completion equivalent is identified. Native opcode `0x128` / S `0x140109910` immediately sets a native read bit, while general persistent-bit writing `0x14007e5e0` serializes through system writer `0x140036b30`, `0x140036da1–0x140036df3`. Neither performs Tsukihime's current-index/session validation or F024 cleanup (`/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:1815–1841`).

[data] Mahoyo has 205 native `0x128` calls, e.g. `mahoyo-steam/raw/1_0.chs:0x7bdc` with index 0. That is usage of the nearest bit primitive; no RDED recipe/handler was identified.

## Differences

[inferred] Missing persistent completion affects native eligibility and IFUR's completion checks; RDST would merge distinct banks and omit validation. Without the F024 path, execution lifetime/timing also changes (T `0x8f910`, `0x8fbb0`, `0x7c858`).

[data] Authored indices span 1..557 (`phase4/scripts/*.txt:1`). [inferred] A future mapping must preserve the rejected-index behavior and durable storage across restart, rather than only setting an in-memory flag (T `0x8f930`; S `0x140036b30`).

## Unverified

[inferred] Complete completion-bit lifetime and F024 cleanup presentation/audio remain unresolved. One check: finish index 533, restart and inspect persistent +0x29a bit 5; within the same instrumented scenario repeat with a wrong current index/unset session bit and then F024 enabled, comparing state writes and cleanup/yield with Switch (T `0x8f910`, `0x7c858`).

---

Engine facts here are lifted from the port's `docs/commands/RDED.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
