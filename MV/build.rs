fn main() {
    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");
    }
}
