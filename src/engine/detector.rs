use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::Duration;
use sysinfo::{ProcessesToUpdate, System};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ServiceState {
    Running(Vec<u32>),      // Running with PIDs
    StoppedClean,           // Not running, no orphan locks
    CrashedOrphanLock(u32), // Not running, but mysql.pid exists with recorded PID
    PortConflict(String),   // Port 3306 taken by another process
    XamppNotFound,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiagnosticReport {
    pub xampp_path: PathBuf,
    pub mysql_bin: PathBuf,
    pub data_dir: PathBuf,
    pub backup_dir: PathBuf,
    pub state: ServiceState,
    pub port_open: bool,
    pub pid_file_exists: bool,
    pub pid_file_value: Option<u32>,
    pub aria_logs_found: Vec<String>,
    pub redo_logs_found: Vec<String>,
    pub user_databases: Vec<String>,
    pub innodb_force_recovery: Option<u32>,
}

pub struct Detector;

impl Detector {
    pub const DEFAULT_CANDIDATES: &'static [&'static str] = &["C:\\xampp", "D:\\xampp", "E:\\xampp"];

    pub fn find_xampp(custom_path: Option<&str>) -> Option<PathBuf> {
        if let Some(cp) = custom_path {
            let p = PathBuf::from(cp);
            if Self::is_valid_xampp(&p) {
                return Some(p);
            }
        }
        for cand in Self::DEFAULT_CANDIDATES {
            let p = PathBuf::from(cand);
            if Self::is_valid_xampp(&p) {
                return Some(p);
            }
        }
        None
    }

    pub fn is_valid_xampp(path: &Path) -> bool {
        path.join("mysql").join("bin").join("mysqld.exe").exists()
            && path.join("mysql").join("data").exists()
    }

    pub fn diagnose(xampp_path: &Path) -> DiagnosticReport {
        let mysql_bin = xampp_path.join("mysql").join("bin").join("mysqld.exe");
        let data_dir = xampp_path.join("mysql").join("data");
        let backup_dir = xampp_path.join("mysql").join("backup");
        let my_ini = xampp_path.join("mysql").join("bin").join("my.ini");

        // 1. Check running processes
        let mut sys = System::new_all();
        sys.refresh_processes(ProcessesToUpdate::All, true);

        let running_pids: Vec<u32> = sys
            .processes()
            .iter()
            .filter(|(_, proc)| {
                let name = proc.name().to_string_lossy().to_lowercase();
                name == "mysqld.exe" || name == "mysqld"
            })
            .map(|(pid, _)| pid.as_u32())
            .collect();

        // 2. Check port 3306
        let port_open = Self::check_port(3306);

        // 3. Check mysql.pid
        let pid_file = data_dir.join("mysql.pid");
        let pid_file_exists = pid_file.exists();
        let pid_file_value = if pid_file_exists {
            std::fs::read_to_string(&pid_file)
                .ok()
                .and_then(|s| s.trim().parse::<u32>().ok())
        } else {
            None
        };

        // 4. Check Aria & Redo logs
        let mut aria_logs_found = Vec::new();
        let mut redo_logs_found = Vec::new();
        let mut user_databases = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&data_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                if path.is_file() {
                    if name.starts_with("aria_log") {
                        aria_logs_found.push(name);
                    } else if name.starts_with("ib_logfile") {
                        redo_logs_found.push(name);
                    }
                } else if path.is_dir() {
                    let sys_schemas = ["mysql", "performance_schema", "phpmyadmin", "test"];
                    if !sys_schemas.contains(&name.to_lowercase().as_str()) {
                        user_databases.push(name);
                    }
                }
            }
        }
        user_databases.sort();

        // 5. Check innodb_force_recovery in my.ini
        let innodb_force_recovery = Self::detect_force_recovery(&my_ini);

        // 6. Determine service state
        let state = if !running_pids.is_empty() {
            ServiceState::Running(running_pids)
        } else if port_open {
            ServiceState::PortConflict("Puerto 3306 ocupado por otro servicio MySQL/MariaDB".into())
        } else if let Some(pid) = pid_file_value {
            ServiceState::CrashedOrphanLock(pid)
        } else {
            ServiceState::StoppedClean
        };

        DiagnosticReport {
            xampp_path: xampp_path.to_path_buf(),
            mysql_bin,
            data_dir,
            backup_dir,
            state,
            port_open,
            pid_file_exists,
            pid_file_value,
            aria_logs_found,
            redo_logs_found,
            user_databases,
            innodb_force_recovery,
        }
    }

    pub fn check_port(port: u16) -> bool {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
    }

    pub fn detect_force_recovery(my_ini: &Path) -> Option<u32> {
        let content = std::fs::read_to_string(my_ini).ok()?;
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('#') && trimmed.contains("innodb_force_recovery") {
                if let Some(val_str) = trimmed.split('=').nth(1) {
                    if let Ok(v) = val_str.trim().parse::<u32>() {
                        return Some(v);
                    }
                }
            }
        }
        None
    }
}
