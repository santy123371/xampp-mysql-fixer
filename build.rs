fn main() {
    println!("cargo:rerun-if-changed=app.ico");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set("ProductName", "XAMPP FIXER");
        res.set("FileDescription", "XAMPP FIXER - Herramienta Automática de Reparación MySQL");
        res.set("CompanyName", "santy123371");
        res.set("LegalCopyright", "Copyright (C) 2026 santy123371");
        res.set("OriginalFilename", "xampp_mysql_fixer.exe");
        res.set("Comments", "Desarrollado por santy123371");
        let _ = res.compile();
    }
}
