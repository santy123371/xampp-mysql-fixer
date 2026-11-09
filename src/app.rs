use crate::engine::backup::{BackupInfo, BackupManager};
use crate::engine::detector::{Detector, DiagnosticReport, ServiceState};
use crate::engine::methods::RepairMethods;
use crate::engine::service::ServiceManager;
use eframe::egui::{
    self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, Vec2,
};
use std::path::PathBuf;

#[derive(PartialEq, Clone, Copy)]
pub enum ActiveTab {
    AutoRepair,
    ManualMethods,
    Backups,
    ErrorLog,
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
    pub report: Option<DiagnosticReport>,
    pub active_tab: ActiveTab,
    pub log_messages: Vec<String>,
    pub backups_list: Vec<BackupInfo>,
    pub error_log_content: String,
    pub force_recovery_level: u32,
    pub status_notification: Option<(String, bool)>,
}

impl XamppFixerApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let xampp_path = Detector::find_xampp(None);
        let custom_path_input = xampp_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "C:\\xampp".into());

        let report = xampp_path.as_ref().map(|p| Detector::diagnose(p));
        let backups_list = xampp_path
            .as_ref()
            .map(|p| BackupManager::list_backups(p))
            .unwrap_or_default();

        let initial_level = report
            .as_ref()
            .and_then(|r| r.innodb_force_recovery)
            .unwrap_or(1);

        let mut app = Self {
            xampp_path,
            custom_path_input,
            report,
            active_tab: ActiveTab::AutoRepair,
            log_messages: vec![
                "Sistemas de diagnóstico iniciados.".into(),
                "Listo para operar en un clic.".into(),
            ],
            backups_list,
            error_log_content: String::new(),
            force_recovery_level: initial_level,
            status_notification: None,
        };

        app.refresh_diagnosis();
        app
    }

    pub fn refresh_diagnosis(&mut self) {
        if let Some(ref path) = self.xampp_path {
            let rep = Detector::diagnose(path);
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

        let mut trigger_refresh = false;
        let mut trigger_start = false;
        let mut trigger_stop = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0_f32, 10.0_f32);

            // 1. TOP HEADER BRAND & QUICK PATH
            ui.horizontal(|ui| {
                ui.label(RichText::new("⚡").size(24.0_f32).color(Theme::PRIMARY));
                ui.vertical(|ui| {
                    ui.label(RichText::new("XAMPP MySQL Medic").size(18.0_f32).strong().color(Color32::WHITE));
                    ui.label(RichText::new("Herramienta Automática de Reparación y Rescate").size(11.0_f32).color(Theme::MUTED));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref p) = self.xampp_path {
                        if ui.button(RichText::new("📂 Abrir Carpeta").size(12.0_f32)).clicked() {
                            ServiceManager::open_in_explorer(p);
                        }
                    }
                    if ui.button(RichText::new("🔄 Refrescar").size(12.0_f32)).clicked() {
                        trigger_refresh = true;
                    }
                    // Small path pill
                    ui.label(RichText::new(format!("Ruta: {}", self.custom_path_input)).size(11.0_f32).color(Theme::MUTED));
                });
            });

            // 2. VITAL SIGNS CARD (DASHBOARD STATUS)
            let report_snapshot = self.report.clone();
            if let Some(rep) = report_snapshot {
                Theme::card_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left: Big Status Icon & Title
                        match &rep.state {
                            ServiceState::Running(pids) => {
                                ui.colored_label(Theme::SUCCESS, RichText::new("●").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MySQL EN EJECUCIÓN").strong().color(Theme::SUCCESS));
                                    ui.label(RichText::new(format!("PIDs: {:?} | Escuchando en puerto 3306", pids)).size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::StoppedClean => {
                                ui.colored_label(Theme::MUTED, RichText::new("○").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MySQL DETENIDO").strong().color(Color32::WHITE));
                                    ui.label(RichText::new("Servicio inactivo de forma limpia.").size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::CrashedOrphanLock(pid) => {
                                ui.colored_label(Theme::DANGER, RichText::new("⚠️").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MySQL CAÍDO / BLOQUEADO").strong().color(Theme::DANGER));
                                    ui.label(RichText::new(format!("Bloqueo activo por archivo mysql.pid (PID: {pid})")).size(11.0_f32).color(Theme::MUTED));
                                });
                            }
                            ServiceState::PortConflict(msg) => {
                                ui.colored_label(Theme::WARNING, RichText::new("⚡").size(22.0_f32));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("CONFLICTO DE PUERTO 3306").strong().color(Theme::WARNING));
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
                        Self::render_pill(ui, "🔌 Puerto 3306", if rep.port_open { "Ocupado" } else { "Libre" }, rep.port_open);
                        Self::render_pill(ui, "🗄️ Bases de Datos", &format!("{}", rep.user_databases.len()), true);
                        Self::render_pill(ui, "📦 Backups", &format!("{}", self.backups_list.len()), true);
                        if !rep.aria_logs_found.is_empty() {
                            Self::render_pill(ui, "⚠️ Logs Aria", &format!("{}", rep.aria_logs_found.len()), false);
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
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::AutoRepair, "⚡ Reparación Inteligente");
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::ManualMethods, "🔧 Métodos Manuales");
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::Backups, "💾 Copias de Seguridad");
                Self::render_tab_button(ui, &mut self.active_tab, ActiveTab::ErrorLog, "📜 Registro de Errores");
            });

            ui.add_space(4.0_f32);

            // 4. TAB CONTENTS
            match self.active_tab {
                ActiveTab::AutoRepair => self.render_auto_repair_tab(ui),
                ActiveTab::ManualMethods => self.render_manual_methods_tab(ui),
                ActiveTab::Backups => self.render_backups_tab(ui),
                ActiveTab::ErrorLog => self.render_error_log_tab(ui),
            }
        });

        // Deferred Actions
        if trigger_start {
            if let Some(ref p) = self.xampp_path.clone() {
                let _ = ServiceManager::start_mysql(p);
                std::thread::sleep(std::time::Duration::from_millis(600));
                self.refresh_diagnosis();
                self.add_log("Comando iniciar MySQL ejecutado.");
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
            self.add_log("Diagnóstico actualizado.");
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

        ui.vertical(|ui| {
            // HERO CARD
            Theme::hero_frame().show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🚀").size(22.0_f32));
                        ui.vertical(|ui| {
                            ui.label(RichText::new("Reparación Automática Inteligente").size(15.0_f32).strong().color(Color32::WHITE));
                            ui.label(RichText::new("Resuelve el 95% de los fallos de MySQL en XAMPP preservando tus datos.").size(11.5_f32).color(Theme::MUTED));
                        });
                    });

                    ui.add_space(8.0_f32);

                    // Workflow steps row
                    ui.horizontal_wrapped(|ui| {
                        Self::render_step_badge(ui, "1", "Matar Huérfanos");
                        ui.label(RichText::new("→").color(Theme::MUTED));
                        Self::render_step_badge(ui, "2", "Backup Preventivo");
                        ui.label(RichText::new("→").color(Theme::MUTED));
                        Self::render_step_badge(ui, "3", "Limpiar Logs Aria");
                        ui.label(RichText::new("→").color(Theme::MUTED));
                        Self::render_step_badge(ui, "4", "Test de Arranque");
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
                            self.add_log("Iniciando reparación automática...");
                            match RepairMethods::auto_repair_cascade(p) {
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
                    "Método 1: Liberar Procesos Huérfanos y Puerto 3306",
                    "Riesgo: Cero",
                    Theme::SUCCESS,
                    "Finaliza cualquier proceso mysqld.exe colgado en segundo plano y elimina los archivos bloqueantes mysql.pid e ib_buffer_pool.",
                    "Útil cuando MySQL dice que el puerto 3306 está tomado o se cerró de golpe.",
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
                                    ui.label(RichText::new(format!("📅 {} | ⚖️ {:.2} MB", bkp.created_at, bkp.size_mb)).size(11.0_f32).color(Theme::MUTED));
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
}
