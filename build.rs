fn main() {
    println!("cargo:rerun-if-changed=xampp_fixer.ico");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("xampp_fixer.ico");
        res.set("ProductName", "XAMPP FIXER");
        res.set("FileDescription", "XAMPP FIXER - Herramienta Automática de Reparación MySQL");
        res.set("CompanyName", "santy123371");
        res.set("LegalCopyright", "Copyright (C) 2026 santy123371");
        res.set("OriginalFilename", "XAMPP_FIXER.exe");
        res.set("Comments", "Desarrollado por santy123371");
        let profile = std::env::var("PROFILE").unwrap_or_default();
        if profile == "release" {
            res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
        }
        let _ = res.compile();
    }
}
