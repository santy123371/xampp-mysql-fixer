fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("app.ico");
        res.set("ProductName", "XAMPP MySQL Medic");
        res.set("FileDescription", "Herramienta de reparación para XAMPP MySQL");
        res.set("LegalCopyright", "Copyright (C) 2026");
        let _ = res.compile();
    }
}
