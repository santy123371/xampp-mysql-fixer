# XAMPP FIXER

<p align="center">
  <img src="icon_square.png" alt="XAMPP FIXER Logo" width="128" height="128">
</p>

<p align="center">
  <b>Fast, safe desktop GUI utility built in Rust to diagnose and auto-repair XAMPP MySQL / MariaDB crashes without losing database tables.</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/GUI-egui%20%2F%20eframe-blue.svg" alt="egui">
  <img src="https://img.shields.io/badge/Platform-Windows-0078D6.svg" alt="Windows">
  <img src="https://img.shields.io/badge/Author-santy123371-crimson.svg" alt="Author">
</p>

<p align="center">
  <a href="https://github.com/santy123371/xampp-mysql-fixer/releases/latest">
    <img src="https://img.shields.io/badge/📥%20Download-XAMPP__FIXER.exe-success?style=for-the-badge&logo=windows" alt="Download Executable">
  </a>
</p>

---

## ⚡ The Problem

When MariaDB/MySQL in XAMPP crashes unexpectedly (e.g. system reboot, power outage, stuck background processes, or corrupted log sequences), XAMPP's Control Panel fails to start MySQL, often showing errors such as:

> *"MySQL shutdown unexpectedly. This may be due to a blocked port, missing dependencies, improper privileges, a crash, or a shutdown of another method."*

Common internet advice tells users to delete the entire `data` folder or copy everything from `backup`, which **destroys user databases, tables, and stored procedures**.

**XAMPP FIXER** solves this safely.

---

## 🛡️ Key Features

- **Zero Data Loss Guarantee**: Automatically creates an instant timestamped backup in `mysql/backups/data_backup_YYYYMMDD_HHMMSS` before any fix is applied.
- **Dynamic Port & Path Detection**: Automatically discovers XAMPP installation (`C:\xampp`) and parses configured ports from `my.ini` (e.g., standard `3306` or custom `3307`).
- **Live Diagnostics**:
  - Checks if target port is occupied or free.
  - Identifies active orphan `mysqld.exe` processes and PIDs.
  - Detects `mysql.pid` lock files.
  - Inspects dirty `aria_log*` sequence corruptions.
- **1-Click Cascading Auto-Repair**: Executes a safe step-by-step diagnostic and repair sequence, testing connectivity after each stage until MySQL successfully initializes.
- **Manual Precision Methods**:
  1. **Kill Zombies & Clear Locks**: Terminates stuck `mysqld.exe` processes and removes stale `mysql.pid` and `ib_buffer_pool`.
  2. **Reset Logs**: Purges uncommitted/dirty `aria_log*` and `ib_logfile*` files safely while preserving `ibdata1` and all database directories.
  3. **Restore System Catalogs**: Replaces corrupted internal catalogs (`mysql`, `performance_schema`, `phpmyadmin`) from pristine XAMPP templates without touching your actual database folders.
  4. **InnoDB Force Recovery**: Allows temporary recovery modes (levels 1 through 6) directly in `my.ini` to extract inaccessible data.
- **Error Log Viewer**: Inspects real-time tail of `mysql_error.log`.
- **Integrated Backup Manager**: Lists existing backups and allows 1-click restore.

---

## 🚀 Building from Source

### Prerequisites

- Windows 10/11 (x64)
- [Rust & Cargo](https://rustup.rs/) (MSVC toolchain recommended)

### Build Instructions

```powershell
# Clone the repository
git clone https://github.com/santy123371/xampp-mysql-fixer.git
cd xampp-mysql-fixer

# Build release executable
cargo build --release
```

The optimized, standalone executable will be located at:
```text
target/release/XAMPP_FIXER.exe
```

---

## 💻 Tech Stack

- **Core**: [Rust](https://www.rust-lang.org/)
- **GUI**: [egui / eframe](https://github.com/emilk/egui) (pure immediate mode GUI, hardware-accelerated via DirectX/wgpu)
- **Process Inspection**: `sysinfo`
- **Windows Integration**: `winres` for native PE metadata and multi-resolution application icons

---

## 👤 Author

- **santy123371** — GitHub: [@santy123371](https://github.com/santy123371)

---

## ⚖️ License

Distributed under the MIT License. See `LICENSE` for details.
