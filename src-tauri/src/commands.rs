use image::{ImageError, ImageReader};
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CropError {
    #[error(transparent)]
    // ImageError is not `Serialize` or `Type`
    Image(#[from] ImageError),
    #[error(transparent)]
    // io::Error is not `Serialize` or `Type`
    Io(#[from] std::io::Error),
}

// we must manually implement serde::Serialize
impl serde::Serialize for CropError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(format!("{:#?}", self).as_str())
    }
}

impl specta::Type for CropError {
    fn inline(
        _type_map: &mut specta::TypeCollection,
        _generics: specta::Generics,
    ) -> specta::datatype::DataType {
        specta::datatype::DataType::Primitive(specta::datatype::PrimitiveType::String)
    }
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
