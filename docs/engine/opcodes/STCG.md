# STCG

Source spelling: `_STCG(img0597)`.

## Operands

- Its sole operand is an image name, looked up against the 23,000-name registry via `0x56d40`; result is passed to `0x76420` (`0x79b1c–0x79b44`).

- For final ids in unsigned 0–3199 it ORs bit `id&7` into the image-seen bitset at the pointer loaded from T `0x351830` (`0x764f8–0x76524`).

- [code] Opcode `0x228` → S `0x140100560`, registration `0x14010673b–0x140106749`; one image/symbol operand (`recipes_data.rs:7196`).

- It obtains/converts the string, then compares its first three UTF-16 characters against **`img`** at S `0x140226d18` (`0x140100676–0x14010069b`).

## Switch build

[code] T dispatch table `0x23562a`, entry 4, reaches inline handler `0x79ac4`. Its sole operand is an image name, looked up against the 23,000-name registry via `0x56d40`; result is passed to `0x76420` (`0x79b1c–0x79b44`). That helper resolves variant/base names where registry metadata bit 8 requires it (`0x76430–0x76494`) and maps extended ids through a tree (`0x76498–0x764f4`). For final ids in unsigned 0–3199 it ORs bit `id&7` into the image-seen bitset at the pointer loaded from T `0x351830` (`0x764f8–0x76524`).

[code] It returns without a pending wait, and dispatch continues at `0x76730` (`0x76528–0x76534`, `0x79b48`). [inferred] This is gallery/image-seen bookkeeping; it does not itself draw a sprite or play sound. Gallery UI eligibility and disk persistence still depend on consumers/serialization of this bitset, not just this write.

## Steam build

[code] Opcode `0x228` → S `0x140100560`, registration `0x14010673b–0x140106749`; one image/symbol operand (`recipes_data.rs:7196`). It obtains/converts the string, then compares its first three UTF-16 characters against **`img`** at S `0x140226d18` (`0x140100676–0x14010069b`). A mismatch jumps directly to cleanup `0x1401007a4`. On a match it replaces/appends the extension `.mzp` (literal S `0x14026bb08`), converts the string and calls `0x140038310` on a manager at VM `+0x10` → `+0xc8` (`0x1401006a1–0x140100790`). It returns 0 (`0x1401007f6–0x140100825`).

[inferred] The native call is consistent with image-seen registration, but the complete persistent state written by `0x140038310` was not traced. Neither the matched path nor rejected-name path installs a script wait (S `0x140100560–0x140100825`).

[data] Mahoyo: **3** occurrences; `1_2.chs@0x338` passes `img0242` following a fade, satisfying the literal name gate.

## Differences

[code] The manager `0x140038310` first checks for an existing filename through `0x140038480`, then appends a hash/string pair to its growable 16-byte-entry list at `+0x38`, incrementing `+0x30`. [inferred] This does not establish Switch variant/base-id aliasing, gallery eligibility, or disk persistence; those consumers still need paired runtime verification.

## Unverified

[inferred] Steam registration's exact persistence schema and how Tsukihime seen bits should map into a gallery remain unresolved (T bitset pointer `0x351830`; S `0x140038310`).

---

Engine facts here are lifted from the port's `docs/commands/STCG.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
