use crate::engine::backup::BackupManager;
use crate::engine::detector::Detector;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};

pub fn make_writable(path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }
}

pub fn remove_file_force(path: &Path) -> Result<(), std::io::Error> {
    make_writable(path);
    fs::remove_file(path)
}

pub fn remove_dir_all_force(dir: &Path) -> Result<(), std::io::Error> {
    if !dir.exists() {
        return Ok(());
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = remove_dir_all_force(&path);
            } else {
                let _ = remove_file_force(&path);
            }
        }
    }
    make_writable(dir);
    fs::remove_dir_all(dir)
}

pub struct RepairMethods;

impl RepairMethods {
    /// Método 1: Matar procesos mysqld.exe huérfanos y borrar mysql.pid, ibtmp1 y locks
    pub fn method_1_kill_orphans(xampp_path: &Path) -> Result<String, String> {
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let mut killed_pids = Vec::new();
        for (pid, proc) in sys.processes() {
            let name = proc.name().to_string_lossy().to_lowercase();
            if name == "mysqld.exe" || name == "mysqld" {
                let pid_u32 = pid.as_u32();
                // Kill process tree via taskkill for guaranteed termination on Windows
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid_u32.to_string()])
                    .output();
                killed_pids.push(pid_u32);
            }
        }

        let data_dir = xampp_path.join("mysql").join("data");
        let stale_locks = ["mysql.pid", "ib_buffer_pool", "ibtmp1", "multi-master.info"];
        let mut removed_locks = Vec::new();

        for file_name in &stale_locks {
            let file_path = data_dir.join(file_name);
            if file_path.exists() && remove_file_force(&file_path).is_ok() {
                removed_locks.push(*file_name);
            }
        }

        let mut msg = String::new();
        if !killed_pids.is_empty() {
            msg.push_str(&format!("Procesos mysqld.exe terminados: {:?}. ", killed_pids));
        } else {
            msg.push_str("No había procesos mysqld.exe colgados. ");
        }

        if !removed_locks.is_empty() {
            msg.push_str(&format!("Archivos de bloqueo y temporales purgados: {:?}.", removed_locks));
        } else {
            msg.push_str("Sin archivos residuales de bloqueo.");
        }

        Ok(msg)
    }

    /// Método 2: Limpieza de logs de transacción corruptos (Aria logs, aria_log_control & InnoDB redo logs)
    /// Este método NO toca tablas ni bases de datos de usuario.
    pub fn method_2_clean_transaction_logs(xampp_path: &Path) -> Result<String, String> {
        // Asegurar que no hay procesos corriendo
        let _ = Self::method_1_kill_orphans(xampp_path);

        let data_dir = xampp_path.join("mysql").join("data");
        if !data_dir.exists() {
            return Err("El directorio data no existe".into());
        }

        let mut removed_files = Vec::new();
        if let Ok(entries) = fs::read_dir(&data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if path.is_file() {
                    if name.starts_with("aria_log")
                        || name == "aria_log_control"
                        || name.starts_with("ib_logfile")
                        || name == "ibtmp1"
                    {
                        if remove_file_force(&path).is_ok() {
                            removed_files.push(name);
                        }
                    }
                }
            }
        }

        if removed_files.is_empty() {
            Ok("No se encontraron logs de Aria ni Redo logs antiguos para limpiar.".into())
        } else {
            Ok(format!(
                "Se eliminaron {} logs corruptos y temporales con éxito: {:?}. MariaDB los regenerará limpios al iniciar.",
                removed_files.len(),
                removed_files
            ))
        }
    }

    /// Método 3: Reconstrucción segura del catálogo del sistema desde backup/
    /// Preserva al 100% las carpetas de bases de datos del usuario e ibdata1.
    pub fn method_3_restore_system_catalog(xampp_path: &Path) -> Result<String, String> {
        let _ = Self::method_1_kill_orphans(xampp_path);

        let data_dir = xampp_path.join("mysql").join("data");
        let backup_dir = xampp_path.join("mysql").join("backup");

        if !backup_dir.exists() {
            return Err("No se encontró la carpeta original mysql/backup de XAMPP".into());
        }

        // 1. Crear backup de seguridad preventivo
        let backup_path = BackupManager::create_backup(xampp_path)?;

        // 2. Esquemas del sistema a restaurar
        let sys_schemas = ["mysql", "performance_schema", "phpmyadmin", "test"];
        let mut restored_schemas = Vec::new();

        for schema in &sys_schemas {
            let src = backup_dir.join(schema);
            let dst = data_dir.join(schema);
            if src.exists() {
                // Eliminar copia rota previa en data con remoción forzada
                if dst.exists() {
                    let _ = remove_dir_all_force(&dst);
                }
                // Copiar la limpia de backup
                if BackupManager::copy_dir_recursive(&src, &dst).is_ok() {
                    restored_schemas.push(*schema);
                }
            }
        }

        // 3. Verificar ibdata1: si está dañado o no existe en data, recién ahí copiarlo
        let ibdata1_data = data_dir.join("ibdata1");
        let ibdata1_backup = backup_dir.join("ibdata1");
        let mut ibdata1_status = "ibdata1 original de usuario preservado";
        if !ibdata1_data.exists() || fs::metadata(&ibdata1_data).map(|m| m.len()).unwrap_or(0) == 0 {
            if ibdata1_backup.exists() {
                make_writable(&ibdata1_data);
                let _ = fs::copy(&ibdata1_backup, &ibdata1_data);
                ibdata1_status = "ibdata1 no existía o estaba vacío y fue restaurado desde backup";
            }
        }

        // 4. Limpiar logs de Aria y Redo logs para arranque limpio
        let _ = Self::method_2_clean_transaction_logs(xampp_path);

        Ok(format!(
            "Restauración de catálogo completada.\n- Backup preventivo guardado en: {}\n- Esquemas de sistema restaurados: {:?}\n- Estado ibdata1: {}\n- Tus bases de datos personales se mantuvieron intactas.",
            backup_path.display(),
            restored_schemas,
            ibdata1_status
        ))
    }

    /// Método 4: Configurar innodb_force_recovery en my.ini
    pub fn method_4_set_force_recovery(xampp_path: &Path, level: Option<u32>) -> Result<String, String> {
        let my_ini = xampp_path.join("mysql").join("bin").join("my.ini");
        if !my_ini.exists() {
            return Err("No se encontró el archivo my.ini".into());
        }

        make_writable(&my_ini);
        let content = fs::read_to_string(&my_ini)
            .map_err(|e| format!("Error leyendo my.ini: {e}"))?;

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Remove existing innodb_force_recovery lines
        lines.retain(|line| {
            let trimmed = line.trim();
            !trimmed.contains("innodb_force_recovery")
        });

        if let Some(lvl) = level {
            // Insert under [mysqld]
            let mut insert_idx = None;
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed == "[mysqld]" {
                    insert_idx = Some(i + 1);
                    break;
                }
            }

            let entry = format!("innodb_force_recovery = {lvl}");
            if let Some(idx) = insert_idx {
                lines.insert(idx, entry);
            } else {
                return Err("No se encontró la sección [mysqld] en my.ini".into());
            }
        }

        let new_content = lines.join("\r\n");
        fs::write(&my_ini, new_content)
            .map_err(|e| format!("Error escribiendo my.ini: {e}"))?;

        match level {
            Some(lvl) => Ok(format!("innodb_force_recovery configurado en nivel {lvl}. Reiniciá MySQL.")),
            None => Ok("innodb_force_recovery desactivado (modo normal restablecido).".into()),
        }
    }

    /// Diagnóstico y Reparación Automática (1-Clic Cascada)
    pub fn auto_repair_cascade(xampp_path: &Path, port: u16) -> Result<String, String> {
        let mut steps_log = Vec::new();
        steps_log.push("--- INICIANDO REPARACIÓN AUTOMÁTICA ---".into());

        // Paso 1: Matar huérfanos y liberar locks
        match Self::method_1_kill_orphans(xampp_path) {
            Ok(msg) => steps_log.push(format!("[Paso 1] Liberar procesos: {msg}")),
            Err(e) => steps_log.push(format!("[Paso 1 - Error] {e}")),
        }

        // Paso 2: Backup preventivo de seguridad
        match BackupManager::create_backup(xampp_path) {
            Ok(path) => steps_log.push(format!("[Paso 2] Backup preventivo creado en: {}", path.display())),
            Err(e) => return Err(format!("Abortado por seguridad: No se pudo crear el backup preventivo ({e})")),
        }

        // Paso 3: Limpiar Aria logs, control y Redo logs
        match Self::method_2_clean_transaction_logs(xampp_path) {
            Ok(msg) => steps_log.push(format!("[Paso 3] Limpieza de logs y temporales: {msg}")),
            Err(e) => steps_log.push(format!("[Paso 3 - Error] {e}")),
        }

        // Paso 4: Test de arranque en el puerto configurado
        steps_log.push(format!("[Paso 4] Verificando si MySQL inicia en puerto {port}..."));
        let test_started = Self::test_start_mysql(xampp_path, port);

        if test_started {
            steps_log.push(format!("¡ÉXITO TOTAL! MySQL inició correctamente en puerto {port} tras limpiar logs corruptos."));
            let _ = Self::method_1_kill_orphans(xampp_path);
            return Ok(steps_log.join("\n"));
        }

        // Paso 5: Reconstrucción de catálogo desde backup/
        steps_log.push("[Paso 5] MySQL aún no levanta. Aplicando reconstrucción segura de catálogo del sistema...".into());
        match Self::method_3_restore_system_catalog(xampp_path) {
            Ok(msg) => steps_log.push(format!("[Paso 5] Catálogo restaurado: {msg}")),
            Err(e) => steps_log.push(format!("[Paso 5 - Error] {e}")),
        }

        // Paso 6: Test de arranque tras Paso 5
        steps_log.push(format!("[Paso 6] Verificando arranque tras reconstrucción de catálogo en puerto {port}..."));
        let test_started_2 = Self::test_start_mysql(xampp_path, port);

        if test_started_2 {
            steps_log.push(format!("¡ÉXITO TOTAL! MySQL levantó perfectamente en puerto {port} tras reparar los esquemas del sistema."));
            let _ = Self::method_1_kill_orphans(xampp_path);
            return Ok(steps_log.join("\n"));
        }

        // Paso 7: Reconciliación automática de secuencia InnoDB (desincronización de LSN)
        steps_log.push("[Paso 7] MySQL requiere reconciliación de secuencia InnoDB. Aplicando recuperación asistida...".into());
        let _ = Self::method_2_clean_transaction_logs(xampp_path);
        if Self::method_4_set_force_recovery(xampp_path, Some(1)).is_ok() {
            let recovery_started = Self::test_start_mysql(xampp_path, port);
            if recovery_started {
                steps_log.push(format!("[Paso 7] InnoDB reconcilió su secuencia en puerto {port}. Restableciendo modo normal..."));
                let _ = Self::method_4_set_force_recovery(xampp_path, None);
                let test_normal = Self::test_start_mysql(xampp_path, port);
                if test_normal {
                    steps_log.push(format!("¡ÉXITO TOTAL! MySQL inició en modo normal en puerto {port} tras reconciliar InnoDB."));
                    let _ = Self::method_1_kill_orphans(xampp_path);
                    return Ok(steps_log.join("\n"));
                } else {
                    let _ = Self::method_4_set_force_recovery(xampp_path, Some(1));
                    steps_log.push("MySQL quedó listo con innodb_force_recovery=1 para permitir exportación y rescate de tablas.".into());
                    return Ok(steps_log.join("\n"));
                }
            } else {
                let _ = Self::method_4_set_force_recovery(xampp_path, None);
            }
        }

        steps_log.push("MySQL aún no inicia. Revisá en el Visor de Logs si hay errores de tablas o librerías faltantes.".into());
        steps_log.push("Sugerencia: Podés probar el Método 4 en nivel 2 a 6 para corrupción severa de datos.".into());
        Ok(steps_log.join("\n"))
    }

    pub fn test_start_mysql(xampp_path: &Path, port: u16) -> bool {
        let mysqld = xampp_path.join("mysql").join("bin").join("mysqld.exe");
        let my_ini = xampp_path.join("mysql").join("bin").join("my.ini");
        if !mysqld.exists() {
            return false;
        }

        // Launch mysqld in background
        let mut child = match Command::new(&mysqld)
            .arg(format!("--defaults-file={}", my_ini.display()))
            .arg("--standalone")
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        // Poll configured port for up to 8.4 seconds (24 x 350ms) to accommodate slow disk I/O and recovery
        let mut started = false;
        for _ in 0..24 {
            std::thread::sleep(Duration::from_millis(350));
            if Detector::check_port(port) {
                started = true;
                break;
            }
            if let Ok(Some(_)) = child.try_wait() {
                break;
            }
        }

        // If MySQL started successfully, perform clean graceful shutdown via mysqladmin
        // so InnoDB flushes checkpoints and prevents LSN mismatch corruption.
        let mysqladmin = xampp_path.join("mysql").join("bin").join("mysqladmin.exe");
        if started && mysqladmin.exists() {
            let _ = Command::new(&mysqladmin)
                .args(["-u", "root", "-P", &port.to_string(), "-h", "127.0.0.1", "shutdown"])
                .output();

            for _ in 0..10 {
                std::thread::sleep(Duration::from_millis(250));
                if let Ok(Some(_)) = child.try_wait() {
                    return true;
                }
            }
        }

        let _ = child.kill();
        let _ = child.wait();
        started
    }
}
