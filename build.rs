fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");

    // Embeds the icon in the .exe (Start menu, shortcuts, Explorer) — same as
    // GitAgent and SpreadRadar. The release workflow makes assets/icon.ico
    // from assets/icon.png just before building.
    #[cfg(target_os = "windows")]
    {
        let icon_path = std::path::Path::new("assets/icon.ico");
        if !icon_path.exists() {
            println!(
                "cargo:warning=assets/icon.ico not found — the .exe will ship without an embedded icon. \
                 Drop a multi-resolution .ico file there to brand the Windows build."
            );
            return;
        }

        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "PublishKit");
        res.set("ProductName", "PublishKit");
        res.set("CompanyName", "Mayorana");
        res.set("LegalCopyright", "© 2026 Mayorana");
        if let Err(e) = res.compile() {
            // Don't hard-fail — the rc.exe/windres dependency isn't always
            // available on every Windows runner.
            println!(
                "cargo:warning=Failed to embed Windows icon resource: {} \
                 (the build will continue without an icon)",
                e
            );
        }
    }
}
