use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

pub struct ServiceManager;

impl ServiceManager {
    /// Inicia el servicio MySQL usando mysql_start.bat o mysqld.exe
    pub fn start_mysql(xampp_path: &Path) -> Result<(), String> {
        let start_bat = xampp_path.join("mysql_start.bat");
        if start_bat.exists() {
            Command::new("cmd")
                .args(["/C", "start", "/B", &start_bat.to_string_lossy()])
                .current_dir(xampp_path)
                .spawn()
                .map_err(|e| format!("Error ejecutando mysql_start.bat: {e}"))?;
            Ok(())
        } else {
            let mysqld = xampp_path.join("mysql").join("bin").join("mysqld.exe");
            let my_ini = xampp_path.join("mysql").join("bin").join("my.ini");
            Command::new(&mysqld)
                .arg(format!("--defaults-file={}", my_ini.display()))
                .arg("--standalone")
                .spawn()
                .map_err(|e| format!("Error ejecutando mysqld.exe: {e}"))?;
            Ok(())
        }
    }

    /// Detiene el servicio MySQL
    pub fn stop_mysql(xampp_path: &Path) -> Result<(), String> {
        let stop_bat = xampp_path.join("mysql_stop.bat");
        if stop_bat.exists() {
            let _ = Command::new("cmd")
                .args(["/C", &stop_bat.to_string_lossy()])
                .current_dir(xampp_path)
                .output();
        }
        
        // Safety: ensure any remaining process is cleaned
        let _ = crate::engine::methods::RepairMethods::method_1_kill_orphans(xampp_path);
        Ok(())
    }

    /// Lee las últimas `max_lines` líneas del archivo mysql_error.log
    pub fn read_error_log(xampp_path: &Path, max_lines: usize) -> Result<String, String> {
        let log_file = xampp_path.join("mysql").join("data").join("mysql_error.log");
        if !log_file.exists() {
            return Err("El archivo mysql_error.log no existe aún en data/".into());
        }

        let file = File::open(&log_file).map_err(|e| format!("Error abriendo log: {e}"))?;
        let reader = BufReader::new(file);

        let mut lines = Vec::new();
        for line in reader.lines() {
            if let Ok(l) = line {
                lines.push(l);
            }
        }

        let start = if lines.len() > max_lines {
            lines.len() - max_lines
        } else {
            0
        };

        Ok(lines[start..].join("\n"))
    }

    /// Abre una carpeta en el explorador de Windows
    pub fn open_in_explorer(path: &Path) {
        let _ = Command::new("explorer").arg(path).spawn();
    }
}
