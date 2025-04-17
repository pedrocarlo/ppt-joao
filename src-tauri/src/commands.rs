use image::{ImageError, ImageReader};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error, specta::Type, Serialize)]
pub enum CropError {
    #[error(transparent)]
    // ImageError is not `Serialize` or `Type`
    Image(
        #[from]
        #[serde(skip)]
        ImageError,
    ),
    #[error(transparent)]
    // io::Error is not `Serialize` or `Type`
    Io(
        #[from]
        #[serde(skip)]
        std::io::Error,
    ),
}

#[tauri::command]
#[specta::specta]
pub fn crop(src: PathBuf, dst: PathBuf) -> Result<Vec<String>, CropError> {
    let files = std::fs::read_dir(src)?;
    let mut file_errors = Vec::new();

    for file in files {
        match file {
            Ok(file) => {
                let img = ImageReader::open(file.path())?.decode()?;

                let top = (0.055 * img.height() as f64) as u32;
                let bottom = (0.124 * img.height() as f64) as u32;
                let height = img.height() - top - bottom;

                let cropped = img.crop_imm(
                    0,
                    libm::ceil(0.055 * img.height() as f64) as u32,
                    img.width(),
                    height,
                );
                let _ = cropped.save(dst.join(file.file_name()))?;
            }
            Err(err) => {
                file_errors.push(err.to_string());
            }
        }
    }

    Ok(file_errors)
}
