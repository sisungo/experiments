# Experimental Field
This repository stores my projects that are currently in experimental stage, or little scripts that are used for once.

🚧🚧🚧🚧🚧🚧

**⚠️ WARNING**: All experiments are incomplete and subject to change.

## List of Active Experiments
 - [`trustedcell`](experiments/trustedcell): Desktop-orinted, interactive LSM that implements application-based dynamic
 access control.
 - [`app-sandbox`](experiments/app-sandbox/): An app sandboxing tool for Linux, built on the top of Landlock.


## List of Inactive Experiments
 - [`kawaii-rustc`](archive/kawaii-rustc): rustc 也要变得可爱！！！
 - [`randvoca`](archive/randvoca): A small tool to generate random vocabulary list for an artifact language.
 - [`fat32x`](archive/fat32x/): FUSE filesystem built on the top of FAT32, which wraps the FAT32 filesystem to support
 large files \(> 4GB\) and soft links, Unix permissions...
 - [`txtfmt`](archive/txtfmt): A simple formatter for plain text.
 - [`wav2bmp`](archive/wav2bmp): WAV-to-BMP converter.
 - [`bmp2wav`](archive/bmp2wav): BMP-to-WAV converter.
 - `airup-classic`: A very early \(late 2019 \~ 2020\) version of the [`Airup`](https://github.com/sisungo/airup) project.
 - `euola-vm`: A general-purpose register-based language virtual machine with minimalized instruction set. Its origin
 source files have been lost.

## Miscellaneous Scripts
Miscellaneous scripts, which are commonly single-file or domain-specific, are stored in the [`misc`](misc) directory.

## FAQ
**Q**: Will experiments keep experimental forever?

**A**: These projects will not be always experimental. Some experiments will be moved into a standalone repository when they
are no longer experimental.

**Q**: Why an experiment is listed in "Past Experiments"?

**A**: The experiments have been inactive for a long time. Issues/PRs are still welcomed and it would be marked active again
\(by moving it into [experiments](experiments)\) after your issue/PR.

**Q**: Can I make an issue or PR for one of the experiments?

**A**: Yes.

**Q**: I see the commit messages are strange. Why?

**A**: In this repository, commit messages begin with `experiment: `, and the following is a sentence from motto or lyrics.
