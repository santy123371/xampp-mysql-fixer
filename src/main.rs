#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod engine;

use app::XamppFixerApp;
use eframe::egui::Vec2;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(780.0_f32, 580.0_f32))
            .with_min_inner_size(Vec2::new(700.0_f32, 500.0_f32))
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
