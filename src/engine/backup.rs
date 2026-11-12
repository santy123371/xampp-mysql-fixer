use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub name: String,
    pub path: PathBuf,
    pub created_at: String,
    pub size_mb: f64,
}

pub struct BackupManager;

impl BackupManager {
    pub fn create_backup(xampp_path: &Path) -> Result<PathBuf, String> {
        let data_dir = xampp_path.join("mysql").join("data");
        if !data_dir.exists() {
            return Err("El directorio data no existe".into());
        }

        let backups_dir = xampp_path.join("mysql").join("backups");
        fs::create_dir_all(&backups_dir).map_err(|e| format!("Error creando carpeta backups: {e}"))?;

        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let target_dir = backups_dir.join(format!("data_backup_{timestamp}"));

        Self::copy_dir_recursive(&data_dir, &target_dir)
            .map_err(|e| format!("Error copiando archivos para el backup: {e}"))?;

        Ok(target_dir)
    }

    pub fn list_backups(xampp_path: &Path) -> Vec<BackupInfo> {
        let mut result = Vec::new();

        // 1. Check mysql/backups/
        let backups_dir = xampp_path.join("mysql").join("backups");
        if let Ok(entries) = fs::read_dir(&backups_dir) {
            for entry in entries.flatten() {
                if let Ok(info) = Self::inspect_backup_entry(&entry.path()) {
                    result.push(info);
                }
            }
        }

        // 2. Check mysql/data_backup_* (carpetas previas en la raíz de mysql)
        let mysql_dir = xampp_path.join("mysql");
        if let Ok(entries) = fs::read_dir(&mysql_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if p.is_dir() && name.starts_with("data_backup_") {
                    if let Ok(info) = Self::inspect_backup_entry(&p) {
                        result.push(info);
                    }
                }
            }
        }

        // Sort newest first
        result.sort_by(|a, b| b.name.cmp(&a.name));
        result
    }

    fn inspect_backup_entry(path: &Path) -> Result<BackupInfo, ()> {
        if !path.is_dir() {
            return Err(());
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        let metadata = fs::metadata(path).map_err(|_| ())?;
        
        let created_at = if let Ok(time) = metadata.modified() {
            let dt: chrono::DateTime<Local> = time.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        } else {
            "Desconocido".into()
        };

        let size_bytes = Self::calc_dir_size(path);
        let size_mb = (size_bytes as f64) / (1024.0 * 1024.0);

        Ok(BackupInfo {
            name,
            path: path.to_path_buf(),
            created_at,
            size_mb,
        })
    }

    pub fn restore_backup(xampp_path: &Path, backup_path: &Path) -> Result<(), String> {
        let data_dir = xampp_path.join("mysql").join("data");

        // 1. Safe guard: move current data to safety first
        if data_dir.exists() {
            let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
            let safety_dir = xampp_path
                .join("mysql")
                .join("backups")
                .join(format!("data_before_restore_{timestamp}"));
            
            let _ = fs::create_dir_all(safety_dir.parent().unwrap());
            fs::rename(&data_dir, &safety_dir)
                .map_err(|e| format!("No se pudo mover el directorio data actual: {e}"))?;
        }

        // 2. Copy backup into data
        Self::copy_dir_recursive(backup_path, &data_dir)
            .map_err(|e| format!("Error restaurando backup a data: {e}"))?;

        Ok(())
    }

    pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dst.join(entry.file_name());
            if path.is_dir() {
                Self::copy_dir_recursive(&path, &dest_path)?;
            } else {
                if dest_path.exists() {
                    crate::engine::methods::make_writable(&dest_path);
                }
                fs::copy(&path, &dest_path)?;
            }
        }
        Ok(())
    }

    pub fn calc_dir_size(path: &Path) -> u64 {
        let mut total = 0;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    total += Self::calc_dir_size(&p);
                } else if let Ok(meta) = p.metadata() {
                    total += meta.len();
                }
            }
        }
        total
    }
}
