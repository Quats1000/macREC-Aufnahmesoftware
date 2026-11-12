//! Bettet unter Windows das Icon in die EXE ein.

fn main() {
    println!("cargo:rerun-if-changed=resources/app.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("resources/app.ico");
        resource.compile().expect("Icon einbetten");
    }
}
