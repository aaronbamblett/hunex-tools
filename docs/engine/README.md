# Engine opcode reference

Script opcodes of the HuneX VM, each described as seen in two engine builds: the Switch build of one game, and the Steam WoH.exe build. Every entry keeps the command name and source spelling, the operand layout, the Switch-build semantics, the Steam-build semantics with its opcode number, the differences between the builds, and the open unverified items. Port-side conversion notes and patch details are not carried over here; each entry covers the engine behavior only.

## Claims and addresses

Facts keep the marks they were recorded with:

- `[code]` — read from disassembly.
- `[data]` — counted or observed in a corpus.
- `[inferred]` — reasoned from the above, not directly observed.

Addresses use these prefixes:

- `T 0x...` — Switch `main.bin` offset or address.
- `S 0x...` — Steam `WoH.unpacked.exe` address.
- `H ...` / `R ...` — paths in this repository's `hunex-chs` crate (compiler recipes and lowering).
- `` `raw/*.chs@...` `` — offset in a Mahoyo Steam script dump.
- `` `dump/scr*.txt` `` — a Tsukihime script-text dump, cited only as evidence for a corpus count.

## How to read an entry

- Title and source spelling: the command name as written in script text.
- `Operands`: what each operand carries, verbatim from the build notes.
- `Switch build`: what the Switch handler does, with `T` addresses.
- `Steam build`: what the Steam side does, with the script opcode number where one exists (`none` marks commands with no stock Steam opcode) and `S` addresses.
- `Differences`: where the two builds diverge.
- `Unverified`: open checks, each naming the addresses and corpus statements that would settle it.

Related: the generated opcode table in [formats/opcodes.md](../formats/opcodes.md), and the per-format notes in [formats/](../formats/README.md).

## Index

### Sprite and stage

| Command | Steam opcode | Meaning |
|---|---|---|
| [STCF](opcodes/STCF.md) | `0x27f` | [code] Wait for selected stage components |
| [STCC](opcodes/STCC.md) | `0x241` | [code] Clear/reset slots |
| [STCH](opcodes/STCH.md) | `0x240` | [code] Create/replace sprite and stage fields |
| [STCP](opcodes/STCP.md) | `0x251` | [code] Set draw priority |
| [STZ4](opcodes/STZ4.md) | `0x246` | [code] Tween x/y scales |
| [STMA](opcodes/STMA.md) | `0x278` | [code] Request authored keyframe animation |
| [STRT](opcodes/STRT.md) | `0x247` | [code] Tween image rotation |
| [STBR](opcodes/STBR.md) | `0x252` | [inferred] Select sprite compositing mode |
| [STEC](opcodes/STEC.md) | `0x253` | [code] Select color/effect preset or RGB |
| [STGS](opcodes/STGS.md) | `0x245` | [code] Tween opacity |
| [STPV](opcodes/STPV.md) | `0x250` | [code] Set pivot and optional angle |
| [STRA](opcodes/STRA.md) | `0x258` | [code] Set auxiliary per-slot effect selector/duration |
| [STNS](opcodes/STNS.md) | `0x257` | [code] Set graphics noise/effect mode |
| [STTR](opcodes/STTR.md) | `0x25b` | [code] Assign a transition-mask resource |
| [STCG](opcodes/STCG.md) | `0x228` | [code] Register seen image |
| [STCL](opcodes/STCL.md) | `0x243 (+ relative 0x244)` | [code] Set/tween absolute or relative position |

### Audio

| Command | Steam opcode | Meaning |
|---|---|---|
| [VPLY](opcodes/VPLY.md) | `0x1a0` | [code] Stage voice for a message |
| [SEPL](opcodes/SEPL.md) | `0x190` | [code] Start or queue SE |
| [SEFD](opcodes/SEFD.md) | `0x195` | [code] Fade or queue SE stop |
| [SEVL](opcodes/SEVL.md) | `0x196` | [code] Ramp or queue SE gain |
| [MPLY](opcodes/MPLY.md) | `0x180` | [code] Play BGM, optionally from a sample frame |
| [MVOL](opcodes/MVOL.md) | `0x186` | [code] Ramp current BGM gain |
| [MFAD](opcodes/MFAD.md) | `0x185` | [code] Fade/stop BGM with optional wait |
| [MSAD](opcodes/MSAD.md) | `0x140 (shares message opcode)` | [code] Continue/display message and consume staged voice |
| [SESP](opcodes/SESP.md) | `0x194` | [code] Stop one/all SE channels |
| [VPL2](opcodes/VPL2.md) | `none` | [code] Start staged voice immediately |
| [VCWT](opcodes/VCWT.md) | `none (barrier uses native op 0x19)` | [code] Wait for voice channel playback to end |

### Text window, page and timing

| Command | Steam opcode | Meaning |
|---|---|---|
| [WNTY](opcodes/WNTY.md) | `none (F822 path uses 0x158)` | [inferred] Select message/window mode |
| [FADS](opcodes/FADS.md) | `0x200` | [inferred] Commit a scene transition and wait |
| [PGST](opcodes/PGST.md) | `0x12a` | [inferred] Record the current page identifier |
| [WTTM](opcodes/WTTM.md) | `0x110` | [inferred] Delay script execution with timer/input policy |
| [CLO3](opcodes/CLO3.md) | `none (same-spelled CLO4 is 0x152)` | [inferred] Close and suppress the old text without waiting |
| [SNX4](opcodes/SNX4.md) | `0x256` | [inferred] Ignored on Switch; Steam image-variant setter |
| [FANX](opcodes/FANX.md) | `0x201` | [inferred] Ignored compatibility command |
| [SNX2](opcodes/SNX2.md) | `0x255` | [inferred] Set the first slot resource parameter |
| [CLO4](opcodes/CLO4.md) | `0x152` | [inferred] Request a timed window close |
| [SNX3](opcodes/SNX3.md) | `0x254` | [inferred] Set the second slot resource parameter |
| [CLO5](opcodes/CLO5.md) | `0x153` | [inferred] Open the message window over a duration |
| [FADE](opcodes/FADE.md) | `none (see FADS 0x200)` | [inferred] Enable/disable the screen-wave distortion |
| [WTSV](opcodes/WTSV.md) | `none` | [inferred] Ignored save-related compatibility spelling |
| [CLO2](opcodes/CLO2.md) | `0x151` | [inferred] Reset and close the message window |
| [TRPY](opcodes/TRPY.md) | `0x138` | [inferred] Ignored on Switch; Steam progression/service hook |

### Flow, variables and control

| Command | Steam opcode | Meaning |
|---|---|---|
| [WKST](opcodes/WKST.md) | `0xe9` | Assign a 16-bit word; C is relative to F823 |
| [WKAD](opcodes/WKAD.md) | `0xeb (+ 0x142 for message counters)` | Add modulo 65536 |
| [WKOR](opcodes/WKOR.md) | `composite (0x26 registered; uses 0xea/0xe9)` | OR a word and mask to 16 bits |
| [IF__](opcodes/IF__.md) | `composite (0xea + 0x30-0x35 + 0x19)` | Textual comparison jumps on false |
| [IFNX](opcodes/IFNX.md) | `0x131` | Undispatched source statement |
| [IFP4](opcodes/IFP4.md) | `0x131` | Jump to operand 1 on Switch |
| [IFSC](opcodes/IFSC.md) | `0x130` | Branch on engine-state predicate |
| [IFUR](opcodes/IFUR.md) | `none` | Gate on completion bits 533..556 |
| [JUMP](opcodes/JUMP.md) | `0xf6 + 0xf3` | Transfer to scene and optional entry |
| [GOTO](opcodes/GOTO.md) | `0x18` | Branch to local label |
| [GOTS](opcodes/GOTS.md) | `0x162` | Branch by selected option value |
| [RTM_](opcodes/RTM_.md) | `0xf4` | Return with source execution state 2 |
| [RTMN](opcodes/RTMN.md) | `none (returns use 0xf4)` | Preserve music and return |
| [RTM2](opcodes/RTM2.md) | `none (returns use 0xf4)` | Arm preservation; continue |
| [END_](opcodes/END_.md) | `0x12c` | Teardown; restore continuation; state 1 |
| [NOP_](opcodes/NOP_.md) | `none` | Yield interpreter to caller |
| [DMMY](opcodes/DMMY.md) | `none` | Ignore all apparent operands |
| [XSTP](opcodes/XSTP.md) | `0x120` | Start asynchronous control action |
| [XRSM](opcodes/XRSM.md) | `0x121` | Signal/release paired action |
| [ZM](opcodes/ZM.md) | `0x140 (shares message opcode)` | Display message and record suffix history |
| [ZD](opcodes/ZD.md) | `none (shares message 0x140)` | Record suffix without displaying text |
| [ZZ](opcodes/ZZ.md) | `none` | Record suffix; define optional label |

### Screen and sprite effects

| Command | Steam opcode | Meaning |
|---|---|---|
| [SQK2](opcodes/SQK2.md) | `0x270` | [inferred] Start bank-1 random sprite displacement |
| [SQS2](opcodes/SQS2.md) | `0x272` | [inferred] Clear bank 1 |
| [SHS2](opcodes/SHS2.md) | `0x272 (STQS)` | [inferred] Clear bank 2, including sway |
| [SSS2](opcodes/SSS2.md) | `0x272` | [inferred] Clear the same bank 2 |
| [SHK2](opcodes/SHK2.md) | `0x270` | [inferred] Start bank-2 sine displacement |
| [SSW2](opcodes/SSW2.md) | `0x274` | [inferred] Start bank-2 sway with authored ranges |
| [MSQK](opcodes/MSQK.md) | `none (see SHOK 0x340)` | [inferred] Start a timed global random shake |
| [STQK](opcodes/STQK.md) | `0x270` | [inferred] Source command is undispatched on Switch |
| [SHOK](opcodes/SHOK.md) | `0x340` | [inferred] Schedule global shock |
| [SCH2](opcodes/SCH2.md) | `0x242` | [inferred] Change image through shared sprite/reference core |
| [SBAS](opcodes/SBAS.md) | `none` | [inferred] Link child to parent slot |
| [SGMA](opcodes/SGMA.md) | `0x279` | [inferred] Capture slot state into an animation frame |
| [SPMA](opcodes/SPMA.md) | `0x27a` | [inferred] Pause/resume sprite animations |
| [SPFC](opcodes/SPFC.md) | `0x222` | [inferred] Prepare image variants with adjustments/tint |
| [SOXY](opcodes/SOXY.md) | `0x259` | [inferred] Write saved draw offsets |
| [SACL](opcodes/SACL.md) | `0x25a` | [inferred] Disable a displacement contribution |
| [SHAF](opcodes/SHAF.md) | `0x341` | [inferred] Clear global shock |
| [SPRC](opcodes/SPRC.md) | `0x221` | [inferred] Release/clear prepared resources |
| [SPRD](opcodes/SPRD.md) | `0x220` | [inferred] Prepare selected image pieces |
| [FCSP](opcodes/FCSP.md) | `0x3a1` | [inferred] Stop/fade focus lines |
| [FCLN](opcodes/FCLN.md) | `0x3a0` | [inferred] Start type-selected focus-line overlay |
| [STRR](opcodes/STRR.md) | `none` | [inferred] Rotate the clipping window independently |

### Choice, save, read-state and movies

| Command | Steam opcode | Meaning |
|---|---|---|
| [SELR](opcodes/SELR.md) | `0x160` | [code] Register text/value/lock row (T 0x796dc) |
| [SELD](opcodes/SELD.md) | `0x161` | [code] Display choice, store value/history (T 0x81ab0) |
| [SELA](opcodes/SELA.md) | `none` | [code] F1118-gated save prompt (T 0x7ca88) |
| [SELS](opcodes/SELS.md) | `none (uses 0x160/0x161)` | [code] Hint Yes/No prompt (T 0x8fcc0) |
| [ATSV](opcodes/ATSV.md) | `0x10e` | [code] Mode-dependent autosave (S 0x140108f70) |
| [RDED](opcodes/RDED.md) | `none (nearest 0x128)` | [code] Validated persistent completion (T 0x8f910) |
| [RDST](opcodes/RDST.md) | `none (nearest 0x128)` | [code] Indexed persistent entered/read bank (T 0x8faf0) |
| [RDAS](opcodes/RDAS.md) | `none (nearest 0x122)` | [code] Completion-sensitive skip control (T 0x78cd4) |
| [RTST](opcodes/RTST.md) | `none (nearest 0x128)` | [code] Session bit and conditional current index (T 0x8fc30) |
| [TOMW](opcodes/TOMW.md) | `0x1c1` | [code] Movie barrier (T 0x7c358) |
| [TOMP](opcodes/TOMP.md) | `0x1c0` | [code] Indexed movie submission (T 0x77784) |
