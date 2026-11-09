use crate::engine::backup::{BackupInfo, BackupManager};
use crate::engine::detector::{Detector, DiagnosticReport, ServiceState};
use crate::engine::methods::RepairMethods;
use crate::engine::service::ServiceManager;
use eframe::egui::{self, Color32, Frame, RichText, ScrollArea, Vec2};
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum ActiveTab {
    AutoRepair,
    ManualMethods,
    Backups,
    ErrorLog,
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
    pub status_notification: Option<(String, bool)>, // (message, is_success)
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
                "Bienvenido a XAMPP MySQL Medic & Auto-Fixer.".into(),
                "Iniciando diagnóstico preventivo...".into(),
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
            if let Ok(log) = ServiceManager::read_error_log(path, 80) {
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
        let mut trigger_refresh = false;
        let mut trigger_start = false;
        let mut trigger_stop = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            // HEADER BAR
            ui.horizontal(|ui| {
                ui.heading(RichText::new("🛠️ XAMPP MySQL Medic & Auto-Fixer").strong().size(20.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 Actualizar Estado").clicked() {
                        trigger_refresh = true;
                    }
                    if let Some(ref p) = self.xampp_path {
                        if ui.button("📁 Abrir XAMPP").clicked() {
                            ServiceManager::open_in_explorer(p);
                        }
                    }
                });
            });

            ui.separator();

            // XAMPP PATH SELECTOR
            ui.horizontal(|ui| {
                ui.label(RichText::new("Ruta de XAMPP:").strong());
                let edit = ui.text_edit_singleline(&mut self.custom_path_input);
                if edit.lost_focus() || ui.button("Establecer").clicked() {
                    let p = PathBuf::from(&self.custom_path_input);
                    if Detector::is_valid_xampp(&p) {
                        self.xampp_path = Some(p);
                        trigger_refresh = true;
                        self.status_notification = Some(("Ruta de XAMPP encontrada con éxito.".into(), true));
                    } else {
                        self.status_notification = Some(("La ruta ingresada no parece tener XAMPP MySQL válido.".into(), false));
                    }
                }
            });

            // STATUS PANEL (Cloned report to avoid borrow conflicts)
            let report_snapshot = self.report.clone();
            if let Some(rep) = report_snapshot {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        match &rep.state {
                            ServiceState::Running(pids) => {
                                ui.colored_label(Color32::from_rgb(46, 204, 113), RichText::new("🟢 EN EJECUCIÓN").strong());
                                ui.label(format!("(PIDs: {:?})", pids));
                                ui.colored_label(Color32::from_rgb(46, 204, 113), "• Puerto 3306 Activo");
                            }
                            ServiceState::StoppedClean => {
                                ui.colored_label(Color32::from_rgb(149, 165, 166), RichText::new("⚪ DETENIDO").strong());
                                ui.label("Servicio detenido limpiamente.");
                            }
                            ServiceState::CrashedOrphanLock(pid) => {
                                ui.colored_label(Color32::from_rgb(231, 76, 60), RichText::new("⚠️ CAÍDO / BLOQUEADO").strong());
                                ui.label(format!("Archivo mysql.pid huérfano con PID {pid}."));
                            }
                            ServiceState::PortConflict(msg) => {
                                ui.colored_label(Color32::from_rgb(230, 126, 34), RichText::new("❌ CONFLICTO").strong());
                                ui.label(msg);
                            }
                            ServiceState::XamppNotFound => {
                                ui.colored_label(Color32::from_rgb(231, 76, 60), "XAMPP no detectado");
                            }
                        }

                        // Quick Service Controls
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("⏹ Detener").clicked() {
                                trigger_stop = true;
                            }
                            if ui.button("▶ Iniciar").clicked() {
                                trigger_start = true;
                            }
                        });
                    });

                    // Extra diagnostic pills
                    ui.horizontal_wrapped(|ui| {
                        if !rep.aria_logs_found.is_empty() {
                            ui.colored_label(Color32::from_rgb(241, 196, 15), format!("⚠️ {} logs Aria detectados", rep.aria_logs_found.len()));
                        }
                        if !rep.redo_logs_found.is_empty() {
                            ui.colored_label(Color32::from_rgb(241, 196, 15), format!("⚠️ {} redo logs detectados", rep.redo_logs_found.len()));
                        }
                        if let Some(lvl) = rep.innodb_force_recovery {
                            ui.colored_label(Color32::from_rgb(230, 126, 34), format!("🚨 innodb_force_recovery = {lvl} ACTIVO"));
                        }
                        ui.label(format!("📁 Bases de datos de usuario detectadas: {}", rep.user_databases.len()));
                    });
                });
            } else {
                ui.colored_label(Color32::from_rgb(231, 76, 60), "⚠️ No se detectó instalación de XAMPP en la ruta indicada.");
            }

            // Notification Banner
            if let Some((ref msg, is_ok)) = self.status_notification {
                let color = if is_ok { Color32::from_rgb(46, 204, 113) } else { Color32::from_rgb(231, 76, 60) };
                ui.colored_label(color, RichText::new(msg).strong());
            }

            ui.add_space(6.0);

            // NAVIGATION TABS
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, ActiveTab::AutoRepair, "⚡ Reparación Automática");
                ui.selectable_value(&mut self.active_tab, ActiveTab::ManualMethods, "🔧 Métodos Manuales");
                ui.selectable_value(&mut self.active_tab, ActiveTab::Backups, "💾 Backups & Respaldo");
                ui.selectable_value(&mut self.active_tab, ActiveTab::ErrorLog, "📜 Visor de Logs");
            });

            ui.separator();

            // TAB CONTENTS
            match self.active_tab {
                ActiveTab::AutoRepair => self.render_auto_repair_tab(ui),
                ActiveTab::ManualMethods => self.render_manual_methods_tab(ui),
                ActiveTab::Backups => self.render_backups_tab(ui),
                ActiveTab::ErrorLog => self.render_error_log_tab(ui),
            }
        });

        // Trigger deferred operations
        if trigger_start {
            if let Some(ref p) = self.xampp_path.clone() {
                let _ = ServiceManager::start_mysql(&p);
                std::thread::sleep(std::time::Duration::from_millis(600));
                self.refresh_diagnosis();
                self.add_log("Comando iniciar MySQL ejecutado.");
            }
        }
        if trigger_stop {
            if let Some(ref p) = self.xampp_path.clone() {
                let _ = ServiceManager::stop_mysql(&p);
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
    fn render_auto_repair_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ui.vertical(|ui| {
            ui.label(RichText::new("Reparación Automática en Cascada (Recomendado)").size(15.0).strong());
            ui.label("Ejecuta en secuencia segura todas las etapas para que MySQL vuelva a levantar:");
            ui.label("1. Mata procesos mysqld.exe huérfanos y elimina bloqueos (mysql.pid).");
            ui.label("2. Crea un BACKUP PREVENTIVO TIMESTAMPED completo de tus bases de datos.");
            ui.label("3. Limpia logs corruptos de Aria / MariaDB y Redo logs (causa del 90% de los crashes).");
            ui.label("4. Prueba arranque; si aún falla, restaura los esquemas del sistema protegiendo tus tablas.");

            ui.add_space(8.0);

            let btn_size = Vec2::new(ui.available_width(), 44.0);
            let btn = egui::Button::new(
                RichText::new("🚀 DIAGNOSTICAR Y REPARAR AUTOMÁTICAMENTE AHORA")
                    .size(15.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(41, 128, 185));

            if ui.add_sized(btn_size, btn).clicked() {
                if let Some(ref p) = xampp_path_opt {
                    self.add_log("Iniciando reparación automática...");
                    match RepairMethods::auto_repair_cascade(p) {
                        Ok(msg) => {
                            self.add_log(msg);
                            self.status_notification = Some(("¡Reparación automática completada!".into(), true));
                        }
                        Err(e) => {
                            self.add_log(format!("ERROR: {e}"));
                            self.status_notification = Some((format!("Error en la reparación: {e}"), false));
                        }
                    }
                    self.refresh_diagnosis();
                } else {
                    self.status_notification = Some(("Error: Ruta de XAMPP no configurada.".into(), false));
                }
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("Registro de Acciones:").strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Limpiar Registro").clicked() {
                        self.log_messages.clear();
                    }
                });
            });

            // Live Log Scroll Area
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .max_height(220.0)
                .show(ui, |ui| {
                    Frame::NONE.fill(Color32::from_rgb(25, 25, 30)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for msg in &self.log_messages {
                            ui.label(RichText::new(msg).monospace().size(12.0).color(Color32::from_rgb(220, 220, 230)));
                        }
                    });
                });
        });
    }

    fn render_manual_methods_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                // METHOD 1
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(RichText::new("Método 1: Liberar Procesos Huérfanos y Puerto 3306").strong().size(14.0));
                    ui.label("Mata cualquier instancia trabada de mysqld.exe y elimina los archivos mysql.pid e ib_buffer_pool.");
                    if ui.button("Ejecutar Método 1").clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            match RepairMethods::method_1_kill_orphans(p) {
                                Ok(msg) => {
                                    self.add_log(msg.clone());
                                    self.status_notification = Some((msg, true));
                                }
                                Err(e) => {
                                    self.add_log(format!("Error: {e}"));
                                    self.status_notification = Some((e, false));
                                }
                            }
                            self.refresh_diagnosis();
                        }
                    }
                });

                ui.add_space(6.0);

                // METHOD 2
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(RichText::new("Método 2: Limpieza de Logs Corruptos (Aria & Redo Logs)").strong().size(14.0));
                    ui.label("Elimina aria_log* e ib_logfile* corruptos tras apagados bruscos. MariaDB los regenera limpios. NO toca tablas.");
                    if ui.button("Ejecutar Método 2").clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            match RepairMethods::method_2_clean_transaction_logs(p) {
                                Ok(msg) => {
                                    self.add_log(msg.clone());
                                    self.status_notification = Some((msg, true));
                                }
                                Err(e) => {
                                    self.add_log(format!("Error: {e}"));
                                    self.status_notification = Some((e, false));
                                }
                            }
                            self.refresh_diagnosis();
                        }
                    }
                });

                ui.add_space(6.0);

                // METHOD 3
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(RichText::new("Método 3: Reconstruir Catálogo del Sistema desde backup/").strong().size(14.0));
                    ui.label("Restaura mysql, phpmyadmin y performance_schema. Preserva al 100% tus bases de datos de usuario e ibdata1.");
                    if ui.button("Ejecutar Método 3 (con Backup Preventivo)").clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            match RepairMethods::method_3_restore_system_catalog(p) {
                                Ok(msg) => {
                                    self.add_log(msg.clone());
                                    self.status_notification = Some(("Catálogo restaurado exitosamente.".into(), true));
                                }
                                Err(e) => {
                                    self.add_log(format!("Error: {e}"));
                                    self.status_notification = Some((e, false));
                                }
                            }
                            self.refresh_diagnosis();
                        }
                    }
                });

                ui.add_space(6.0);

                // METHOD 4
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.label(RichText::new("Método 4: Forzar Recuperación InnoDB (innodb_force_recovery)").strong().size(14.0));
                    ui.label("Útil cuando una tabla InnoDB está corrupta. Permite arrancar en modo solo lectura para rescatar datos.");
                    
                    ui.horizontal(|ui| {
                        ui.label("Nivel (1 a 6):");
                        ui.add(egui::Slider::new(&mut self.force_recovery_level, 1..=6));
                        
                        if ui.button("Activar Nivel").clicked() {
                            if let Some(ref p) = xampp_path_opt {
                                let _ = RepairMethods::method_4_set_force_recovery(p, Some(self.force_recovery_level));
                                self.add_log(format!("innodb_force_recovery establecido en {}", self.force_recovery_level));
                                self.refresh_diagnosis();
                            }
                        }

                        if ui.button("Desactivar (Modo Normal)").clicked() {
                            if let Some(ref p) = xampp_path_opt {
                                let _ = RepairMethods::method_4_set_force_recovery(p, None);
                                self.add_log("innodb_force_recovery desactivado.");
                                self.refresh_diagnosis();
                            }
                        }
                    });
                });
            });
        });
    }

    fn render_backups_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Backups y Copias de Seguridad").size(15.0).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("➕ Crear Backup Ahora").clicked() {
                        if let Some(ref p) = xampp_path_opt {
                            match BackupManager::create_backup(p) {
                                Ok(path) => {
                                    self.add_log(format!("Backup creado en: {}", path.display()));
                                    self.status_notification = Some(("Backup creado exitosamente.".into(), true));
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

            ui.separator();

            if self.backups_list.is_empty() {
                ui.label("No se encontraron copias de seguridad en mysql/backups/.");
            } else {
                ScrollArea::vertical().show(ui, |ui| {
                    for bkp in self.backups_list.clone() {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&bkp.name).strong());
                                    ui.label(format!("Fecha: {} | Tamaño: {:.2} MB", bkp.created_at, bkp.size_mb));
                                });

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Restaurar Este").clicked() {
                                        if let Some(ref p) = xampp_path_opt {
                                            match BackupManager::restore_backup(p, &bkp.path) {
                                                Ok(()) => {
                                                    self.add_log(format!("Backup restaurado: {}", bkp.name));
                                                    self.status_notification = Some(("Backup restaurado exitosamente.".into(), true));
                                                }
                                                Err(e) => {
                                                    self.status_notification = Some((format!("Error restaurando: {e}"), false));
                                                }
                                            }
                                            self.refresh_diagnosis();
                                        }
                                    }
                                    if ui.button("Abrir Carpeta").clicked() {
                                        ServiceManager::open_in_explorer(&bkp.path);
                                    }
                                });
                            });
                        });
                        ui.add_space(4.0);
                    }
                });
            }
        });
    }

    fn render_error_log_tab(&mut self, ui: &mut egui::Ui) {
        let xampp_path_opt = self.xampp_path.clone();

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Últimas líneas de mysql_error.log").size(15.0).strong());
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

            ui.separator();

            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    Frame::NONE.fill(Color32::from_rgb(18, 18, 22)).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(
                            RichText::new(&self.error_log_content)
                                .monospace()
                                .size(11.0)
                                .color(Color32::from_rgb(200, 200, 210)),
                        );
                    });
                });
        });
    }
}
