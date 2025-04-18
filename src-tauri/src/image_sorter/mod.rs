use std::sync::Mutex;
use std::time::Instant;
use std::{fs::DirEntry, path::PathBuf};

use image::{DynamicImage, GenericImageView, ImageError, ImageFormat, open};
use imageproc::filter::bilateral_filter;
use kalosm_ocr::OcrInferenceSettings;
use kalosm_ocr::{LoadOcrError, Ocr};
use rayon::iter::{ParallelBridge as _, ParallelIterator as _};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager as _, State};
use tauri_specta::Event;

use crate::crop::AppError;

macro_rules! try_image_error {
    ($res:expr, $file:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => {
                let err = AppError::Image {
                    filepath: $file.path().to_string_lossy().to_string(),
                    error: err,
                };
                log::warn!("{err}");
                return Some(err);
            }
        }
    };
}

macro_rules! try_ocr_error {
    ($res:expr, $file:expr) => {
        match $res {
            Ok(val) => val,
            Err(err) => {
                let err = AppError::Ocr {
                    filepath: $file.path().to_string_lossy().to_string(),
                    error: err,
                };
                log::warn!("{err}");
                return Some(err);
            }
        }
    };
}

const LEFT_PERCENTAGE: f64 = 60.0 / 473.0;
const RIGHT_PERCENTAGE: f64 = 230.0 / 473.0;
const TOP_PERCENTAGE: f64 = 17.0 / 842.0;
const BOTTOM_PERCENTAGE: f64 = 53.0 / 842.0;

pub type OcrModel = Mutex<Ocr>;

// Add `tauri_specta::Event` to your event
#[derive(Serialize, Deserialize, Debug, Clone, Type, Event)]
pub struct OcrLoadEvent(f32);

pub async fn load_ocr(handle: AppHandle) -> std::result::Result<(), LoadOcrError> {
    let copy_handle = handle.clone();
    let ocr = Ocr::builder()
        .build_with_loading_handler(move |progress| {
            let _ = OcrLoadEvent(progress.progress()).emit(&copy_handle);
        })
        .await?;
    handle.manage(Mutex::new(ocr));
    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub fn sort_images(state: State<'_, OcrModel>, dir: PathBuf) -> Result<Vec<AppError>, AppError> {
    let files = std::fs::read_dir(dir)?;
    let t_start = Instant::now();
    let file_errors: Vec<AppError> = files
        .par_bridge()
        .filter_map(|file| match file {
            Ok(file) => {
                if file.file_name().eq_ignore_ascii_case(".ds_store") {
                    return None;
                }

                try_image_error!(ImageFormat::from_path(&file.path()), file);

                let img = try_image_error!(preprocess(&file), file);

                let text = {
                    let mut ocr = state.lock().unwrap();

                    try_ocr_error!(ocr.recognize_text(OcrInferenceSettings::new(img)), file)
                };

                log::info!(
                    "image ocr: {} - recognize_text: {}",
                    file.path().to_string_lossy(),
                    text
                );

                None
            }
            Err(err) => {
                log::warn!("{err}");
                Some(AppError::Custom(err.to_string()))
            }
        })
        .collect();
    let t_end = Instant::now();
    println!("Time taken: {} ms", (t_end - t_start).as_millis());
    Ok(file_errors)
}

/// Ideally want to remove background as much as possible and have only text left
fn preprocess(file: &DirEntry) -> Result<DynamicImage, ImageError> {
    let img = open(file.path())?;

    let (width, height) = img.dimensions();

    let left = (width as f64 * LEFT_PERCENTAGE) as u32;
    let top = (height as f64 * TOP_PERCENTAGE) as u32;
    let right = (width as f64 * RIGHT_PERCENTAGE) as u32;
    let bottom = (height as f64 * BOTTOM_PERCENTAGE) as u32;

    let width = right - left;
    let height = bottom - left;

    log::debug!("preprocess::crop({left}, {top}, {width}, {height})");

    let img = img
        .crop_imm(left, top, width, height)
        .grayscale()
        .into_luma8();
    // TODO maybe gamma correct first?
    let mut img: DynamicImage = bilateral_filter(&img, 5, 75.0, 75.0).into();

    img.invert();

    Ok(img)
}
