use image::RgbaImage;
use image_dds::{dds_from_image, ImageFormat, Mipmaps, Quality};
use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;

#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    pub current: usize,
    pub max: usize,
}

pub fn replace_all_strings_in_bytes(data: &mut [u8], to_replace: &str, replacement: &str) {
    let replacement = elongate(replacement, 'X', to_replace.len(), true);
    let replacement_bytes = replacement.as_bytes();
    let to_replace_bytes = to_replace.as_bytes();
    let mut position = 0;

    while let Some(start) = data[position..]
        .windows(to_replace_bytes.len())
        .position(|window| window == to_replace_bytes)
    {
        let start = start + position;
        position = start + to_replace_bytes.len();
        data[start..position].copy_from_slice(replacement_bytes);
    }
}

pub fn replace_first_string_in_bytes(data: &mut [u8], to_replace: &str, replacement: &str) {
    let replacement = elongate(replacement, 'X', to_replace.len(), true);
    let replacement_bytes = replacement.as_bytes();
    let first_occurrence = match data
        .windows(to_replace.len())
        .position(|window| window == to_replace.as_bytes())
    {
        Some(pos) => pos,
        None => return, // No occurrences found, return the original data
    };
    data[first_occurrence..first_occurrence + replacement_bytes.len()]
        .copy_from_slice(replacement_bytes);
}

pub fn elongate(string: &str, character: char, length: usize, leading: bool) -> String {
    let mut result = string.to_string();
    while result.len() < length {
        if leading {
            result.insert(0, character);
        } else {
            result.push(character);
        }
    }
    result
}

pub fn find_and_replace_float(buffer: &mut [u8], target: f32, replacement: f32) {
    let buffer_len = buffer.len();
    let new_value_bytes = replacement.to_le_bytes();

    for i in 0..buffer_len - 4 {
        if let Ok(bytes) = buffer[i..i + 4].try_into() {
            let value = f32::from_le_bytes(bytes);
            if target == value {
                buffer[i..i + 4].copy_from_slice(&new_value_bytes);
            }
        }
    }
}

pub fn save_as_dds(image: &RgbaImage, output_path: String, mipmaps: Mipmaps) {
    let dds_image = dds_from_image(image, ImageFormat::BC1RgbaUnorm, Quality::Slow, mipmaps)
        .expect("Failed to convert to dds");
    let mut writer = BufWriter::new(File::create(output_path).unwrap());
    dds_image.write(&mut writer).unwrap();
}

// pub fn time_string_to_number(string: &str) -> f32 {
//     let parts: Vec<&str> = string.split(':').collect();
//     let minutes: u32 = parts[0].parse().unwrap();
//     let seconds: f32 = parts[1].parse().unwrap();
//     seconds + minutes as f32 * 60f32
// }

pub fn time_number_to_string(number: f64) -> String {
    let minutes: u32 = (number / 60f64) as u32;
    let seconds = number % 60f64;
    format!("{minutes:02}:{seconds:04.1}")
}
