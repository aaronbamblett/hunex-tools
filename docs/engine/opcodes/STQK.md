# STQK

Source spelling: `_STQK(29,0,10,100,200,0,2)`.

## Operands

- Operands are zero-based.

- [code] Source STQK has no Switch command handler: the dispatch tables route it to unknown-command continuation T `0x76730`, so its seven supplied fields do not install a quake or wait.

- [code] `STQK`, opcode `0x270`, handler S `0x140103bc0`, pops slot, x/y amplitudes, period, lifetime, delay and mode.

- It writes one slot-quake request at `+0x1a0…+0x1c0`; the callback S `0x1401455e0` consumes it.

- [data] Mahoyo uses the matching opcode **614** times in the 250 supplied raw CHS files.

## Switch build

[code] Source STQK has no Switch command handler: the dispatch tables route it to unknown-command continuation T `0x76730`, so its seven supplied fields do not install a quake or wait. This is an established dispatch result reused from (table T `0x2355d0`), not a claim that Switch SQK2 is inert.

## Steam build

[code] `STQK`, opcode `0x270`, handler S `0x140103bc0`, pops slot, x/y amplitudes, period, lifetime, delay and mode. It writes one slot-quake request at `+0x1a0…+0x1c0`; the callback S `0x1401455e0` consumes it. Input mode 0 selects random displacement (S `0x140145b16`); input mode 2 selects the sine branch (S `0x140145c5c`). The handler returns zero, without creating a wait. Recipe: `/var/tmp/hunex-chs-work/wt-int/crates/hunex-chs/src/recipes_data.rs:8199`.

[data] Mahoyo uses the matching opcode **614** times in the 250 supplied raw CHS files. First occurrence: `/var/tmp/tsukihime-port/mahoyo-steam/raw/1_0.chs`, assembly-dump line 3762. All 250 dumps returned success; this is native usage, not proof of Tsukihime compatibility.

## Differences

[code] Passing source STQK through would add motion that Switch never requested; matching names alone would be wrong (T `0x76730`; S `0x140103bc0`). [inferred] The three source occurrences should be inert even though their authored operands resemble a valid Steam effect.

## Unverified

[inferred] One runtime check: compare scr0508 statement 1208 with and without the source command on Switch, then confirm the translated script has no added motion at that point. No runtime observation of those three source calls was made (T `0x76730`; source `scr0508.txt:1`).

---

Engine facts here are lifted from the port's `docs/commands/STQK.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
