fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "Cut Ready");
        res.set_icon("assets/icon.ico");
        res.set_manifest_file("app.manifest");
        res.compile().unwrap();
    }
}
