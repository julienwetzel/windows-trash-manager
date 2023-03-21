fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icons/logo.ico");
        res.compile().expect("failed to build executable logo.");
    }
}
