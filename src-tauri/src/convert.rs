use crate::utility::{save_as_dds, time_number_to_string, DownloadProgress};
use image::GenericImageView;
use image_dds::Mipmaps;
use rayon::prelude::*;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Emitter, Window};

pub fn convert_video(
    window: &Window,
    progress: &mut DownloadProgress,
    input: PathBuf,
    mod_identifier: &str,
    video_identifier: &str,
    frame_size: u32,
    auto_scale: bool,
    framerate: u32,
) -> Result<(u8, f32, String), String> {
    let audio_path = format!("output/Sound/Videos/{mod_identifier}");
    fs::create_dir_all(&audio_path).unwrap();
    let wav_name = format!("{video_identifier}.wav");
    let wav_path = format!("{audio_path}/{wav_name}");
    let input_str = input.to_str().unwrap();

    if !Path::new("./autovideo cache/frames").exists() {
        fs::create_dir_all("./autovideo cache/frames").unwrap();
    }
    let padded_video_path = "./autovideo cache/Video.mp4";
    if auto_scale {
        match Command::new("ffmpeg")
            .args([
                "-i",
                input_str,
                "-c:a",
                "copy",
                "-vf",
                "pad=max(iw\\,ih*4/3):max(ih\\,iw*3/4):(ow-iw)/2:(oh-ih)/2",
                "-crf",
                "18",
                "-y",
                padded_video_path,
            ])
            .status()
        {
            Ok(ffmpeg_status) => {
                if !ffmpeg_status.success() {
                    return Err("Failed to pad video".to_string());
                }
            }
            Err(e) => return Err(format!("{}: ffmpeg is not installed!", e)),
        }
        progress.current = progress.current + 1;
        window.emit("listener", progress.clone()).unwrap();
    }

    let video_path = if auto_scale {
        padded_video_path
    } else {
        input_str
    };

    let has_sound = match Command::new("ffprobe")
        .args([
            "-i",
            video_path,
            "-show_streams",
            "-select_streams",
            "a",
            "-loglevel",
            "error",
        ])
        .output()
    {
        Ok(probe) => {
            if !probe.status.success() {
                return Err(format!(
                    "Failed to probe video for audio track: {input_str}"
                ));
            } else {
                !String::from_utf8(probe.stdout).unwrap().trim().is_empty()
            }
        }
        Err(e) => return Err(format!("{e}: ffprobe not found!")),
    };

    let mut args: Vec<String> = [
        "-i",
        video_path,
        "-filter:v",
        &format!("scale={frame_size}:{frame_size}"),
        "-r",
        &format!("{framerate}"),
        "-f",
        "image2",
        "-c:v",
        "png",
        "-b:v",
        "2M",
        "-maxrate",
        "2M",
        "-bufsize",
        "1M",
        "-async",
        "44000",
        "autovideo cache/frames/%04d.png",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    if has_sound {
        args.extend(["-ac", "1", &wav_path].map(|s| s.to_string()));
    }
    match Command::new("ffmpeg").args(args).status() {
        Ok(ffmpeg_status) => {
            if !ffmpeg_status.success() {
                return Err(format!("Failed to convert video: {input_str}"));
            }
        }
        Err(e) => return Err(format!("{}: ffmpeg is not installed!", e)),
    }
    progress.current = progress.current + 1;
    window.emit("listener", progress.clone()).unwrap();
    let xwm_name = format!("{video_identifier}.xwm");
    let xwm_path = format!("{audio_path}/{xwm_name}");
    let xwma_encoder_path = Path::new("./autovideo cache/xWMAEncode.exe");
    if !xwma_encoder_path.exists() {
        let encoder_bytes = include_bytes!("./assets/xWMAEncode.exe");
        let mut encoder_file = File::create(xwma_encoder_path).unwrap();
        encoder_file.write_all(encoder_bytes).unwrap();
    }
    Command::new("./autovideo cache/xWMAEncode")
        .args([&wav_path, &xwm_path])
        .status()
        .unwrap();
    let xwm_exists = Path::new(&xwm_path).exists();
    if xwm_exists {
        fs::remove_file(&wav_path).unwrap();
    }
    if auto_scale {
        fs::remove_file(padded_video_path).unwrap_or_else(|e| println!("{}", e));
    }

    println!("\nReading frames ...");
    let frames: Vec<_> = fs::read_dir("./autovideo cache/frames")
        .unwrap()
        .flatten()
        .collect();
    let frames: Vec<_> = frames
        .par_iter()
        .map(|f| image::open(f.path()).unwrap())
        .chunks(256)
        .collect();
    fs::remove_dir_all("./autovideo cache").unwrap();

    let grid_amount = frames.len();
    if grid_amount > 24 {
        let mut max_time = 614.4;
        if framerate != 10 {
            max_time = max_time / framerate as f64 * 10f64;
        }
        return Err(format!(
            "Video {} is longer than {} (24 grids). Reduce FPS or use a shorter video.",
            input.file_stem().unwrap().to_str().unwrap(),
            time_number_to_string(max_time)
        ));
    }
    let last_chunk_frame_amount = frames.last().unwrap().len();

    let grids_path_string = format!("output/textures/Videos/{mod_identifier}/{video_identifier}");
    let grids_path = Path::new(&grids_path_string);
    fs::create_dir_all(grids_path).unwrap();

    println!("Converting frames to grids ...");
    let dimension = frame_size * 16;
    frames
        .into_par_iter()
        .enumerate()
        .for_each(|(grid_index, grid)| {
            let mut output_grid =
                image::RgbaImage::from_pixel(dimension, dimension, [0, 0, 0, 255].into());
            for (index, frame) in grid.iter().enumerate() {
                let x_offset = (index as u32 % 16) * frame_size;
                let y_offset = (index as u32 / 16) * frame_size;
                for (x, y, pixel) in frame.pixels() {
                    output_grid.put_pixel(x + x_offset, y + y_offset, pixel);
                }
            }
            save_as_dds(
                &output_grid,
                format!("{}/Grid{:0>2}.dds", grids_path_string, grid_index + 1),
                Mipmaps::Disabled,
            );
        });

    progress.current = progress.current + 1;
    window.emit("listener", progress.clone()).unwrap();

    Ok((
        grid_amount as u8,
        last_chunk_frame_amount as f32 / 10f32,
        if xwm_exists { xwm_name } else { wav_name },
    ))
}
