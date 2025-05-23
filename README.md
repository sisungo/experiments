# Experimental Field
This repository stores my projects that are currently in experimental stage, or little scripts that are used for once.

🚧🚧🚧🚧🚧🚧

**⚠️ WARNING**: All experiments are incomplete and subject to change.

## List of Active Experiments
These experiments are currently under active development.

 - [`trustedcell`](experiments/trustedcell): Desktop-orinted, interactive LSM that implements application-based dynamic
 access control.
 - [`landlockwrap`](experiments/landlockwrap): An app sandboxing tool for Linux, built on the top of Landlock. Originally `app-sandbox`.
 - [`vinyld`](experiments/vinyld): A Spotify-like web music backend service.

## List of Future Experiments
These experiments are currently under development, but not public yet.

 - `pessica`: A highly flexible hybrid kernel for operating systems.
 - `patina-lang`: A new strong-and-statically-typed programming language, running on the top of PatinaVM.
 - `patina-vm`: A language virtual machine which mainly focus on interpreter mode, providing dynamic features like GC,
 reflection, dynamic code executation, stackful coroutines, etc, and small in size. Aiming to have similar usage to Lua.

## List of Inactive Experiments
These experiments are not actively maintained.

 - [`app-sandbox`](archive/app-sandbox): An app sandboxing tool for Linux, built on the top of Landlock. Replaced by new experiment `landlockwrap`.
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
Miscellaneous scripts, which are commonly single-file or domain-specific, are stored in the [`misc`](misc) directory. Note that
**not** all the scripts are listed here.

 - [`clean.sh`](misc/clean.sh): Clears all temporary files generated by build systems in its parent directory.
 - [`toolkit`](misc/toolkit): Toolkit for writing other misc scripts in Rust.
 - [`music_convert`](misc/music_convert): A tool for converting new raw music file to the author's music database format.

## FAQ
**Q**: Will experiments keep experimental forever?

**A**: These projects will not be always experimental. Some experiments will be moved into a standalone repository when they
are no longer experimental.

**Q**: Why an experiment is listed in "Past Experiments"?

**A**: The experiments have been inactive for a long time. Issues/PRs are still welcomed and it would be marked active again
\(by moving it into [experiments](experiments)\) after your issue/PR.

**Q**: Can I make an issue or PR for one of the experiments?

**A**: Yes.

**Q**: I see the commit messages strange. Why?

**A**: In this repository, commit messages begin with `experiment: `, and the following is a sentence from motto or lyrics.
