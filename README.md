# XAMPP FIXER

<p align="center">
  <img src="icon_square.png" alt="XAMPP FIXER Logo" width="128" height="128">
</p>

<p align="center">
  <b>Herramienta nativa en Rust con interfaz gráfica para diagnosticar y reparar automáticamente caídas de MySQL/MariaDB en XAMPP sin pérdida de datos.</b><br>
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
    <img src="https://img.shields.io/github/v/release/santy123371/xampp-mysql-fixer?label=Descargar%20%2F%20Download%20EXE&style=for-the-badge&logo=windows&color=success" alt="Download Executable">
  </a>
</p>

---

## 🇪🇸 Versión en Español

### ⚡ El Problema
Cuando MariaDB/MySQL en XAMPP se detiene de forma inesperada (cortes de energía, reinicios forzados del sistema, procesos huérfanos o corrupción en las secuencias de logs), el Panel de Control de XAMPP no logra iniciarlo nuevamente, generando el mensaje de error habitual:

> *"MySQL shutdown unexpectedly. This may be due to a blocked port, missing dependencies, improper privileges, a crash, or a shutdown of another method."*

La solución más difundida en foros consiste en eliminar el directorio `data` o reemplazarlo con la carpeta `backup`, lo cual **destruye todas las bases de datos del usuario, tablas y esquemas de desarrollo**.

**XAMPP FIXER** soluciona este problema desde la raíz de forma segura y automatizada.

### 🛡️ Características Principales
- **Garantía de Cero Pérdida de Datos**: Genera un respaldo de seguridad completo y fechado en `mysql/backups/data_backup_YYYYMMDD_HHMMSS` antes de aplicar cualquier modificación.
- **Detección Automática**:
  - Localiza la ruta de instalación de XAMPP (`C:\xampp`).
  - Identifica el puerto configurado en `my.ini` (soporta puertos estándar como `3306` o personalizados como `3307`).
- **Diagnóstico en Tiempo Real**:
  - Comprueba si el puerto objetivo se encuentra ocupado o disponible mediante sockets TCP.
  - Detecta procesos huérfanos de `mysqld.exe` y sus identificadores (PIDs).
  - Identifica archivos residuales de bloqueo (`mysql.pid`).
  - Detecta secuencias de logs corruptas (`aria_log*`, `ib_logfile*`).
- **Auto-Reparación en Cascada (1 Clic)**: Ejecuta una secuencia progresiva de diagnóstico y recuperación, verificando la conectividad hasta que el servicio inicia con éxito.
- **4 Métodos Manuales de Precisión**:
  1. **Finalizar Procesos Huérfanos y Liberar Bloqueos**: Cierra procesos colgados y elimina `mysql.pid` junto con `ib_buffer_pool`.
  2. **Restablecer Logs Dañados**: Purga secuencias incompletas de `aria_log*` e `ib_logfile*`, manteniendo intactos `ibdata1` y las carpetas de bases de datos de usuario.
  3. **Restaurar Catálogos del Sistema**: Reconstruye únicamente los esquemas internos del motor (`mysql`, `performance_schema`, `phpmyadmin`) a partir de la plantilla original de XAMPP sin alterar las bases de datos del usuario.
  4. **Modo de Recuperación InnoDB**: Configura `innodb_force_recovery` (niveles 1 a 6) en `my.ini` para operaciones de rescate.
- **Visor del Registro de Errores**: Muestra en tiempo real las últimas 100 líneas del archivo `mysql_error.log`.
- **Administrador de Respaldos**: Permite consultar el historial de copias de seguridad y restaurar cualquier estado previo con 1 solo clic.
- **Detección de Actualizaciones**: Comprueba automáticamente si existen nuevas versiones disponibles en GitHub y permite actualizar la aplicación directamente.

### 🚀 Compilación desde el Código Fuente
```powershell
# Clonar el repositorio
git clone https://github.com/santy123371/xampp-mysql-fixer.git
cd xampp-mysql-fixer

# Compilar en modo Release
cargo build --release
```
El archivo binario ejecutable se generará en:
```text
target/release/XAMPP_FIXER.exe
```

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
