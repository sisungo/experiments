include!("shared/rust_common.h.rs");

use std::path::{Path, PathBuf};

// Working Directories
const MUSIC_LIBRARY_DIR: &str = "/Volumes/sisungo/Music";
const MUSIC_CONVERT_WORK_DIR: &str = "/Volumes/sisungo/Downloads/tmp.musicqueue/tmp.1111";

// Parameters
const VALID_BITRATES: &[&str] = &["32k", "48k", "64k", "96k", "128k", "192k", "256k", "320k"];

// Tools
fn preprocess(path: &Path) -> Result<PathBuf, StdError> {
    Ok(path.to_path_buf())
}

fn mangle_name(title: &str, artist: &str) -> String {
    let should_shadow: Vec<char> =
        " ,.?!@#$%^&*()[]{}|~/-=_+`\\:;'\"<>～·！¥（）——【】「」、｜；：‘’“”《》，。"
            .chars()
            .collect();
    let mangle = |s: &str| s.replace(|x| should_shadow.contains(&x), "_");
    let title = mangle(title);
    let artist = mangle(artist);

    format!("{title}@{artist}")
}

fn main() -> Result<(), StdError> {
    // Parse command-line arguments first.
    let origin_input_path = std::env::args()
        .nth(1)
        .ok_or_else(|| -> StdError { Box::from("no input file") })?;

    // Ensure an empty `MUSIC_CONVERT_WORK_DIR` exists.
    std::fs::remove_dir_all(MUSIC_CONVERT_WORK_DIR).ok();
    std::fs::create_dir_all(MUSIC_CONVERT_WORK_DIR)?;
    std::env::set_current_dir(MUSIC_CONVERT_WORK_DIR)?;

    let input_path = preprocess(Path::new(&origin_input_path))?;
    std::fs::copy(&input_path, "./InputCompressedWaveFile")?;
    perform(&[
        "ffmpeg",
        "-i",
        "./InputCompressedWaveFile",
        "-map_metadata",
        "-1",
        "./InputWaveFile.wav",
    ])??;

    // Begin Ask User Stage
    let title = ask("Title: ")?;
    let album = ask("Album: ")?;
    let artist = ask("Artist: ")?;
    let cover_path = ask("Path of \"Cover (front)\" image (`null` if not existent): ")?;
    let bitrate = loop {
        let bitrate = ask("Bitrate: ")?;
        if !VALID_BITRATES.contains(&&bitrate[..]) {
            continue;
        }
        break bitrate;
    };

    // Process something
    let cover_path = match &cover_path[..] {
        "null" => None,
        x => Some(x),
    };
    let has_cover = cover_path.is_some();
    if let Some(x) = &cover_path {
        perform(&[
            "ffmpeg",
            "-i",
            x,
            "-map_metadata",
            "-1",
            "-vf",
            "scale=300*300",
            "./InputCoverFront.png",
        ])??;
    }

    // Begin converting
    let mangled = mangle_name(&title, &artist);
    let filename = format!("{mangled}.opus");
    let path = Path::new(MUSIC_LIBRARY_DIR).join(&filename);
    let path = path.to_string_lossy();
    let mut command = vec![
        "opusenc",
        "--title",
        &title,
        "--artist",
        &artist,
        "--album",
        &album,
        "--bitrate",
        &bitrate,
    ];
    if has_cover {
        command.append(&mut vec!["--picture", "./InputCoverFront.png"]);
    }
    command.push("./InputWaveFile.wav");
    command.push(&path[..]);
    perform(&command)??;

    // Delete files before exit.
    if let Some(x) = &cover_path {
        std::fs::remove_file(x)?;
    }
    std::fs::remove_file(origin_input_path)?;
    std::fs::remove_file(input_path).ok(); // Preprocessed input file path may be same as origin, so errors are ignored.
    std::env::set_current_dir("/")?; // At least we have access to root directory.
    std::fs::remove_dir_all(MUSIC_CONVERT_WORK_DIR)?;

    Ok(())
}
