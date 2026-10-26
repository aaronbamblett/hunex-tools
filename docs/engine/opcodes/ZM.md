# ZM

Source spelling: `_ZM01801($000107)`.

## Operands

- The five suffix characters encode a three-hex-digit group and two-hex-digit sequence, decoded into signed-16-expanded state values at T `0x77c10–0x77cb0`.
- This suffix is distinct from the decimal `$NNNNNN` text ID in the argument.

## Switch build

- [code] Message form `ZM<hex-group><hex-sequence>(text)` enters the shared Z bookkeeping path. Group/sequence are stored in current and remembered fields through `[0x3518a8]`, `[0x3518b0]`, `[0x3518b8]`, `[0x3518c0]`; associated read-history/skip checks inspect group ≤0xfb3 and per-group byte at +0xd (`0x77ccc–0x77cf0`). [code] Only M enters display: T `0x7849c–0x784ac` branches to `0x7c380`, which calls message routine `0x6db80` with the payload and yields. That routine updates message/window state, parses content (`0x6dccc–0x6dd8c`), and consumes pending voice through `0x6e804–0x6e87c`. Text can therefore display and trigger staged voice; it is not an immediate pure string operation. [inferred] Click/auto/skip/voice state affects when the message progression releases; a universal fixed wait or unconditional click requirement is not established by this entry slice (T `0x6db80`; shared read/skip guards `0x77cb4–0x784ac`).

## Steam build

- [code] The source parser strips all-hex suffixes from ZM's canonical name while preserving raw spelling (H `src/source.rs:294–304`). Compiler `lower_message` uses the payload, converts `^` to newline, assigns a new sequential message index, emits metadata `0x7b` with magic 0x12345678, then pushes mode,1,0,0,text and calls 0x140 (H `compile::lower_message`). Original suffix values are not encoded by this lowering. [code] Mode is 0 when a later WKST with value 63 is found before WNTY/ZM/MSAD; otherwise 1 for `@n`, otherwise WNTY+1 (H `compile::lower_message`). Handler S `0x14010a170` parses/stages text and a native task; ordinary task installation at VM +0x24e8 returns 1, while its special immediate branch executes callbacks and returns 0 (`0x14010a264–0x14010a2b9`, `0x14010aeee–0x14010aff7`). This has native message/input timing, not the original suffix/read history. [data] Mahoyo has 23,139 calls to `0x140` in 250 raw files. `raw/1_0.chs:0x738–0x760` pushes 1,1,0,0 and a decimal text token. These calls may also correspond to MSAD, which shares the opcode.

## Differences

- [code] Switch decodes five fixed suffix positions using digit/lowercase-hex arithmetic; the compiler accepts any nonempty all-hex suffix, including uppercase and other lengths. Such spellings are not proved equivalent (T `0x77c14–0x77c88`; H `src/source.rs:294–304`). [code] Decimal text ID, hex suffix group/sequence and compiler-generated message ordinal are three separate identities (T `0x77c10–0x77cb0`; H `compile::lower_message`). Equal text does not imply equal history state. Presence of displayed text does not validate auto/skip or voice synchronization (H `compile::lower_message`; [WKST](WKST.md)).

## Unverified

- [inferred] One paired multi-message-page run in normal, auto and skip modes, with staged voice and save/load, should compare displayed text, release timing and suffix/read-history state against the compiled message tasks (T `0x77c98`, `0x6e804`; S `0x14010a170`). Exact release predicates and full history parity remain unresolved.
