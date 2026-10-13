# Text form (asm format)

`hunex chs dump --asm` writes this form and `hunex chs asm` reads it. Any file dumped and assembled again gives the same bytes. The examples here are synthetic.

## Lines

Each line holds one statement. Blank lines are ignored. A line whose first non-blank character is `#` is a comment. Tokens are separated by spaces or tabs; a quoted string may contain spaces.

## Directives

| Directive | Meaning |
|---|---|
| `format hunex` or `format buriko` | Required, once. |
| `header_size N` | Required, once. Decimal or `0x` hex. |
| `include "name"` | Zero or more, in order. |
| `header_padding 00 a5 ...` | Optional, once. Hex bytes, two digits each. Must match the header size. |
| `string S_0000 "text"` | An original pool entry. Entries keep their order. |
| `string new S_0000 "text"` | A new string, written after the original entries. |
| `tail "text"` | Optional, once. Bytes after the last pool NUL. |

Every `header_size` must equal 8 plus the include names (each with its NUL) plus the padding bytes. The assembler checks this.

## Labels

`L_0000:` defines a label on the next instruction. A label placed after the last instruction labels the end of the code. Label names start with `L_` and must be unique.

## Instructions

An instruction is an opcode followed by its operands:

- The opcode is a mnemonic from the opcode table (for example `PushString`), or a hex id such as `0x0f7` for opcodes without a mnemonic. Ids must be registered.
- Operands are:
  - a number: `0x10` or `16`;
  - a label reference: `L_0000`, allowed only as the first operand of `PushOffset` (0x01);
  - a string reference: `S_0000`, allowed only as the first operand of `PushString` (0x03) and of the 0x7F record;
  - a quoted string: `"text"`, which creates a new string at that point.

Operand counts follow the opcode table. Any other operand kind is an error.

## Strings and escapes

Quoted text escapes `\n`, `\r`, `\t`, `\\`, `\"` and `\xNN` (two hex digits, any byte). Text that is valid UTF-8 is written as is; any other byte is written as `\xNN`. Raw control characters inside quotes are errors. A string may not contain a NUL byte, even as `\x00`, because pool entries end with a NUL.

## Editing rules

- New strings go in `string new` entries or as quoted operands. They are written after the original entries, and only if code references them.
- An edited original `string` line keeps its place in the pool; the assembler does not know it was edited. Use `string new` to add an entry that is written after the originals.
- Inserting an instruction moves labels that sit at or after its position, so a label stays on the instruction it named.
- A label cannot point past the end of the code.
- The code must end with a return (0x1B).
- Pointers are recomputed on lowering. A pointer that is not a label or string operand is not moved.

## Pool layout

When the script is lowered, the pool is written in this order:

1. Original entries, starting at the first original string that code references, then in their original order. Unreferenced original entries after that point are kept.
2. New strings that code references, in table order.
3. The tail, if any.

Entries before the first referenced original string are dropped, and unreferenced new strings are dropped. This keeps the decoder's pool-start rule valid: the first pool entry is always a referenced string.

## Example

```text
format hunex
header_size 8

    PushDword 0x5
    PushString S_0000
    PushOffset L_0000
L_0000:
    Return
string S_0000 "example"
```
