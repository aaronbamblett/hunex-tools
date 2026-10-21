# STCL

Source spelling: `_STCL(0,2,125,13,125,13,0,0)`.

## Operands

- Operand 0 selects absolute mode 0 or relative nonzero mode; operand 1 is slot, default 0.

- Absolute mode operands 2/3 are initial x/y, 4/5 target x/y, 6 duration and 7 interpolation selector (`0x62410–0x62610`).

- Absent endpoints retain saved coordinates; absent duration/selector are 0 (`0x62588`, `0x625e8`, `0x62518`).

- Relative mode adds operands 2/3 to saved x/y and takes duration/selector at operands 4/5 (`0x62364–0x6240c`, `0x62680–0x62704`).

- Duration converts to `trunc(60*n/1000)` ticks, setter `0x338f0` starts the movement request with live target `+0x28/+0x2c`, counter `+0x78`, deltas `+0xf4/+0xf8`; selector setter `0x33870` writes `+0x76/+0x96` (`0x62610–0x6265c`, `0x338f0–0x3398c`).

- The eight-field form appends a null anchor value; the nine-field form supports the authored empty initial fields and CC anchor.

- [code] Absolute core resolves missing endpoints from pending/current state and registry-based anchor, then writes request `+0xd8`, initial/target coordinates `+0xe4..+0xf0`, pivot `+0xf4/+0xf8`, duration/selector `+0xfc/+0x100` (`0x140101f20–0x140102211`).

## Switch build

[code] T dispatch `0x79b7c` calls `0x62260`. Operand 0 selects absolute mode 0 or relative nonzero mode; operand 1 is slot, default 0. **It clears saved/live STMA animation ids before validating the slot** (`0x62284–0x622fc`, setters `0x92e44` at `0x622c8` and `0x622e4`). Normal valid slots are 0–39. Absolute mode operands 2/3 are initial x/y, 4/5 target x/y, 6 duration and 7 interpolation selector (`0x62410–0x62610`). Absent endpoints retain saved coordinates; absent duration/selector are 0 (`0x62588`, `0x625e8`, `0x62518`). Relative mode adds operands 2/3 to saved x/y and takes duration/selector at operands 4/5 (`0x62364–0x6240c`, `0x62680–0x62704`).

[code] Coordinates scale by x640/1280, y480/720 into saved floats at `+0xa4/+0x144 + slot*4` (`0x62440–0x625d4`). Duration converts to `trunc(60*n/1000)` ticks, setter `0x338f0` starts the movement request with live target `+0x28/+0x2c`, counter `+0x78`, deltas `+0xf4/+0xf8`; selector setter `0x33870` writes `+0x76/+0x96` (`0x62610–0x6265c`, `0x338f0–0x3398c`). Returns 0 and dispatch continues (`0x62660–0x6267c`, `0x79b84`); movement completion is separate STCF mask 1 (`0x33d10–0x33d2c`). No audio start is present.

## Steam build

[code] Matching opcode depends on recipe: absolute eight-/nine-field forms use **`0x243` → S `0x140102240`**, calling core `0x140101f20`; relative seven-field form uses **`0x244` → `0x140102350`** (`recipes_data.rs:7507–7700`; registration `0x14010678f–0x1401067b2`). The eight-field form appends a null anchor value; the nine-field form supports the authored empty initial fields and CC anchor. Mode mismatches diagnose instead of choosing a different relative/absolute path (`0x1401022eb–0x14010232d`, `0x1401023d9–0x1401023ef`).

[code] Absolute core resolves missing endpoints from pending/current state and registry-based anchor, then writes request `+0xd8`, initial/target coordinates `+0xe4..+0xf0`, pivot `+0xf4/+0xf8`, duration/selector `+0xfc/+0x100` (`0x140101f20–0x140102211`). Relative mode adds converted deltas to the pending target (`0x1401023f4–0x1401024eb`). Slots are 0–63. The inspected paths return 0 without installing a wait (`0x140102220`, `0x1401024f9`). They do not explicitly clear the STMA request block `+0x17c..+0x198`.

[data] Mahoyo: **648** occurrences: 645 absolute opcode `0x243` and three relative opcode `0x244`. `1_0.chs@0x2a6c` uses mode 0, slot 13, retained initial x/y, target (240,136), duration 3000, selector 0 and CC anchor. Relative example `8dot5.chs@0x52bdc` uses mode 1, slot 14, y delta -46, duration 300, selector 0 and CC anchor.

## Differences

[code] Source STCL always clears animation ids; no corresponding clear occurs in the inspected Steam position setter paths (T `0x622c8/0x622e4`; S `0x140101f20–0x140102239`, `0x140102350–0x140102514`). [inferred] An active Steam keyframe animation may therefore compete with STCL, pending consumer-level confirmation. Source durations/coordinates use quantized ticks/floats; Steam converts coordinates to integer engine units and stores dword duration. Out-of-range source slots can be indexed **before** the guard; do not extrapolate safety from the 0–39 check.

## Unverified

[inferred] Absolute recipe runtime behavior, implicit animation cancellation in downstream consumers, coordinate rounding and selector-35 timing remain unverified (T `0x338f0`; S `0x140101f20`). **One runtime check:** start STMA on a visible slot, execute absolute STCL with selector 35 and a 500-ms move, then inspect movement, animation state and STCF completion on both engines.

---

Engine facts here are lifted from the port's `docs/commands/STCL.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
