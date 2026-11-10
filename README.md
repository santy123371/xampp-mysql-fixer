# XAMPP FIXER

<p align="center">
  <img src="icon_square.png" alt="XAMPP FIXER Logo" width="128" height="128">
</p>

<p align="center">
  <b>Herramienta nativa en Rust con GUI para diagnosticar y reparar automáticamente caídas de MySQL/MariaDB en XAMPP sin perder bases de datos.</b><br>
  <i>Fast, safe desktop GUI utility built in Rust to diagnose and auto-repair XAMPP MySQL/MariaDB crashes without losing database tables.</i>
</p>

<p align="center">
  <a href="#-versión-en-español"><b>🇪🇸 Versión en Español</b></a> • 
  <a href="#-english-version"><b>🇬🇧 English Version</b></a>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/GUI-egui%20%2F%20eframe-blue.svg" alt="egui">
  <img src="https://img.shields.io/badge/Platform-Windows-0078D6.svg" alt="Windows">
  <img src="https://img.shields.io/badge/Author-santy123371-crimson.svg" alt="Author">
</p>

<p align="center">
  <a href="https://github.com/santy123371/xampp-mysql-fixer/releases/latest">
    <img src="https://img.shields.io/badge/📥%20Descargar%20%2F%20Download-XAMPP__FIXER.exe-success?style=for-the-badge&logo=windows" alt="Download Executable">
  </a>
</p>

---

## 🇪🇸 Versión en Español

### ⚡ El Problema
Cuando MariaDB/MySQL en XAMPP se apaga de forma inesperada (cortes de luz, reinicios forzados, procesos huérfanos o corrupción en los archivos de log), el Panel de Control de XAMPP no puede volver a iniciarlo, arrojando el clásico error:

> *"MySQL shutdown unexpectedly. This may be due to a blocked port, missing dependencies, improper privileges, a crash, or a shutdown of another method."*

La solución común en foros suele ser borrar la carpeta `data` o sobreescribir con `backup`, lo cual **destruye todas tus bases de datos, tablas y datos de desarrollo**.

**XAMPP FIXER** resuelve este problema de raíz y de forma segura.

### 🛡️ Características Principales
- **Garantía Cero Pérdida de Datos**: Crea un backup de seguridad completo y fechado en `mysql/backups/data_backup_YYYYMMDD_HHMMSS` antes de realizar cualquier cambio.
- **Detección Automática**:
  - Encuentra la ruta de instalación de XAMPP (`C:\xampp`).
  - Detecta el puerto configurado en `my.ini` (soporta puertos estándar como `3306` o personalizados como `3307`).
- **Diagnóstico en Vivo de Constantes Vitales**:
  - Verifica si el puerto está ocupado o libre mediante socket TCP.
  - Detecta procesos huérfanos `mysqld.exe` y sus PIDs.
  - Detecta archivos de bloqueo residuales (`mysql.pid`).
  - Identifica logs sucios o corruptos (`aria_log*`, `ib_logfile*`).
- **Auto-Reparación en Cascada (1-Click)**: Ejecuta una secuencia progresiva de diagnóstico y reparación automática hasta que el servicio arranca con éxito.
- **4 Métodos Manuales Quirúrgicos**:
  1. **Matar Huérfanos y Liberar Bloqueos**: Cierra procesos colgados y elimina `mysql.pid` e `ib_buffer_pool`.
  2. **Resetear Logs Sucios**: Purga secuencias incompletas de `aria_log*` e `ib_logfile*` manteniendo intacto `ibdata1` y las tablas de usuario.
  3. **Restaurar Catálogos del Sistema**: Reconstruye únicamente las bases de datos internas del motor (`mysql`, `performance_schema`, `phpmyadmin`) a partir de la plantilla limpia de XAMPP sin tocar tus bases de datos.
  4. **Modo Rescate InnoDB**: Configura `innodb_force_recovery` (niveles 1 a 6) en `my.ini` para rescatar tablas corruptas.
- **Visor de Logs de Error**: Visualiza en tiempo real las últimas 100 líneas de `mysql_error.log`.
- **Gestor de Backups**: Lista los backups creados y permite restaurar cualquier estado anterior con 1 click.
- **Actualizaciones Automáticas**: Detecta nuevas versiones publicadas en GitHub y se actualiza automáticamente.

### 🚀 Compilación desde el Código Fuente
```powershell
# Clonar el repositorio
git clone https://github.com/santy123371/xampp-mysql-fixer.git
cd xampp-mysql-fixer

# Compilar en modo Release
cargo build --release
```
El ejecutable optimizado se generará en `target/release/XAMPP_FIXER.exe`.

---

## 🇬🇧 English Version

### ⚡ The Problem
When MariaDB/MySQL in XAMPP crashes unexpectedly (reboots, power outages, hung background processes, or log sequence corruption), XAMPP's Control Panel fails to restart, showing:

> *"MySQL shutdown unexpectedly. This may be due to a blocked port, missing dependencies, improper privileges, a crash, or a shutdown of another method."*

Most tutorials advice deleting the `data` directory or overwriting it with `backup`, which **destroys all user database tables and schemas**.

**XAMPP FIXER** fixes this safely without data loss.

### 🛡️ Key Features
- **Zero Data Loss Guarantee**: Instant timestamped backup to `mysql/backups/data_backup_YYYYMMDD_HHMMSS` before any mutation.
- **Dynamic Detection**: Auto-detects XAMPP path (`C:\xampp`) and active configured port in `my.ini` (`3306`, `3307`, etc.).
- **Real-Time Vital Signs**:
  - Live TCP socket probe on target port.
  - Identification of orphan `mysqld.exe` processes and PIDs.
  - Stale `mysql.pid` lock file detection.
  - Dirty Aria & InnoDB redo log corruption detection.
- **1-Click Cascading Auto-Repair**: Step-by-step diagnostic sequence that repairs and verifies connectivity until MySQL starts successfully.
- **4 Surgical Manual Methods**:
  1. **Kill Zombies & Clear Locks**: Terminates orphan processes and deletes stale locks.
  2. **Reset Dirty Logs**: Clears broken `aria_log*` and `ib_logfile*` sequences without touching `ibdata1` or database folders.
  3. **Restore System Catalogs**: Rebuilds internal schemas (`mysql`, `performance_schema`, `phpmyadmin`) preserving user databases.
  4. **InnoDB Force Recovery**: Configures `innodb_force_recovery = 1..6` for rescue operations.
- **Live Error Log Viewer**: Tail of the last 100 lines of `mysql_error.log`.
- **Backup Manager**: View and restore previous backups with a single click.
- **Auto-Updates**: Checks for new GitHub releases and updates itself seamlessly.

### 🚀 Building from Source
```powershell
# Clone repository
git clone https://github.com/santy123371/xampp-mysql-fixer.git
cd xampp-mysql-fixer

# Compile in release mode
cargo build --release
```
Standalone executable produced at `target/release/XAMPP_FIXER.exe`.

---

## 💻 Tech Stack
- **Language**: [Rust](https://www.rust-lang.org/)
- **GUI**: [egui / eframe 0.31](https://github.com/emilk/egui) (immediate-mode GPU accelerated)
- **Process Management**: `sysinfo`
- **Windows Integration**: `winres` (native PE metadata & multi-resolution icons)

## 👤 Author
- **santy123371** (Santiago Diaz) — GitHub: [@santy123371](https://github.com/santy123371)

## ⚖️ License
Distributed under the MIT License. See [LICENSE](LICENSE) for details.
