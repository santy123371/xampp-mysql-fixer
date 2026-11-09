#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod engine;

use app::XamppFixerApp;
use eframe::egui::Vec2;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(760.0, 560.0))
            .with_min_inner_size(Vec2::new(640.0, 480.0))
            .with_title("XAMPP MySQL Medic & Auto-Fixer")
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "XAMPP MySQL Medic & Auto-Fixer",
        native_options,
        Box::new(|cc| Ok(Box::new(XamppFixerApp::new(cc)))),
    )
}
