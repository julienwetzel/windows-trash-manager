#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions {
        min_window_size: Some(egui::vec2(400.0, 240.0)),
        initial_window_size: Some(egui::vec2(800.0, 360.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Windows trash manager",
        native_options,
        Box::new(|cc| Box::new(windows_trash_manager::TemplateApp::new(cc))),
    )
}
