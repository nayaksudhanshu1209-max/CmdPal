// Embeds the application icon and version info into the .exe (Windows only).
// A failure here must never break the build: the app falls back to the default icon.
fn main() {
    println!("cargo:rerun-if-changed=assets/app-icon/cmdpal.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").map(|o| o == "windows").unwrap_or(false) {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app-icon/cmdpal.ico");
        res.set("ProductName", "CmdPal");
        res.set("FileDescription", "CmdPal command palette");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed icon/resources: {e}");
        }
    }
}
