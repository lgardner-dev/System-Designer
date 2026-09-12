fn main() {
    println!("cargo:rerun-if-changed=packaging/windows/app.rc");
    println!("cargo:rerun-if-changed=assets/branding/windows/system-designer.ico");
    #[cfg(feature = "desktop")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let root =
            std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
        let icon = std::path::Path::new(&root).join("assets/branding/windows/system-designer.ico");
        let define = format!("APP_ICON=\"{}\"", icon.to_string_lossy().replace('\\', "/"));
        embed_resource::compile_for("packaging/windows/app.rc", ["system-designer"], [define])
            .manifest_required()
            .expect("could not embed the System Designer Windows icon");
    }
}
