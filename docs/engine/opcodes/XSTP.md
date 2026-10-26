# XSTP

Source spelling: `_XSTP()`.

## Operands

- [code] Operandless control action: T `0x77314` branches to `0x7c5b4`.

- It then writes **F036=1** and calls action constructor `0x40550(1,1,0x24,0,0,0)`, then yields (`0x7c5b4–0x7c600`).

- Thus completion depends on engine action status, not a source duration operand.

## Switch build

- [code] Operandless control action: T `0x77314` branches to `0x7c5b4`. If the active word through `[0x3515d8]` is nonzero, it calls `0x5db90`, clears that word and sets control word `0x9e5720` to zero through `0x40140`. It then writes **F036=1** and calls action constructor `0x40550(1,1,0x24,0,0,0)`, then yields (`0x7c5b4–0x7c600`). [code] The action constructor sets an action record under `0x9e5720` with stride `0x134`, active state, parameters and callback table `0x4638b8` (`0x40570–0x40618`). This is asynchronous engine work, not an unconditional VM stop or music-stop command. No direct sound call appears in the dispatch block. Exact presentation and completion predicate are not established by this constructor slice.

## Steam build

- [code] Recipe emits **0x120** (H `src/recipes_data.rs:10611–10632`), S `0x1401097a0`. If global handle `0x1405aaf30` already exists it returns 0. Otherwise it requests engine action **0x5a40** through `0x14006f960`, creates a task at VM +0x24e8 with vtable `0x140284130`, and returns **1** (`0x1401097b2–0x140109852`). [code] Task update is S `0x140156ec0`: while task +0x10 is zero it polls `0x1400e42d0`; result 0 advances to completion and calls `0x1400e4520`, result 2 returns 1, and other initial results return −1 (`0x140156ed8–0x140156f24`). Thus completion depends on engine action status, not a source duration operand. A task cancellation field +0xc also ends the update (`0x140156eca`, `0x140156f25`). The full user-visible meaning of action 0x5a40 remains unverified. [data] Mahoyo uses `0x120` three times in 250 raw files; `raw/d_8.chs:0x217e4` follows presentation commands.

## Differences

- [code] Switch sets F036 and constructs action (1,1,36...); Steam uses global handle/action 0x5a40 plus a task. Native Steam XSTP does not explicitly assign VM F036 in its handler (`0x7c5e8`; S `0x1401097a0–0x140109852`). [inferred] A task whose engine action never reaches completion can hold progression; a generic stop/resume description would conceal this dependency (S `0x140156ee3`). Exact pixels, input suppression and cleanup need a paired runtime trace.

## Unverified

- [inferred] One paired trace through that complete XSTP→XRSM interval should record action/task status, F036, input availability and presentation frames, and confirm the first following statement executes. This would settle the unknown visible effect and possible noncompletion (T `0x40550`; S `0x140156ec0`).

---

Engine facts here are lifted from the port's `docs/commands/XSTP.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
