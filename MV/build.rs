fn main() {
    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");

        // Set the executable icon
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/logo.ico");
        println!("cargo:warning=Setting icon to assets/logo.ico");
        match res.compile() {
            Ok(_) => println!("cargo:warning=Icon set successfully"),
            Err(e) => println!("cargo:warning=Failed to set icon: {}", e),
        }
    }
}
