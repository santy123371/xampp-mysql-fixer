use crate::engine::backup::{BackupInfo, BackupManager};
use crate::engine::detector::{Detector, DiagnosticReport, ServiceState};
use crate::engine::methods::RepairMethods;
use crate::engine::service::ServiceManager;
use crate::engine::updater::{self, UpdateInfo};
use eframe::egui::{
    self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, Vec2,
};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(PartialEq, Clone, Copy)]
pub enum ActiveTab {
    AutoRepair,
    ManualMethods,
    Backups,
    ErrorLog,
    Updates,
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate(String),
    Available(UpdateInfo),
    Downloading,
    Failed(String),
}

pub enum UpdateEvent {
    CheckComplete(Result<Option<UpdateInfo>, String>),
    DownloadComplete(Result<(), String>),
}

// Colors for the modern dark palette
pub struct Theme;
#[allow(dead_code)]
impl Theme {
    pub const BG_MAIN: Color32 = Color32::from_rgb(15, 17, 23);
    pub const CARD_BG: Color32 = Color32::from_rgb(22, 27, 38);
    pub const CARD_BORDER: Color32 = Color32::from_rgb(38, 45, 64);

    pub const PRIMARY: Color32 = Color32::from_rgb(59, 130, 246);
    pub const SUCCESS: Color32 = Color32::from_rgb(16, 185, 129);
    pub const SUCCESS_BG: Color32 = Color32::from_rgb(6, 78, 59);
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);
    pub const WARNING_BG: Color32 = Color32::from_rgb(120, 53, 15);
    pub const DANGER: Color32 = Color32::from_rgb(239, 68, 68);
    pub const DANGER_BG: Color32 = Color32::from_rgb(127, 29, 29);
    pub const MUTED: Color32 = Color32::from_rgb(148, 163, 184);
    pub const CONSOLE_BG: Color32 = Color32::from_rgb(10, 12, 16);

    pub fn card_frame() -> Frame {
        Frame::NONE
            .fill(Self::CARD_BG)
            .corner_radius(CornerRadius::same(8))
            .stroke(Stroke::new(1.0_f32, Self::CARD_BORDER))
            .inner_margin(Margin::same(12))
    }

    pub fn hero_frame() -> Frame {
        Frame::NONE
            .fill(Color32::from_rgb(25, 33, 50))
            .corner_radius(CornerRadius::same(10))
            .stroke(Stroke::new(1.5_f32, Self::PRIMARY))
            .inner_margin(Margin::same(14))
    }
}

pub struct XamppFixerApp {
    pub xampp_path: Option<PathBuf>,
    pub custom_path_input: String,
    pub custom_port_input: String,
    pub active_port: u16,
    pub report: Option<DiagnosticReport>,
    pub active_tab: ActiveTab,
    pub log_messages: Vec<String>,
    pub backups_list: Vec<BackupInfo>,
    pub error_log_content: String,
    pub force_recovery_level: u32,
    pub status_notification: Option<(String, bool)>,
    pub logo_texture: Option<egui::TextureHandle>,
    pub update_status: UpdateStatus,
    pub update_rx: Option<Receiver<UpdateEvent>>,
    pub update_tx: Sender<UpdateEvent>,
    pub show_update_modal: bool,
}

impl XamppFixerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let xampp_path = Detector::find_xampp(None);
        let custom_path_input = xampp_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\xampp".into());

        let detected_port = xampp_path
            .as_ref()
            .map(|p| Detector::detect_port(p))
            .unwrap_or(3306);
        let custom_port_input = detected_port.to_string();
        let active_port = detected_port;

        let report = xampp_path.as_ref().map(|p| Detector::diagnose(p, Some(active_port)));
        let backups_list = xampp_path
            .as_ref()
            .map(|p| BackupManager::list_backups(p))
            .unwrap_or_default();

        let initial_level = report
            .as_ref()
            .and_then(|r| r.innodb_force_recovery)
            .unwrap_or(1);

        let (update_tx, update_rx) = channel();

        // Spawn initial background check for updates
        let tx_init = update_tx.clone();
        std::thread::spawn(move || {
            let res = updater::check_for_updates();
            let _ = tx_init.send(UpdateEvent::CheckComplete(res));
        });

        let mut app = Self {
            xampp_path,
            custom_path_input,
            custom_port_input,
            active_port,
            report,
            active_tab: ActiveTab::AutoRepair,
            log_messages: vec![
                format!("Sistemas de diagnóstico iniciados (Puerto configurado: {active_port})."),
                "Listo para operar en un clic.".into(),
            ],
            backups_list,
            error_log_content: String::new(),
            force_recovery_level: initial_level,
            status_notification: None,
            logo_texture: None,
            update_status: UpdateStatus::Checking,
            update_rx: Some(update_rx),
            update_tx,
            show_update_modal: false,
        };

        app.refresh_diagnosis();
        app
    }

    pub fn trigger_check_updates(&mut self) {
        self.update_status = UpdateStatus::Checking;
        let tx = self.update_tx.clone();
        std::thread::spawn(move || {
            let res = updater::check_for_updates();
            let _ = tx.send(UpdateEvent::CheckComplete(res));
        });
    }

    pub fn refresh_diagnosis(&mut self) {
        if let Ok(p) = self.custom_port_input.trim().parse::<u16>() {
            self.active_port = p;
        }
        if let Some(ref path) = self.xampp_path {
            let rep = Detector::diagnose(path, Some(self.active_port));
            self.backups_list = BackupManager::list_backups(path);
            if let Ok(log) = ServiceManager::read_error_log(path, 100) {
                self.error_log_content = log;
            }
            self.report = Some(rep);
        } else {
            self.report = None;
        }
    }

    fn add_log(&mut self, text: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.log_messages.push(format!("[{timestamp}] {}", text.into()));
    }
}

impl eframe::App for XamppFixerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configure visuals for clean dark mode
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Theme::BG_MAIN;
        visuals.window_fill = Theme::BG_MAIN;
        visuals.widgets.noninteractive.bg_fill = Theme::CARD_BG;
        visuals.widgets.inactive.bg_fill = Color32::from_rgb(26, 31, 44);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(6);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(38, 45, 64);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(6);
        visuals.widgets.active.bg_fill = Theme::PRIMARY;
        visuals.widgets.active.corner_radius = CornerRadius::same(6);
        ctx.set_visuals(visuals);

        let mut events = Vec::new();
        if let Some(ref rx) = self.update_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            match event {
                UpdateEvent::CheckComplete(res) => match res {
                    Ok(Some(info)) => {
                        self.add_log(format!("Nueva versión disponible: v{}", info.version));
                        self.update_status = UpdateStatus::Available(info);
                        self.show_update_modal = true;
                        self.status_notification = Some(("¡Nueva versión disponible! Consultá la pestaña Actualizaciones.".into(), true));
                    }
                    Ok(None) => {
                        self.update_status = UpdateStatus::UpToDate(updater::current_version().to_string());
                    }
                    Err(e) => {
                        self.update_status = UpdateStatus::Failed(e);
                    }
                },
                UpdateEvent::DownloadComplete(res) => {
                    if let Err(e) = res {
                        self.update_status = UpdateStatus::Failed(e.clone());
                        self.status_notification = Some((format!("Error en actualización: {e}"), false));
                    }
                }
            }
        }

        if self.logo_texture.is_none() {
            let img_bytes = include_bytes!("../icon_square.png");
            if let Ok(dyn_img) = image::load_from_memory(img_bytes) {
                let rgba = dyn_img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
                self.logo_texture = Some(ctx.load_texture("app_logo", color_image, egui::TextureOptions::LINEAR));
            }
        }

        let mut trigger_refresh = false;
        let mut trigger_start = false;
        let mut trigger_stop = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0_f32, 10.0_f32);

            // 1. TOP HEADER BRAND
            ui.horizontal(|ui| {
                if let Some(ref tex) = self.logo_texture {
                    ui.image((tex.id(), Vec2::new(38.0_f32, 38.0_f32)));
                } else {
                    ui.label(RichText::new("⚡").size(24.0_f32).color(Theme::PRIMARY));
                }
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("XAMPP ").size(19.0_f32).strong().color(Color32::WHITE));
                        ui.label(RichText::new("FIXER").size(19.0_f32).strong().color(Color32::from_rgb(239, 68, 68)));
                    });
                    ui.label(RichText::new("Herramienta Automática de Reparación MySQL • santy123371").size(11.0_f32).color(Theme::MUTED));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref p) = self.xampp_path {
                        if ui.button(RichText::new("📂 Abrir XAMPP").size(12.0_f32)).clicked() {
                            ServiceManager::open_in_explorer(p);
                        }
                    }
                    if ui.button(RichText::new("🔄 Refrescar").size(12.0_f32)).clicked() {
                        trigger_refresh = true;
                    }

                    match &self.update_status {
                        UpdateStatus::Available(info) => {
                            if ui.button(RichText::new(format!("🚀 v{} Disponible", info.version)).size(11.5_f32).color(Theme::SUCCESS).strong()).clicked() {
                                self.show_update_modal = true;
                                self.active_tab = ActiveTab::Updates;
                            }
                        }
                        UpdateStatus::Checking => {
                            ui.label(RichText::new("Buscando v...").size(11.0_f32).color(Theme::MUTED));
                        }
                        UpdateStatus::Downloading => {
                            ui.label(RichText::new("⏳ Descargando...").size(11.0_f32).color(Theme::WARNING));
                        }
                        _ => {
                            if ui.button(RichText::new(format!("v{}", updater::current_version())).size(11.0_f32).color(Theme::MUTED)).clicked() {
                                self.active_tab = ActiveTab::Updates;
                            }
                        }
                    }
                });
            });

            // PATH & PORT CONFIGURATION BAR
            Theme::card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Ruta:").strong().size(12.0_f32));
                    let path_edit = ui.add(egui::TextEdit::singleline(&mut self.custom_path_input).desired_width(260.0_f32));

                    ui.add_space(8.0_f32);
                    ui.label(RichText::new("Puerto:").strong().size(12.0_f32));
                    let port_edit = ui.add(egui::TextEdit::singleline(&mut self.custom_port_input).desired_width(55.0_f32));

                    if path_edit.lost_focus() || port_edit.lost_focus() || ui.button(RichText::new("Aplicar").size(11.5_f32)).clicked() {
                        if let Ok(p) = self.custom_port_input.trim().parse::<u16>() {
                            self.active_port = p;
                        }
                        let p = PathBuf::from(&self.custom_path_input);
                        if Detector::is_valid_xampp(&p) {
                            self.xampp_path = Some(p);
                            trigger_refresh = true;
                            self.status_notification = Some((format!("Configuración aplicada: Puerto {}", self.active_port), true));
                        } else {
                            self.status_notification = Some(("La ruta ingresada no parece tener XAMPP MySQL válido.".into(), false));
                        }
                    }

                    if let Some(ref p) = self.xampp_path {
                        if ui.button(RichText::new("Auto-detectar de my.ini").size(11.0_f32)).clicked() {
                            let auto_p = Detector::detect_port(p);
                            self.active_port = auto_p;
                            self.custom_port_input = auto_p.to_string();
                            trigger_refresh = true;
                            self.status_notification = Some((format!("Puerto {} detectado automáticamente desde my.ini", auto_p), true));
                        }
                    }
                });
            });

            // 2. VITAL SIGNS CARD (DASHBOARD STATUS)
            let report_snapshot = self.report.clone();
            let current_port = self.active_port;

            if let Some(rep) = report_snapshot {
                Theme::card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left: Big Status Icon & Title
                        match &rep.state {
                            ServiceState::Running(pids) => {
                                ui.colored_label(Theme::SUCCESS, RichText::new("●").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MySQL EN EJECUCIÓN").strong().color(Theme::SUCCESS));
                                    ui.label(RichText::new(format!("PIDs: {:?} | Escuchando en puerto {current_port}", pids)).size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::StoppedClean => {
                                ui.colored_label(Theme::MUTED, RichText::new("○").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MySQL DETENIDO").strong().color(Color32::WHITE));
                                    ui.label(RichText::new(format!("Servicio inactivo de forma limpia (Puerto objetivo: {current_port}).")).size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::CrashedOrphanLock(pid) => {
                                ui.colored_label(Theme::DANGER, RichText::new("⚠").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MySQL CAÍDO / BLOQUEADO").strong().color(Theme::DANGER));
                                    ui.label(RichText::new(format!("Bloqueo activo por archivo mysql.pid (PID: {pid})")).size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::PortConflict(msg) => {
                                ui.colored_label(Theme::WARNING, RichText::new("⚡").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(format!("CONFLICTO DE PUERTO {current_port}")).strong().color(Theme::WARNING));
                                    ui.label(RichText::new(msg).size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::XamppNotFound => {
                                ui.colored_label(Theme::DANGER, RichText::new("✕").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("XAMPP NO ENCONTRADO").strong().color(Theme::DANGER));
                                    ui.label(RichText::new("Verificá la ruta configurada.").size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                        }

                        // Right: Quick Service Controls
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let btn_stop = egui::Button::new(RichText::new("⏹ Detener").size(12.0_f32))
                                .fill(Color32::from_rgb(180, 40, 40));
                            if ui.add(btn_stop).clicked() {
                                trigger_stop = true;
                            }

                            let btn_start = egui::Button::new(RichText::new("▶ Iniciar").size(12.0_f32).strong())
                                .fill(Color32::from_rgb(16, 140, 90));
                            if ui.add(btn_start).clicked() {
                                trigger_start = true;
                            }
                        });
                    });

                    ui.add_space(4.0_f32);
                    ui.separator();

                    // Metric Pills Row
                    ui.horizontal_wrapped(|ui| {
                        Self::render_pill(ui, &format!("🔌 Puerto {current_port}"), if rep.port_open { "Ocupado" } else { "Libre" }, rep.port_open);
                        Self::render_pill(ui, "📁 Bases de Datos", &format!("{}", rep.user_databases.len()), true);
                        Self::render_pill(ui, "📦 Backups", &format!("{}", self.backups_list.len()), true);
                        if !rep.aria_logs_found.is_empty() {
                            Self::render_pill(ui, "⚠ Logs Aria", &format!("{}", rep.aria_logs_found.len()), false);
                        }
                        if let Some(lvl) = rep.innodb_force_recovery {
                            Self::render_pill(ui, "🚨 Recovery Mode", &format!("Nivel {lvl}"), false);
                        }
                    });
                });
            }

            // Notification Banner
            if let Some((ref msg, is_ok)) = self.status_notification {
                let (bg, border, text_color, icon) = if is_ok {
                    (Theme::SUCCESS_BG, Theme::SUCCESS, Color32::WHITE, "✓")
                } else {
                    (Theme::DANGER_BG, Theme::DANGER, Color32::WHITE, "✕")
                };

                Frame::NONE
                    .fill(bg)
                    .corner_radius(CornerRadius::same(6))
                    .stroke(Stroke::new(1.0_f32, border))
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).strong().color(text_color));
                            ui.label(RichText::new(msg).size(12.0_f32).color(text_color));
                        });
                    });
            }

            // 3. MODERN SEGMENTED TAB BAR
            ui.add_space(2.0_f32);
            ui.horizontal(|ui| {
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::AutoRepair, "⚡ Reparación");
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::ManualMethods, "🔧 Métodos");
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::Backups, "💾 Backups");
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::ErrorLog, "📋 Logs");
                let updates_label = match &self.update_status {
                    UpdateStatus::Available(_) => "🚀 Actualizaciones (1)",
                    _ => "🚀 Actualizaciones",
                };
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::Updates, updates_label);
            });

            ui.add_space(4.0_f32);

            // 4. TAB CONTENTS
            match self.active_tab {
                ActiveTab::AutoRepair => self.render_auto_repair_tab(ui),
                ActiveTab::ManualMethods => self.render_manual_methods_tab(ui),
                ActiveTab::Backups => self.render_backups_tab(ui),
                ActiveTab::ErrorLog => self.render_error_log_tab(ui),
                ActiveTab::Updates => self.render_updates_tab(ui),
            }
        });

        // Deferred Actions
        if trigger_start {
            if let Some(ref p) = self.xampp_path.clone() {
                let _ = ServiceManager::start_mysql(p);
                std::thread::sleep(std::time::Duration::from_millis(600));
                self.refresh_diagnosis();
                self.add_log(format!("Comando iniciar MySQL ejecutado (esperando en puerto {})...", self.active_port));
            }
        }
        if trigger_stop {
            if let Some(ref p) = self.xampp_path.clone() {
                let _ = ServiceManager::stop_mysql(p);
                self.refresh_diagnosis();
                self.add_log("Comando detener MySQL ejecutado.");
            }
        }
        if trigger_refresh {
            self.refresh_diagnosis();
            self.add_log(format!("Diagnóstico actualizado para puerto {}.", self.active_port));
        }

        let mut trigger_modal_update_url: Option<String> = None;
        if self.show_update_modal {
            if let UpdateStatus::Available(ref info) = self.update_status {
                let info_clone = info.clone();
                egui::Window::new("🚀 Actualización de XAMPP FIXER")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .frame(
                        Frame::NONE
                            .fill(Color32::from_rgb(22, 27, 38))
                            .corner_radius(CornerRadius::same(12))
                            .stroke(Stroke::new(1.5_f32, Theme::PRIMARY))
                            .inner_margin(Margin::same(18)),
                    )
                    .show(ctx, |ui| {
                        ui.set_width(380.0_f32);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("⚡").size(26.0_f32).color(Theme::SUCCESS));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(format!("¡Nueva versión v{} disponible!", info_clone.version)).strong().size(15.0_f32).color(Color32::WHITE));
                                    ui.label(RichText::new(format!("Versión actual: v{}", updater::current_version())).size(11.5_f32).color(Theme::MUTED));
                                });
                            });

                            ui.add_space(8.0_f32);
                            ui.label(RichText::new("¿Deseás descargar e instalar esta actualización ahora?").size(12.5_f32).color(Color32::WHITE));
                            ui.label(RichText::new("La app descargará el nuevo binario y se reiniciará automáticamente al instante sin tocar tus bases de datos.").size(11.0_f32).color(Theme::MUTED));

                            if !info_clone.notes.is_empty() {
                                ui.add_space(6.0_f32);
                                ui.label(RichText::new("Novedades:").strong().size(11.5_f32).color(Theme::MUTED));
                                ScrollArea::vertical().max_height(80.0_f32).show(ui, |ui| {
                                    Frame::NONE
                                        .fill(Theme::CONSOLE_BG)
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::same(6))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.label(RichText::new(&info_clone.notes).size(10.5_f32).color(Color32::from_rgb(200, 205, 215)));
                                        });
                                });
                            }

                            ui.add_space(12.0_f32);
                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("✓ Sí, Actualizar Ahora").strong().size(12.5_f32).color(Color32::WHITE)).clicked() {
                                    trigger_modal_update_url = Some(info_clone.download_url);
                                    self.show_update_modal = false;
                                }
                                if ui.button(RichText::new("Más tarde").size(12.0_f32)).clicked() {
                                    self.show_update_modal = false;
                                }
                            });
                        });
                    });
            } else {
                self.show_update_modal = false;
            }
        }

        if let Some(url) = trigger_modal_update_url {
            self.update_status = UpdateStatus::Downloading;
            self.status_notification = Some(("Descargando e instalando nueva versión de XAMPP FIXER...".into(), true));
            self.add_log("Descargando actualización desde GitHub...");
            let tx = self.update_tx.clone();
            std::thread::spawn(move || {
                let res = updater::apply_update(&url);
                let _ = tx.send(UpdateEvent::DownloadComplete(res));
            });
        }
    }
}

impl XamppFixerApp {
    fn render_pill(ui: &mut egui::Ui, label: &str, value: &str, is_positive: bool) {
        let (bg, border, text_col) = if is_positive {
            (Color32::from_rgb(26, 32, 48), Color32::from_rgb(46, 56, 84), Color32::from_rgb(200, 215, 240))
        } else {
            (Color32::from_rgb(48, 30, 24), Color32::from_rgb(84, 50, 40), Color32::from_rgb(255, 180, 160))
        };

        Frame::NONE
            .fill(bg)
            .corner_radius(CornerRadius::same(4))
            .stroke(Stroke::new(1.0_f32, border))
            .inner_margin(Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(label).size(10.5_f32).color(Theme::MUTED));
                    ui.label(RichText::new(value).size(11.0_f32).strong().color(text_col));
                });
            });
    }

    fn render_tab_button(ui: &mut egui::Ui, current: &mut ActiveTab, target: ActiveTab, text: &str) {
        let is_selected = *current == target;
        let (bg, text_color) = if is_selected {
            (Theme::PRIMARY, Color32::WHITE)
        } else {
            (Color32::from_rgb(24, 28, 40), Theme::MUTED)
        };

        let btn = egui::Button::new(RichText::new(text).size(12.0_f32).strong().color(text_color))
            .fill(bg)
            .corner_radius(CornerRadius::same(6));

        if ui.add(btn).clicked() {
            *current = target;
        }
    }

    fn render_auto_repair_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();
        let target_port = self.active_port;

        ui.vertical(|ui| {
            // HERO CARD
            Theme::hero_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🚀").size(22.0_f32));
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Reparación Automática Inteligente").size(15.0_f32).strong().color(Color32::WHITE));
                            ui.label(RichText::new(format!("Repara el arranque de MySQL verificando en puerto {target_port} sin tocar tus datos.")).size(11.5_f32).color(Theme::MUTED));
                        });
                    });

                    ui.add_space(8.0_f32);

                    // Workflow steps row
                    ui.horizontal_wrapped(|ui| {
                        Self::render_step_badge(ui, "1", "Matar Huérfanos");
                        ui.label(RichText::new("»").color(Theme::MUTED));
                        Self::render_step_badge(ui, "2", "Backup Preventivo");
                        ui.label(RichText::new("»").color(Theme::MUTED));
                        Self::render_step_badge(ui, "3", "Limpiar Logs Aria");
                        ui.label(RichText::new("»").color(Theme::MUTED));
                        Self::render_step_badge(ui, "4", &format!("Test Puerto {target_port}"));
                    });

                    ui.add_space(10.0_f32);

                    let btn_size = Vec2::new(ui.available_width(), 42.0_f32);
                    let btn = egui::Button::new(
                        RichText::new("⚡ EJECUTAR REPARACIÓN INTELIGENTE AHORA")
                            .size(14.0_f32)
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(Theme::PRIMARY)
                    .corner_radius(CornerRadius::same(6));

                    if ui.add_sized(btn_size, btn).clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            self.add_log(format!("Iniciando reparación automática para puerto {target_port}..."));
                            match RepairMethods::auto_repair_cascade(p, target_port) {
                                Ok(msg) => {
                                    self.add_log(msg);
                                    self.status_notification = Some(("¡Reparación automática completada con éxito!".into(), true));
                                }
                                Err(e) => {
                                    self.add_log(format!("ERROR: {e}"));
                                    self.status_notification = Some((format!("Error en reparación: {e}"), false));
                                }
                            }
                            self.refresh_diagnosis();
                        } else {
                            self.status_notification = Some(("Error: Ruta de XAMPP no configurada.".into(), false));
                        }
                    }
                });
            });

            ui.add_space(4.0_f32);

            // LOG CONSOLE CARD
            Theme::card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Terminal de Operaciones:").size(12.0_f32).strong().color(Color32::WHITE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("Limpiar").size(11.0_f32)).clicked() {
                            self.log_messages.clear();
                        }
                    });
                });

                ui.add_space(4.0_f32);

                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .max_height(190.0_f32)
                    .show(ui, |ui| {
                        Frame::NONE
                            .fill(Theme::CONSOLE_BG)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                for msg in &self.log_messages {
                                    let color = if msg.contains("ÉXITO") {
                                        Theme::SUCCESS
                                    } else if msg.contains("ERROR") {
                                        Theme::DANGER
                                    } else if msg.contains("INICIANDO") {
                                        Theme::PRIMARY
                                    } else {
                                        Color32::from_rgb(210, 215, 230)
                                    };
                                    ui.label(RichText::new(msg).monospace().size(11.5_f32).color(color));
                                }
                            });
                    });
            });
        });
    }

    fn render_step_badge(ui: &mut egui::Ui, num: &str, label: &str) {
        Frame::NONE
            .fill(Color32::from_rgb(18, 24, 38))
            .corner_radius(CornerRadius::same(4))
            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(40, 52, 78)))
            .inner_margin(Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(num).strong().color(Theme::PRIMARY).size(10.0_f32));
                    ui.label(RichText::new(label).color(Color32::WHITE).size(11.0_f32));
                });
            });
    }

    fn render_manual_methods_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                // METHOD 1 CARD
                Self::render_method_card(
                    ui,
                    "Método 1: Liberar Procesos Huérfanos y Archivos de Bloqueo",
                    "Riesgo: Cero",
                    Theme::SUCCESS,
                    "Finaliza cualquier proceso mysqld.exe colgado en segundo plano y elimina los archivos bloqueantes mysql.pid e ib_buffer_pool.",
                    "Útil cuando MySQL dice que el puerto está tomado o se cerró de golpe.",
                    "Ejecutar Liberación",
                    || {
                        if let Some(ref p) = xampp_path_opt {
                            RepairMethods::method_1_kill_orphans(p)
                        } else {
                            Err("Ruta no configurada".into())
                        }
                    },
                    &mut self.log_messages,
                    &mut self.status_notification,
                );

                ui.add_space(4.0_f32);

                // METHOD 2 CARD
                Self::render_method_card(
                    ui,
                    "Método 2: Limpieza de Logs Corruptos (Aria & Redo Logs)",
                    "Riesgo: Cero (Conserva datos)",
                    Theme::SUCCESS,
                    "Elimina los archivos aria_log* e ib_logfile* dañados tras un corte de luz o apagado forzado. MariaDB los regenera limpios automáticamente.",
                    "Solución al 90% de los crashes donde MySQL no pasa del arranque inicial.",
                    "Ejecutar Limpieza de Logs",
                    || {
                        if let Some(ref p) = xampp_path_opt {
                            RepairMethods::method_2_clean_transaction_logs(p)
                        } else {
                            Err("Ruta no configurada".into())
                        }
                    },
                    &mut self.log_messages,
                    &mut self.status_notification,
                );

                ui.add_space(4.0_f32);

                // METHOD 3 CARD
                Self::render_method_card(
                    ui,
                    "Método 3: Reconstruir Catálogo del Sistema desde backup/",
                    "Riesgo: Seguro (Backup previo)",
                    Theme::WARNING,
                    "Restaura los esquemas del sistema (mysql, phpmyadmin, performance_schema) desde la copia limpia de XAMPP. Preserva al 100% tus bases de datos e ibdata1.",
                    "Usar cuando el catálogo interno de MySQL está corrupto.",
                    "Restaurar Catálogo",
                    || {
                        if let Some(ref p) = xampp_path_opt {
                            RepairMethods::method_3_restore_system_catalog(p)
                        } else {
                            Err("Ruta no configurada".into())
                        }
                    },
                    &mut self.log_messages,
                    &mut self.status_notification,
                );

                ui.add_space(4.0_f32);

                // METHOD 4 CARD
                Theme::card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Método 4: Forzar Recuperación InnoDB (innodb_force_recovery)").strong().size(13.0_f32));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(Theme::DANGER, RichText::new("Modo Rescate").size(11.0_f32));
                        });
                    });
                    ui.label(RichText::new("Permite arrancar MySQL ignorando páginas corruptas para hacer un volcado/exportación de emergencia.").size(11.5_f32).color(Theme::MUTED));
                    
                    ui.add_space(4.0_f32);
                    ui.horizontal(|ui| {
                        ui.label("Nivel de forzado (1-6):");
                        ui.add(egui::Slider::new(&mut self.force_recovery_level, 1..=6));

                        if ui.button("Aplicar Nivel").clicked() {
                            if let Some(ref p) = self.xampp_path {
                                let _ = RepairMethods::method_4_set_force_recovery(p, Some(self.force_recovery_level));
                                self.add_log(format!("innodb_force_recovery establecido en {}", self.force_recovery_level));
                                self.status_notification = Some((format!("Recovery nivel {} aplicado. Reiniciá MySQL.", self.force_recovery_level), true));
                                self.refresh_diagnosis();
                            }
                        }

                        if ui.button("Desactivar (Normal)").clicked() {
                            if let Some(ref p) = self.xampp_path {
                                let _ = RepairMethods::method_4_set_force_recovery(p, None);
                                self.add_log("innodb_force_recovery desactivado.");
                                self.status_notification = Some(("Modo normal restablecido.".into(), true));
                                self.refresh_diagnosis();
                            }
                        }
                    });
                });
            });
        });
    }

    fn render_method_card(
        ui: &mut egui::Ui,
        title: &str,
        badge: &str,
        badge_color: Color32,
        desc: &str,
        hint: &str,
        action_label: &str,
        action: impl FnOnce() -> Result<String, String>,
        logs: &mut Vec<String>,
        notif: &mut Option<(String, bool)>,
    ) {
        Theme::card_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(title).strong().size(13.0_f32).color(Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.colored_label(badge_color, RichText::new(badge).size(11.0_f32));
                });
            });
            ui.label(RichText::new(desc).size(11.5_f32).color(Color32::from_rgb(210, 215, 230)));
            ui.label(RichText::new(format!("💡 {}", hint)).size(10.5_f32).color(Theme::MUTED));

            ui.add_space(4.0_f32);
            if ui.button(RichText::new(action_label).size(12.0_f32).strong()).clicked() {
                match action() {
                    Ok(msg) => {
                        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                        logs.push(format!("[{timestamp}] {msg}"));
                        *notif = Some(("Acción completada con éxito.".into(), true));
                    }
                    Err(e) => {
                        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                        logs.push(format!("[{timestamp}] ERROR: {e}"));
                        *notif = Some((format!("Error: {e}"), false));
                    }
                }
            }
        });
    }

    fn render_backups_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Copias de Seguridad (Backups)").size(14.0_f32).strong().color(Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("➕ Crear Backup Preventivo Ahora").size(12.0_f32).strong()).clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            match BackupManager::create_backup(p) {
                                Ok(path) => {
                                    self.add_log(format!("Backup creado en: {}", path.display()));
                                    self.status_notification = Some(("Backup creado con éxito.".into(), true));
                                }
                                Err(e) => {
                                    self.status_notification = Some((format!("Error: {e}"), false));
                                }
                            }
                            self.refresh_diagnosis();
                        }
                    }
                });
            });

            ui.add_space(4.0_f32);

            if self.backups_list.is_empty() {
                Theme::card_frame().show(ui, |ui| {
                    ui.label(RichText::new("No se encontraron copias de seguridad en mysql/backups/").color(Theme::MUTED));
                });
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    for bkp in self.backups_list.clone() {
                        Theme::card_frame().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&bkp.name).strong().color(Color32::WHITE).size(12.5_f32));
                                    ui.label(RichText::new(format!("📅 {} • {:.2} MB", bkp.created_at, bkp.size_mb)).size(11.0_f32).color(Theme::MUTED));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("Restaurar").size(11.5_f32)).clicked() {
                                        if let Some(ref p) = xampp_path_opt {
                                            match BackupManager::restore_backup(p, &bkp.path) {
                                                Ok(()) => {
                                                    self.add_log(format!("Backup restaurado: {}", bkp.name));
                                                    self.status_notification = Some(("Backup restaurado con éxito.".into(), true));
                                                }
                                                Err(e) => {
                                                    self.status_notification = Some((format!("Error: {e}"), false));
                                                }
                                            }
                                            self.refresh_diagnosis();
                                        }
                                    }
                                    if ui.button(RichText::new("Abrir Carpeta").size(11.5_f32)).clicked() {
                                        ServiceManager::open_in_explorer(&bkp.path);
                                    }
                                });
                            });
                        });
                        ui.add_space(2.0_f32);
                    }
                });
            }
        });
    }

    fn render_error_log_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Visor de mysql_error.log").size(14.0_f32).strong().color(Color32::WHITE));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Recargar Log").clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            if let Ok(l) = ServiceManager::read_error_log(p, 100) {
                                self.error_log_content = l;
                            }
                        }
                    }
                });
            });

            ui.add_space(4.0_f32);

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    Frame::NONE
                        .fill(Theme::CONSOLE_BG)
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for line in self.error_log_content.lines() {
                                let color = if line.contains("[ERROR]") || line.contains("Fatal") {
                                    Theme::DANGER
                                } else if line.contains("[Warning]") {
                                    Theme::WARNING
                                } else if line.contains("[Note]") {
                                    Color32::from_rgb(120, 180, 240)
                                } else {
                                    Color32::from_rgb(200, 205, 215)
                                };
                                ui.label(RichText::new(line).monospace().size(11.0_f32).color(color));
                            }
                        });
                });
        });
    }

    fn render_updates_tab(&mut self, ui: &mut egui::Ui) {
        let current_v = updater::current_version();
        let mut trigger_check = false;
        let mut trigger_download_url: Option<String> = None;

        ui.vertical(|ui| {
            Theme::card_frame().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🚀").size(24.0_f32));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Centro de Actualizaciones").strong().size(16.0_f32).color(Color32::WHITE));
                        ui.label(RichText::new(format!("Versión instalada: v{current_v} • Repositorio: santy123371/xampp-mysql-fixer")).size(11.0_f32).color(Theme::MUTED));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let is_busy = matches!(self.update_status, UpdateStatus::Checking | UpdateStatus::Downloading);
                        if ui.add_enabled(!is_busy, egui::Button::new(RichText::new("🔄 Buscar Actualizaciones").size(12.0_f32))).clicked() {
                            trigger_check = true;
                        }
                    });
                });
            });

            ui.add_space(6.0_f32);

            match &self.update_status {
                UpdateStatus::Idle => {
                    Theme::card_frame().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("ℹ").size(18.0_f32));
                            ui.label(RichText::new("Hacé clic en 'Buscar Actualizaciones' para verificar si hay una nueva versión en GitHub.").color(Theme::MUTED));
                        });
                    });
                }
                UpdateStatus::Checking => {
                    Theme::card_frame().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.add_space(6.0_f32);
                            ui.label(RichText::new("Consultando la API de GitHub en busca de nuevas versiones...").color(Color32::WHITE));
                        });
                    });
                }
                UpdateStatus::UpToDate(v) => {
                    Frame::NONE
                        .fill(Theme::SUCCESS_BG)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, Theme::SUCCESS))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.colored_label(Theme::SUCCESS, RichText::new("✓").size(24.0_f32).strong());
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(format!("¡Estás en la última versión disponible (v{v})!")).strong().size(14.0_f32).color(Color32::WHITE));
                                    ui.label(RichText::new("Tu aplicación cuenta con todos los últimos métodos de reparación y diagnósticos actualizados.").size(11.5_f32).color(Color32::from_rgb(209, 250, 229)));
                                });
                            });
                        });
                }
                UpdateStatus::Available(info) => {
                    let info_clone = info.clone();
                    Theme::hero_frame().show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("✨").size(24.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(format!("¡Nueva versión v{} disponible!", info_clone.version)).strong().size(15.0_f32).color(Theme::SUCCESS));
                                    ui.label(RichText::new(format!("Título: {} • Tamaño: {:.2} MB", info_clone.title, info_clone.file_size as f64 / 1_048_576.0)).size(11.5_f32).color(Theme::MUTED));
                                });
                            });

                            ui.add_space(8.0_f32);

                            if !info_clone.notes.is_empty() {
                                ui.label(RichText::new("Novedades de la versión:").strong().size(12.0_f32).color(Color32::WHITE));
                                ScrollArea::vertical().max_height(100.0_f32).show(ui, |ui| {
                                    Frame::NONE
                                        .fill(Theme::CONSOLE_BG)
                                        .corner_radius(CornerRadius::same(6))
                                        .inner_margin(Margin::same(8))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.label(RichText::new(&info_clone.notes).size(11.0_f32).color(Color32::from_rgb(220, 225, 235)));
                                        });
                                });
                                ui.add_space(8.0_f32);
                            }

                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("⚡ Actualizar Automáticamente Ahora").strong().size(13.0_f32).color(Color32::WHITE)).clicked() {
                                    trigger_download_url = Some(info_clone.download_url);
                                }
                                ui.label(RichText::new("La app descargará el nuevo binario y se reiniciará automáticamente al instante.").size(11.0_f32).color(Theme::MUTED));
                            });
                        });
                    });
                }
                UpdateStatus::Downloading => {
                    Frame::NONE
                        .fill(Theme::WARNING_BG)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, Theme::WARNING))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.add_space(8.0_f32);
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("Descargando e instalando nueva versión...").strong().size(14.0_f32).color(Color32::WHITE));
                                    ui.label(RichText::new("Por favor no cierres la app. En breve se reiniciará con la versión actualizada.").size(11.5_f32).color(Color32::from_rgb(254, 243, 199)));
                                });
                            });
                        });
                }
                UpdateStatus::Failed(err) => {
                    Frame::NONE
                        .fill(Theme::DANGER_BG)
                        .corner_radius(CornerRadius::same(8))
                        .stroke(Stroke::new(1.0_f32, Theme::DANGER))
                        .inner_margin(Margin::same(14))
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.colored_label(Theme::DANGER, RichText::new("⚠").size(22.0_f32));
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new("No se pudo verificar o descargar la actualización").strong().color(Color32::WHITE));
                                        ui.label(RichText::new(err).size(11.0_f32).color(Color32::from_rgb(254, 202, 202)));
                                    });
                                });
                                ui.add_space(6.0_f32);
                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("Reintentar").size(11.5_f32)).clicked() {
                                        trigger_check = true;
                                    }
                                    if ui.button(RichText::new("Abrir Releases en GitHub").size(11.5_f32)).clicked() {
                                        let _ = std::process::Command::new("cmd")
                                            .args(["/C", "start", "", "https://github.com/santy123371/xampp-mysql-fixer/releases"])
                                            .spawn();
                                    }
                                });
                            });
                        });
                }
            }
        });

        if trigger_check {
            self.trigger_check_updates();
            self.add_log("Búsqueda manual de actualizaciones iniciada...");
        }

        if let Some(url) = trigger_download_url {
            self.update_status = UpdateStatus::Downloading;
            self.status_notification = Some(("Descargando e instalando nueva versión de XAMPP FIXER...".into(), true));
            self.add_log("Descargando nueva versión desde GitHub...");
            let tx = self.update_tx.clone();
            std::thread::spawn(move || {
                let res = updater::apply_update(&url);
                let _ = tx.send(UpdateEvent::DownloadComplete(res));
            });
        }
    }
}
