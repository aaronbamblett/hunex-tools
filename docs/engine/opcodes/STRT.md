# STRT

Source spelling: `_STRT(0,0,0,0,0)`.

## Operands

- Operand 0 is slot 0–39, default 0.
- Operand 1 is initial image angle, default saved angle `+0x284`; operand 2 is target angle, default 0, and becomes the saved angle.
- Operand 3 is duration, default 0, converted to `trunc(60*n/1000)` ticks.
- Operand 4 is an interpolation control stored at live `+0x82` through `0x33890`; operands 5/6 are auxiliary rotation controls, default 0, stored through setter `0x33990` (`0x628ac–0x62a10`).

## Switch build

[code] T dispatch jump table `0x235646` entry 19 reaches `0x79720` → `0x62800`.

[code] Setter `0x33990` stores initial-minus-target delta at `+0x6c`, target at `+0x50`, controls at `+0x72/+0x74` and timer `+0x84/+0x86`. Handler returns 0 (`0x62a14–0x62a2c`) and dispatch continues (`0x79728`). [inferred] It rotates the image over subsequent updates and does not wait or emit audio; it is distinct from region/clip rotation STRR. Source angle units are centidegrees in the related clip-angle path; precise image-angle conversion still needs end-to-end confirmation (T `0x33990`; S below).

## Steam build

[code] Opcode `0x247` → S `0x140102920`, registration `0x1401067e3–0x1401067f1`; seven-field recipe `_STRT:0`. Absent operand 1 compiles as 0, but an explicit empty operand 1 as `0x80000000`; remaining tail defaults are 0. Handler pops seven fields but retains initial/target angle, duration and selector; the last two are consumed without corresponding stored controls (`0x14010292f–0x140102997`, `0x140102aaf–0x140102ada`). Angles pass through scale/truncate/sign conversion into radians and sign inversion (`0x1401029e6–0x140102aaf`). Request state is `+0x14c`, initial/target `+0x154/+0x158`, duration/selector `+0x15c/+0x160`. Return 0 (`0x140102ae6`); sync later applies rotation (`0x140149f16–0x140149fa4`).

[data] Mahoyo: 18,916 occurrences; `1_0.chs@0x264` supplies seven zero fields after STCH, before STGS.

## Differences

[code] Switch absent initial angle uses saved state; the compiler emits zero for an absent field and a retain-state sentinel for an explicit empty one (`0x628bc–0x628dc`; `_STRT:0`). Switch consumes controls 5/6, while Steam discards their popped values (T `0x6299c–0x629dc`; S `0x140102988–0x140102ada`). [inferred] Common explicit-zero calls are compatible candidates; whole-command exactness, wrapping and easing are not established. This does not restore lost clip rotation (T `0x63190`; S `0x1400762f0`).

## Unverified

[inferred] Exact angle units/wrapping, easing codes and auxiliary-control effects remain unresolved (T `0x33990`; S `0x1401029e6`). Start at a nonzero angle, run STRT with an omitted initial angle and nonzero controls 5/6, and capture per-frame angles plus STCF completion in both engines.
