# Getting started

This guide explains which files you need, how to make a working copy, and how to check that `hunex` reads them. The other guides start from here.

## Prerequisites

- A POSIX shell. The commands in these guides use POSIX shell syntax. On Linux and macOS, use your terminal. On Windows, use WSL or Git Bash.
- `hunex` on your `PATH`. See [Install hunex](#install-hunex).
- `cmp` and `sed`. The `sed -i` commands use GNU sed. On macOS, which ships BSD sed, write `sed -i ''` instead of `sed -i`.
- Only the guides that paint images need Python 3 with the Pillow library.
- Only the audio guide needs `ffmpeg` built with the libvorbis encoder.

## Install hunex

Follow the install steps in the [README](../../README.md#install). Then check that the command runs:

```sh
hunex --version
```

Output:

```text
hunex 0.1.0
```

If the command is not found after `cargo install`, add Cargo's bin folder to your `PATH`. Cargo puts the binary in `~/.cargo/bin` by default.

## Find the game's files

The game's files are in the game's folder under `steamapps/common` in your Steam library folder. The archives are named `data*.hfa`. They sit at the top level of that folder, beside the game's executable.

To list them, run this in a shell, with the game's folder in place of the placeholder:

```sh
ls "<game folder>"/data*.hfa
```

## Make a working copy

Make a work folder, and copy only the archives a guide needs into it. Each guide names its archive. `cp` reads the source and does not change it:

```sh
mkdir -p work
cp "<game folder>/data00300.hfa" work/
```

Keep the work folder outside the game's folder. Large scratch output belongs under `/var/tmp` or another disk with space, not `/tmp`, which many systems keep in RAM.

## Stored and decompressed entries

An archive entry is stored in one of two forms:

- The stored form is the entry's bytes as they are in the archive. `hunex hfa unpack` without `--decompress` writes this form. The guides call it the stored form.
- The decompressed form is the plain bytes inside a LenZu entry. `hunex hfa unpack --decompress` writes this form. It applies only to entries that are LenZu streams.

Each guide says which form it needs.

## Check the archive

Print the version, then list an archive:

```sh
hunex --version
hunex hfa list work/data00300.hfa
```

Output:

```text
hunex 0.1.0
7 entries
   0      32611  raw    SSEDecided.hw
   1       7565  raw    SSECancelled.hw
   2       9205  raw    SSEChoiced.hw
   3      18698  raw    SSETurnedPage.hw
   4      18979  raw    SSEUnable.hw
   5      13188  raw    SSEBookDecided.hw
   6       8765  raw    SSEBookChoiced.hw
```

The columns are the entry number, the size in bytes, the kind of data, and the name. The `raw` kind means the entry is stored as it is. The `lenzu` kind means a LenZu stream.

These values come from the Steam release of Witch on the Holy Night. Another game, or another release, has different sizes and counts.

## The archive layout

An HFA archive starts with a 12-byte magic and a 4-byte entry count. Each entry has a 0x80-byte record that holds its name, the offset and size of its data, and reserved bytes. The data follows the records. The full layout is in the [HFA format note](../formats/hfa-format.md).

## Next steps

- [Editing script text](editing-text.md)
- [Editing scripts](editing-scripts.md)
- [Editing images](editing-images.md)
- [Editing audio](editing-audio.md)
- [Editing fonts](editing-fonts.md)
- [Reading Switch archives](switch-files.md)
- [Repacking an archive](repacking.md)
- The command reference is in [../cli.md](../cli.md).

Verified in game: not applicable. This guide only reads files.

Format details: [HFA format note](../formats/hfa-format.md).
