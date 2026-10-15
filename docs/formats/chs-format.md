# HuneX compiled script (.chs) format

## Overview

A `.chs` file is a compiled script of the HuneX engine. The `hunex-chs` crate decodes it, re-encodes it exactly, and lifts it into a symbolic form that can be edited and lowered again. The format has no official specification.

Checked on sample files: decode and re-encode give identical bytes for the 250 sample scripts, and lifting and lowering an unchanged file gives identical bytes too. Numbers are unsigned 32-bit little-endian dwords unless stated.

## Layout

```
magic | header size | include count | include names | padding | code | string pool
```

The file starts with a magic string that selects the variant:

- Hunex: `HunexCompiledScriptVer1.00` followed by a NUL byte (27 bytes including the NUL).
- Buriko: `BurikoCompiledScriptVer1.00` followed by a NUL byte (28 bytes including the NUL). The decoder accepts this variant; no sample file uses it.

Anything else is rejected.

### Header

- At the end of the magic: the header size dword. The code base is the magic length plus this value. The value must be at least 8, so the two header dwords fit.
- Then the include count dword, followed by that many NUL-terminated include names. The names must end before the code base. Include names are raw bytes.
- The bytes from the end of the include names up to the code base are padding. All 250 sample files have zero padding; the decoder keeps the bytes so the file re-encodes exactly.

The variant name comes from the magic string. In the samples the Hunex variant has a header size of 21, so its code base is 0x30.

### Code

Code starts at the code base and is a sequence of instructions. Each instruction is an opcode dword followed by a fixed number of inline operand dwords, given by the opcode table (see [opcodes.md](opcodes.md)). The table holds 160 ids: 71 base opcodes up to 0xFE and 89 commands from 0x100. The Steam executable registers all of them except 0x25C and 0x25E, which a patched executable adds (see `_SBAS` and `_FCSP` below). An id outside the table is an error.

- Opcode 0x7B is followed by three inline dwords that the interpreter skips. In 23,139 of the 23,209 places where 0x7B occurs, the first dword after it is 0x12345678. Its meaning is open.
- Opcode 0x7F has two inline dwords: a string pointer to a source file name, and a line number.

Pointers are stored relative to the code base:

- The first operand of 0x01 is a code pointer. It must land on the start of an instruction.
- The first operand of 0x03 and of 0x7F is a string pointer into the string pool.
- The third operand of 0x7B is a string pointer when it names a pool string. Lifting keeps it as a plain dword otherwise, and lowering writes it back as a pointer when it is a string.

### String pool

The pool runs from the pool start to the end of the file. It is a sequence of NUL-terminated entries. Bytes after the last NUL, if any, are a tail. A string pointer must name a string start: the pool start itself, or a position right after a NUL. The string must end with a NUL inside the file. String bytes are checked as UTF-8 only when they are read as text.

## Encoding rules

### Walk and pool start

The file does not store where the code ends, so the decoder works it out:

1. Decode instructions from the code base.
2. Keep the smallest string pointer seen so far. The pool start is the code base plus that pointer.
3. Stop decoding when the position reaches the pool start. An instruction that would cross the pool start is an error. The decoder never reads bytes at or after the pool start as code.
4. The walk must end exactly at the pool start, and the last instruction must be a return (0x1B).

Only the 0x03 and 0x7F pointers take part in this rule. The 0x7B third operand is also a pointer, and in the samples it is never below the pool start, but it does not define the pool start.

A file without any string pointer has no pool start, so its code must run exactly to the end of the file.

### Re-encoding and lowering

`encode` writes the magic, the header dwords, include names, padding, each instruction (opcode and operands as dwords) and the pool bytes.

`lift` replaces raw pointers with typed operands: a label for each code pointer target and a string id for each string operand. `lower` recomputes all pointers from the symbolic form, and its pool layout rule is described in [asm-format.md](asm-format.md).

### Source text and normalized form

Checked on sample files: the `source` parser reads all 249 Switch scripts into 493,227 statements (including 14 malformed `_KWRS` tokens), matching the reference parser statement for statement. The `normalize` helper drops every Debug instruction and resolves string and jump targets, which is the form used to compare compiler output against the sample files.

The `compile` lowering covers the engine recipe table (prologue dispatch, labels, conditionals, jumps, messages, typed encoders, epilogue return) and compares against all 249 scenes with identical per-scene classes. The three counts are cumulative: 162 scenes match exactly, 216 including shape matches (source-numeric drift only), and 242 including reviewed authored edits. Jump targets normalize to their own operand variant so a target never compares equal to a literal number.

Checked by static disassembly of the Steam handlers: numeric recipe fallbacks preserve audio tag payloads instead of limiting them to corpus literals. SEPL `0x190` (`0x1401075b0`) pops slot, mode, asset string, volume, pan, fade and delay; defaults are volume 100, pan 64, fade/delay 0. MPLY `0x180` (`0x140106d80`) pops track, volume, mode, fade and delay; the supported numeric shapes use mode 1, volume 100 and fade/delay 0 unless supplied. Tag 069 remains unsupported. SEFD `0x195` (`0x140107fb0`) pops slot, an unused zero, fade and delay. SEVL `0x196` (`0x140108060`) pops slot, two unused zeros, volume, fade and delay. MFAD/MVOL retain their existing three-field encodings and defaults. Tag predicates require the exact code and one numeric payload; unsupported tags are not discarded. These addresses correct the earlier port report's SEFD/SEVL identifications. Cross-engine audio timing parity remains unverified.

Checked by static disassembly: flag addition `0xeb` (`0x14007dce0`) pops bank, index and increment; ordinary flag recipes preserve all three, after existing special flag recipes. SPFC `0x222` (`0x140100230`) consumes slot, asset string and six numeric fields, with empty numeric fields defaulting to zero; one trailing empty source field is accepted. SPMA `0x27a` (`0x140105290`) consumes its numeric animation control, including zero. STMA `0x278` (`0x140104c70`) consumes slot, animation, third-field control, mode and flags; the four-field fallback defaults the third field and flags to zero. VPLY `0x1a0` (`0x1401086e0`) consumes the asset string and numeric control; its existing unused source identifier remains unused. Existing recipes retain selection precedence, and the Mahoyo gate remains 162 exact, 216 shape and 242 including authored edits, with all 249 classes agreeing. Runtime behavior of the new shapes is unverified.

To compile one Switch script, run `hunex chs compile <script.txt> --scene-map <map.txt> -o <out.chs>`: the general scene number map is a UTF-8 text file with one `<number> <scene name>` per line (blank lines and `#` comment lines allowed). For the Mahoyo Switch layout, `--names <nam.bin>` is a convenience: the scene names come from the Switch `nam.bin` (32-byte records; records 1–137 add 199, 138–150 add 362, 151–219 add 849, 220–245 add 1780, record 247 is scene 4000). The output uses the Steam header layout with a Debug-free program and a first-use string pool. Selecting a zero-emission predicate that is not proven on the corpus prints a warning to stderr; the selection still emits nothing.

### Errors

The decoder reports: bad magic, truncated data, a malformed header, an unregistered opcode (with its offset), a walk that does not end on a return at the pool start, a string pointer that does not name a string, a code pointer that does not land on an instruction (during lifting), and invalid UTF-8 when reading text.

## What the game accepts

- Verified in game (2026-10-13): edited scripts play through in the Steam release of Witch on the Holy Night. The edits were an inserted instruction at the start, which moves every code and string pointer, a changed image string, and changed text. The relocation worked for those scripts.
- Runtime-computed pointers are not covered by these tests.

## Known unknowns

- The pool start is the smallest string pointer. A crafted file whose smallest pointer targets a return inside the code is accepted, and the code after that return is read as pool data. Real files cannot be told apart from this case, because the format has no explicit code length.
- Pointers computed at runtime, or stored in plain 0x00 dwords, are not recognized as pointers. Lowering does not move them. The sample files show none.
- The 0x7B counter (its second dword) is kept as a plain dword. Its meaning is unknown, and edits do not renumber it.
- The stack effect of each opcode and the operands of most commands are not modeled.
- Whether the game accepts a script whose code is not in the sample layout, beyond the edited scripts above.

### Additional source shapes (static handler evidence)

Checked in the Steam image, not in game: `_SESP(slot)` for literal slots 0–19
pushes zero delay, zero fade and the slot, then calls 0x194. Its handler at
`0x140107ab0` pops slot/fade/delay at `0x140107acc`, `0x140107add` and
`0x140107af2`. Higher slots need the source modulo-20 behavior; this recipe
does not accept them. The existing negative-slot recipe keeps precedence.

Eight-integer `_STCL(0,slot,x0,y0,x1,y1,duration,delay)` pushes a numeric zero
coordinate pointer followed by the eight fields in reverse order, then 0x243.
The handler at `0x140102240` pops eight words and a pointer
(`0x140102273–0x1401022e6`). The setter at `0x140101f20` distinguishes a null
coordinate pointer (`0x140102134–0x1401021a6`) from a supplied coordinate string.
The source's absent coordinate field takes its default half-size path
(Mahoyo Switch `0x42b38–0x42b5c`); an empty string or `CC` would select a
different path. Existing seven- and nine-field recipes keep precedence.

`_SBAS(child,parent)` pushes the parent, then the child, and calls 0x25C.
The stock Steam executable does not register 0x25C: its registration chain
(`0x1401066b0–0x140106a6e`) skips 0x25C–0x26F, and the table it fills is
zero-filled when allocated (`0x14007a446–0x14007a45d`). A script that uses `_SBAS` runs only
with an executable patched to register a handler that pops the child and then
the parent. The source handler stores the parent per child slot (Tsukihime
Switch `0x797b8–0x7981c`).

`_FCSP(duration)` with one integer pushes it and calls 0x25E; `_FCSP()` keeps
0x3A1. The stock 0x3A1 handler (`0x140106670`) reads no operand and stores a
fixed 400 ms stop fade (`0x14010669f`). 0x25E is not registered by the stock
executable; a patched executable registers a handler that pops the duration
and stores it in place of the 400. The source handler reads operand 0 in ms
(Tsukihime Switch `0x8f6d4–0x8f700`).

The remaining SELD selection index, VCWT wait, FADE effect codes and STCH
fields beyond 17 have no established Steam lowering. SELD 0x161 at
`0x140049c20` pops four words into object offsets 0x70/0x74/0x78/0x7c
(`0x140049c5d–0x140049c8a`, `0x140049e0f–0x140049e1b`); those reads alone do
not establish a mapping for the source's persistent selection index. STCH
0x240 at `0x140101610` supplies literal -1 to its setter at `0x14010178b`.
Do not append stack operands to that opcode or discard the source values.
