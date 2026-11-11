# Format notes

Each note describes one file format as the crate reads and writes it: the layout, the encoding rules, what the game accepts, and the open questions.

Every claim in a note carries one of three labels:

- **Checked on sample files.** The crate's output was run on the sample files, and the result matches the note. The sample files are the owner's own copies of the game's files, which the gated tests read. See [CONTRIBUTING](../../CONTRIBUTING.md#tests-that-use-real-files).
- **Verified in game.** The result was seen in the Steam release of Witch on the Holy Night, on the date given. Only the in-game results are listed under this label.
- **Open.** The behavior is not yet checked on the sample files or in the game. The note states what is not known.

A note that is not labeled as verified in game does not claim the game accepts the format.

Some claims come from the engine's code or the game's data rather than from this crate's output. These carry a source mark:

- **[code]** Read from code, at the address or file given.
- **[data]** Read from the game's data files, at the samples or counts given.
- **[code; visible effect unverified]** Read from the code, but not checked on screen.

T in an address is an offset in the Switch main module; S is a virtual address in Mahoyo's `WoH.exe`.

[Animation tables](animation-table.md): the per-animation keyframe data the engine's evaluator reads. The current tools do not read or write it.
