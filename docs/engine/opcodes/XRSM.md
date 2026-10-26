# XRSM

Source spelling: `_XRSM()`.

## Switch build

[code] Operandless control action at T `0x77a70`: calls `0x40980(1,1)` and continues at `0x76730`.

- [code] The helper writes 1 to action record +0x2c at `0x9e5720 + 0x134*1 + 0x2c` and returns zero (`0x40980–0x40998`). It signals the earlier action; it does not explicitly clear F036, draw, play, or yield in this dispatch block. [inferred] The paired placement after XSTP suggests a resume/release role, but the exact input/presentation effect requires tracing the action consumer; the setter alone proves only the state change (T `0x40994`; `scr0445.txt:1`, statements 2345–2434).

## Steam build

- [code] Recipe emits 0x121 (H `_XRSM:0`), handler S `0x140109860`. If global handle `0x1405aaf30` is nonnull it calls `0x14006fd60(engine,handle)` and clears that global; if null it does nothing. Returns 0 with no VM wait/task creation or direct sound operation (`0x140109864–0x14010988d`). [data] Mahoyo uses `0x121` three times in 250 raw files, e.g. `raw/d_8.chs:0x238f4`, after an explicit WTTM call. XSTP's matching global handle is created at S `0x1401097ea`.

## Differences

- [code] Source dispatch continues without waiting; Steam also returns 0, but clears the stored handle immediately after the engine call. Neither block establishes the full visible/input restoration lifecycle (same addresses). [inferred] Native handle ownership, cancellation, repeat calls and save/load mid-action can change the result even when the script pair compiles (`0x1405aaf30`; T action record `0x9e5880`).

## Unverified

- [inferred] One paired trace across that release should compare the source +0x2c signal and Steam handle removal, then verify input/presentation restoration and continued execution. The same trace should include the preceding XSTP; it has not been run (T `0x40980`; S `0x140109860`).
