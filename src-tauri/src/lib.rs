#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use commands::crop;
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use tauri_plugin_log::fern::colors::ColoredLevelConfig;
use tauri_specta::{collect_commands, Builder};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = Builder::<tauri::Wry>::new()
        // Then register them (separated by a comma)
        .commands(collect_commands![crop,]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    builder
        .export(
            Typescript::default()
                .formatter(specta_typescript::formatter::biome)
                .header("/* eslint-disable */")
                .header("// @ts-nocheck"),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .with_colors(ColoredLevelConfig::default())
                .level_for("tauri", log::LevelFilter::Warn)
                .level_for("tao", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // This is also required if you want to use events
            builder.mount_events(app);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
