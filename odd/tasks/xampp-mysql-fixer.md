# Feature: XAMPP MySQL Auto-Fixer Desktop GUI

XAMPP MySQL Medic & Repair Utility built in Rust with eframe (egui) native GUI for Windows.

## Specs
S1: Auto-detect XAMPP installation path (defaults to `C:\xampp`, checks `D:\xampp`, allows custom path) and verify MySQL components (`bin/mysqld.exe`, `data`, `backup`, `my.ini`).
S2: Real-time system diagnostic panel showing status of MySQL process (Running, Stopped, Orphaned PID), Port 3306 availability, and file locks / corrupted logs detection.
S3: Automatic safety shield: create full timestamped backup of `mysql/data` before applying any mutating repair action, with rollback / restore capability.
S4: Repair Method 1: Terminate orphaned `mysqld.exe` processes and remove stale lock files (`*.pid`, socket locks).
S5: Repair Method 2: Clean dirty MariaDB/Aria logs (`aria_log*`, `aria_log_control`) and dirty InnoDB redo logs (`ib_logfile*`) while preserving user tables.
S6: Repair Method 3: Safe catalog reconstruction from `backup/` preserving all user database directories and InnoDB dictionary (`ibdata1`).
S7: Repair Method 4: InnoDB force recovery mode configurer (`innodb_force_recovery = 1..6`) for heavily damaged tablespaces.
S8: 1-Click "Diagnóstico y Reparación Automática": cascading automated repair with test start verification and immediate feedback.
S9: Real-time `mysql_error.log` viewer and test start/stop controls directly in the UI.

## Tasks
T1: Configure Cargo.toml with eframe, sysinfo, chrono, and windows subsystem flags -> S1 -> route: direct inline -> commit: completed
T2: Implement core diagnostic and repair engine in `src/engine/mod.rs` (process killer, port checker, backup manager, repair methods 1-4) -> S2, S3, S4, S5, S6, S7 -> route: direct inline -> commit: completed
T3: Implement GUI with eframe in `src/app.rs` with dashboard, 1-click button, individual methods, log viewer, and backup manager -> S1, S8, S9 -> route: direct inline -> commit: completed
T4: Integrate, verify build release, test execution on live XAMPP files safely -> S1-S9 -> route: direct inline -> commit: completed

## Log
L1: User: "broder vamos a crear programa un .exe con interfaz para arreglar el xampp que se encargue de arreglar eso de que mysql se cae a cda rato y no inicia sabes que tenga distintos metodos y que los haga automaticamente pero esto que te digo que lenguaje podriamos usar rust? python o que propones y por que"
L2: User: "dale hagamoslo"
L3: User: "ya instale rust bro"
