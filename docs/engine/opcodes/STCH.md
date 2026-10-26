# STCH

Source spelling: `_STCH(1,WHITE,240,136,1000,255,2,,CC)`.

## Operands

- [code] Dispatch T `0x79b4c–0x79b78` parses and calls core `0x60530` with create mode 1.

- Operand 0 is slot 0–39; operand 1 is registry image name (`0x60554–0x60590`, `0x605dc–0x605ec`).

- [code] Operands are zero-based; the following are the established fields in T `0x60530–0x61d70`:

- | Operand | Meaning / write |
|---|---|
| 2, 3 | Position x/y, converted by 640/1280 and 480/720 (`0x619b4–0x61a44`) |
| 4 | Uniform zoom, per-mille value stored as halfword (`0x61a98–0x61aa4`) |
| 5 | Opacity, upper-clamped to 255 (`0x61978–0x6199c`) |
| 6 | Draw priority/order (`0x61b50–0x61b5c`) |
| 7, 8 | Auxiliary numeric field and anchor text; complete Switch parsing/consumer mapping remains unverified here |
| 9, 10 | Image variant components, also recoverable from a referenced image's name (`0x60600–0x607bc`) |
| 11 | Mask/transition reference, saved at `+0x8ec` (`0x61314`, `0x61ae0`) |
| 12–15 | Four region coordinates/dimensions at `+0xa2c..+0xa32` (`0x61c20–0x61cd8`) |
| 16, 17 | Region auxiliary halfword/byte, `+0xb6c/+0xbbc`; exact flag meanings unverified (`0x61d08`, `0x61d50`) |
| 18 | Parent/reference slot, default -1 (`0x60574–0x60664`) |
| 19 | Clip-window angle, stored at `+0xbe4` (`0x61880`) |

- The precise 7/8/16/17 semantics must not be filled in from their names or defaults (T `0x60530`; Steam core below).

- `recipes_data.rs:7248` accepts 2–18 operands.

- Its absent/empty defaults in order are `0, empty-name, 0, 0, 1000, 255, 0, 0, CC, 0, 0, -1, 0, 0, 0, 0, 0, 0`.

- These are **compiler defaults**, not a proof of Switch fresh/reuse defaults.

- Anchor calculation uses registry dimensions; missing metadata diagnoses and continues through guards (`0x140100ac4–0x140100b01`).

## Switch build

[code] Dispatch T `0x79b4c–0x79b78` parses and calls core `0x60530` with create mode 1. Operand 0 is slot 0–39; operand 1 is registry image name (`0x60554–0x60590`, `0x605dc–0x605ec`). State is saved/live stage data, including image id, position, pivot, zoom, alpha, priority, region, parenting and animation. A fresh create resets auxiliary state and animation (`0x61220–0x61310`); reuse/reference paths copy existing state and remap children (`0x60c50`, `0x615a8–0x616c0`, also). Dispatch continues, rather than installing an STCF-style wait (`0x79b64–0x79b78`).

[code] Operands are zero-based; the following are the established fields in T `0x60530–0x61d70`:

| Operand | Meaning / write |
|---|---|
| 2, 3 | Position x/y, converted by 640/1280 and 480/720 (`0x619b4–0x61a44`) |
| 4 | Uniform zoom, per-mille value stored as halfword (`0x61a98–0x61aa4`) |
| 5 | Opacity, upper-clamped to 255 (`0x61978–0x6199c`) |
| 6 | Draw priority/order (`0x61b50–0x61b5c`) |
| 7, 8 | Auxiliary numeric field and anchor text; complete Switch parsing/consumer mapping remains unverified here |
| 9, 10 | Image variant components, also recoverable from a referenced image's name (`0x60600–0x607bc`) |
| 11 | Mask/transition reference, saved at `+0x8ec` (`0x61314`, `0x61ae0`) |
| 12–15 | Four region coordinates/dimensions at `+0xa2c..+0xa32` (`0x61c20–0x61cd8`) |
| 16, 17 | Region auxiliary halfword/byte, `+0xb6c/+0xbbc`; exact flag meanings unverified (`0x61d08`, `0x61d50`) |
| 18 | Parent/reference slot, default -1 (`0x60574–0x60664`) |
| 19 | Clip-window angle, stored at `+0xbe4` (`0x61880`) |

[inferred] The command shows/replaces an image with these transforms; no audio effect is established by the handler. The precise 7/8/16/17 semantics must not be filled in from their names or defaults (T `0x60530`; Steam core below).

## Steam build

[code] Opcode `0x240` → S `0x140101610` → core `0x140100830` (registration `0x140106757`). `recipes_data.rs:7248` accepts 2–18 operands. Its absent/empty defaults in order are `0, empty-name, 0, 0, 1000, 255, 0, 0, CC, 0, 0, -1, 0, 0, 0, 0, 0, 0`. These are **compiler defaults**, not a proof of Switch fresh/reuse defaults. Core stores position/pivot, scale, rotation, opacity, priority, variants and region in slot stride `0x220` (`0x14010132d–0x140101493`). Anchor calculation uses registry dimensions; missing metadata diagnoses and continues through guards (`0x140100ac4–0x140100b01`). It returns 0 (`0x140101554`); render/resource processing follows stage synchronization.

[data] Mahoyo: **33,549** occurrences; `1_0.chs@0x21c` creates slot 0 with image `img0408`, then STRT at `0x264` and STGS at `0x29c`.

## Differences

[code] Clip angle is lost; Steam region setter S `0x1400762f0` has no angle, while Switch renders rotated clipping at T `0x885c8–0x88840`. [inferred] Parent support is conditional, with unsaved links. Steam allows 64 slots; Switch 40 (T `0x6058c`; S `0x140101580`).

## Unverified

[inferred] Fresh versus reused omitted fields, operand 7/8/16/17 consumer semantics, and save/load parenting remain incomplete (T `0x60530–0x61d70`; S `0x140100830`). **One runtime check:** replay a scr0137 panel create/recreate and save/load sequence while logging every parsed field and parent link and comparing the resulting clip boundaries against Switch.

---

Engine facts here are lifted from the port's `docs/commands/STCH.md` (engine sections only); port conversion and patch notes are not carried over. Claims keep their [code]/[data]/[inferred] marks.
