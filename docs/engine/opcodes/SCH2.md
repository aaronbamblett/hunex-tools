# SCH2

Source spelling: `_SCH2(7,img2343,,,,0,,,CC,,,,,,,,,,2)`.

## Operands

- Operands are zero-based.

- [code] T `0x78ee8…0x78efc` parses fields then enters shared sprite core `0x60530` with mode 0; STCH uses mode 1 (`0x79b4c…0x79b64`).

- Slot is 0…39; operand 1 is registry name, 2/3 position, 4 zoom, 5 opacity (clamped 0…255), 6 priority.

- Operand 8 carries anchor text such as CC; 9/10 participate in piece selection (`0x60780…0x607c4`); operand 11 writes a slot-like signed id (`+0x89c`, `0x61ae0`; full role unresolved); 12…15 are four signed region fields (`+0xa2c…+0xa32`, `0x61c20…0x61cd8`); 16/17 write `+0xb6c` and byte `+0xbbc`; 18 defaults to −1 (`0x60574`, `0x60658`); prior documentation calls it parent, while the shared core also uses this value as a reference for copying saved slot state (`0x6092c…0x6097c`).

- Operand 19 sets clip angle (`0x6112c`, `0x61880`).

- [data] Native Mahoyo has **2626** matching opcode occurrences across 250 supplied CHS files, all dumped successfully.

## Switch build

[code] T `0x78ee8…0x78efc` parses fields then enters shared sprite core `0x60530` with mode 0; STCH uses mode 1 (`0x79b4c…0x79b64`). It changes an image/slot using reference state rather than simply being an alternate spelling of a fresh STCH. Slot is 0…39; operand 1 is registry name, 2/3 position, 4 zoom, 5 opacity (clamped 0…255), 6 priority. Position conversion is x*640/1280 and y*480/724 (T `0x619b4…0x61a48`). Operand 8 carries anchor text such as CC; 9/10 participate in piece selection (`0x60780…0x607c4`); operand 11 writes a slot-like signed id (`+0x89c`, `0x61ae0`; full role unresolved); 12…15 are four signed region fields (`+0xa2c…+0xa32`, `0x61c20…0x61cd8`); 16/17 write `+0xb6c` and byte `+0xbbc`; 18 defaults to −1 (`0x60574`, `0x60658`); prior documentation calls it parent, while the shared core also uses this value as a reference for copying saved slot state (`0x6092c…0x6097c`). Operand 19 sets clip angle (`0x6112c`, `0x61880`). Full flag meanings for 7,16,17 remain unverified. Core copies/initializes saved state and marks the scene dirty (`0x79b64…0x79b78`); it does not install a script completion wait or request audio.

## Steam build

[code] Opcode `0x242`, S `0x140101d00`, forwards a 19-field command into common core `0x140100830` (call `0x140101f02`), which checks slot 0…63, resolves image metadata through `0x1400a0dc0` and updates the slot request. Recipe accepts 2…19 fields: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:1946`. Final rendering occurs through the slot-sync path `0x1401494b0…0x14014a635`; no dedicated SCH2 wait task is created in the wrapper.

[data] Native Mahoyo has **2626** matching opcode occurrences across 250 supplied CHS files, all dumped successfully. First: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_0.chs`, assembly-dump line 3984. These are static counts, not reachability.

## Differences

[code] Switch can read operand 19 for clip angle; the Steam recipe stops at operand 18, so general absent/present/default behavior and fields 7/16/17 are not fully reconciled (T `0x6112c`; recipe `recipes_data.rs:1946`). Metadata origin/logical-size differences can change CC anchoring; a Steam registry miss continues without recovering those values. [inferred] Reference inheritance and parent/clip behavior need validation even when compilation succeeds.

## Unverified

[inferred] Full operand defaults, the distinction between operand-11 and operand-18 references/parenting, reference-copy details and flag meanings remain unresolved. One runtime check: replay scr0001 statement 210 with distinct source/reference position, pivot, parent and clip state, recording both engines immediately after SCH2 and after the following display transition (T `0x60530`; S `0x140100830`).

---

Engine facts here are lifted from the port's `docs/commands/SCH2.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
