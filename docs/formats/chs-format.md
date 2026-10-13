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

Code starts at the code base and is a sequence of instructions. Each instruction is an opcode dword followed by a fixed number of inline operand dwords, given by the opcode table (see [opcodes.md](opcodes.md)). The table holds 158 registered ids: 71 base opcodes up to 0xFE and 87 commands from 0x100. An unregistered id is an error.

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
