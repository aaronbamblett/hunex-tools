# STEC

Source spelling: `_STEC(12,48)`.

## Operands

- Operand 0 selects slot 0–39 (absent defaults to -1 and exits); operand 1 is a color/effect preset, default 0.

## Switch build

[code] T dispatch `0x79a24–0x79a34` calls `0x3e10`. Presets 10–76 store their number at saved `+0x1404 + slot*2` and reset the associated color state to white (`0x3ec4–0x3f1c`). Other presets up to 82 dispatch through byte table `0x219986` (`0x3f38–0x3f58`): one branch resets the preset to 0, one sets it to 4, and one parses operand 2 as a hexadecimal color string and installs preset 1 (`0x3f5c–0x3ffc`). Preset parameter records are 0x50 bytes at T `0x2199dc`, read by `0x4000`.

[code] Saved/live color records are set through T `0x3b70/0x3de0`; the live color record starts at slot `+0x1cc` (`0x3ef4–0x3f1c`, `0x3fc8–0x3fe4`). The handler returns 0 and dispatch continues, with no wait (`0x3ea0–0x3eb4`, `0x79a34`). [inferred] It changes later sprite color/effect rendering; no audio action is established by this setter. A named interpretation for every preset has not been traced (T `0x2199dc`, `0x4000`).

## Steam build

[code] Opcode `0x253` → S `0x140102e00`, registration `0x140106837–0x140106845`; compiler three-field recipe with zero defaults (`_STEC:0`). Handler accepts slots 0–63, stores raw operand 1 as dword `+0x48`, unpacks operand 2 into RGB bytes at `+0x54/+0x58/+0x5c`, and sets dirty/update flags (`0x140102e3b–0x140102eb7`). Return 0 (`0x140102ebb–0x140102ec2`); it does not allocate a wait task. There is no preset-number translation in this handler.

[data] Mahoyo: 15,441 occurrences; `1_0.chs@0x4a60` uses slot 22, mode 1, RGB `0x8e0707`, between STCH and STBR.

## Differences

[code] The Switch handles a Tsukihime preset table and hexadecimal-color parsing; the Steam setter stores raw mode plus RGB. Source has a defined upper bound of 82 on its dispatch path; the Steam handler does not enforce that same bound (T `0x3f38`; S `0x140102e86–0x140102eb7`). [inferred] Passing modes 10–76 through does not prove the Steam renderer has corresponding Tsukihime presets. Their actual differences require consumer/table comparison or runtime output; this page does not claim every preset is missing.

## Unverified

[inferred] The complete Steam preset consumer, mapping of all 83 source presets and their color math remain unverified (T `0x4000`; S `0x140102eaf`). Capture a fixed sprite with STEC mode 48 and explicit RGB mode 1 in both engines, recording the resolved preset/color state and pixel output; this distinguishes a missing preset from a difference in color math.
