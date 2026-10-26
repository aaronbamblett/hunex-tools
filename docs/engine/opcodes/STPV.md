# STPV

Source spelling: `_STPV(6,459,178,)`.

## Operands

- Operand 0 selects the sprite slot; operands 1/2 replace the pivot x/y when supplied, and operand 3 supplies an auxiliary angle when supplied (`0x63890–0x63924`, `0x63c90–0x63ed8`, `0x641dc–0x641ec`).

## Switch build

[code] T dispatch `0x78ffc–0x79004` calls `0x63780`. This handler uses signed-halfword slot indexing; a normal unsigned 0–39 guard was not established in the inspected path (`0x638ac–0x638bc`, `0x63aac–0x63ab8`). Omitted coordinates retain saved pivot values. Pivot calculation depends on the image registry, image type and related variant-name lookup (`0x638bc–0x63910`, `0x63ac0–0x63bf4`).

[code] Saved pivot fields are floats at `+0x374/+0x414 + slot*4`; setter `0x33e90` copies them into live slot `+0x64/+0x68` (`0x63f98–0x63fb0`). Tail code updates the stage snapshot and negative pivot halfwords at saved `slot*0x12 + 0x16d6/+0x16d8` (`0x641f0–0x64248`). Return 0 (`0x6420c–0x64250`), and dispatch continues. [inferred] It changes the sprite transform's pivot, affecting subsequent scaling/rotation, without waiting or emitting sound. Exact image-type scaling factors require tracing these calculation branches rather than assuming raw pixels.

## Steam build

[code] Opcode `0x250` → S `0x140102b00`, registration `0x1401067f8–0x140106806`; four-field recipe with slot 0 and retain-state `0x80000000` for each other field (`_STPV:0`). It accepts slots 0–63. Explicit pivot x/y are converted by integer arithmetic to engine coordinates, missing fields retain `+0x7c/+0x80`; the optional angle is converted and sign-inverted, missing angle retains `+0x94` (`0x140102b47–0x140102c4a`). It writes pivot `+0x7c/+0x80`, angle `+0x94`, dirty/update flags and returns 0 (`0x140102c4a–0x140102c70`). Sync consumes pivot through `0x140149d05–0x140149d73` and rotation through `0x140149f16–0x140149fa4`.

[data] Mahoyo: 1,661 occurrences; `1_1.chs@0x4594` supplies slot 27, x15, y69 and retained angle, after STBR and before the next source statement.

## Differences

[code] Switch derives coordinates partly from image type/variant metadata and saves floats plus negative halfwords; Steam uses its own integer coordinate conversion and stores dwords (T `0x638bc–0x63924`, `0x64230–0x64248`; S `0x140102b9f–0x140102c66`). [inferred] Identical operands do not establish equal pivots for paired/variant images. Invalid-slot behavior is particularly unsafe to infer from other ST commands because this Switch path does not show their usual guard.

## Unverified

[inferred] Image-type/variant scaling, optional-angle equivalence and invalid-slot handling remain unresolved (T `0x63780`; S `0x140102b00`). Set the scr0005 pivot on its actual image, then rotate and nonuniformly scale it while logging both pivots and comparing a fixed visible landmark in both engines.
