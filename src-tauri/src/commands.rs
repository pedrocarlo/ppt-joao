use image::{ImageError, ImageFormat, ImageReader};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    // ImageError is not `Serialize` or `Type`
    Image(#[from] ImageError),
    #[error(transparent)]
    // io::Error is not `Serialize` or `Type`
    Io(#[from] std::io::Error),
}

// we must manually implement serde::Serialize
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(format!("{:#?}", self).as_str())
    }
}

impl specta::Type for AppError {
    fn inline(
        _type_map: &mut specta::TypeCollection,
        _generics: specta::Generics,
    ) -> specta::datatype::DataType {
        specta::datatype::DataType::Primitive(specta::datatype::PrimitiveType::String)
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn crop(src: PathBuf, dst: PathBuf) -> Result<Vec<String>, AppError> {
    log::debug!(
        "crop(src: {}, dst: {})",
        src.to_string_lossy(),
        dst.to_string_lossy()
    );
    let files = std::fs::read_dir(src)?;
    let file_errors: Vec<String> = files
        .par_bridge()
        .filter_map(|file| match file {
            Ok(file) => {
                if file.file_name().eq_ignore_ascii_case(".ds_store") {
                    return None;
                }

                if let Err(err) = ImageFormat::from_path(&file.path()) {
                    log::warn!("File: {}\n {err}", &file.path().to_string_lossy());
                    return Some(err.to_string());
                }

                if let Err(err) = crop_image_file(&file, &dst) {
                    log::warn!("File: {}\n {err}", &file.path().to_string_lossy());
                    return Some(err.to_string());
                }

                None
            }
            Err(err) => {
                log::warn!("{err}");
                Some(err.to_string())
            }
        })
        .collect();

    Ok(file_errors)
}

fn crop_image_file(file: &DirEntry, dst: &Path) -> Result<(), AppError> {
    let img = ImageReader::open(file.path())?.decode()?;

    let top = libm::ceil(0.055 * img.height() as f64) as u32;
    let bottom = (0.124 * img.height() as f64) as u32;
    let height = img.height() - (top + bottom);

    let cropped = img.crop_imm(
        0,
        top,
        img.width(),
        height,
    );
    let _ = cropped.save(dst.join(file.file_name()))?;
    Ok(())
}


