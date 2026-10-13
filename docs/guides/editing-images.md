# Editing images

Background images are MZP containers. A container holds a header entry and one entry per tile, and each tile is an MZX0 stream. The `image` commands convert a whole background to PNG and back. The tiles you do not change keep their original bytes.

## Prerequisites

- `hunex` and a POSIX shell (see [Getting started](getting-started.md)).
- Python 3 with the Pillow library, to paint the PNG in the steps below. Any image editor also works.
- `cmp`.

## Get an image

Background images are entries of the large HFA archives `data02000.hfa` and `data02001.hfa`. The sample used here, `img1155.mzp`, is in `data02001.hfa`. Those archives are about 1.6 GB each. Unpacking one writes the whole archive, so use a disk with at least that much free space:

```sh
mkdir -p work/images/data02001
hunex hfa unpack "<game folder>/data02001.hfa" -o work/images/data02001
```

If you already have an extracted copy of `img1155.mzp`, copy that file instead:

```sh
mkdir -p work/images/data02001
cp "<extracted folder>/img1155.mzp" work/images/data02001/
```

The steps below use `work/images/data02001/img1155.mzp`.

## Convert to PNG

```sh
hunex image to-png work/images/data02001/img1155.mzp -o work/images/img1155.png
```

The command prints nothing on success. The PNG is the full canvas, 3,072 by 1,280 pixels for this image.

## Edit the PNG

This example paints a red rectangle from x 150 to 1,349 and y 350 to 699, with Python and Pillow:

```sh
python3 -c "
from PIL import Image, ImageDraw
im = Image.open('work/images/img1155.png').convert('RGBA')
ImageDraw.Draw(im).rectangle([150, 350, 1349, 699], fill=(200, 30, 30, 255))
im.save('work/images/painted.png')
"
```

Keep the PNG the same size as the original.

## Rebuild the image

Use `--like` with the original container. It takes the canvas, the tile grid and the flags from the original, and it keeps the bytes of every tile you did not change:

```sh
hunex image from-png work/images/painted.png -o work/images/img1155-edit.mzp --like work/images/data02001/img1155.mzp
```

Output (one line for each changed tile):

```text
tile 6: 58644 pixels changed, 212 colors, exact, max error 0, mean error 0.000
tile 7: 82944 pixels changed, 136 colors, exact, max error 0, mean error 0.000
tile 8: 52812 pixels changed, 210 colors, exact, max error 0, mean error 0.000
tile 12: 68056 pixels changed, 198 colors, exact, max error 0, mean error 0.000
tile 13: 96256 pixels changed, 133 colors, exact, max error 0, mean error 0.000
tile 14: 61288 pixels changed, 206 colors, exact, max error 0, mean error 0.000
```

Each line gives the tile number, the number of changed pixels, the number of colors, whether the tile was reduced to 256 colors (`quantized`), and the error against the PNG. A tile with more than 256 colors is reduced, and its line says `quantized`.

The new container's size depends on your edit. In this run it was 2,388,840 bytes, against the original's 2,674,144 bytes.

## Check the result

Convert the new container to PNG and check two pixels, one inside the rectangle and one outside it:

```sh
hunex image to-png work/images/img1155-edit.mzp -o work/images/check.png
python3 -c "
from PIL import Image
im = Image.open('work/images/check.png').convert('RGBA')
print(im.getpixel((400, 500)), im.getpixel((50, 50)))
"
```

Output:

```text
(200, 30, 30, 255) (232, 223, 224, 255)
```

The pixel inside the rectangle is red. The pixel outside it keeps its original color.

## Put the image back in the archive

Copy the new container into the unpacked folder under its original name, and pack the archive with `--base`:

```sh
cp work/images/img1155-edit.mzp work/images/data02001/img1155.mzp
hunex hfa pack work/images/data02001 -o work/images/data02001-edited.hfa --base "<game folder>/data02001.hfa"
```

This step writes the whole archive, about 1.6 GB.

## CompressedBG images

`.cbg` files are CompressedBG images, a different format with its own commands. The title logo is one of them. It is in `data00000.hfa`, which is about 157 MB. Unpack that archive to a work folder, then convert the logo:

```sh
mkdir -p work/cbg
hunex hfa unpack "<game folder>/data00000.hfa" -o work/cbg/unpacked
hunex cbg info work/cbg/unpacked/title_titlelogo_en.cbg
```

Output:

```text
size:          952 x 303
bits per pixel: 32 (RGBA)
stripe height: 60 (6 stripes)
file size:     343941 bytes
```

Convert the logo to PNG, paint a red bar across its top with Pillow, and save the result as `title-red.png`:

```sh
hunex cbg to-png work/cbg/unpacked/title_titlelogo_en.cbg -o work/cbg/title.png
python3 -c "
from PIL import Image, ImageDraw
im = Image.open('work/cbg/title.png').convert('RGBA')
ImageDraw.Draw(im).rectangle([0, 0, 951, 39], fill=(200, 30, 30, 255))
im.save('work/cbg/title-red.png')
"
```

Rebuild the logo with `--like`. The output goes to a new file, so the original stays in place:

```sh
hunex cbg from-png work/cbg/title-red.png -o work/cbg/title_titlelogo_en-edit.cbg --like work/cbg/unpacked/title_titlelogo_en.cbg
```

`from-png` takes the color depth and stripe height from the original, so the PNG must be the same size. Copy the new logo into the unpacked folder under its original name, and pack the archive with `--base`:

```sh
cp work/cbg/title_titlelogo_en-edit.cbg work/cbg/unpacked/title_titlelogo_en.cbg
hunex hfa pack work/cbg/unpacked -o work/cbg/data00000-edited.hfa --base "<game folder>/data00000.hfa"
hunex hfa unpack work/cbg/data00000-edited.hfa -o work/cbg/check
hunex cbg to-png work/cbg/check/title_titlelogo_en.cbg -o work/cbg/check.png
```

Check a pixel inside the bar. It should be red:

```sh
python3 -c "
from PIL import Image
print(Image.open('work/cbg/check.png').convert('RGBA').getpixel((100, 10)))
"
```

Output:

```text
(200, 30, 30, 255)
```

Verified in game: a background image repainted with `hunex image from-png --like` displays cleanly, and the Steam release of Witch on the Holy Night showed a repainted title logo, alpha edges included.

Format details: [image format note](../formats/mzp-image-format.md), and [CompressedBG format note](../formats/cbg-format.md).
