# FCLN

Source spelling: `_FCLN(600,200,,,32,4)`.

## Operands

- Operands 0…4 are unread.

## Switch build

[code] T `0x8f610` reads only operand 5 as overlay type (default 0), writes saved byte `+0x19a9` and calls `0x83bc0(type,0)`. Types 4…33 select 400-entry pools (count table `0x236ffc`, entries `0x13583f4`, stride `0x38`). Initializer `0x83990` reads per-type parameters at `0x235f1c` with stride `0x90`; draw `0x84730` produces focus-line overlays. The handler returns zero and sets dirty state, without waiting or audio.

## Steam build

[code] Opcode `0x3a0`, handler S `0x140106610`, consumes five operands without reading their values and schedules the fixed focus-line overlay at global context `+0x108…+0x124`, returning zero. Initializer S `0x140151320` reads table `0x140580530`; update is `0x140151ef0`. S `0x140151010` is on the destruction path, not the constructor. Recipe: `_FCLN:source`.

[data] Mahoyo: 1 call(s) (`raw/wik_q_a.chs`, assembly-dump line 7956).

## Differences

[code] Switch selects overlay type from 400-entry pools while Steam schedules one fixed overlay (T `0x236ffc`; S `0x140106610`): type selection has no Steam counterpart. The established Switch-game comparison is 400 entries versus Mahoyo Switch's fixed type-4 pool of 20 (T `0x236ffc`; Mahoyo T `0x65960`). [inferred] Do not transfer that pool-size observation to Steam without tracing its allocation; Steam's fixed geometry/density still has no proven match to the Tsukihime type.

## Unverified

[inferred] Full Steam task mapping and matching geometry/density remain unresolved. Show source types 4, 5 and 32 consecutively and compare both engines through an immediate and a 2000-ms FCSP stop; each type should remain distinct (T `0x83990`, `0x84730`; S `0x140151320`).
