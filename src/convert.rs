use std::{
    path::Path,
    process::{Command, CommandArgs, ExitStatus, Stdio},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("failed to convert to path to str: {0}")]
    Path(String),
    #[error("failed to start command: {}", source)]
    Command {
        #[from]
        source: std::io::Error,
    },
    #[error("command exited with exit code: {0}")]
    ExitCode(ExitStatus),
    #[error("parse command stdout failed: {}", source)]
    Stdout {
        #[from]
        source: std::string::FromUtf8Error,
    },
    #[error("failed to convert duration to f32: {}", source)]
    F32Convert {
        #[from]
        source: std::num::ParseFloatError,
    },
}

pub fn convert_webp(
    file_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
) -> Result<(), ConvertError> {
    let file_path = file_path.as_ref();
    let out_path = out_path.as_ref();
    let status = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-nostats",
            "-i",
            file_path
                .to_str()
                .ok_or(ConvertError::Path(file_path.to_string_lossy().to_string()))?,
            out_path
                .to_str()
                .ok_or(ConvertError::Path(file_path.to_string_lossy().to_string()))?,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        Err(ConvertError::ExitCode(status))
    } else {
        Ok(())
    }
}

pub fn convert_webm(
    file_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
) -> Result<(), ConvertError> {
    let file_path = file_path.as_ref().to_str().ok_or(ConvertError::Path(
        file_path.as_ref().to_string_lossy().to_string(),
    ))?;
    let out_path = out_path.as_ref().to_str().ok_or(ConvertError::Path(
        out_path.as_ref().to_string_lossy().to_string(),
    ))?;
    let tmp_dir = temp_dir::TempDir::new()?;
    let frames_path = tmp_dir.path().join("%03d.png");
    let frames_path = frames_path.to_str().ok_or(ConvertError::Path(
        frames_path.to_string_lossy().to_string(),
    ))?;
    let palette_path = tmp_dir.path().join("palette.png");
    let palette_path = palette_path.to_str().ok_or(ConvertError::Path(
        palette_path.to_string_lossy().to_string(),
    ))?;
    let unoptimized_gif_path = tmp_dir.path().join("out.gif");
    let unoptimized_gif_path = unoptimized_gif_path.to_str().ok_or(ConvertError::Path(
        unoptimized_gif_path.to_string_lossy().to_string(),
    ))?;

    let output = Command::new("ffprobe")
        .args([
            "-loglevel",
            "quiet",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            file_path,
        ])
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    if !output.status.success() {
        return Err(ConvertError::ExitCode(output.status));
    }
    let duration = String::from_utf8(output.stdout)?;
    let duration = std::time::Duration::from_secs_f32(duration.trim().parse::<f32>()?);
    let status = Command::new("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-nostats",
            "-y",
            "-c:v",
            "libvpx-vp9",
            "-i",
            file_path,
            "-pix_fmt",
            "rgba",
            "-r",
            "15",
            "-s",
            "256x256",
            frames_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-nostats",
            "-y",
            "-i",
            frames_path,
            "-vf",
            "palettegen",
            palette_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-nostats",
            "-y",
            "-framerate",
            "15",
            "-i",
            frames_path,
            "-i",
            palette_path,
            "-lavfi",
            "paletteuse",
            "-t",
            &duration.as_millis().to_string(),
            unoptimized_gif_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("gifsicle")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-O3",
            "--colors",
            "64",
            "-i",
            unoptimized_gif_path,
            "-o",
            out_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    Ok(())
}

pub fn convert_tgs(
    file_path: impl AsRef<Path>,
    out_path: impl AsRef<Path>,
) -> Result<(), ConvertError> {
    let file_path = file_path.as_ref().to_str().ok_or(ConvertError::Path(
        file_path.as_ref().to_string_lossy().to_string(),
    ))?;
    let out_path = out_path.as_ref().to_str().ok_or(ConvertError::Path(
        out_path.as_ref().to_string_lossy().to_string(),
    ))?;
    let tmp_dir = temp_dir::TempDir::new()?;
    let tmp_dir_str = tmp_dir.path().to_str().ok_or(ConvertError::Path(
        tmp_dir.path().to_string_lossy().to_string(),
    ))?;
    let uncompressed_file_path = tmp_dir.path().join("out.tgs");
    let uncompressed_file = std::fs::File::create(&uncompressed_file_path)?;
    let uncompressed_file_path = uncompressed_file_path.to_str().ok_or(ConvertError::Path(
        uncompressed_file_path.to_string_lossy().to_string(),
    ))?;
    let frames_path = tmp_dir.path().join("%03d.png");
    let frames_path = frames_path.to_str().ok_or(ConvertError::Path(
        frames_path.to_string_lossy().to_string(),
    ))?;
    let palette_path = tmp_dir.path().join("palette.png");
    let palette_path = palette_path.to_str().ok_or(ConvertError::Path(
        palette_path.to_string_lossy().to_string(),
    ))?;
    let unoptimized_gif_path = tmp_dir.path().join("out.gif");
    let unoptimized_gif_path = unoptimized_gif_path.to_str().ok_or(ConvertError::Path(
        unoptimized_gif_path.to_string_lossy().to_string(),
    ))?;
    let status = Command::new("gunzip")
        .args(["-dc", file_path])
        .stdout(Stdio::from(uncompressed_file))
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("lottie_to_png")
        .args([
            "--width",
            "216",
            "--height",
            "216",
            "--fps",
            "15",
            "--threads",
            "1",
            "--output",
            tmp_dir_str,
            uncompressed_file_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-nostats",
            "-y",
            "-i",
            frames_path,
            "-vf",
            "palettegen",
            palette_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("ffmpeg")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-hide_banner",
            "-loglevel",
            "quiet",
            "-nostats",
            "-y",
            "-framerate",
            "15",
            "-i",
            frames_path,
            "-i",
            palette_path,
            "-lavfi",
            "paletteuse",
            unoptimized_gif_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    let status = Command::new("gifsicle")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .args([
            "-O3",
            "--colors",
            "64",
            "-i",
            unoptimized_gif_path,
            "-o",
            out_path,
        ])
        .spawn()?
        .wait()?;
    if !status.success() {
        return Err(ConvertError::ExitCode(status));
    }
    Ok(())
}
