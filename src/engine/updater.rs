use serde::Deserialize;
use std::fs::File;
use std::io::copy;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateInfo {
    pub tag: String,
    pub version: String,
    pub title: String,
    pub notes: String,
    pub download_url: String,
    pub file_size: u64,
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    size: u64,
    browser_download_url: String,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let clean = s.trim().trim_start_matches('v').trim_start_matches('V');
    let mut parts = clean.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next().unwrap_or("0").split('-').next()?.parse().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer(remote: &str, current: &str) -> bool {
    if let (Some(r), Some(c)) = (parse_version(remote), parse_version(current)) {
        r > c
    } else {
        false
    }
}

pub fn check_for_updates() -> Result<Option<UpdateInfo>, String> {
    let url = "https://api.github.com/repos/santy123371/xampp-mysql-fixer/releases/latest";
    let resp: GitHubRelease = ureq::get(url)
        .set("User-Agent", "XAMPP-FIXER-App")
        .call()
        .map_err(|e| format!("Error de conexión al buscar actualizaciones: {}", e))?
        .into_json()
        .map_err(|e| format!("Error al procesar respuesta de GitHub: {}", e))?;

    let remote_ver = resp.tag_name.trim();
    let current_ver = current_version();

    if is_newer(remote_ver, current_ver) {
        // Find the Windows executable asset
        let asset = resp
            .assets
            .into_iter()
            .find(|a| a.name.ends_with(".exe"))
            .ok_or_else(|| "La nueva versión no contiene un binario ejecutable para Windows.".to_string())?;

        Ok(Some(UpdateInfo {
            tag: resp.tag_name.clone(),
            version: remote_ver.trim_start_matches('v').to_string(),
            title: resp.name.unwrap_or_else(|| resp.tag_name.clone()),
            notes: resp.body.unwrap_or_default(),
            download_url: asset.browser_download_url,
            file_size: asset.size,
        }))
    } else {
        Ok(None)
    }
}

pub fn cleanup_old_executable() {
    if let Ok(exe) = std::env::current_exe() {
        let old_exe = exe.with_extension("exe.old");
        if old_exe.exists() {
            let _ = std::fs::remove_file(old_exe);
        }
    }
}

pub fn apply_update(download_url: &str) -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| format!("No se pudo determinar la ruta del ejecutable actual: {}", e))?;
    let temp_exe = current_exe.with_extension("exe.new");
    let old_exe = current_exe.with_extension("exe.old");

    // 1. Download new binary to temporary file
    let resp = ureq::get(download_url)
        .set("User-Agent", "XAMPP-FIXER-App")
        .call()
        .map_err(|e| format!("Error al descargar el ejecutable: {}", e))?;

    let mut reader = resp.into_reader();
    let mut out_file = File::create(&temp_exe)
        .map_err(|e| format!("Error al crear archivo temporal: {}", e))?;

    copy(&mut reader, &mut out_file)
        .map_err(|e| format!("Error al guardar nueva versión: {}", e))?;
    drop(out_file);

    // 2. Remove any previous .old file
    if old_exe.exists() {
        let _ = std::fs::remove_file(&old_exe);
    }

    // 3. Rename current running .exe to .old (permitted on Windows NTFS)
    std::fs::rename(&current_exe, &old_exe)
        .map_err(|e| format!("No se pudo renombrar el binario actual: {}", e))?;

    // 4. Place new binary in place
    if let Err(e) = std::fs::rename(&temp_exe, &current_exe) {
        // Rollback if something failed
        let _ = std::fs::rename(&old_exe, &current_exe);
        return Err(format!("No se pudo posicionar el nuevo ejecutable: {}", e));
    }

    // 5. Spawn new process
    Command::new(&current_exe)
        .spawn()
        .map_err(|e| format!("Error al reiniciar la aplicación actualizada: {}", e))?;

    // 6. Terminate this process
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(parse_version("1.0.0"), Some((1, 0, 0)));
        assert_eq!(parse_version("v1.0.1"), Some((1, 0, 1)));
        assert_eq!(parse_version("V2.3.4-beta"), Some((2, 3, 4)));
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("v1.0.1", "1.0.0"));
        assert!(is_newer("v1.1.0", "1.0.5"));
        assert!(is_newer("2.0.0", "1.9.9"));
        assert!(!is_newer("1.0.0", "1.0.0"));
        assert!(!is_newer("v1.0.0", "1.0.0"));
        assert!(!is_newer("0.9.9", "1.0.0"));
    }
}
