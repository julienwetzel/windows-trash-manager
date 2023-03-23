#![warn(clippy::all, rust_2021_compatibility)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod assets;
use crate::assets::*;
use egui::Vec2;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    // Set up native GUI integration options.
    let native_options = native_options();

    eframe::run_native(
        "Windows trash manager",
        native_options,
        Box::new(|cc| Box::new(windows_trash_manager::TemplateApp::new(cc))),
    )
}

fn app_icon() -> eframe::IconData {
    let icon = image::io::Reader::new(std::io::Cursor::new(WTM_ICON))
        .with_guessed_format()
        .expect("Should be infallible")
        .decode()
        .expect("Could not decode image");
    let icon = icon.into_rgba8();
    let width = icon.width();
    let height = icon.height();
    eframe::IconData {
        rgba: icon.into_raw(),
        width,
        height,
    }
}

fn native_options() -> eframe::NativeOptions {
    let icon = app_icon();
    let window_size = Vec2::new(800.0, 360.0);
    eframe::NativeOptions {
        decorated: true,
        icon_data: Some(icon),
        initial_window_size: Some(window_size),
        ..Default::default()
    }
}
