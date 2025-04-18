use image::{open, ImageError, ImageFormat};
use kalosm_ocr::OcrInferenceError;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs::DirEntry,
    path::{Path, PathBuf},
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum AppError {
    // #[error(transparent)]
    // Image(#[from] ImageError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid image: `{filepath}`. Error: {error}")]
    Image {
        filepath: String,
        #[source]
        error: ImageError,
    },
    #[error("ocr failed on image: `{filepath}`. Error: {error}")]
    Ocr {
        filepath: String,
        #[source]
        error: OcrInferenceError,
    },
    #[error("{0}")]
    Custom(String),
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
pub fn crop(src: PathBuf, dst: PathBuf) -> Result<Vec<AppError>, AppError> {
    log::debug!(
        "crop(src: {}, dst: {})",
        src.to_string_lossy(),
        dst.to_string_lossy()
    );
    let files = std::fs::read_dir(src)?;
    let file_errors: Vec<AppError> = files
        .par_bridge()
        .filter_map(|file| match file {
            Ok(file) => {
                if file.file_name().eq_ignore_ascii_case(".ds_store") {
                    return None;
                }

                if let Err(err) = ImageFormat::from_path(&file.path()) {
                    let err = AppError::Image {
                        filepath: file.path().to_string_lossy().to_string(),
                        error: err,
                    };
                    log::warn!("{err}");
                    return Some(err);
                }

                if let Err(err) = crop_image_file(&file, &dst) {
                    let err = AppError::Image {
                        filepath: file.path().to_string_lossy().to_string(),
                        error: err,
                    };
                    log::warn!("{err}");
                    return Some(err);
                }

                None
            }
            Err(err) => {
                log::warn!("{err}");
                Some(AppError::Custom(err.to_string()))
            }
        })
        .collect();

    Ok(file_errors)
}

fn crop_image_file(file: &DirEntry, dst: &Path) -> Result<(), ImageError> {
    let img = open(file.path())?;

    let top = libm::ceil(0.055 * img.height() as f64) as u32;
    let bottom = (0.124 * img.height() as f64) as u32;
    let height = img.height() - (top + bottom);

    let img = img.crop_imm(0, top, img.width(), height);
    let _ = img.save(dst.join(file.file_name()))?;
    Ok(())
}
