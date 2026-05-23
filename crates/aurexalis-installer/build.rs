//! Embebe el icono Windows en el ejecutable del instalador.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let icon = std::path::Path::new(&manifest)
            .join("..")
            .join("..")
            .join("assets")
            .join("branding")
            .join("aurexalis.ico");
        println!("cargo:rerun-if-changed={}", icon.display());
        if icon.is_file() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(icon.to_string_lossy().as_ref());
            if let Err(error) = res.compile() {
                eprintln!("cargo:warning=icono Windows no aplicado: {error}");
            }
        } else {
            eprintln!("cargo:warning=no se encontro {}", icon.display());
        }
    }
}
