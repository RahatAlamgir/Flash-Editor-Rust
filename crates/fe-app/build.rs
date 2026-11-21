//! Embeds the app icon and version info into `flash-editor.exe` (Windows only).

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icons/app/FlashEditor.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icons/app/FlashEditor.ico");
        res.set("FileDescription", "Flash Editor");
        res.set("ProductName", "Flash Editor");
        if let Err(err) = res.compile() {
            println!("cargo:warning=could not embed the app icon: {err}");
        }
    }
}
