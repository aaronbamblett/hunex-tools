# HuneX compiled script (.chs) format

This describes the format as the hunex-chs crate reads and writes it. It is based on our own analysis of the sample files and on the game's loader; it is not an official specification. Numbers are unsigned 32-bit little-endian dwords unless stated.

## Overall layout

```
magic | header size | include count | include names | padding | code | string pool
```

The file starts with a magic string that selects the variant:

- Hunex: `HunexCompiledScriptVer1.00` followed by a NUL byte (27 bytes including the NUL).
- Buriko: `BurikoCompiledScriptVer1.00` followed by a NUL byte (28 bytes including the NUL). The decoder accepts this variant; no sample file uses it.

Anything else is rejected.

## Header

- At the end of the magic: the header size dword. The code base is the magic length plus this value. The value must be at least 8, so the two header dwords fit.
- Then the include count dword, followed by that many NUL-terminated include names. The names must end before the code base. Include names are raw bytes.
- The bytes from the end of the include names up to the code base are padding. All 250 sample files have zero padding; the decoder keeps the bytes so the file re-encodes exactly.

For the Hunex variant the header size is 21 in the samples, so the code base is 0x30.

## Code

Code starts at the code base and is a sequence of instructions. Each instruction is an opcode dword followed by a fixed number of inline operand dwords, given by the opcode table (see [opcodes.md](opcodes.md)). The table holds 158 registered ids: 71 base opcodes up to 0xFE and 87 commands from 0x100. An unregistered id is an error.

- Opcode 0x7B has three inline dwords. They are skipped by the interpreter and kept as operands.
- Opcode 0x7F has two inline dwords: a string pointer and a line number.

Pointers are stored relative to the code base:

- The first operand of 0x01 is a code pointer. It must land on the start of an instruction.
- The first operand of 0x03 and of 0x7F is a string pointer into the string pool.

## Walk and pool start

The string pool is the data after the code. The file does not store where the code ends, so the decoder works it out:

1. Decode instructions from the code base.
2. Keep the smallest string pointer seen so far. The pool start is the code base plus that pointer.
3. Stop decoding when the position reaches the pool start. An instruction that would cross the pool start is an error. The decoder never reads bytes at or after the pool start as code.
4. The walk must end exactly at the pool start, and the last instruction must be a return (0x1B).

A file without any string pointer has no pool start, so its code must run exactly to the end of the file.

## String pool

- The pool runs from the pool start to the end of the file.
- It is a sequence of NUL-terminated entries. Bytes after the last NUL, if any, are a tail.
- A string pointer must name a string start: the pool start itself, or a position right after a NUL. The string must end with a NUL inside the file.
- String bytes are only checked as UTF-8 when they are read as text. Raw bytes are always available.

## Errors

The decoder reports: bad magic, truncated data, a malformed header, an unregistered opcode (with its offset), a walk that does not end on a return at the pool start, a string pointer that does not name a string, a code pointer that does not land on an instruction (during lifting), and invalid UTF-8 when reading text.

## Re-encoding

`encode` writes the magic, the header dwords, include names, padding, each instruction (opcode and operands as dwords) and the pool bytes. For the 250 sample files the result is identical to the input.

## Symbolic form and lowering

`lift` replaces raw pointers with typed operands: a label for each code pointer target and a string id for each string operand. Labels name instruction positions, and the string table holds the pool entries in order.

`lower` recomputes all pointers from the symbolic form. Its pool layout rule is described in [asm-format.md](asm-format.md). Lifting and lowering an unchanged file gives identical bytes for all 250 sample files.

## Known limits

- The pool start is the smallest string pointer. A crafted file whose smallest pointer targets a return inside the code is accepted, and the code after that return is read as pool data. Real files cannot be told apart from this case, because the format has no explicit code length.
- Pointers computed at runtime, or stored in plain 0x00 dwords, are not recognised as pointers. Lowering does not move them. The sample files show none, but this is not proven until a relocated script runs in the game.
- The stack effect of each opcode and the operands of most commands are not modelled.
- Relocation and editing are verified on sample files and in the library tests, not yet inside the game.
