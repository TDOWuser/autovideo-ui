mod convert;
mod scriptwrite;
mod utility;

use std::cmp::Ordering;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::Window;
use utility::DownloadProgress;

use crate::scriptwrite::ScriptInfo;
use crate::utility::{
    elongate, find_and_replace_float, replace_all_strings_in_bytes, replace_first_string_in_bytes,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
async fn convert_files(
    window: Window,
    inputs: Vec<PathBuf>,
    input_esp: Option<PathBuf>,
    input_esp_drive_in: Option<PathBuf>,
    mod_name: String,
    input_framerate: u32,
    short_names: bool,
    video_name: Option<String>,
    size: u32,
    keep_aspect_ratio: bool,
    script_info: Option<ScriptInfo>,
) -> Result<(), String> {
    let mut videos = vec![];
    let path_to_name_and_framerate = |path: &PathBuf| -> (String, u32) {
        let mut name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let mut framerate = input_framerate;
        let split: Vec<&str> = name.split('.').collect();
        if split.len() > 1 {
            if let Ok(fps) = split[split.len() - 1].parse::<u32>() {
                framerate = fps;
                name = split[0..split.len() - 1].join("_");
            }
        }
        if short_names && name.len() > 10 {
            name = name[0..10].to_string();
        }
        (name.replace(' ', "_"), framerate)
    };
    let only_one = inputs.len() == 1;
    for input in inputs {
        let (filename, file_framerate) = path_to_name_and_framerate(&input);
        let name = if only_one {
            video_name.clone().unwrap_or(filename)
        } else {
            filename
        };
        videos.push((name, input, file_framerate));
    }

    let mut tv_esp_bytes = if let Some(input_esp) = input_esp {
        let mut bytes = vec![];
        File::open(input_esp)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    } else {
        include_bytes!("./assets/TemplateVideos_10.esp").into()
    };
    let mut di_esp_bytes = if let Some(input_esp) = input_esp_drive_in {
        let mut bytes = vec![];
        File::open(input_esp)
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    } else {
        include_bytes!("./assets/TemplateDriveIn_10.esp").into()
    };

    let mut write_drivein_esp = false;
    let mut script_video_data = Vec::new();

    let elongated_mod_identifier = elongate(&mod_name, 'X', 10, true);
    let leading_spaced_mod_identifier = elongate(&mod_name, ' ', 10, true);
    let trailing_spaced_mod_identifier = elongate(&mod_name, ' ', 10, false);

    let mut download_progress = DownloadProgress {
        current: 0,
        max: if keep_aspect_ratio {
            videos.len() * 3
        } else {
            videos.len() * 2
        },
    };

    for (video_name, video_path, video_framerate) in videos {
        let elongated_video_identifier = elongate(&video_name, 'X', 10, true);
        let trailing_spaced_video_identifier = elongate(&video_name, ' ', 10, false);

        let (grid_amount, last_stop_time, audio_name) = convert::convert_video(
            &window,
            &mut download_progress,
            video_path,
            &elongated_mod_identifier,
            &elongated_video_identifier,
            size,
            keep_aspect_ratio,
            video_framerate,
        )?;
        if !write_drivein_esp {
            write_drivein_esp = grid_amount <= 8;
        }

        if script_info.is_some() {
            script_video_data.push((
                elongated_video_identifier.clone(),
                video_name.clone(),
                audio_name.clone(),
                grid_amount <= 8,
            ));
        } else {
            let mut esps = vec![&mut tv_esp_bytes];
            if grid_amount <= 8 {
                esps.push(&mut di_esp_bytes);
            }
            for bytes in &mut esps {
                replace_all_strings_in_bytes(bytes, "AUTOCIDENT", &elongated_mod_identifier);
                for key in ["AUTOVIDENT", "AUTOSIDENT", "AUTOPIDENT"] {
                    replace_first_string_in_bytes(bytes, key, &elongated_video_identifier);
                }
                replace_all_strings_in_bytes(bytes, "AUTOTIDENT", &trailing_spaced_mod_identifier);
                replace_all_strings_in_bytes(bytes, "AUTOMIDENT", &leading_spaced_mod_identifier);
                replace_first_string_in_bytes(
                    bytes,
                    "ZAUTONIDEN",
                    &trailing_spaced_video_identifier,
                );
                replace_first_string_in_bytes(bytes, "AUTOIDENTSOUND", &audio_name);
            }
        }

        let tv_mesh_bytes: &[u8] = if grid_amount <= 8 {
            include_bytes!("./assets/TV 8 Grids.nif")
        } else {
            include_bytes!("./assets/TV 24 Grids.nif")
        };
        let pr_mesh_bytes: &[u8] = if grid_amount <= 8 {
            include_bytes!("./assets/PR 8 Grids.nif")
        } else {
            include_bytes!("./assets/PR 24 Grids.nif")
        };
        let mut mesh_bytes: Vec<(&str, &[u8])> =
            vec![("Television", tv_mesh_bytes), ("Projector", pr_mesh_bytes)];
        if grid_amount <= 8 {
            let di_8_grid_bytes = include_bytes!("./assets/DI 8 Grids.nif");
            mesh_bytes.push(("DriveIn", di_8_grid_bytes));
        }
        for (key, bytes) in mesh_bytes {
            let mut this_mesh_bytes = bytes.to_vec();
            replace_all_strings_in_bytes(
                &mut this_mesh_bytes,
                "AUTOCIDENT",
                &elongated_video_identifier,
            );
            replace_all_strings_in_bytes(
                &mut this_mesh_bytes,
                "AUTOMIDENT",
                &elongated_mod_identifier,
            );
            for grid_nr in 1..25 {
                let controller_float = match grid_nr.cmp(&grid_amount) {
                    Ordering::Less => 25.6,
                    Ordering::Equal => last_stop_time,
                    Ordering::Greater => 0f32,
                };
                let textkey_float = if controller_float == 0f32 || video_framerate == 10 {
                    controller_float
                } else {
                    controller_float / video_framerate as f32 * 10f32
                };
                find_and_replace_float(
                    &mut this_mesh_bytes,
                    121200f32 + grid_nr as f32,
                    textkey_float,
                );
                find_and_replace_float(
                    &mut this_mesh_bytes,
                    141400f32 + grid_nr as f32,
                    controller_float,
                );
            }
            find_and_replace_float(
                &mut this_mesh_bytes,
                1313f32,
                (video_framerate as f32) / 10f32,
            );
            let nif_path = format!("output/meshes/Videos/{key}/{elongated_mod_identifier}");
            let nif_path = Path::new(&nif_path);
            fs::create_dir_all(nif_path).unwrap();
            let mut file =
                File::create(nif_path.join(format!("{elongated_video_identifier}.nif"))).unwrap();
            file.write_all(&this_mesh_bytes).unwrap();
        }
    }
    if let Some(script_info) = script_info {
        scriptwrite::generate_script(
            &mod_name,
            &elongated_mod_identifier,
            &script_video_data,
            script_info,
        )?;
    } else {
        let mut esp_file = File::create(format!("output/VotW_{}.esp", mod_name)).unwrap();
        esp_file.write_all(&tv_esp_bytes).unwrap();
        if write_drivein_esp {
            let mut esp_file =
                File::create(format!("output/VotW_{}_DriveIn.esp", mod_name)).unwrap();
            esp_file.write_all(&di_esp_bytes).unwrap();
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![convert_files])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
