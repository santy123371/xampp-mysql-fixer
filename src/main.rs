#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod engine;

use app::XamppFixerApp;
use eframe::egui::{IconData, Vec2};
use std::sync::Arc;

fn load_icon() -> Option<Arc<IconData>> {
    let png_bytes = include_bytes!("../icon_square.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(Arc::new(IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }))
    } else {
        None
    }
}

fn main() -> eframe::Result {
    engine::updater::cleanup_old_executable();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size(Vec2::new(780.0_f32, 580.0_f32))
        .with_min_inner_size(Vec2::new(700.0_f32, 500.0_f32))
        .with_title("XAMPP FIXER - MySQL Medic & Auto-Repair")
        .with_active(true);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "XAMPP FIXER",
        native_options,
        Box::new(|cc| Ok(Box::new(XamppFixerApp::new(cc)))),
    )
}
